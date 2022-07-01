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

pub struct AudioQueue {
    spec: AudioSpec,
    commands: VecDeque<AudioCommand>,
}

impl AudioQueue {
    pub fn from(audio: &Audio) -> Self {
        AudioQueue {
            spec: audio.spec,
            commands: VecDeque::new(),
        }
    }

    #[inline]
    pub fn spec(&self) -> &AudioSpec {
        &self.spec
    }

    pub fn stop_channel(&mut self, channel_index: usize) -> Result<(), AudioDeviceError> {
        if channel_index >= NUM_CHANNELS {
            Err(AudioDeviceError::ChannelIndexOutOfRange(channel_index))
        } else {
            self.commands.push_back(AudioCommand::StopChannel(channel_index));
            Ok(())
        }
    }

    pub fn stop_all(&mut self) {
        self.commands.push_back(AudioCommand::StopAllChannels);
    }

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

    pub fn play_generator(
        &mut self,
        generator: Box<dyn AudioGenerator>,
        loops: bool,
    ) -> Result<(), AudioDeviceError> {
        self.commands.push_back(AudioCommand::PlayGenerator { generator, loops });
        Ok(())
    }

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

    pub fn apply(&mut self, device: &mut AudioDevice) -> Result<(), AudioDeviceError> {
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
                        device.play_generator(generator, loops);
                    },
                    PlayGeneratorOnChannel { channel, generator, loops } => {
                        device.play_generator_on_channel(channel, generator, loops);
                    },
                }
            } else {
                return Ok(())
            }
        }
    }
}