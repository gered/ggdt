use std::fmt::Formatter;

use byte_slice_cast::AsByteSlice;
use sdl2::{AudioSubsystem, Sdl, TimerSubsystem, VideoSubsystem};
use sdl2::audio::AudioSpecDesired;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, WindowCanvas};
use thiserror::Error;

use crate::{DEFAULT_SCALE_FACTOR, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::audio::*;
use crate::graphics::*;

pub use self::event::*;
pub use self::input_devices::*;
pub use self::input_devices::keyboard::*;
pub use self::input_devices::mouse::*;
pub use self::res::*;
pub use self::res::dos_like::*;

pub mod event;
pub mod input_devices;
pub mod res;

fn is_x11_compositor_skipping_problematic() -> bool {
	/*
	this is _probably_ a bit of a hack.

	currently on linux systems, SDL2 (2.0.8+), tries to "skip" (disable) the X11 server
	compositor when starting up. this is to reduce/remove any added latency from the SDL program
	that is usually introduced by the compositor when it is enabled for the window. if SDL did
	disable the compositor in this manner, it will re-enable it when SDL shuts down. the
	intention is for the compositor to be disabled for just the SDL window(s) only and to affect
	nothing else running concurrently.

	this works great for several desktop environments, but it unfortunately has a global effect
	on KDE/Kwin, where users may notice a visible screen flicker, other concurrently running
	applications may exhibit visual artifacts/weirdness, and (all?) other application windows
	while the SDL app is running will also have the compositor disabled for them too.

	not great! this function is a quick, hacky, and probably-not-bullet-proof method to detect
	if KDE/Kwin is the current desktop environment. in the future other known problem
	configurations could be added here and/or this could/should be updated with a better method
	to check for this.
	 */
	match std::env::consts::OS {
		"linux" | "freebsd" | "netbsd" | "openbsd" => {
			match std::env::var("XDG_SESSION_DESKTOP") {
				Ok(value) => value.eq_ignore_ascii_case("KDE"),
				Err(_) => false
			}
		}
		_ => false,
	}
}

#[derive(Error, Debug)]
pub enum SystemError {
	#[error("System SDL error: {0}")]
	SDLError(String),

	#[error("System audio error: {0}")]
	AudioError(#[from] AudioError),

	#[error("SystemResources error: {0}")]
	SystemResourcesError(#[from] SystemResourcesError),
}

/// Builder for configuring and constructing an instance of [`System`].
#[derive(Debug)]
pub struct SystemBuilder {
	window_title: String,
	vsync: bool,
	target_framerate: Option<u32>,
	resizable: bool,
	show_mouse: bool,
	relative_mouse_scaling: bool,
	skip_x11_compositor: bool,
}

impl SystemBuilder {
	/// Returns a new [`SystemBuilder`] with a default configuration.
	pub fn new() -> SystemBuilder {
		SystemBuilder {
			window_title: String::new(),
			vsync: false,
			target_framerate: None,
			resizable: true,
			show_mouse: false,
			relative_mouse_scaling: true,
			skip_x11_compositor: !is_x11_compositor_skipping_problematic(),
		}
	}

	/// Set the window title for the [`System`] to be built.
	pub fn window_title(mut self, window_title: &str) -> Self {
		self.window_title = window_title.to_string();
		self
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

	/// Sets whether the window will be resizable by the user for the [`System`] being built.
	pub fn resizable(mut self, enable: bool) -> Self {
		self.resizable = enable;
		self
	}

	/// Enables or disables mouse cursor display by the operating system when the cursor is over
	/// the window for the [`System`] being built. Disable this if you intend to render your own
	/// custom mouse cursor.
	pub fn show_mouse(mut self, enable: bool) -> Self {
		self.show_mouse = enable;
		self
	}

	/// Enables or disables automatic DPI scaling of mouse relative movement values (delta values)
	/// available via the [`Mouse`] input device.
	pub fn relative_mouse_scaling(mut self, enable: bool) -> Self {
		self.relative_mouse_scaling = enable;
		self
	}

	/// Enables or disables skipping the X11 server compositor on Linux systems only. This can be
	/// set to manually control the underlying SDL hint that is used to control this setting. The
	/// default setting that [`SystemBuilder`] configures is to follow the SDL default, except where
	/// the setting affects the system globally (in certain desktop environments, e.g. KDE/Kwin)
	/// which may be undesired by end-users, at the cost of some additional input latency.
	pub fn skip_x11_compositor(mut self, enable: bool) -> Self {
		self.skip_x11_compositor = enable;
		self
	}

	/// Builds and returns a [`System`] from the current configuration.
	pub fn build<ConfigType: SystemResourcesConfig>(
		&self,
		config: ConfigType,
	) -> Result<System<ConfigType::SystemResourcesType>, SystemError> {

		sdl2::hint::set("SDL_RENDER_VSYNC", if self.vsync { "1" } else { "0" });
		sdl2::hint::set("SDL_MOUSE_RELATIVE_SCALING", if self.relative_mouse_scaling { "1" } else { "0" });
		sdl2::hint::set("SDL_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", if self.skip_x11_compositor { "1" } else { "0" });

		// build all the individual SDL subsystems

		let sdl_context = match sdl2::init() {
			Ok(sdl_context) => sdl_context,
			Err(message) => return Err(SystemError::SDLError(message)),
		};

		let sdl_timer_subsystem = match sdl_context.timer() {
			Ok(timer_subsystem) => timer_subsystem,
			Err(message) => return Err(SystemError::SDLError(message)),
		};

		let sdl_video_subsystem = match sdl_context.video() {
			Ok(video_subsystem) => video_subsystem,
			Err(message) => return Err(SystemError::SDLError(message)),
		};

		let sdl_event_pump = match sdl_context.event_pump() {
			Ok(event_pump) => event_pump,
			Err(message) => return Err(SystemError::SDLError(message)),
		};

		let sdl_audio_subsystem = match sdl_context.audio() {
			Ok(audio_subsystem) => audio_subsystem,
			Err(message) => return Err(SystemError::SDLError(message)),
		};

		// create the window with an initial default size that will be overridden during
		// SystemResources initialization

		let mut window_builder = &mut (sdl_video_subsystem.window(
			self.window_title.as_str(),
			640,
			480,
		));
		if self.resizable {
			window_builder = window_builder.resizable();
		}
		let sdl_window = match window_builder.build() {
			Ok(window) => window,
			Err(error) => return Err(SystemError::SDLError(error.to_string())),
		};

		sdl_context.mouse().show_cursor(self.show_mouse);

		let system_resources = match config.build(
			&sdl_video_subsystem,
			&sdl_audio_subsystem,
			sdl_window
		) {
			Ok(system_resources) => system_resources,
			Err(error) => return Err(SystemError::SystemResourcesError(error)),
		};

		let event_pump = SystemEventPump::from(sdl_event_pump);

		Ok(System {
			sdl_context,
			sdl_audio_subsystem,
			sdl_video_subsystem,
			sdl_timer_subsystem,
			res: system_resources,
			event_pump,
			vsync: self.vsync,
			target_framerate: self.target_framerate,
			target_framerate_delta: None,
			next_tick: 0,
		})
	}
}

/// Holds all primary structures necessary for interacting with the operating system and for
/// applications to render to the display, react to input device events, etc. through the
/// "virtual machine" exposed by this library.
#[allow(dead_code)]
pub struct System<SystemResType>
where SystemResType: SystemResources {
	sdl_context: Sdl,
	sdl_audio_subsystem: AudioSubsystem,
	sdl_video_subsystem: VideoSubsystem,
	sdl_timer_subsystem: TimerSubsystem,

	vsync: bool,
	target_framerate: Option<u32>,
	target_framerate_delta: Option<i64>,
	next_tick: i64,

	pub res: SystemResType,

	pub event_pump: SystemEventPump,
}

impl<SystemResType> std::fmt::Debug for System<SystemResType>
where SystemResType: SystemResources {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("System")
			.field("res", &self.res)
			.field("vsync", &self.vsync)
			.field("target_framerate", &self.target_framerate)
			.field("target_framerate_delta", &self.target_framerate_delta)
			.field("next_tick", &self.next_tick)
			.finish_non_exhaustive()
	}
}

impl<SystemResType> System<SystemResType>
where SystemResType: SystemResources {
	/// Displays the current backbuffer on to the window. If a `target_framerate` is set, this will
	/// attempt to apply some timing to achieve that framerate. If V-sync is enabled, that will take
	/// priority instead. You must call this in your application's main loop to display anything on screen.
	pub fn display(&mut self) -> Result<(), SystemError> {
		self.res.display()?;

		// if a specific target framerate is desired, apply some loop timing/delay to achieve it
		// TODO: do this better. delaying when running faster like this is a poor way to do this..

		if let Some(target_framerate) = self.target_framerate {
			if self.target_framerate_delta.is_some() {
				// normal path for every other loop iteration except the first
				let delay = self.next_tick - self.ticks() as i64;
				if delay < 0 {
					// this loop iteration took too long, no need to delay
					self.next_tick -= delay;
				} else {
					// this loop iteration completed before next_tick time, delay by the remainder
					// time period so we're running at about the desired framerate
					self.delay(((delay * 1000) / self.tick_frequency() as i64) as u32);
				}
			} else {
				// this branch will occur on the first main loop iteration. we use the fact that
				// target_framerate_delta was not yet set to avoid doing any delay on the first
				// loop, just in case there was some other processing between the System struct
				// being created and the actual beginning of the first loop ...
				self.target_framerate_delta = Some((self.tick_frequency() / target_framerate as u64) as i64);
			}

			// expected time for the next display() call to happen by
			self.next_tick = (self.ticks() as i64) + self.target_framerate_delta.unwrap();
		}

		Ok(())
	}

	/// Checks for and responds to all SDL2 events waiting in the queue. Each event is passed to
	/// all [`InputDevice`]'s automatically to ensure input device state is up to date. Returns
	/// true if a [`SystemEvent::Quit`] event is encountered, in which case, the application
	/// should quit. Otherwise, returns false.
	///
	/// ```no_run
	/// use ggdt::system::*;
	///
	/// let config = DosLikeConfig::new();
	/// let mut system = SystemBuilder::new().window_title("Example").build(config).unwrap();
	///
	/// while !system.do_events().unwrap() {
	///     // ... the body of your main loop here ...
	/// }
	/// ```
	///
	/// If your application needs to react to [`SystemEvent`]s, then instead of using
	/// [`System::do_events`], you should instead manually take care of event polling in your
	/// main loop. For example:
	///
	/// ```no_run
	/// use ggdt::system::*;
	///
	/// let config = DosLikeConfig::new();
	/// let mut system = SystemBuilder::new().window_title("Example").build(config).unwrap();
	///
	/// 'mainloop: loop {
	///     system.res.update_event_state().unwrap();
	///     for event in system.event_pump.poll_iter() {
	///         system.res.handle_event(&event).unwrap();
	///         match event {
	///             SystemEvent::Quit => {
	///                 break 'mainloop
	///             },
	///             _ => {},
	///         }
	///     }
	///
	///     //  ...the rest of the body of your main loop here ...
	/// }
	/// ```
	pub fn do_events(&mut self) -> Result<bool, SystemError> {
		let mut should_quit = false;
		self.res.update_event_state()?;
		for event in self.event_pump.poll_iter() {
			self.res.handle_event(&event)?;
			if event == SystemEvent::Quit {
				should_quit = true;
			}
		}
		Ok(should_quit)
	}

	/// Perform any per-frame hardware resource and system updates. This includes important state management such
	/// as ensuring audio queues are fed to the audio device, etc. You should call this in your application's
	/// main loop.
	pub fn update(&mut self) -> Result<(), SystemError> {
		if let Err(error) = self.res.update() {
			return Err(SystemError::SystemResourcesError(error));
		}
		Ok(())
	}

	/// Returns true if the current configuration has V-sync enabled.
	#[inline]
	pub fn vsync(&self) -> bool {
		self.vsync
	}

	/// Returns the current configuration's target framerate, or `None` if not set.
	#[inline]
	pub fn target_framerate(&self) -> Option<u32> {
		self.target_framerate
	}

	pub fn ticks(&self) -> u64 {
		self.sdl_timer_subsystem.performance_counter()
	}

	pub fn tick_frequency(&self) -> u64 {
		self.sdl_timer_subsystem.performance_frequency()
	}

	/// Returns the number of milliseconds elapsed since SDL was initialized.
	pub fn millis(&self) -> u32 {
		self.sdl_timer_subsystem.ticks()
	}

	/// Delays (blocks) for about the number of milliseconds specified.
	pub fn delay(&mut self, millis: u32) {
		self.sdl_timer_subsystem.delay(millis);
	}
}
