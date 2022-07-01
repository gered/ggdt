use sdl2::audio::{AudioFormat, AudioFormatNum, AudioSpecDesired};
use sdl2::AudioSubsystem;
use thiserror::Error;

pub use self::buffer::*;
pub use self::device::*;

pub mod buffer;
pub mod device;

pub const NUM_CHANNELS: usize = 8;

pub const AUDIO_FREQUENCY_44KHZ: u32 = 44100;
pub const AUDIO_FREQUENCY_22KHZ: u32 = 22050;
pub const AUDIO_FREQUENCY_11KHZ: u32 = 11025;

pub const SILENCE: u8 = AudioFormatNum::SILENCE;

pub const TARGET_AUDIO_FREQUENCY: u32 = AUDIO_FREQUENCY_22KHZ;
pub const TARGET_AUDIO_CHANNELS: u8 = 1;

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AudioSpec {
    frequency: u32,
    channels: u8,
    format: AudioFormat,
}

impl AudioSpec {
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

    #[inline]
    pub fn format(&self) -> AudioFormat {
        self.format
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

pub trait AudioGenerator: Send {
    fn gen_sample(&mut self, position: usize) -> Option<u8>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("Failed to open audio device for playback: {0}")]
    OpenDeviceFailed(String),
}

pub struct Audio {
    spec: AudioSpec,
    sdl_audio_device: sdl2::audio::AudioDevice<AudioDevice>,
}

impl Audio {
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

    #[inline]
    pub fn spec(&self) -> &AudioSpec {
        &self.spec
    }

    #[inline]
    pub fn status(&self) -> sdl2::audio::AudioStatus {
        self.sdl_audio_device.status()
    }

    #[inline]
    pub fn pause(&mut self) {
        self.sdl_audio_device.pause()
    }

    #[inline]
    pub fn resume(&mut self) {
        self.sdl_audio_device.resume()
    }

    #[inline]
    pub fn lock(&mut self) -> sdl2::audio::AudioDeviceLockGuard<AudioDevice> {
        self.sdl_audio_device.lock()
    }
}
