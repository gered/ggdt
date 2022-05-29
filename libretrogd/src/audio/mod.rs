use std::ops::{Index, IndexMut};

use sdl2::audio::{AudioCallback, AudioFormat, AudioSpecDesired};
use sdl2::AudioSubsystem;
use thiserror::Error;

pub use self::wav::*;

pub mod wav;

pub const NUM_CHANNELS: usize = 8;

pub const AUDIO_FREQUENCY_44KHZ: u32 = 44100;
pub const AUDIO_FREQUENCY_22KHZ: u32 = 22050;
pub const AUDIO_FREQUENCY_11KHZ: u32 = 11025;

pub const SILENCE: u8 = sdl2::audio::AudioFormatNum::SILENCE;

pub const TARGET_AUDIO_SPEC: AudioSpec = AudioSpec {
    frequency: AUDIO_FREQUENCY_22KHZ,
    channels: 1,
    format: AudioFormat::U8,
};

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

#[derive(Debug, Clone)]
pub struct AudioChannel {
    pub playing: bool,
    pub loops: bool,
    pub data: Vec<u8>,
    pub volume: f32,
    pub position: usize,
}

impl AudioChannel {
    pub fn new() -> Self {
        AudioChannel {
            playing: false,
            loops: false,
            volume: 1.0,
            position: 0,
            data: Vec::new(),
        }
    }

    /// Returns the next sample from this channel's buffer. If this channel's buffer is done
    /// playing or there is no buffer data at all, `None` is returned. If the next sample was
    /// successfully loaded from the buffer, the channel's current position is advanced by 1.
    ///
    /// The returned sample will be a byte value, but in an `i16` with the buffer's original `u8`
    /// value centered around 0 (meaning the returned sample will be within the range -128 to 127
    /// instead of 0 to 255).
    #[inline]
    fn next_sample(&mut self) -> Option<i16> {
        if let Some(sample) = self.data.get(self.position) {
            self.position += 1;
            Some((*sample as i16) - 128)
        } else {
            None
        }
    }

    /// Samples the channel's current audio buffer, advancing the position within that buffer by 1.
    /// The channel will automatically stop playing when the end of the buffer is reached and if
    /// the channel is not set to loop. `None` is returned if no data can be read from the buffer
    /// for any reason, or if the channel is not currently playing.
    ///
    /// The returned sample will be a byte value, but in an `i16` with the buffer's original `u8`
    /// value centered around 0 (meaning the returned sample will be within the range -128 to 127
    /// instead of 0 to 255).
    #[inline]
    pub fn sample(&mut self) -> Option<i16> {
        if !self.playing {
            return None;
        } else if self.position >= self.data.len() {
            if self.loops {
                self.position = 0;
            } else {
                self.stop();
                return None;
            }
        }

        if let Some(raw_sample) = self.next_sample() {
            Some((raw_sample as f32 * self.volume) as i16)
        } else {
            None
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.data.clear();
        self.position = 0;
        self.playing = false;
    }

    #[inline]
    pub fn play_buffer(&mut self, buffer: &AudioBuffer, loops: bool) {
        self.data.clear();
        self.data.extend(&buffer.data);
        self.position = 0;
        self.playing = true;
        self.loops = loops;
    }

    #[inline]
    pub fn play(&mut self, loops: bool) {
        if !self.data.is_empty() {
            self.position = 0;
            self.playing = true;
            self.loops = loops;
        }
    }

    #[inline]
    pub fn stop(&mut self) {
        self.playing = false;
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Error)]
pub enum AudioDeviceError {
    #[error("That buffer's AudioSpec does not match the device's AudioSpec")]
    AudioSpecMismatch,
}

pub struct AudioDevice {
    spec: AudioSpec,
    channels: Vec<AudioChannel>,
}

impl AudioCallback for AudioDevice {
    type Channel = u8;

    fn callback(&mut self, out: &mut [u8]) {
        for dest in out.iter_mut() {
            let mut sample: i16 = 0;
            for channel in self.channels.iter_mut() {
                if let Some(this_sample) = channel.sample() {
                    sample += this_sample;
                }
            }
            *dest = (sample.clamp(-128, 127) + 128) as u8;
        }
    }
}

impl AudioDevice {
    pub fn new(spec: AudioSpec) -> Self {
        let mut channels = Vec::new();
        for _ in 0..NUM_CHANNELS {
            channels.push(AudioChannel::new());
        }
        AudioDevice { spec, channels }
    }

    #[inline]
    pub fn spec(&self) -> &AudioSpec {
        &self.spec
    }

    #[inline]
    pub fn is_playing(&self) -> bool {
        self.channels.iter().any(|channel| channel.playing)
    }

    pub fn stop_all(&mut self) {
        for channel in self.channels.iter_mut() {
            channel.stop();
        }
    }

    pub fn play_buffer(
        &mut self,
        buffer: &AudioBuffer,
        loops: bool,
    ) -> Result<Option<&mut AudioChannel>, AudioDeviceError> {
        if buffer.spec != self.spec {
            Err(AudioDeviceError::AudioSpecMismatch)
        } else {
            if let Some(channel) = self.stopped_channels_iter_mut().next() {
                channel.play_buffer(buffer, loops);
                Ok(Some(channel))
            } else {
                Ok(None)
            }
        }
    }

    #[inline]
    pub fn playing_channels_iter(&mut self) -> impl Iterator<Item = &AudioChannel> {
        self.channels.iter().filter(|channel| channel.playing)
    }

    #[inline]
    pub fn playing_channels_iter_mut(&mut self) -> impl Iterator<Item = &mut AudioChannel> {
        self.channels.iter_mut().filter(|channel| channel.playing)
    }

    #[inline]
    pub fn stopped_channels_iter(&mut self) -> impl Iterator<Item = &AudioChannel> {
        self.channels.iter().filter(|channel| !channel.playing)
    }

    #[inline]
    pub fn stopped_channels_iter_mut(&mut self) -> impl Iterator<Item = &mut AudioChannel> {
        self.channels.iter_mut().filter(|channel| !channel.playing)
    }

    #[inline]
    pub fn channels_iter(&mut self) -> impl Iterator<Item = &AudioChannel> {
        self.channels.iter()
    }

    #[inline]
    pub fn channels_iter_mut(&mut self) -> impl Iterator<Item = &mut AudioChannel> {
        self.channels.iter_mut()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&AudioChannel> {
        self.channels.get(index)
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut AudioChannel> {
        self.channels.get_mut(index)
    }
}

impl Index<usize> for AudioDevice {
    type Output = AudioChannel;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl IndexMut<usize> for AudioDevice {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
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

//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum AudioBufferError {
    #[error("Error during format conversion: {0}")]
    ConversionError(String),
}

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    spec: AudioSpec,
    pub data: Vec<u8>,
}

impl AudioBuffer {
    pub fn new(spec: AudioSpec) -> Self {
        AudioBuffer {
            spec,
            data: Vec::new(),
        }
    }

    #[inline]
    pub fn spec(&self) -> &AudioSpec {
        &self.spec
    }

    pub fn convert(self, to_spec: &AudioSpec) -> Result<Self, AudioBufferError> {
        if self.spec == *to_spec {
            Ok(self)
        } else {
            let converter = sdl2::audio::AudioCVT::new(
                self.spec.format(),
                self.spec.channels(),
                self.spec.frequency() as i32,
                to_spec.format(),
                to_spec.channels(),
                to_spec.frequency() as i32,
            );
            match converter {
                Ok(converter) => {
                    let mut result = AudioBuffer::new(*to_spec);
                    result.data = converter.convert(self.data);
                    Ok(result)
                }
                Err(string) => Err(AudioBufferError::ConversionError(string)),
            }
        }
    }
}
