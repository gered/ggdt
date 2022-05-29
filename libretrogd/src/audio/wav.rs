use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use sdl2::audio::AudioFormat;
use thiserror::Error;

use crate::audio::{AudioBuffer, AudioSpec};
use crate::utils::io::StreamSize;

#[derive(Error, Debug)]
pub enum WavError {
    #[error("Bad or unsupported WAV file: {0}")]
    BadFile(String),

    #[error("WAV I/O error")]
    IOError(#[from] std::io::Error),
}

#[derive(Debug, Copy, Clone)]
struct ChunkId {
    id: [u8; 4],
}

impl ChunkId {
    pub fn read<T: Read>(reader: &mut T) -> Result<Self, WavError> {
        let mut id = [0u8; 4];
        reader.read_exact(&mut id)?;
        Ok(ChunkId { id })
    }

    #[allow(dead_code)]
    pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), WavError> {
        writer.write_all(&self.id)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
struct SubChunkHeader {
    chunk_id: ChunkId,
    size: u32,
}

impl SubChunkHeader {
    pub fn read<T: ReadBytesExt>(reader: &mut T) -> Result<Self, WavError> {
        let chunk_id = ChunkId::read(reader)?;
        let size = reader.read_u32::<LittleEndian>()?;
        Ok(SubChunkHeader { chunk_id, size })
    }

    #[allow(dead_code)]
    pub fn write<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), WavError> {
        self.chunk_id.write(writer)?;
        writer.write_u32::<LittleEndian>(self.size)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
struct WavHeader {
    file_chunk: SubChunkHeader,
    file_container_id: ChunkId,
}

impl WavHeader {
    pub fn read<T: ReadBytesExt>(reader: &mut T) -> Result<Self, WavError> {
        let file_chunk = SubChunkHeader::read(reader)?;
        let file_container_id = ChunkId::read(reader)?;
        Ok(WavHeader {
            file_chunk,
            file_container_id,
        })
    }

    #[allow(dead_code)]
    pub fn write<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), WavError> {
        self.file_chunk.write(writer)?;
        self.file_container_id.write(writer)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FormatChunk {
    compression_code: u16,
    channels: u16,
    frequency: u32,
    bytes_per_second: u32,
    block_alignment: u16,
    bits_per_sample: u16,
    additional_data_length: u16,
    additional_data: Option<Box<[u8]>>,
}

impl FormatChunk {
    pub fn read<T: ReadBytesExt>(
        reader: &mut T,
        chunk_header: &SubChunkHeader,
    ) -> Result<Self, WavError> {
        let compression_code = reader.read_u16::<LittleEndian>()?;
        let channels = reader.read_u16::<LittleEndian>()?;
        let frequency = reader.read_u32::<LittleEndian>()?;
        let bytes_per_second = reader.read_u32::<LittleEndian>()?;
        let block_alignment = reader.read_u16::<LittleEndian>()?;
        let bits_per_sample = reader.read_u16::<LittleEndian>()?;
        let additional_data_length;
        let additional_data;
        if chunk_header.size > 16 {
            additional_data_length = reader.read_u16::<LittleEndian>()?;
            let mut buffer = vec![0u8; additional_data_length as usize];
            reader.read(&mut buffer)?;
            additional_data = Some(buffer.into_boxed_slice());
        } else {
            additional_data_length = 0;
            additional_data = None;
        }

        Ok(FormatChunk {
            compression_code,
            channels,
            frequency,
            bytes_per_second,
            block_alignment,
            bits_per_sample,
            additional_data_length,
            additional_data,
        })
    }

    #[allow(dead_code)]
    pub fn write<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), WavError> {
        writer.write_u16::<LittleEndian>(self.compression_code)?;
        writer.write_u16::<LittleEndian>(self.channels)?;
        writer.write_u32::<LittleEndian>(self.frequency)?;
        writer.write_u32::<LittleEndian>(self.bytes_per_second)?;
        writer.write_u16::<LittleEndian>(self.block_alignment)?;
        writer.write_u16::<LittleEndian>(self.bits_per_sample)?;
        if self.additional_data_length > 0 {
            writer.write_u16::<LittleEndian>(self.additional_data_length)?;
            writer.write_all(&self.additional_data.as_ref().unwrap())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct DataChunk {
    data: Box<[u8]>,
}

impl DataChunk {
    pub fn read<T: ReadBytesExt>(
        reader: &mut T,
        chunk_header: &SubChunkHeader,
        is_probably_naive_file: bool,
    ) -> Result<Self, WavError> {
        let mut buffer;
        if is_probably_naive_file {
            // in this scenario, we have doubts about the chunk size being recorded correctly
            // (the tool that created this file may have used a buggy calculation). we assume that
            // in this case, this wav file is probably written in a "naive" manner and likely only
            // contains "fmt" and "data" chunks with the "data" chunk being at the end of the file.
            // if this assumption is correct, then we can just read everything until EOF here as
            // the "data" chunk contents and that should be ok (assuming this file isn't corrupt
            // anyway).
            buffer = Vec::new();
            reader.read_to_end(&mut buffer)?;
        } else {
            // alternatively, this scenario means we are assuming the file was written out more
            // properly and we will assume the chunk size is correct and read that many bytes.
            // this is best if there is the possibility that there are more chunks than just "fmt"
            // and "data" in this wav file and maybe they are in a different order, etc.
            // it is important to note that this seems to be an uncommon case for wav files. most
            // wav files seem to be written in a fairly "naive" manner.
            buffer = vec![0u8; chunk_header.size as usize];
            reader.read_exact(&mut buffer)?;
        }
        Ok(DataChunk {
            data: buffer.into_boxed_slice(),
        })
    }

    #[allow(dead_code)]
    pub fn write<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), WavError> {
        writer.write_all(self.data.as_ref())?;
        Ok(())
    }
}

impl AudioBuffer {
    pub fn load_wav_bytes<T: ReadBytesExt + Seek>(reader: &mut T) -> Result<AudioBuffer, WavError> {
        let file_size = reader.stream_size()?;

        let header = WavHeader::read(reader)?;
        if header.file_chunk.chunk_id.id != *b"RIFF" {
            return Err(WavError::BadFile(String::from(
                "Unexpected RIFF chunk id, probably not a WAV file",
            )));
        }
        if header.file_container_id.id != *b"WAVE" {
            return Err(WavError::BadFile(String::from(
                "Unexpected RIFF container id, probably not a WAV file",
            )));
        }

        // some tools like sfxr and jsfxr incorrectly calculate data sizes, seemingly using a
        // hardcoded 16-bit sample size in their calculations even when the file being created has
        // 8-bit sample sizes. this means the chunk size here and the "data" chunk size will be
        // larger than they should be for the _actual_ data present in the file.
        // of course, if the chunk size here is wrong, it could also mean a corrupt file. but we'll
        // proceed regardless, with the assumption that an incorrect size here probably means that
        // the file was created using these semi-broken tools and should act accordingly later on.
        // if the file is actually corrupt (maybe truncated accidentally or something), then we'll
        // hit an EOF earlier than expected somewhere too ...
        let is_probably_naive_file = file_size - 8 != header.file_chunk.size as u64;

        let mut format: Option<FormatChunk> = None;
        let mut data: Option<DataChunk> = None;

        loop {
            let chunk_header = match SubChunkHeader::read(reader) {
                Ok(header) => header,
                Err(WavError::IOError(io_error))
                    if io_error.kind() == io::ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(err) => return Err(err),
            };
            let chunk_data_position = reader.stream_position()?;

            // read only the chunks we recognize / care about
            if chunk_header.chunk_id.id == *b"fmt " {
                format = Some(FormatChunk::read(reader, &chunk_header)?);
                if format.as_ref().unwrap().compression_code != 1 {
                    return Err(WavError::BadFile(String::from(
                        "Only PCM format WAV files are supported",
                    )));
                }
                if format.as_ref().unwrap().bits_per_sample != 8 &&
                    format.as_ref().unwrap().bits_per_sample != 16 {
                    return Err(WavError::BadFile(String::from(
                        "Only 8-bit and 16-bit sample WAV files are supported",
                    )));
                }
            } else if chunk_header.chunk_id.id == *b"data" {
                data = Some(DataChunk::read(reader, &chunk_header, is_probably_naive_file)?);
            }

            // move to the start of the next chunk (possibly skipping over the current chunk if we
            // didn't recognize it above ...)
            reader.seek(SeekFrom::Start(
                chunk_data_position + chunk_header.size as u64,
            ))?;
        }

        // all done reading the file, now convert the read data into an AudioBuffer ...

        let mut audio_buffer;

        if let Some(format) = format {
            let sample_format = match format.bits_per_sample {
                8 => AudioFormat::U8,
                16 => AudioFormat::S16LSB,
                // this shouldn't be able to happen given the above checks when reading the
                // "fmt" chunk
                _ => return Err(WavError::BadFile(String::from("Unsupported sample bit size.")))
            };
            let spec = AudioSpec::new(format.frequency, format.channels as u8, sample_format);
            audio_buffer = AudioBuffer::new(spec);
        } else {
            return Err(WavError::BadFile(String::from("No 'fmt ' chunk was found")));
        }

        if let Some(data) = data {
            audio_buffer.data = data.data.into_vec();
        } else {
            return Err(WavError::BadFile(String::from("No 'data' chunk was found")));
        }

        Ok(audio_buffer)
    }

    pub fn load_wav_file(path: &Path) -> Result<AudioBuffer, WavError> {
        let f = File::open(path)?;
        let mut reader = BufReader::new(f);
        Self::load_wav_bytes(&mut reader)
    }
}

#[cfg(test)]
mod tests {
    use crate::audio::*;

    use super::*;

    #[test]
    pub fn load_wav_file() -> Result<(), WavError> {
        let wav_buffer = AudioBuffer::load_wav_file(Path::new("./test-assets/22khz_8bit_1ch.wav"))?;
        assert_eq!(AUDIO_FREQUENCY_22KHZ, wav_buffer.spec().frequency());
        assert_eq!(1, wav_buffer.spec().channels());
        assert_eq!(AudioFormat::U8, wav_buffer.spec.format);
        assert_eq!(11248, wav_buffer.data.len());

        let wav_buffer = AudioBuffer::load_wav_file(Path::new("./test-assets/44khz_8bit_1ch.wav"))?;
        assert_eq!(AUDIO_FREQUENCY_44KHZ, wav_buffer.spec().frequency());
        assert_eq!(1, wav_buffer.spec().channels());
        assert_eq!(AudioFormat::U8, wav_buffer.spec.format);
        assert_eq!(22496, wav_buffer.data.len());

        let wav_buffer = AudioBuffer::load_wav_file(Path::new("./test-assets/22khz_16bit_1ch.wav"))?;
        assert_eq!(AUDIO_FREQUENCY_22KHZ, wav_buffer.spec().frequency());
        assert_eq!(1, wav_buffer.spec().channels());
        assert_eq!(AudioFormat::S16LSB, wav_buffer.spec.format);
        assert_eq!(22496, wav_buffer.data.len());

        let wav_buffer = AudioBuffer::load_wav_file(Path::new("./test-assets/44khz_16bit_1ch.wav"))?;
        assert_eq!(AUDIO_FREQUENCY_44KHZ, wav_buffer.spec().frequency());
        assert_eq!(1, wav_buffer.spec().channels());
        assert_eq!(AudioFormat::S16LSB, wav_buffer.spec.format);
        assert_eq!(44992, wav_buffer.data.len());

        Ok(())
    }
}
