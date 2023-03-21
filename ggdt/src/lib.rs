extern crate core;
extern crate sdl2;

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

pub const LOW_RES: bool = if cfg!(feature = "low_res") {
	true
} else {
	false
};
pub const WIDE_SCREEN: bool = if cfg!(feature = "wide") {
	true
} else {
	false
};

pub const SCREEN_WIDTH: u32 = if cfg!(feature = "low_res") {
	if cfg!(feature = "wide") {
		214
	} else {
		160
	}
} else {
	if cfg!(feature = "wide") {
		428
	} else {
		320
	}
};
pub const SCREEN_HEIGHT: u32 = if cfg!(feature = "low_res") {
	120
} else {
	240
};

pub const SCREEN_TOP: u32 = 0;
pub const SCREEN_LEFT: u32 = 0;
pub const SCREEN_RIGHT: u32 = SCREEN_WIDTH - 1;
pub const SCREEN_BOTTOM: u32 = SCREEN_HEIGHT - 1;

pub const DEFAULT_SCALE_FACTOR: u32 = if cfg!(feature = "low_res") {
	6
} else {
	3
};

pub const NUM_COLORS: usize = 256; // i mean ... the number of colors is really defined by the size of u8 ...

// using this to hold common unit test helper functions
// (since apparently rust doesn't really have a great alternative ... ?)
#[cfg(test)]
pub mod tests {
	use std::fs::File;
	use std::io;
	use std::io::{BufReader, Read};
	use std::path::{Path, PathBuf};

	use byteorder::{LittleEndian, ReadBytesExt};

	const ASSETS_PATH: &str = "./assets/";
	const TEST_ASSETS_PATH: &str = "./test-assets/";

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

	pub fn load_raw_argb(bin_file: &Path) -> Result<Box<[u32]>, io::Error> {
		let f = File::open(bin_file)?;
		let mut reader = BufReader::new(f);
		let mut buffer = Vec::new();
		loop {
			buffer.push(match reader.read_u32::<LittleEndian>() {
				Ok(value) => value,
				Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => break,
				Err(err) => return Err(err),
			});
		}
		Ok(buffer.into_boxed_slice())
	}
}