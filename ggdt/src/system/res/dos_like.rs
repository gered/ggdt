//! This provides a "DOS-like" implementation of [`SystemResources`] which is used in conjunction with a [`System`]
//! instance to provide something resembling an old DOS VGA mode 13h style experience (there are differences, however).
//!
//! ```no_run
//! use ggdt::graphics::*;
//! use ggdt::system::*;
//!
//! let config = DosLikeConfig::new();
//! let mut system = SystemBuilder::new()
//! 	.window_title("Example")
//! 	.build(config)
//! 	.unwrap();
//!
//! while !system.do_events().unwrap() {
//! 	if system.res.keyboard.is_key_pressed(Scancode::Escape) {
//! 		break;
//! 	}
//!
//! 	system.update().unwrap();
//!
//!     system.res.video.clear(0);
//!     system.res.video.set_pixel(10, 10, 4);
//!     system.res.video.print_string("Hello, world!", 10, 50, FontRenderOpts::Color(10), &system.res.font);
//!
//! 	system.display().unwrap();
//! }
//! ```
//!

use sdl2::video::Window;

use crate::system::*;

/// Configuration / builder for configuring and constructing an instance of [`DosLike`].
pub struct DosLikeConfig {
	screen_width: u32,
	screen_height: u32,
	initial_scale_factor: u32,
	integer_scaling: bool,
}

impl DosLikeConfig {
	/// Returns a new [`DosLikeConfig`] with a default configuration.
	pub fn new() -> Self {
		DosLikeConfig {
			screen_width: SCREEN_WIDTH,
			screen_height: SCREEN_HEIGHT,
			initial_scale_factor: DEFAULT_SCALE_FACTOR,
			integer_scaling: false,
		}
	}

	// TODO: add customization ability for setting different screen dimensions instead of it being hardcoded

	/// Sets an integer scaling factor for the [`System`] being built to up-scale the virtual
	/// framebuffer to when displaying it on screen.
	pub fn scale_factor(mut self, scale_factor: u32) -> Self {
		self.initial_scale_factor = scale_factor;
		self
	}

	/// Enables or disables restricting the final rendered output to always be integer scaled,
	/// even if that result will not fully fill the area of the window.
	pub fn integer_scaling(mut self, enable: bool) -> Self {
		self.integer_scaling = enable;
		self
	}
}

impl SystemResourcesConfig for DosLikeConfig {
	type SystemResourcesType = DosLike;

	fn build(
		self,
		_video_subsystem: &VideoSubsystem,
		audio_subsystem: &AudioSubsystem,
		mut window: Window,
	) -> Result<Self::SystemResourcesType, SystemResourcesError> {
		let texture_pixel_size = 4; // 32-bit ARGB format

		let window_width = self.screen_width * self.initial_scale_factor;
		let window_height = self.screen_height * self.initial_scale_factor;
		if let Err(error) = window.set_size(window_width, window_height) {
			return Err(SystemResourcesError::SDLError(error.to_string()));
		}

		// turn the window into a canvas (under the hood, an SDL Renderer that owns the window)

		let mut canvas_builder = window.into_canvas();
		let mut sdl_canvas = match canvas_builder.build() {
			Ok(canvas) => canvas,
			Err(error) => return Err(SystemResourcesError::SDLError(error.to_string())),
		};
		if let Err(error) = sdl_canvas.set_logical_size(self.screen_width, self.screen_height) {
			return Err(SystemResourcesError::SDLError(error.to_string()));
		};

		// TODO: newer versions of rust-sdl2 support this directly off the WindowCanvas struct
		unsafe {
			sdl2::sys::SDL_RenderSetIntegerScale(
				sdl_canvas.raw(),
				if self.integer_scaling {
					sdl2::sys::SDL_bool::SDL_TRUE
				} else {
					sdl2::sys::SDL_bool::SDL_FALSE
				},
			);
		}

		// create an SDL texture which we will be uploading to every frame to display the
		// application's framebuffer

		let sdl_texture = match sdl_canvas.create_texture_streaming(
			Some(PixelFormatEnum::ARGB8888),
			self.screen_width,
			self.screen_height,
		) {
			Ok(texture) => texture,
			Err(error) => return Err(SystemResourcesError::SDLError(error.to_string())),
		};
		let sdl_texture_pitch = (sdl_texture.query().width * texture_pixel_size) as usize;

		// create a raw 32-bit RGBA buffer that will be used as the temporary source for
		// SDL texture uploads each frame. necessary as applications are dealing with 8-bit indexed
		// bitmaps, not 32-bit RGBA pixels, so this temporary buffer is where we convert the final
		// application framebuffer to 32-bit RGBA pixels before it is uploaded to the SDL texture
		let texture_pixels_size = (self.screen_width * self.screen_height * texture_pixel_size) as usize;
		let texture_pixels = vec![0u32; texture_pixels_size].into_boxed_slice();

		// create the Bitmap object that will be exposed to the application acting as the system
		// backbuffer

		let framebuffer = match Bitmap::new(SCREEN_WIDTH, SCREEN_HEIGHT) {
			Ok(bmp) => bmp,
			Err(error) => return Err(SystemResourcesError::SDLError(error.to_string())),
		};

		// create the default palette, initialized to the VGA default palette. also exposed to the
		// application for manipulation

		let palette = match Palette::new_vga_palette() {
			Ok(palette) => palette,
			Err(error) => return Err(SystemResourcesError::SDLError(error.to_string())),
		};

		// create the default font, initialized to the VGA BIOS default font.

		let font = match BitmaskFont::new_vga_font() {
			Ok(font) => font,
			Err(error) => return Err(SystemResourcesError::SDLError(error.to_string())),
		};

		// create audio device and queue

		let audio_spec = AudioSpecDesired {
			freq: Some(TARGET_AUDIO_FREQUENCY as i32),
			channels: Some(TARGET_AUDIO_CHANNELS),
			samples: None,
		};
		let mut audio = Audio::new(audio_spec, &audio_subsystem)?;
		audio.resume();
		let audio_queue = AudioQueue::new(&audio);

		// create all of the input device objects

		let keyboard = Keyboard::new();
		let mouse = Mouse::new();
		let cursor = CustomMouseCursor::new();

		Ok(DosLike {
			sdl_canvas,
			sdl_texture,
			sdl_texture_pitch,
			texture_pixels,
			audio,
			audio_queue,
			palette,
			video: framebuffer,
			font,
			keyboard,
			mouse,
			cursor,
        })
	}
}

/// A [`SystemResources`] implementation that provides indexed-colour [`Bitmap`]s for graphics, simple 8-bit / 22khz
/// audio via [`Audio`] and keyboard/mouse input.
pub struct DosLike {
	sdl_canvas: WindowCanvas,
	sdl_texture: Texture,
	sdl_texture_pitch: usize,
	texture_pixels: Box<[u32]>,

	/// An [`Audio`] instance that allows interacting with the system's audio output device.
	pub audio: Audio,

	/// An [`AudioQueue`] instance that can queue up playback/stop commands to be issued to the
	/// system's [`Audio`] instance a bit more flexibly. If you use this, your application must
	/// manually call [`AudioQueue::apply`] or [`AudioQueue::apply_to_device`] in your loop to
	/// flush the queued commands, otherwise this queue will not do anything.
	pub audio_queue: AudioQueue,

	/// The [`Palette`] that will be used in conjunction with the `video` backbuffer to
	/// render the final output to the screen whenever [`System::display`] is called.
	pub palette: Palette,

	/// The primary backbuffer [`Bitmap`] that will be rendered to the screen whenever
	/// [`System::display`] is called. Regardless of the actual window size, this bitmap is always
	/// [`SCREEN_WIDTH`]x[`SCREEN_HEIGHT`] pixels in size.
	pub video: Bitmap,

	/// A pre-loaded [`Font`] that can be used for text rendering.
	pub font: BitmaskFont,

	/// The current keyboard state. To ensure it is updated each frame, you should call
	/// [`System::do_events`] or [`System::do_events_with`] each frame.
	pub keyboard: Keyboard,

	/// The current mouse state. To ensure it is updated each frame, you should call
	/// [`System::do_events`] or [`System::do_events_with`] each frame.
	pub mouse: Mouse,

	/// Manages custom mouse cursor graphics and state. Use this to set/unset a custom mouse cursor bitmap.
	/// When set, rendering should occur automatically during calls to [`SystemResources::display`].
	pub cursor: CustomMouseCursor,
}

impl std::fmt::Debug for DosLike {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DosLike")
            .field("audio", &self.audio)
            .field("audio_queue", &self.audio_queue)
            .field("palette", &self.palette)
            .field("video", &self.video)
            .field("font", &self.font)
            .field("keyboard", &self.keyboard)
            .field("mouse", &self.mouse)
            .finish_non_exhaustive()
    }
}

impl SystemResources for DosLike {
	fn update(&mut self) -> Result<(), SystemResourcesError> {
		self.cursor.update(&self.mouse);

		match self.audio_queue.apply(&mut self.audio) {
			Ok(_) => Ok(()),
			Err(error) => Err(SystemResourcesError::AudioDeviceError(error))
		}
	}

	/// Takes the `video` backbuffer bitmap and `palette` and renders it to the window, up-scaled
	/// to fill the window (preserving aspect ratio of course).
	fn display(&mut self) -> Result<(), SystemResourcesError> {
        self.cursor.render(&mut self.video);

        // convert application framebuffer to 32-bit RGBA pixels, and then upload it to the SDL
        // texture so it will be displayed on screen

        self.video.copy_as_argb_to(&mut self.texture_pixels, &self.palette);

        let texture_pixels = self.texture_pixels.as_byte_slice();
        if let Err(error) = self.sdl_texture.update(None, texture_pixels, self.sdl_texture_pitch) {
            return Err(SystemResourcesError::SDLError(error.to_string()));
        }
        self.sdl_canvas.clear();
        if let Err(error) = self.sdl_canvas.copy(&self.sdl_texture, None, None) {
            return Err(SystemResourcesError::SDLError(error));
        }
        self.sdl_canvas.present();

        self.cursor.hide(&mut self.video);

        Ok(())
	}

	fn update_event_state(&mut self) -> Result<(), SystemResourcesError> {
		self.keyboard.update();
		self.mouse.update();
		Ok(())
	}

	fn handle_event(&mut self, event: &SystemEvent) -> Result<bool, SystemResourcesError> {
		if self.keyboard.handle_event(event) {
			return Ok(true);
		}
		if self.mouse.handle_event(event) {
			return Ok(true);
		}
		Ok(false)
	}

	#[inline]
	fn width(&self) -> u32 {
		self.video.width()
	}

	#[inline]
	fn height(&self) -> u32 {
		self.video.height()
	}
}
