use std::ops::{Index, IndexMut};

use sdl2::audio::AudioCallback;
use thiserror::Error;

use crate::audio::*;

/// Represents a "channel" of audio playback that will be mixed together with all of the other
/// actively playing audio channels to get the final audio playback.
pub struct AudioChannel {
    /// Whether the channel is currently playing or not.
    pub playing: bool,
    /// Whether this channel is playing on a loop or not. If not, once the end of the [`data`]
    /// buffer is reached, or the [`AudioGenerator::gen_sample`] method returns `None`, playback
    /// on this channel will automatically stop and [`playing`] will be changed to `false`.
    pub loops: bool,
    /// The audio data buffer (samples) that this channel will play from, **only** if [`generator`]
    /// is `None`.
    pub data: Vec<u8>,
    /// An [`AudioGenerator`] instance that will be used to dynamically generate audio data to play
    /// on this channel _instead of_ playing from [`data`]. Set this to `None` to play from audio
    /// data in [`data`] instead.
    pub generator: Option<Box<dyn AudioGenerator>>,
    /// The volume level to play this channel at. 1.0 is "normal", 0.0 is completely silent.
    pub volume: f32,
    /// The current playback position (index). 0 is the start of playback. The end position is
    /// either the (current) size of the [`data`] buffer or dependant on the implementation of this
    /// channel's current [`generator`] if not `None`.
    pub position: usize,
}

impl AudioChannel {
    pub fn new() -> Self {
        AudioChannel {
            playing: false,
            loops: false,
            volume: 1.0,
            position: 0,
            generator: None,
            data: Vec::new(),
        }
    }

    /// Returns the audio sample for the given position, or `None` if that position is invalid.
    #[inline]
    fn data_at(&mut self, position: usize) -> Option<u8> {
        if let Some(generator) = &mut self.generator {
            generator.gen_sample(position)
        } else {
            self.data.get(self.position).copied()
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
        if let Some(sample) = self.data_at(self.position) {
            self.position += 1;
            Some(sample as i16 - 128)
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
        }

        if let Some(sample) = self.next_sample() {
            Some((sample as f32 * self.volume) as i16)
        } else {
            if self.loops {
                self.position = 0;
                None
            } else {
                self.stop();
                None
            }
        }
    }

    /// Resets the audio channel to a "blank slate", clearing the audio buffer, setting no current
    /// audio generator, and turning playback off.
    #[inline]
    pub fn reset(&mut self) {
        self.data.clear();
        self.generator = None;
        self.position = 0;
        self.playing = false;
    }

    /// Copies the data from the given audio buffer into this channel's buffer (clearing it first,
    /// and extending the size of the buffer if necessary) and then begins playback from position 0.
    /// This also sets the associated [`generator`] to `None`.
    #[inline]
    pub fn play_buffer(&mut self, buffer: &AudioBuffer, loops: bool) {
        self.data.clear();
        self.data.extend(&buffer.data);
        self.generator = None;
        self.position = 0;
        self.playing = true;
        self.loops = loops;
    }

    /// Begins playback on this channel from the given [`AudioGenerator`] instance from position 0.
    /// This also clears the existing audio buffer contents.
    #[inline]
    pub fn play_generator(&mut self, generator: Box<dyn AudioGenerator>, loops: bool) {
        self.data.clear();
        self.generator = Some(generator);
        self.position = 0;
        self.playing = true;
        self.loops = loops;
    }

    /// Returns true if this channel has something that can be played back currently.
    #[inline]
    pub fn is_playable(&self) -> bool {
        !self.data.is_empty() || self.generator.is_some()
    }

    /// Begins playback on this channel, only if playback is currently possible with its current
    /// state (if it has some sample data in the buffer or if an [`AudioGenerator`] is set).
    /// Resets the position to 0 if playback is started and returns true, otherwise returns false.
    #[inline]
    pub fn play(&mut self, loops: bool) -> bool {
        if self.is_playable() {
            self.position = 0;
            self.playing = true;
            self.loops = loops;
            true
        } else {
            false
        }
    }

    /// Stops playback on this channel.
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

    #[error("The channel index {0} is out of range")]
    ChannelIndexOutOfRange(usize),
}

/// Represents the audio device and performs mixing of all of the [`AudioChannel`]s that are
/// currently playing. You should not be creating this manually, but obtaining it as needed via
/// [`Audio::lock`].
pub struct AudioDevice {
    spec: AudioSpec,
    channels: Vec<AudioChannel>,
    pub volume: f32,
}

/// SDL audio callback implementation which performs audio mixing, generating the final sample data
/// that will be played by the system's audio device.
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
            sample = ((sample as f32) * self.volume) as i16;
            *dest = (sample.clamp(-128, 127) + 128) as u8;
        }
    }
}

impl AudioDevice {
    /// Creates a new [`AudioDevice`] instance, using the given spec as its playback format.
    pub fn new(spec: AudioSpec) -> Self {
        let mut channels = Vec::new();
        for _ in 0..NUM_CHANNELS {
            channels.push(AudioChannel::new());
        }
        AudioDevice {
            spec,
            channels,
            volume: 1.0,
        }
    }

    /// Returns the spec that this device is currently set to play. All audio to be played via
    /// this device must be pre-converted to match this spec!
    #[inline]
    pub fn spec(&self) -> &AudioSpec {
        &self.spec
    }

    /// Returns true if any of the audio channels are currently playing, false otherwise.
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.channels.iter().any(|channel| channel.playing)
    }

    /// Stops the specified channel's playback, or does nothing if that channel was not currently
    /// playing. This does not affect the channel's other state (data buffer, etc).
    pub fn stop_channel(&mut self, channel_index: usize) -> Result<(), AudioDeviceError> {
        if channel_index >= NUM_CHANNELS {
            Err(AudioDeviceError::ChannelIndexOutOfRange(channel_index))
        } else {
            self.channels[channel_index].stop();
            Ok(())
        }
    }

    /// Stops playback of all channels.
    pub fn stop_all(&mut self) {
        for channel in self.channels.iter_mut() {
            channel.stop();
        }
    }

    /// Tries to play the given [`AudioBuffer`] on the first channel found that is not already
    /// playing. If a free channel is found, playback will be started by copying the buffer's
    /// contents to the channel. The index of the channel is returned. If playback was not started
    /// because no channel is free currently, then `None` is returned.
    pub fn play_buffer(
        &mut self,
        buffer: &AudioBuffer,
        loops: bool,
    ) -> Result<Option<usize>, AudioDeviceError> {
        if *buffer.spec() != self.spec {
            Err(AudioDeviceError::AudioSpecMismatch)
        } else {
            if let Some((index, channel)) = self.stopped_channels_iter_mut().enumerate().next() {
                channel.play_buffer(buffer, loops);
                Ok(Some(index))
            } else {
                Ok(None)
            }
        }
    }

    /// Plays the given [`AudioBuffer`] on the specified channel. Whatever that channel was playing
    /// will be interrupted and replaced with a copy of the given buffer's data.
    pub fn play_buffer_on_channel(
        &mut self,
        channel_index: usize,
        buffer: &AudioBuffer,
        loops: bool,
    ) -> Result<(), AudioDeviceError> {
        if *buffer.spec() != self.spec {
            Err(AudioDeviceError::AudioSpecMismatch)
        } else if channel_index >= NUM_CHANNELS {
            Err(AudioDeviceError::ChannelIndexOutOfRange(channel_index))
        } else {
            self.channels[channel_index].play_buffer(buffer, loops);
            Ok(())
        }
    }

    /// Tries to play the given [`AudioGenerator`] on the first channel found that is not already
    /// playing. If a free channel is found, playback will be started and the index of the channel
    /// will be returned. If playback was not started because no channel is free currently, then
    /// `None` is returned.
    pub fn play_generator(
        &mut self,
        generator: Box<dyn AudioGenerator>,
        loops: bool,
    ) -> Result<Option<usize>, AudioDeviceError> {
        if let Some((index, channel)) = self.stopped_channels_iter_mut().enumerate().next() {
            channel.play_generator(generator, loops);
            Ok(Some(index))
        } else {
            Ok(None)
        }
    }

    /// Plays the given [`AudioGenerator`] on the specified channel. Whatever that channel was
    /// playing will be interrupted and replaced.
    pub fn play_generator_on_channel(
        &mut self,
        channel_index: usize,
        generator: Box<dyn AudioGenerator>,
        loops: bool,
    ) -> Result<(), AudioDeviceError> {
        if channel_index >= NUM_CHANNELS {
            Err(AudioDeviceError::ChannelIndexOutOfRange(channel_index))
        } else {
            self.channels[channel_index].play_generator(generator, loops);
            Ok(())
        }
    }

    /// Returns an iterator of any [`AudioChannel`]s that are currently playing.
    #[inline]
    pub fn playing_channels_iter(&mut self) -> impl Iterator<Item = &AudioChannel> {
        self.channels.iter().filter(|channel| channel.playing)
    }

    /// Returns an iterator of mutable [`AudioChannel`]s that are currently playing.
    #[inline]
    pub fn playing_channels_iter_mut(&mut self) -> impl Iterator<Item = &mut AudioChannel> {
        self.channels.iter_mut().filter(|channel| channel.playing)
    }

    /// Returns an iterator of [`AudioChannel`]s that are not currently playing.
    #[inline]
    pub fn stopped_channels_iter(&mut self) -> impl Iterator<Item = &AudioChannel> {
        self.channels.iter().filter(|channel| !channel.playing)
    }

    /// Returns an iterator of mutable [`AudioChannel`]s that are not currently playing.
    #[inline]
    pub fn stopped_channels_iter_mut(&mut self) -> impl Iterator<Item = &mut AudioChannel> {
        self.channels.iter_mut().filter(|channel| !channel.playing)
    }

    /// Returns an iterator of all [`AudioChannel`]s.
    #[inline]
    pub fn channels_iter(&mut self) -> impl Iterator<Item = &AudioChannel> {
        self.channels.iter()
    }

    /// Returns an iterator of all [`AudioChannel`]s as mutable references.
    #[inline]
    pub fn channels_iter_mut(&mut self) -> impl Iterator<Item = &mut AudioChannel> {
        self.channels.iter_mut()
    }

    /// Returns a reference to the specified [`AudioChannel`] or `None` if the index specified
    /// is not valid.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&AudioChannel> {
        self.channels.get(index)
    }

    /// Returns a mutable reference to the specified [`AudioChannel`] or `None` if the index
    /// specified is not valid.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut AudioChannel> {
        self.channels.get_mut(index)
    }
}

impl Index<usize> for AudioDevice {
    type Output = AudioChannel;

    /// Returns a reference to the specified [`AudioChannel`] or panics if the index specified is
    /// not valid.
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl IndexMut<usize> for AudioDevice {
    /// Returns a mutable reference to the specified [`AudioChannel`] or panics if the index
    /// specified is not valid.
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
