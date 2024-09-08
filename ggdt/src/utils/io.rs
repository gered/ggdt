use byteorder::{ReadBytesExt, WriteBytesExt};
use std::{
	io::{Error, SeekFrom},
	path::PathBuf,
};

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

/// Returns the application root directory (the directory that the application executable is
/// located in).
///
/// First tries to automatically detect this from the `CARGO_MANIFEST_DIR` environment variable,
/// if present, to catch scenarios where the application is running from a Cargo workspace as a
/// sub-project/binary within that workspace. In such a case, the application root directory is
/// the same as that sub-project/binary's `Cargo.toml`.
///
/// If `CARGO_MANIFEST_DIR` is not present, then an attempt is made to determine the application
/// root directory from the running executable's location.
///
/// If this fails for some reason, then this returns the current working directory.
pub fn app_root_dir() -> Result<PathBuf, Error> {
	if let Some(manifest_path) = std::env::var_os("CARGO_MANIFEST_DIR") {
		return Ok(PathBuf::from(manifest_path));
	}

	let mut exe_path = std::env::current_exe()?.canonicalize()?;
	if exe_path.pop() {
		return Ok(exe_path);
	}

	std::env::current_dir()
}
