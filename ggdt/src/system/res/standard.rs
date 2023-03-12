use byte_slice_cast::AsByteSlice;

use crate::{DEFAULT_SCALE_FACTOR, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::audio::{Audio, TARGET_AUDIO_CHANNELS, TARGET_AUDIO_FREQUENCY};
use crate::audio::queue::AudioQueue;
use crate::graphics::bitmap::rgb::RgbaBitmap;
use crate::graphics::font::BitmaskFont;
use crate::system::event::{SystemEvent, SystemEventHandler};
use crate::system::input_devices::InputDevice;
use crate::system::input_devices::keyboard::Keyboard;
use crate::system::input_devices::mouse::cursor::CustomMouseCursor;
use crate::system::input_devices::mouse::Mouse;
use crate::system::res::{SystemResources, SystemResourcesConfig, SystemResourcesError};

pub struct StandardConfig {
	screen_width: u32,
	screen_height: u32,
	initial_scale_factor: u32,
	integer_scaling: bool,
}

impl StandardConfig {
	/// Returns a new [`DosLikeConfig`] with a default configuration.
	pub fn new() -> Self {
		StandardConfig {
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

impl SystemResourcesConfig for StandardConfig {
	type SystemResourcesType = Standard;

	fn build(
		self,
		_video_subsystem: &sdl2::VideoSubsystem,
		audio_subsystem: &sdl2::AudioSubsystem,
		mut window: sdl2::video::Window
	) -> Result<Self::SystemResourcesType, SystemResourcesError> {
		let texture_pixel_size = 4; // 32-bit ARGB format

		let window_width = self.screen_width * self.initial_scale_factor;
		let window_height = self.screen_height * self.initial_scale_factor;
		if let Err(error) = window.set_size(window_width, window_height) {
			return Err(SystemResourcesError::SDLError(error.to_string()));
		}

		// turn the window into a canvas (under the hood, an SDL Renderer that owns the window)

		let canvas_builder = window.into_canvas();
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
			Some(sdl2::pixels::PixelFormatEnum::ARGB8888),
			self.screen_width,
			self.screen_height,
		) {
			Ok(texture) => texture,
			Err(error) => return Err(SystemResourcesError::SDLError(error.to_string())),
		};
		let sdl_texture_pitch = (sdl_texture.query().width * texture_pixel_size) as usize;

		// create the Bitmap object that will be exposed to the application acting as the system
		// backbuffer

		let framebuffer = match RgbaBitmap::new(SCREEN_WIDTH, SCREEN_HEIGHT) {
			Ok(bmp) => bmp,
			Err(error) => return Err(SystemResourcesError::SDLError(error.to_string())),
		};

		// create the default font, initialized to the VGA BIOS default font.

		let font = match BitmaskFont::new_vga_font() {
			Ok(font) => font,
			Err(error) => return Err(SystemResourcesError::SDLError(error.to_string())),
		};

		// create audio device and queue

		let audio_spec = sdl2::audio::AudioSpecDesired {
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

		Ok(Standard {
			sdl_canvas,
			sdl_texture,
			sdl_texture_pitch,
			audio,
			audio_queue,
			video: framebuffer,
			font,
			keyboard,
			mouse,
			cursor,
		})

	}
}

pub struct Standard {
	sdl_canvas: sdl2::render::WindowCanvas,
	sdl_texture: sdl2::render::Texture,
	sdl_texture_pitch: usize,

	/// An [`Audio`] instance that allows interacting with the system's audio output device.
	pub audio: Audio,

	/// An [`AudioQueue`] instance that can queue up playback/stop commands to be issued to the
	/// system's [`Audio`] instance a bit more flexibly. If you use this, your application must
	/// manually call [`AudioQueue::apply`] or [`AudioQueue::apply_to_device`] in your loop to
	/// flush the queued commands, otherwise this queue will not do anything.
	pub audio_queue: AudioQueue,

	/// The primary backbuffer [`Bitmap`] that will be rendered to the screen whenever
	/// [`System::display`] is called. Regardless of the actual window size, this bitmap is always
	/// [`SCREEN_WIDTH`]x[`SCREEN_HEIGHT`] pixels in size.
	pub video: RgbaBitmap,

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
	pub cursor: CustomMouseCursor<RgbaBitmap>,
}

impl std::fmt::Debug for Standard {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Standard")
			.field("audio", &self.audio)
			.field("audio_queue", &self.audio_queue)
			.field("video", &self.video)
			.field("font", &self.font)
			.field("keyboard", &self.keyboard)
			.field("mouse", &self.mouse)
			.finish_non_exhaustive()
	}
}

impl SystemResources for Standard {
	fn update(&mut self) -> Result<(), SystemResourcesError> {
		self.cursor.update(&self.mouse);

		match self.audio_queue.apply(&mut self.audio) {
			Ok(_) => Ok(()),
			Err(error) => Err(SystemResourcesError::AudioDeviceError(error))
		}
	}

	fn display(&mut self) -> Result<(), SystemResourcesError> {
		self.cursor.render(&mut self.video);

		let texture_pixels = self.video.pixels().as_byte_slice();
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
