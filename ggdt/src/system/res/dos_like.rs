use sdl2::video::Window;

use crate::system::*;

pub struct DosLikeConfig {
	screen_width: u32,
	screen_height: u32,
	vsync: bool,
	target_framerate: Option<u32>,
	initial_scale_factor: u32,
	integer_scaling: bool,
}

impl DosLikeConfig {
	pub fn new() -> Self {
		DosLikeConfig {
			screen_width: SCREEN_WIDTH,
			screen_height: SCREEN_HEIGHT,
			vsync: false,
			target_framerate: None,
			initial_scale_factor: DEFAULT_SCALE_FACTOR,
			integer_scaling: false,
		}
	}

	/// Enables or disables V-Sync for the [`System`] to be built. Enabling V-sync automatically
	/// disables `target_framerate`.
	pub fn vsync(mut self, enable: bool) -> Self {
		self.vsync = enable;
		self.target_framerate = None;
		self
	}

	/// Sets a target framerate for the [`System`] being built to run at. This is intended to be
	/// used when V-sync is not desired, so setting a target framerate automatically disables
	/// `vsync`.
	pub fn target_framerate(mut self, target_framerate: u32) -> Self {
		self.target_framerate = Some(target_framerate);
		self.vsync = false;
		self
	}

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
		if self.vsync {
			canvas_builder = canvas_builder.present_vsync();
		}
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

		Ok(DosLike {
			sdl_canvas,
			sdl_texture,
			sdl_texture_pitch,
			texture_pixels,
			vsync: self.vsync,
			target_framerate: self.target_framerate,
			audio,
			audio_queue,
			palette,
			video: framebuffer,
			font,
			keyboard,
			mouse,
        })
	}
}

pub struct DosLike {
	sdl_canvas: WindowCanvas,
	sdl_texture: Texture,
	sdl_texture_pitch: usize,
	texture_pixels: Box<[u32]>,
	vsync: bool,
	target_framerate: Option<u32>,

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
            .field("vsync", &self.vsync)
            .field("target_framerate", &self.target_framerate)
            .finish_non_exhaustive()
    }
}

impl SystemResources for DosLike {
	fn update(&mut self) -> Result<(), SystemResourcesError> {
		match self.audio_queue.apply(&mut self.audio) {
			Ok(_) => Ok(()),
			Err(error) => Err(SystemResourcesError::AudioDeviceError(error))
		}
	}

	fn display(&mut self) -> Result<(), SystemResourcesError> {
        self.mouse.render_cursor(&mut self.video);

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

        self.mouse.hide_cursor(&mut self.video);

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

	#[inline]
	fn vsync(&self) -> bool {
		self.vsync
	}

	#[inline]
	fn target_framerate(&self) -> Option<u32> {
		self.target_framerate
	}
}
