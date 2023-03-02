//! GIF-variant implementation of LZW (Lempel-Ziv-Welch) compression and decompression.
//!
//! The GIF-specific changes/limitations from other LZW implementations are:
//!
//! * LZW-encoded data is packed into a series of one or more sub-chunks of data which are blocks
//!   of at most 256 bytes in size, with the first byte of each chunk indicating the size of that
//!   chunk (limited to 255 maximum, because it's only one byte). The sequence of sub-chunks is
//!   terminated with a zero byte.
//! * Variable/dynamic code bit sizes are used. The minimum bit size supported is 2, while the
//!   maximum bit size is 12, after which the code table will be reset before encoding/decoding
//!   resumes.
//! * The input "minimum_code_size" parameter for both encoding and decoding must be a bit size
//!   between 2 and 8.
//! * Internally the code table is always initialized with all entries needed for the minimum bit
//!   size specified, plus two extra special code values are added which are used by the GIF
//!   encoding process, a "clear code" and an "end of information" code.
//! * The LZW-encoded stream always starts with a "clear code" and ends with an "end of information"
//!   code. The "clear code" may also appear at other times within the stream.

// TODO: LZW encode/decode algorithm optimizations. specifically, moving away from use of HashMaps

use std::collections::HashMap;

use byteorder::{ReadBytesExt, WriteBytesExt};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LzwBytePackingError {
	#[error("Code size bits {0} is unsupported")]
	UnsupportedCodeSizeBits(usize),

	#[error("Not enough bits available in the buffer to push new value in")]
	NotEnoughBits,
}

#[derive(Error, Debug)]
pub enum LzwError {
	#[error("Code size bits {0} is unsupported")]
	UnsupportedCodeSizeBits(usize),

	#[error("LZW byte packing/unpacking error")]
	BytePackingError(#[from] LzwBytePackingError),

	#[error("Encoding/decoding error: {0}")]
	EncodingError(String),

	#[error("LZW I/O error")]
	IOError(#[from] std::io::Error),
}

type LzwCode = u16;

const GIF_MAX_SUB_CHUNK_SIZE: usize = 255;
const GIF_MAX_CODE_SIZE_BITS: usize = 8;
const MIN_BITS: usize = 2;
const MAX_BITS: usize = 12;
const MAX_CODE_VALUE: LzwCode = (1 as LzwCode).wrapping_shl(MAX_BITS as u32) - 1;

fn is_valid_code_size_bits(code_size_bits: usize) -> bool {
	code_size_bits >= MIN_BITS && code_size_bits <= MAX_BITS
}

fn is_valid_gif_min_code_size_bits(min_code_size_bits: usize) -> bool {
	min_code_size_bits >= MIN_BITS && min_code_size_bits <= GIF_MAX_CODE_SIZE_BITS
}

fn get_bitmask_for_bits(bits: usize) -> u32 {
	let mut bitmask = 0;
	for i in 0..bits {
		bitmask |= 1u32.wrapping_shl(i as u32);
	}
	bitmask
}

fn get_table_size_for_bits(bits: usize) -> usize {
	1usize.wrapping_shl(bits as u32)
}

fn get_max_code_value_for_bits(bits: usize) -> LzwCode {
	(1 as LzwCode).wrapping_shl(bits as u32) - 1
}

#[derive(Debug)]
struct LzwBytePacker {
	buffer: u32,
	buffer_length: usize,
	current_bit_size: usize,
	bitmask: u32,
	initial_bit_size: usize,
}

impl LzwBytePacker {
	pub fn new(initial_bit_size: usize) -> Result<Self, LzwBytePackingError> {
		if !is_valid_code_size_bits(initial_bit_size) {
			return Err(LzwBytePackingError::UnsupportedCodeSizeBits(initial_bit_size));
		}

		Ok(LzwBytePacker {
			buffer: 0,
			buffer_length: 0,
			current_bit_size: initial_bit_size,
			bitmask: get_bitmask_for_bits(initial_bit_size),
			initial_bit_size,
		})
	}

	#[inline]
	fn remaining_space(&self) -> usize {
		32 - self.buffer_length
	}

	pub fn increase_bit_size(&mut self) -> Result<usize, LzwBytePackingError> {
		if self.current_bit_size >= MAX_BITS {
			return Err(LzwBytePackingError::UnsupportedCodeSizeBits(self.current_bit_size + 1));
		} else {
			self.current_bit_size += 1;
			self.bitmask = get_bitmask_for_bits(self.current_bit_size);
			Ok(self.current_bit_size)
		}
	}

	pub fn reset_bit_size(&mut self) {
		self.current_bit_size = self.initial_bit_size;
		self.bitmask = get_bitmask_for_bits(self.current_bit_size);
	}

	pub fn push_code(&mut self, code: LzwCode) -> Result<(), LzwBytePackingError> {
		if self.remaining_space() >= self.current_bit_size {
			let value = (code as u32 & self.bitmask).wrapping_shl(self.buffer_length as u32);
			self.buffer |= value;
			self.buffer_length += self.current_bit_size;
			Ok(())
		} else {
			Err(LzwBytePackingError::NotEnoughBits)
		}
	}

	pub fn take_byte(&mut self) -> Option<u8> {
		if self.buffer_length >= 8 {
			let byte = (self.buffer & 0xff) as u8;
			self.buffer = self.buffer.wrapping_shr(8);
			self.buffer_length -= 8;
			Some(byte)
		} else {
			None
		}
	}

	pub fn flush_byte(&mut self) -> Option<u8> {
		if self.buffer_length > 0 {
			let byte = (self.buffer & 0xff) as u8;
			self.buffer = self.buffer.wrapping_shr(8);
			if self.buffer_length >= 8 {
				self.buffer_length -= 8;
			} else {
				self.buffer_length = 0;
			}
			Some(byte)
		} else {
			None
		}
	}
}

#[derive(Debug)]
struct LzwBytesWriter {
	packer: LzwBytePacker,
	buffer: Vec<u8>,
}

impl LzwBytesWriter {
	pub fn new(code_size_bits: usize) -> Result<Self, LzwError> {
		if !is_valid_code_size_bits(code_size_bits) {
			return Err(LzwError::UnsupportedCodeSizeBits(code_size_bits));
		}

		Ok(LzwBytesWriter {
			packer: LzwBytePacker::new(code_size_bits)?,
			buffer: Vec::with_capacity(GIF_MAX_SUB_CHUNK_SIZE),
		})
	}

	#[inline]
	pub fn increase_bit_size(&mut self) -> Result<usize, LzwError> {
		Ok(self.packer.increase_bit_size()?)
	}

	#[inline]
	pub fn reset_bit_size(&mut self) {
		self.packer.reset_bit_size()
	}

	fn write_buffer<T: WriteBytesExt>(&mut self, writer: &mut T) -> Result<(), LzwError> {
		if !self.buffer.is_empty() {
			writer.write_u8(self.buffer.len() as u8)?;
			writer.write_all(&self.buffer)?;
			self.buffer.clear();
		}
		Ok(())
	}

	pub fn write_code<T: WriteBytesExt>(
		&mut self,
		writer: &mut T,
		code: LzwCode,
	) -> Result<(), LzwError> {
		self.packer.push_code(code)?;

		while let Some(byte) = self.packer.take_byte() {
			self.buffer.push(byte);
			if self.buffer.len() == GIF_MAX_SUB_CHUNK_SIZE {
				self.write_buffer(writer)?;
			}
		}

		Ok(())
	}

	pub fn flush<T: WriteBytesExt>(&mut self, writer: &mut T) -> Result<(), LzwError> {
		while let Some(byte) = self.packer.flush_byte() {
			self.buffer.push(byte);
			if self.buffer.len() == GIF_MAX_SUB_CHUNK_SIZE {
				self.write_buffer(writer)?;
			}
		}
		self.write_buffer(writer)?;
		// block terminator for data sub-block sequence
		writer.write_u8(0)?;
		Ok(())
	}
}

#[derive(Debug)]
struct LzwByteUnpacker {
	buffer: u32,
	buffer_length: usize,
	current_bit_size: usize,
	bitmask: u32,
	initial_bit_size: usize,
}

impl LzwByteUnpacker {
	pub fn new(initial_bit_size: usize) -> Result<Self, LzwBytePackingError> {
		if !is_valid_code_size_bits(initial_bit_size) {
			return Err(LzwBytePackingError::UnsupportedCodeSizeBits(initial_bit_size));
		}

		Ok(LzwByteUnpacker {
			buffer: 0,
			buffer_length: 0,
			current_bit_size: initial_bit_size,
			bitmask: get_bitmask_for_bits(initial_bit_size),
			initial_bit_size,
		})
	}

	#[inline]
	fn remaining_space(&self) -> usize {
		32 - self.buffer_length
	}

	pub fn increase_bit_size(&mut self) -> Result<usize, LzwBytePackingError> {
		if self.current_bit_size >= MAX_BITS {
			return Err(LzwBytePackingError::UnsupportedCodeSizeBits(self.current_bit_size + 1));
		} else {
			self.current_bit_size += 1;
			self.bitmask = get_bitmask_for_bits(self.current_bit_size);
			Ok(self.current_bit_size)
		}
	}

	pub fn reset_bit_size(&mut self) {
		self.current_bit_size = self.initial_bit_size;
		self.bitmask = get_bitmask_for_bits(self.current_bit_size);
	}

	pub fn push_byte(&mut self, byte: u8) -> Result<(), LzwBytePackingError> {
		if self.remaining_space() >= 8 {
			let value = (byte as u32).wrapping_shl(self.buffer_length as u32);
			self.buffer |= value;
			self.buffer_length += 8;
			Ok(())
		} else {
			Err(LzwBytePackingError::NotEnoughBits)
		}
	}

	pub fn take_code(&mut self) -> Option<LzwCode> {
		if self.buffer_length >= self.current_bit_size {
			let code = (self.buffer & self.bitmask) as LzwCode;
			self.buffer = self.buffer.wrapping_shr(self.current_bit_size as u32);
			self.buffer_length -= self.current_bit_size;
			Some(code)
		} else {
			None
		}
	}
}

#[derive(Debug)]
struct LzwBytesReader {
	unpacker: LzwByteUnpacker,
	sub_chunk_remaining_bytes: u8,
	reached_end: bool,
}

impl LzwBytesReader {
	pub fn new(code_size_bits: usize) -> Result<Self, LzwError> {
		if !is_valid_code_size_bits(code_size_bits) {
			return Err(LzwError::UnsupportedCodeSizeBits(code_size_bits));
		}

		Ok(LzwBytesReader {
			unpacker: LzwByteUnpacker::new(code_size_bits)?,
			sub_chunk_remaining_bytes: 0,
			reached_end: false,
		})
	}

	#[inline]
	pub fn increase_bit_size(&mut self) -> Result<usize, LzwError> {
		Ok(self.unpacker.increase_bit_size()?)
	}

	pub fn reset_bit_size(&mut self) {
		self.unpacker.reset_bit_size()
	}

	fn read_byte<T: ReadBytesExt>(&mut self, reader: &mut T) -> Result<Option<u8>, LzwError> {
		if self.reached_end {
			return Ok(None);
		}
		// if we reached the end of the current sub-chunk, read the length of the next sub-chunk.
		// if that length is zero, then we're done reading all the sub-chunks in the series (as
		// there should always be a terminator zero byte at the end of the sequence).
		if self.sub_chunk_remaining_bytes == 0 {
			self.sub_chunk_remaining_bytes = reader.read_u8()?;
			if self.sub_chunk_remaining_bytes == 0 {
				self.reached_end = true;
				return Ok(None);
			}
		}

		self.sub_chunk_remaining_bytes -= 1;
		Ok(Some(reader.read_u8()?))
	}

	pub fn read_code<T: ReadBytesExt>(
		&mut self,
		reader: &mut T,
	) -> Result<Option<LzwCode>, LzwError> {
		loop {
			if let Some(code) = self.unpacker.take_code() {
				return Ok(Some(code));
			} else {
				match self.read_byte(reader) {
					Ok(Some(byte)) => self.unpacker.push_byte(byte)?,
					Ok(None) => return Ok(None),
					Err(LzwError::IOError(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
						return Ok(None);
					}
					Err(error) => return Err(error),
				};
			}
		}
	}
}

/// Encodes data read from the `src` using LZW (GIF-variant) compression, writing the encoded
/// data out to `dest`. The LZW minimum code bit size is specified via `min_code_size`.
pub fn lzw_encode<S, D>(
	src: &mut S,
	dest: &mut D,
	min_code_size: usize,
) -> Result<(), LzwError>
	where
		S: ReadBytesExt,
		D: WriteBytesExt
{
	if !is_valid_gif_min_code_size_bits(min_code_size) {
		return Err(LzwError::UnsupportedCodeSizeBits(min_code_size));
	}

	// initialize the table, special codes, bit size info, etc
	// note that we do not add clear_code or end_of_info_code to the table since they aren't really
	// needed in the table (they are never looked up in it). this also saves us the trouble of
	// needing the table to be able to hold buffers containing u16's instead of just u8's.
	// this does mean that the size of the table is always 2 less than the number of created codes.

	let initial_table_size = get_table_size_for_bits(min_code_size);
	let clear_code = initial_table_size as LzwCode;
	let end_of_info_code = initial_table_size as LzwCode + 1;
	let mut table = HashMap::<Vec<u8>, LzwCode>::with_capacity(initial_table_size + 2);
	for i in 0..initial_table_size {
		table.insert(vec![i as u8], i as LzwCode);
	}
	let mut current_bit_size = min_code_size + 1;
	let mut max_code_value_for_bit_size = get_max_code_value_for_bits(current_bit_size);
	let mut next_code = initial_table_size as LzwCode + 2;

	// begin the output code stream. always write the min_code_size first as a normal byte. then
	// write out the clear_code, being sure to encode it using our normal dynamic bit sizing
	dest.write_u8(min_code_size as u8)?;
	let mut writer = LzwBytesWriter::new(current_bit_size)?;
	writer.write_code(dest, clear_code)?;

	// read first byte to start things off before the main loop.
	// if we eof here for some reason, just end the lzw stream like "normal" ... even though this
	// isn't really a normal situation
	let byte = match src.read_u8() {
		Ok(byte) => byte,
		Err(ref error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
			writer.write_code(dest, end_of_info_code)?;
			writer.flush(dest)?;
			return Ok(());
		}
		Err(error) => return Err(LzwError::IOError(error))
	};

	let mut buffer = vec![byte];

	loop {
		// grab the next byte
		let byte = match src.read_u8() {
			Ok(byte) => byte,
			Err(ref error) if error.kind() == std::io::ErrorKind::UnexpectedEof => break,
			Err(error) => return Err(LzwError::IOError(error))
		};

		// check if the table currently contains a string composed of the current buffer plus
		// the byte we just read (
		let mut buffer_plus_byte = buffer.clone();
		buffer_plus_byte.push(byte);

		if table.contains_key(&buffer_plus_byte) {
			// we have a match, so lets just keep collecting bytes in our buffer ...
			buffer.push(byte);
		} else {
			// no match in the table, so we need to create a new code in the table for this
			// string of bytes (buffer + byte) and also emit the code for _just_ the buffer string

			let new_code = next_code;
			next_code += 1;

			table.insert(buffer_plus_byte, new_code);

			if let Some(code) = table.get(&buffer) {
				writer.write_code(dest, *code)?;
			} else {
				return Err(LzwError::EncodingError(format!("Expected to find code in table for buffer {:?} but none was found", buffer)));
			}

			// bump up to the next bit size once we've seen enough codes to necessitate it ...
			// note that this just means codes that exist in the table, not _necessarily_ codes
			// which have actually been written out yet ...
			if new_code > max_code_value_for_bit_size {
				current_bit_size += 1;
				max_code_value_for_bit_size = get_max_code_value_for_bits(current_bit_size);
				writer.increase_bit_size()?;
			}

			// reset the table and code bit sizes once we've seen enough codes to fill all our
			// allowed bits. again, this is just based on codes that exist in the table!
			if new_code == MAX_CODE_VALUE {
				// we reached the maximum code bit size, time to re-initialize the code table
				table = HashMap::with_capacity(initial_table_size + 2);
				for i in 0..initial_table_size {
					table.insert(vec![i as u8], i as LzwCode);
				}
				current_bit_size = min_code_size + 1;
				max_code_value_for_bit_size = get_max_code_value_for_bits(current_bit_size);
				next_code = initial_table_size as LzwCode + 2;

				// reset the output code stream
				writer.write_code(dest, clear_code)?;
				writer.reset_bit_size();
			}

			buffer = vec![byte];
		}
	}

	// flush the remaining buffer and finish up the output code stream

	if let Some(code) = table.get(&buffer) {
		writer.write_code(dest, *code)?;
	} else {
		return Err(LzwError::EncodingError(format!("No matching code for buffer {:?} at end of input stream", buffer)));
	}

	writer.write_code(dest, end_of_info_code)?;
	writer.flush(dest)?;

	Ok(())
}

/// Decodes data read from the `src` using LZW (GIF-variant) decompression, writing the decoded
/// data out to `dest`.
pub fn lzw_decode<S, D>(
	src: &mut S,
	dest: &mut D,
) -> Result<(), LzwError>
	where
		S: ReadBytesExt,
		D: WriteBytesExt
{
	let min_code_size = src.read_u8()? as usize;

	if !is_valid_gif_min_code_size_bits(min_code_size) {
		return Err(LzwError::UnsupportedCodeSizeBits(min_code_size));
	}

	// initialize some basic properties for decoding and the table here ... we initialize the
	// actual table and the rest of the decoding properties we need a bit further on below

	let mut current_bit_size = min_code_size + 1;
	let initial_table_size = get_table_size_for_bits(min_code_size);
	let clear_code = initial_table_size as LzwCode;
	let end_of_info_code = initial_table_size as LzwCode + 1;

	let mut reader = LzwBytesReader::new(current_bit_size)?;

	// read the first code from the input code stream.
	// we also return immediately without writing anything to the destination byte stream if there
	// are no codes to read (kind of a weird situation, but no real reason to error ...?)
	let mut code = match reader.read_code(src)? {
		Some(code) => code,
		None => return Ok(())
	};

	// the first code in the stream SHOULD be a clear code ... which we can just ignore because
	// our table is freshly reset right now anyway. but we should flag this as an error if for some
	// reason we didn't just read a clear code!
	if code != clear_code {
		return Err(LzwError::EncodingError(String::from("Unexpected first code value (not a clear code)")));
	}

	'outer: loop {
		// initialize the table and some extra bits of info here so that whenever we read in a
		// clear code from the input stream, we can just loop back here to handle it

		// note that we do not add clear_code or end_of_info_code to the table since they aren't really
		// needed in the table (they are never looked up in it). this also saves us the trouble of
		// needing the table to be able to hold buffers containing u16's instead of just u8's.
		// this does mean that the size of the table is always 2 less than the number of created codes.

		let mut table = vec![None; 1usize.wrapping_shl(MAX_BITS as u32)];
		for i in 0..initial_table_size {
			table[i] = Some(vec![i as u8]);
		}
		let mut max_code_value_for_bit_size = get_max_code_value_for_bits(current_bit_size);
		let mut next_code = initial_table_size as LzwCode + 2;

		// read the next code which should actually be the first "interesting" value of the code stream
		code = match reader.read_code(src)? {
			Some(code) if code > MAX_CODE_VALUE => return Err(LzwError::EncodingError(format!("Encountered code that is too large: {}", code))),
			Some(code) if code == end_of_info_code => return Ok(()),
			Some(code) => code,
			None => return Err(LzwError::EncodingError(String::from("Unexpected end of code stream"))),
		};

		// ok, now we're able to get started!

		// simply write out the table string associated with the first code
		if let Some(string) = table.get(code as usize).unwrap() {
			dest.write_all(string)?;
		} else {
			return Err(LzwError::EncodingError(format!("No table entry for code {}", code)));
		}

		let mut prev_code = code;

		'inner: loop {
			// grab the next code
			code = match reader.read_code(src)? {
				Some(code) if code > MAX_CODE_VALUE => return Err(LzwError::EncodingError(format!("Encountered code that is too large: {}", code))),
				Some(code) if code == end_of_info_code => break 'outer,
				Some(code) if code == clear_code => {
					// reset the bit size and reader and then loop back to the outer loop which
					// will handle actually resetting the code table
					current_bit_size = min_code_size + 1;
					reader.reset_bit_size();
					break 'inner;
				}
				Some(code) => code,
				None => return Err(LzwError::EncodingError(String::from("Unexpected end of code stream"))),
			};

			// note: prev_code should always be present since we looked it up in the table during a
			// previous loop iteration ...
			let prev_code_string = match table.get(prev_code as usize).unwrap() {
				Some(prev_code_string) => prev_code_string,
				None => {
					return Err(LzwError::EncodingError(format!("Previous code {} not found in table", prev_code)));
				}
			};

			let new_code = next_code;
			next_code += 1;

			if let Some(string) = table.get(code as usize).unwrap() {
				// write out the matching table string for the code just read
				dest.write_all(string)?;

				// update the table accordingly
				let k = string.first().unwrap();
				let mut new_string = prev_code_string.clone();
				new_string.push(*k);
				table[new_code as usize] = Some(new_string);
			} else {
				// code is not yet present in the table.
				// add prev_code string + the code we just read to the table and also write it out
				let k = prev_code_string.first().unwrap();
				let mut new_string = prev_code_string.clone();
				new_string.push(*k);
				dest.write_all(&new_string)?;
				table[new_code as usize] = Some(new_string);
			}

			if new_code == max_code_value_for_bit_size {
				current_bit_size += 1;
				max_code_value_for_bit_size = get_max_code_value_for_bits(current_bit_size);
				reader.increase_bit_size()?;
			}

			prev_code = code;
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use super::*;

	struct LzwTestData<'a> {
		min_code_size: usize,
		packed: &'a [u8],
		unpacked: &'a [u8],
	}

	static LZW_TEST_DATA: &[LzwTestData] = &[
		LzwTestData {
			min_code_size: 2,
			packed: &[0x02, 0x16, 0x8c, 0x2d, 0x99, 0x87, 0x2a, 0x1c, 0xdc, 0x33, 0xa0, 0x02, 0x75, 0xec, 0x95, 0xfa, 0xa8, 0xde, 0x60, 0x8c, 0x04, 0x91, 0x4c, 0x01, 0x00],
			unpacked: &[1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1],
		},
		LzwTestData {
			min_code_size: 4,
			packed: &[0x04, 0x21, 0x70, 0x49, 0x79, 0x6a, 0x9d, 0xcb, 0x39, 0x7b, 0xa6, 0xd6, 0x96, 0xa4, 0x3d, 0x0f, 0xd8, 0x8d, 0x64, 0xb9, 0x1d, 0x28, 0xa9, 0x2d, 0x15, 0xfa, 0xc2, 0xf1, 0x37, 0x71, 0x33, 0xc5, 0x61, 0x4b, 0x04, 0x00],
			unpacked: &[11, 11, 11, 11, 11, 7, 7, 7, 7, 7, 11, 11, 11, 11, 14, 14, 7, 7, 7, 7, 11, 11, 11, 14, 14, 14, 14, 7, 7, 7, 11, 11, 14, 14, 15, 15, 14, 14, 7, 7, 11, 14, 14, 15, 15, 15, 15, 14, 14, 7, 7, 14, 14, 15, 15, 15, 15, 14, 14, 11, 7, 7, 14, 14, 15, 15, 14, 14, 11, 11, 7, 7, 7, 14, 14, 14, 14, 11, 11, 11, 7, 7, 7, 7, 14, 14, 11, 11, 11, 11, 7, 7, 7, 7, 7, 11, 11, 11, 11, 11],
		},
		LzwTestData {
			min_code_size: 8,
			packed: &[0x08, 0x0b, 0x00, 0x51, 0xfc, 0x1b, 0x28, 0x70, 0xa0, 0xc1, 0x83, 0x01, 0x01, 0x00],
			unpacked: &[0x28, 0xff, 0xff, 0xff, 0x28, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
		}
	];

	#[test]
	fn lzw_compresses() -> Result<(), LzwError> {
		for LzwTestData { packed, unpacked, min_code_size } in LZW_TEST_DATA {
			let mut src = Cursor::new(*unpacked);
			let mut dest = vec![0u8; 0];
			lzw_encode(&mut src, &mut dest, *min_code_size)?;
			assert_eq!(dest, *packed);
		}

		Ok(())
	}

	#[test]
	fn lzw_decompresses() -> Result<(), LzwError> {
		for LzwTestData { packed, unpacked, min_code_size: _ } in LZW_TEST_DATA {
			let mut src = Cursor::new(*packed);
			let mut dest = vec![0u8; 0];
			lzw_decode(&mut src, &mut dest)?;
			assert_eq!(dest, *unpacked);
		}

		Ok(())
	}
}
