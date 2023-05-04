use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Error, SeekFrom};

/// Provides a convenience method for determining the total size of a stream. This is provided
/// as a temporary alternative to [std::io::Seek::stream_len] which is currently marked unstable.
pub trait StreamSize {
	fn stream_size(&mut self) -> Result<u64, std::io::Error>;
}

impl<T: std::io::Read + std::io::Seek> StreamSize for T {
	fn stream_size(&mut self) -> Result<u64, Error> {
		let old_pos = self.stream_position()?;
		let len = self.seek(SeekFrom::End(0))?;

		// Avoid seeking a third time when we were already at the end of the
		// stream. The branch is usually way cheaper than a seek operation.
		if old_pos != len {
			self.seek(SeekFrom::Start(old_pos))?;
		}

		Ok(len)
	}
}

pub trait ReadType {
	type OutputType;
	type ErrorType;

	fn read<T: ReadBytesExt>(reader: &mut T) -> Result<Self::OutputType, Self::ErrorType>;
}

pub trait WriteType {
	type ErrorType;

	fn write<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), Self::ErrorType>;
}
