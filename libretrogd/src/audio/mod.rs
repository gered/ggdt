use std::ops::{Index, IndexMut};

use sdl2::audio::AudioCallback;
use thiserror::Error;

pub use self::wav::*;

pub mod wav;

pub const NUM_CHANNELS: usize = 8;

pub const AUDIO_FREQUENCY_44KHZ: u32 = 44100;
pub const AUDIO_FREQUENCY_22KHZ: u32 = 22050;
pub const AUDIO_FREQUENCY_11KHZ: u32 = 11025;

pub const SILENCE: u8 = sdl2::audio::AudioFormatNum::SILENCE;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AudioSpec {
    pub frequency: u32,
    pub channels: u8,
}

impl AudioSpec {
    #[inline]
    pub fn frequency(&self) -> u32 {
        self.frequency
    }

    #[inline]
    pub fn channels(&self) -> u8 {
        self.channels
    }
}

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

    #[inline]
    fn next_sample(&mut self) -> i16 {
        if let Some(sample) = self.data.get(self.position) {
            self.position += 1;
            (*sample as i16) - 128
        } else {
            0
        }
    }

    #[inline]
    pub fn get_audio_frame(&mut self) -> i16 {
        if !self.playing {
            return 0;
        } else if self.position >= self.data.len() {
            if self.loops {
                self.position = 0;
            } else {
                self.stop();
                return 0;
            }
        }

        let raw_sample = self.next_sample();
        (raw_sample as f32 * self.volume) as i16
    }

    pub fn reset(&mut self) {
        self.data.clear();
        self.position = 0;
        self.playing = false;
    }

    pub fn play_buffer(&mut self, buffer: &AudioBuffer, loops: bool) {
        self.data.clear();
        self.data.extend(&buffer.data);
        self.position = 0;
        self.playing = true;
        self.loops = loops;
    }

    pub fn play(&mut self, loops: bool) {
        if !self.data.is_empty() {
            self.position = 0;
            self.playing = true;
            self.loops = loops;
        }
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }
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
                sample += channel.get_audio_frame();
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

    pub fn play_buffer(&mut self, buffer: &AudioBuffer, loops: bool) -> Option<&mut AudioChannel> {
        if let Some(channel) = self.stopped_channels_iter_mut().next() {
            channel.play_buffer(buffer, loops);
            Some(channel)
        } else {
            None
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
    pub fn new(frequency: u32, channels: u8) -> Self {
        AudioBuffer {
            spec: AudioSpec {
                frequency,
                channels,
            },
            data: Vec::new(),
        }
    }

    #[inline]
    pub fn spec(&self) -> &AudioSpec {
        &self.spec
    }

    pub fn convert(self, frequency: u32, channels: u8) -> Result<Self, AudioBufferError> {
        if self.spec.frequency == frequency && self.spec.channels == channels {
            Ok(self)
        } else {
            use sdl2::audio::AudioFormat;
            let converter = sdl2::audio::AudioCVT::new(
                AudioFormat::U8,
                self.spec.channels,
                self.spec.frequency as i32,
                AudioFormat::U8,
                channels,
                frequency as i32,
            );
            match converter {
                Ok(converter) => {
                    let mut result = AudioBuffer::new(frequency, channels);
                    result.data = converter.convert(self.data);
                    Ok(result)
                }
                Err(string) => Err(AudioBufferError::ConversionError(string)),
            }
        }
    }
}
