#![feature(portable_simd)]

pub mod audio;
pub mod base;
pub mod entities;
pub mod events;
pub mod graphics;
pub mod math;
pub mod states;
pub mod system;
pub mod utils;

pub mod prelude;

// using this to hold common unit test helper functions
// (since apparently rust doesn't really have a great alternative ... ?)
#[cfg(test)]
mod tests {
	use std::fs::File;
	use std::io;
	use std::io::{BufReader, Read};
	use std::path::{Path, PathBuf};

	use crate::graphics::RGBA;
	use crate::utils::ReadType;

	#[allow(dead_code)]
	const ASSETS_PATH: &str = "./assets/";

	const TEST_ASSETS_PATH: &str = "./test-assets/";

	#[allow(dead_code)]
	pub fn assets_file(file: &Path) -> PathBuf {
		PathBuf::from(ASSETS_PATH).join(file)
	}

	pub fn test_assets_file(file: &Path) -> PathBuf {
		PathBuf::from(TEST_ASSETS_PATH).join(file)
	}

	pub fn load_raw_indexed(bin_file: &Path) -> Result<Box<[u8]>, io::Error> {
		let f = File::open(bin_file)?;
		let mut reader = BufReader::new(f);
		let mut buffer = Vec::new();
		reader.read_to_end(&mut buffer)?;
		Ok(buffer.into_boxed_slice())
	}

	pub fn load_raw_rgba(bin_file: &Path) -> Result<Box<[RGBA]>, io::Error> {
		let f = File::open(bin_file)?;
		let mut reader = BufReader::new(f);
		let mut buffer = Vec::new();
		loop {
			buffer.push(match RGBA::read(&mut reader) {
				Ok(value) => value,
				Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => break,
				Err(err) => return Err(err),
			});
		}
		Ok(buffer.into_boxed_slice())
	}
}
