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
