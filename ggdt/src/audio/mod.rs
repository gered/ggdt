use std::fmt::Formatter;

use sdl2::audio::{AudioFormat, AudioFormatNum, AudioSpecDesired};
use sdl2::AudioSubsystem;
use thiserror::Error;

pub use self::buffer::*;
pub use self::device::*;
pub use self::queue::*;

pub mod buffer;
pub mod device;
pub mod queue;

/// The number of simultaneously playing audio channels supported by this library currently.
pub const NUM_CHANNELS: usize = 8;

pub const AUDIO_FREQUENCY_44KHZ: u32 = 44100;
pub const AUDIO_FREQUENCY_22KHZ: u32 = 22050;
pub const AUDIO_FREQUENCY_11KHZ: u32 = 11025;

pub const SILENCE: u8 = AudioFormatNum::SILENCE;

/// The target audio frequency supported by this library currently.
pub const TARGET_AUDIO_FREQUENCY: u32 = AUDIO_FREQUENCY_22KHZ;
/// The number of channels per audio buffer supported by this library currently.
pub const TARGET_AUDIO_CHANNELS: u8 = 1;

//////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents an "audio specification" for an audio buffer or the audio device itself. Useful
/// to know what format an audio buffer is in and to specify conversion formats, etc.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AudioSpec {
	frequency: u32,
	channels: u8,
	format: AudioFormat,
}

impl AudioSpec {
	/// Creates a new `AudioSpec` with the properties specified.
	///
	/// # Arguments
	///
	/// * `frequency`: the frequency of the audio
	/// * `channels`: the number of channels of the audio (e.g. 1 = mono, 2 = stereo, etc)
	/// * `format`: indicates the format of the bytes making up the audio buffer.
	pub fn new(frequency: u32, channels: u8, format: AudioFormat) -> Self {
		AudioSpec {
			frequency,
			channels,
			format,
		}
	}

	#[inline]
	pub fn frequency(&self) -> u32 {
		self.frequency
	}

	#[inline]
	pub fn channels(&self) -> u8 {
		self.channels
	}

	/// An SDL2 [`sdl2::audio::AudioFormat`] value indicating the audio format of the bytes making
	/// up an audio buffer.
	#[inline]
	pub fn format(&self) -> AudioFormat {
		self.format
	}
}

//////////////////////////////////////////////////////////////////////////////////////////////////

// NOTE: this is currently hardcoded such that 8-bit samples must always be used!

/// Used to implement custom/dynamic audio generation.
pub trait AudioGenerator: Send {
	/// Generates and returns the sample for the given playback position. `None` is returned if
	/// there is no sample for that position (e.g. it might be past the "end").
	fn gen_sample(&mut self, position: usize) -> Option<u8>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Error)]
pub enum AudioError {
	#[error("Failed to open audio device for playback: {0}")]
	OpenDeviceFailed(String),
}

/// Top-level abstraction over the system's audio output device. To play audio or change other
/// playback properties, you will need to lock the audio device via [`Audio::lock`] to obtain an
/// [`AudioDevice`].
pub struct Audio {
	spec: AudioSpec,
	sdl_audio_device: sdl2::audio::AudioDevice<AudioDevice>,
}

impl std::fmt::Debug for Audio {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Audio")
			.field("spec", &self.spec)
			.finish_non_exhaustive()
	}
}

impl Audio {
	/// Creates a new [`Audio`] instance, wrapping the given SDL [`sdl2::audio::AudioSubsystem`].
	/// The `desired_spec` given specifies the target audio playback format.
	///
	/// Ideally, you should not be creating an instance of this yourself and should just use the
	/// one provided by [`crate::system::System`].
	pub fn new(
		desired_spec: AudioSpecDesired,
		sdl_audio_subsystem: &AudioSubsystem,
	) -> Result<Self, AudioError> {
		let mut spec = None;
		let sdl_audio_device =
			match sdl_audio_subsystem.open_playback(None, &desired_spec, |opened_spec| {
				let our_spec = AudioSpec::new(
					opened_spec.freq as u32,
					opened_spec.channels,
					opened_spec.format,
				);
				spec = Some(our_spec);
				AudioDevice::new(our_spec)
			}) {
				Ok(audio_device) => audio_device,
				Err(error) => return Err(AudioError::OpenDeviceFailed(error)),
			};

		if let Some(spec) = spec {
			Ok(Audio {
				spec,
				sdl_audio_device,
			})
		} else {
			Err(AudioError::OpenDeviceFailed(String::from(
				"Device initialization failed to set AudioSpec",
			)))
		}
	}

	/// Returns current audio device's audio specification/format for playback. All [`AudioBuffer`]s
	/// that are to be used for playback must be converted to match this before they can be played.
	#[inline]
	pub fn spec(&self) -> &AudioSpec {
		&self.spec
	}

	/// Returns the current status of the audio device (e.g. whether it is paused, stopped, etc).
	#[inline]
	pub fn status(&self) -> sdl2::audio::AudioStatus {
		self.sdl_audio_device.status()
	}

	/// Pauses all audio playback.
	#[inline]
	pub fn pause(&mut self) {
		self.sdl_audio_device.pause()
	}

	/// Resumes all audio playback.
	#[inline]
	pub fn resume(&mut self) {
		self.sdl_audio_device.resume()
	}

	/// Locks the audio device so that new audio data can be provided or playback altered. A
	/// [`AudioDevice`] instance is returned on successful lock which can be used to interact with
	/// the actual system's audio playback. The audio device is unlocked once this instance is
	/// dropped. Ideally, you will want to keep the audio device for **as _short_ a time as
	/// possible!**
	#[inline]
	pub fn lock(&mut self) -> sdl2::audio::AudioDeviceLockGuard<AudioDevice> {
		self.sdl_audio_device.lock()
	}
}
