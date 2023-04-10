use thiserror::Error;

use crate::audio::device::AudioDeviceError;
use crate::audio::AudioError;
use crate::graphics::Pixel;
use crate::system::event::SystemEvent;
use crate::system::framebuffer::SdlFramebufferError;

pub mod dos_like;
pub mod standard;

#[derive(Error, Debug)]
pub enum SystemResourcesError {
	#[error("SystemResources SDL error: {0}")]
	SDLError(String),

	#[error("SdlFramebufferError: {0}")]
	SdlFramebufferError(#[from] SdlFramebufferError),

	#[error("System audioerror: {0}")]
	AudioError(#[from] AudioError),

	#[error("System audio device error: {0}")]
	AudioDeviceError(#[from] AudioDeviceError),
}

/// Trait used to implement a configuration / builder for an associated [`SystemResources`] implementation.
pub trait SystemResourcesConfig {
	type SystemResourcesType: SystemResources;

	/// Builds and returns an instance of the associated [`SystemResources`] type from the current configuration,
	/// using the provided SDL resources. This is not intended to be called directly by your applications, but is
	/// instead called internally by [`SystemBuilder::build`].
	fn build(
		self,
		video_subsystem: &sdl2::VideoSubsystem,
		audio_subsystem: &sdl2::AudioSubsystem,
		window: sdl2::video::Window,
	) -> Result<Self::SystemResourcesType, SystemResourcesError>;
}

/// Trait used to implement structs which get used by [`System`] to provide access to hardware resources like
/// audio, video and input devices.
pub trait SystemResources: std::fmt::Debug {
	type PixelType: Pixel;

	/// Perform any per-frame hardware resource updates. You should prefer to call [`System::update`] instead of this
	/// in your main loop.
	fn update(&mut self) -> Result<(), SystemResourcesError>;

	/// Displays the current backbuffer on the application window. You should prefer to call [`System::display`]
	/// instead of this in your main loop.
	fn display(&mut self) -> Result<(), SystemResourcesError>;

	/// Takes care of per-frame state management/housekeeping that should preceed `SystemEvent` processing. In other
	/// words, if you're manually calling [`SystemResources::handle_event`] in your main loop, you should call this
	/// first. If you are using [`System::do_events`] then you should not call this at all.
	fn update_event_state(&mut self) -> Result<(), SystemResourcesError>;

	/// Processes the data from the given [`SystemEvent`]. Returns true if the processing actually
	/// recognized the passed event and handled it, or false if the event was ignored. If you are using
	/// [`System::do_events`] then you should not call this at all.
	fn handle_event(&mut self, event: &SystemEvent) -> Result<bool, SystemResourcesError>;

	/// Returns the width of the current video backbuffer, in pixels.
	fn width(&self) -> u32;

	/// Returns the height of the current video backbuffer, in pixels.
	fn height(&self) -> u32;
}
