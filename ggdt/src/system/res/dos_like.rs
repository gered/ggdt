//! This provides a "DOS-like" implementation of [`SystemResources`] which is used in conjunction with a [`System`]
//! instance to provide something resembling an old DOS VGA mode 13h style experience (there are differences, however).
//!
//! ```no_run
//! use ggdt::prelude::*;
//!
//! let config = DosLikeConfig::default();
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

use crate::audio::queue::AudioQueue;
use crate::audio::{Audio, TARGET_AUDIO_CHANNELS, TARGET_AUDIO_FREQUENCY};
use crate::graphics::bitmap::indexed::IndexedBitmap;
use crate::graphics::font::BitmaskFont;
use crate::graphics::palette::Palette;
use crate::system::event::{SystemEvent, SystemEventHandler, WindowEvent};
use crate::system::framebuffer::{calculate_logical_screen_size, SdlFramebuffer};
use crate::system::input_devices::keyboard::Keyboard;
use crate::system::input_devices::mouse::cursor::CustomMouseCursor;
use crate::system::input_devices::mouse::Mouse;
use crate::system::input_devices::InputDevice;
use crate::system::res::{SystemResources, SystemResourcesConfig, SystemResourcesError};

const DEFAULT_SCREEN_WIDTH: u32 = 320;
const DEFAULT_SCREEN_HEIGHT: u32 = 240;
const DEFAULT_SCALE_FACTOR: u32 = 3;

/// Configuration / builder for configuring and constructing an instance of [`DosLike`].
pub struct DosLikeConfig {
	screen_width: u32,
	screen_height: u32,
	fixed_screen_size: bool,
	initial_scale_factor: u32,
	integer_scaling: bool,
}

impl Default for DosLikeConfig {
	/// Returns a new [`DosLikeConfig`] with a default configuration.
	fn default() -> Self {
		DosLikeConfig::fixed_screen_size(DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT, false)
	}
}

impl DosLikeConfig {
	/// Creates a configuration that will use a fixed screen size at a set scaling factor. Any window resizing
	/// will simply scale up or down the final image on screen, but the application will always use the same
	/// logical screen resolution, `screen_width` and `screen_height`, at runtime.
	pub fn fixed_screen_size(screen_width: u32, screen_height: u32, integer_scaling: bool) -> Self {
		DosLikeConfig {
			screen_width,
			screen_height,
			initial_scale_factor: DEFAULT_SCALE_FACTOR,
			integer_scaling,
			fixed_screen_size: true,
		}
	}

	/// Creates a configuration that allows the screen size to be automatically updated at runtime to match the
	/// current window size, including any arbitrary user window resizing. The final image on screen will always be
	/// scaled up by the factor given. The logical screen size at runtime (as seen by the application code) is
	/// always based on:
	///
	/// `logical_screen_width = ceil(window_width / scale_factor)`  
	/// `logical_screen_height = ceil(window_height / scale_factor)`
	pub fn variable_screen_size(initial_width: u32, initial_height: u32) -> Self {
		DosLikeConfig {
			screen_width: initial_width,
			screen_height: initial_height,
			initial_scale_factor: DEFAULT_SCALE_FACTOR,
			integer_scaling: false,
			fixed_screen_size: false,
		}
	}

	/// Sets an integer scaling factor for the [`System`] being built to up-scale the virtual
	/// framebuffer to when displaying it on screen.
	pub fn scale_factor(mut self, scale_factor: u32) -> Self {
		self.initial_scale_factor = scale_factor;
		self
	}
}

impl SystemResourcesConfig for DosLikeConfig {
	type SystemResourcesType = DosLike;

	fn build(
		self,
		_video_subsystem: &sdl2::VideoSubsystem,
		audio_subsystem: &sdl2::AudioSubsystem,
		mut window: sdl2::video::Window,
	) -> Result<Self::SystemResourcesType, SystemResourcesError> {
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

		// TODO: newer versions of rust-sdl2 support this directly off the WindowCanvas struct
		unsafe {
			sdl2::sys::SDL_RenderSetIntegerScale(
				sdl_canvas.raw(),
				if self.integer_scaling {
					sdl2::sys::SDL_bool::SDL_TRUE //
				} else {
					sdl2::sys::SDL_bool::SDL_FALSE
				},
			);
		}

		// create the SDL framebuffer at the initial logical screen size

		let framebuffer = SdlFramebuffer::new(&mut sdl_canvas, self.screen_width, self.screen_height, true)?;

		// create the Bitmap object that will be exposed to the application acting as the system
		// backbuffer

		let screen_bitmap = match IndexedBitmap::new(self.screen_width, self.screen_height) {
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

		let audio_spec = sdl2::audio::AudioSpecDesired {
			freq: Some(TARGET_AUDIO_FREQUENCY as i32),
			channels: Some(TARGET_AUDIO_CHANNELS),
			samples: None,
		};
		let mut audio = Audio::new(audio_spec, audio_subsystem)?;
		audio.resume();
		let audio_queue = AudioQueue::new(&audio);

		// create all of the input device objects

		let keyboard = Keyboard::new();
		let mouse = Mouse::new();
		let cursor = CustomMouseCursor::new();

		Ok(DosLike {
			sdl_canvas,
			framebuffer,
			scale_factor: self.initial_scale_factor,
			fixed_screen_size: self.fixed_screen_size,
			audio,
			audio_queue,
			palette,
			video: screen_bitmap,
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
	sdl_canvas: sdl2::render::WindowCanvas,
	framebuffer: SdlFramebuffer,
	scale_factor: u32,
	fixed_screen_size: bool,

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
	pub video: IndexedBitmap,

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
	pub cursor: CustomMouseCursor<IndexedBitmap>,
}

impl std::fmt::Debug for DosLike {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("DosLike") //
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
	type PixelType = u8;

	fn update(&mut self) -> Result<(), SystemResourcesError> {
		self.cursor.update(&self.mouse);

		match self.audio_queue.apply(&mut self.audio) {
			Ok(_) => Ok(()),
			Err(error) => Err(SystemResourcesError::AudioDeviceError(error)),
		}
	}

	/// Takes the `video` backbuffer bitmap and `palette` and renders it to the window, up-scaled
	/// to fill the window (preserving aspect ratio of course).
	fn display(&mut self) -> Result<(), SystemResourcesError> {
		self.cursor.render(&mut self.video);
		self.framebuffer.display_indexed_bitmap(&mut self.sdl_canvas, &self.video, &self.palette)?;
		self.cursor.hide(&mut self.video);
		Ok(())
	}

	fn update_event_state(&mut self) -> Result<(), SystemResourcesError> {
		self.keyboard.update();
		self.mouse.update();
		Ok(())
	}

	fn handle_event(&mut self, event: &SystemEvent) -> Result<bool, SystemResourcesError> {
		if let SystemEvent::Window(WindowEvent::SizeChanged(width, height)) = event {
			if !self.fixed_screen_size {
				self.resize_screen(*width as u32, *height as u32)?;
			}
			return Ok(true);
		}

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

impl DosLike {
	fn resize_screen(&mut self, new_width: u32, new_height: u32) -> Result<(), SystemResourcesError> {
		let (logical_width, logical_height) = calculate_logical_screen_size(new_width, new_height, self.scale_factor);

		let framebuffer = SdlFramebuffer::new(&mut self.sdl_canvas, logical_width, logical_height, true)?;

		let screen_bitmap = match IndexedBitmap::new(logical_width, logical_height) {
			Ok(bmp) => bmp,
			Err(error) => return Err(SystemResourcesError::SDLError(error.to_string())),
		};

		self.framebuffer = framebuffer;
		self.video = screen_bitmap;

		Ok(())
	}
}
