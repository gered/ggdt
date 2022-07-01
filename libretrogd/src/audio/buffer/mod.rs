use crate::audio::*;

pub use self::wav::*;

pub mod wav;

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
