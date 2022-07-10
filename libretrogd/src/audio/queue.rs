use std::collections::VecDeque;
use std::rc::Rc;

use crate::audio::*;

pub enum AudioCommand {
    StopChannel(usize),
    StopAllChannels,
    PlayBuffer {
        buffer: AudioBuffer,
        loops: bool,
    },
    PlayRcBuffer {
        buffer: Rc<AudioBuffer>,
        loops: bool,
    },
    PlayBufferOnChannel {
        channel: usize,
        buffer: AudioBuffer,
        loops: bool,
    },
    PlayRcBufferOnChannel {
        channel: usize,
        buffer: Rc<AudioBuffer>,
        loops: bool,
    },
    PlayGenerator {
        generator: Box<dyn AudioGenerator>,
        loops: bool,
    },
    PlayGeneratorOnChannel {
        channel: usize,
        generator: Box<dyn AudioGenerator>,
        loops: bool,
    },
}

/// A convenience abstraction that can be used to queue up commands to be issued to an
/// [`AudioDevice`]. This can be more useful to utilize in applications versus needing to directly
/// lock the [`AudioDevice`] and then determine what your application needs to do and issue those
/// commands that time. [`AudioQueue`] lets you play/stop audio in more of a "fire-and-forget"
/// manner.
pub struct AudioQueue {
    spec: AudioSpec,
    commands: VecDeque<AudioCommand>,
}

impl AudioQueue {
    /// Creates and returns a new [`AudioQueue`] instance.
    pub fn new(audio: &Audio) -> Self {
        AudioQueue {
            spec: audio.spec,
            commands: VecDeque::new(),
        }
    }

    /// Returns the spec that this queue is currently set to play. All audio to be played via
    /// this queue must be pre-converted to match this spec! This spec is a copy of the one that
    /// was obtained from the [`Audio`] instance used to create this [`AudioQueue`].
    #[inline]
    pub fn spec(&self) -> &AudioSpec {
        &self.spec
    }

    /// Queues a stop command for the given channel.
    pub fn stop_channel(&mut self, channel_index: usize) -> Result<(), AudioDeviceError> {
        if channel_index >= NUM_CHANNELS {
            Err(AudioDeviceError::ChannelIndexOutOfRange(channel_index))
        } else {
            self.commands.push_back(AudioCommand::StopChannel(channel_index));
            Ok(())
        }
    }

    /// Queues a command that will stop playback on all channels.
    pub fn stop_all(&mut self) {
        self.commands.push_back(AudioCommand::StopAllChannels);
    }

    /// Queues a command to play a copy of the given [`AudioBuffer`]'s data. The buffer will be
    /// played on the first channel found that is not already playing. If all channels are already
    /// playing, then nothing will be done.
    pub fn play_buffer(
        &mut self,
        buffer: &AudioBuffer,
        loops: bool,
    ) -> Result<(), AudioDeviceError> {
        if *buffer.spec() != self.spec {
            Err(AudioDeviceError::AudioSpecMismatch)
        } else {
            self.commands.push_back(AudioCommand::PlayBuffer {
                buffer: buffer.clone(),
                loops,
            });
            Ok(())
        }
    }

    /// Queues a command to play the given [`AudioBuffer`]'s data. The buffer will be played on
    /// the first channel found that is not already playing. If all channels are already playing,
    /// then nothing will be done. This method is more performant than [`AudioQueue::play_buffer`],
    /// as that method will always immediately copy the given buffer to create the queued command.
    pub fn play_buffer_rc(
        &mut self,
        buffer: Rc<AudioBuffer>,
        loops: bool,
    ) -> Result<(), AudioDeviceError> {
        if *buffer.spec() != self.spec {
            Err(AudioDeviceError::AudioSpecMismatch)
        } else {
            self.commands.push_back(AudioCommand::PlayRcBuffer {
                buffer,
                loops,
            });
            Ok(())
        }
    }

    /// Queues a command to play a copy of the given [`AudioBuffer`]'s data on the channel
    /// specified. Whatever that channel was playing will be interrupted to begin playing this
    /// buffer.
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
            self.commands.push_back(AudioCommand::PlayBufferOnChannel {
                channel: channel_index,
                buffer: buffer.clone(),
                loops,
            });
            Ok(())
        }
    }

    /// Queues a command to play the given [`AudioBuffer`]'s data on the channel specified. Whatever
    /// that channel was playing will be interrupted to begin playing this buffer. This method is
    /// more performant than [`AudioQueue::play_buffer_on_channel`], as that method will always
    /// immediately copy the given buffer to create the queued command.
    pub fn play_buffer_rc_on_channel(
        &mut self,
        channel_index: usize,
        buffer: Rc<AudioBuffer>,
        loops: bool,
    ) -> Result<(), AudioDeviceError> {
        if *buffer.spec() != self.spec {
            Err(AudioDeviceError::AudioSpecMismatch)
        } else if channel_index >= NUM_CHANNELS {
            Err(AudioDeviceError::ChannelIndexOutOfRange(channel_index))
        } else {
            self.commands.push_back(AudioCommand::PlayRcBufferOnChannel {
                channel: channel_index,
                buffer,
                loops,
            });
            Ok(())
        }
    }

    /// Queues a command to play the given [`AudioGenerator`] on the first channel found that is
    /// not already playing. If all channels are already playing, then nothing will be done.
    pub fn play_generator(
        &mut self,
        generator: Box<dyn AudioGenerator>,
        loops: bool,
    ) -> Result<(), AudioDeviceError> {
        self.commands.push_back(AudioCommand::PlayGenerator { generator, loops });
        Ok(())
    }

    /// Queues a command to play the given [`AudioGenerator`] on the channel specified. Whatever
    /// that channel was playing will be interrupted to begin playing this generator.
    pub fn play_generator_on_channel(
        &mut self,
        channel_index: usize,
        generator: Box<dyn AudioGenerator>,
        loops: bool,
    ) -> Result<(), AudioDeviceError> {
        self.commands.push_back(AudioCommand::PlayGeneratorOnChannel {
            channel: channel_index,
            generator,
            loops,
        });
        Ok(())
    }

    /// Flushes the queued commands, issuing them in the same order they were created, to the
    /// given [`AudioDevice`].
    pub fn apply_to_device(&mut self, device: &mut AudioDevice) -> Result<(), AudioDeviceError> {
        loop {
            if let Some(command) = self.commands.pop_front() {
                use AudioCommand::*;
                match command {
                    StopChannel(channel_index) => {
                        device.stop_channel(channel_index)?;
                    },
                    StopAllChannels => {
                        device.stop_all();
                    },
                    PlayBuffer { buffer, loops } => {
                        device.play_buffer(&buffer, loops)?;
                    }
                    PlayRcBuffer { buffer, loops } => {
                        device.play_buffer(&buffer, loops)?;
                    },
                    PlayBufferOnChannel { channel, buffer, loops } => {
                        device.play_buffer_on_channel(channel, &buffer, loops)?;
                    }
                    PlayRcBufferOnChannel { channel, buffer, loops } => {
                        device.play_buffer_on_channel(channel, &buffer, loops)?;
                    },
                    PlayGenerator { generator, loops } => {
                        device.play_generator(generator, loops)?;
                    },
                    PlayGeneratorOnChannel { channel, generator, loops } => {
                        device.play_generator_on_channel(channel, generator, loops)?;
                    },
                }
            } else {
                return Ok(())
            }
        }
    }

    /// Flushes the queued commands, issuing them in the same order they were created, to the
    /// given [`Audio`] instance. This method automatically handles obtaining a locked
    /// [`AudioDevice`], and so is a bit more convenient to use if you don't actually need to
    /// interact with the [`AudioDevice`] itself in your code.
    pub fn apply(&mut self, audio: &mut Audio) -> Result<(), AudioDeviceError> {
        let mut device = audio.lock();
        self.apply_to_device(&mut device)
    }
}