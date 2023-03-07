use thiserror::Error;

use crate::audio::{AudioDeviceError, AudioError};
use crate::system::SystemEvent;

pub mod dos_like;

#[derive(Error, Debug)]
pub enum SystemResourcesError {
	#[error("SystemResources SDL error: {0}")]
	SDLError(String),

	#[error("System audioerror: {0}")]
	AudioError(#[from] AudioError),

	#[error("System audio device error: {0}")]
	AudioDeviceError(#[from] AudioDeviceError),
}

pub trait SystemResourcesConfig {
	type SystemResourcesType: SystemResources;

	fn build(
		self,
		video_subsystem: &sdl2::VideoSubsystem,
		audio_subsystem: &sdl2::AudioSubsystem,
		window: sdl2::video::Window,
	) -> Result<Self::SystemResourcesType, SystemResourcesError>;
}

pub trait SystemResources : std::fmt::Debug {
	fn update(&mut self) -> Result<(), SystemResourcesError>;
	fn display(&mut self) -> Result<(), SystemResourcesError>;
	fn update_event_state(&mut self) -> Result<(), SystemResourcesError>;
	fn handle_event(&mut self, event: &SystemEvent) -> Result<bool, SystemResourcesError>;

	fn width(&self) -> u32;
	fn height(&self) -> u32;
	fn vsync(&self) -> bool;
	fn target_framerate(&self) -> Option<u32>;
}
