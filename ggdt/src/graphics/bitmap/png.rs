use std::fs::File;
use std::hash::Hasher;
use std::io;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::graphics::bitmap::Bitmap;
use crate::graphics::bitmap::indexed::IndexedBitmap;
use crate::graphics::bitmap::rgb::RgbaBitmap;
use crate::graphics::palette::Palette;
use crate::graphics::Pixel;
use crate::prelude::{from_argb32, from_rgb32, PaletteError, PaletteFormat, to_argb32, to_rgb32};
use crate::utils::bytes::ReadFixedLengthByteArray;

const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

// this is fairly arbitrary ...
const PNG_WRITE_IDAT_CHUNK_SIZE: usize = 1024 * 8;

#[derive(Error, Debug)]
pub enum PngError {
	#[error("Bad or unsupported PNG file: {0}")]
	BadFile(String),

	#[error("PNG palette data error")]
	BadPalette(#[from] PaletteError),

	#[error("Unsupported IHDR color format: {0}")]
	UnsupportedColorType(u8),

	#[error("Unsupported filter: {0}")]
	UnsupportedFilter(u8),

	#[error("PNG I/O error")]
	IOError(#[from] std::io::Error),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ColorFormat {
	Grayscale = 0,
	RGB = 2,
	IndexedColor = 3,
	GrayscaleAlpha = 4,
	RGBA = 6,
}

impl ColorFormat {
	pub fn from(value: u8) -> Result<Self, PngError> {
		use ColorFormat::*;
		match value {
			0 => Ok(Grayscale),
			2 => Ok(RGB),
			3 => Ok(IndexedColor),
			4 => Ok(GrayscaleAlpha),
			6 => Ok(RGBA),
			_ => Err(PngError::UnsupportedColorType(value)),
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct ChunkHeader {
	size: u32,
	name: [u8; 4],
}

impl ChunkHeader {
	pub fn read<T: ReadBytesExt>(reader: &mut T) -> Result<Self, PngError> {
		Ok(ChunkHeader {
			size: reader.read_u32::<BigEndian>()?,
			name: reader.read_bytes()?,
		})
	}

	pub fn write<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), PngError> {
		writer.write_u32::<BigEndian>(self.size)?;
		writer.write(&self.name)?;
		Ok(())
	}
}

#[derive(Debug, Copy, Clone)]
struct ImageHeaderChunk {
	width: u32,
	height: u32,
	bpp: u8,
	format: ColorFormat,
	compression: u8,
	filter: u8,
	interlace: u8,
}

impl ImageHeaderChunk {
	pub fn read<T: ReadBytesExt>(reader: &mut T) -> Result<Self, PngError> {
		Ok(ImageHeaderChunk {
			width: reader.read_u32::<BigEndian>()?,
			height: reader.read_u32::<BigEndian>()?,
			bpp: reader.read_u8()?,
			format: ColorFormat::from(reader.read_u8()?)?,
			compression: reader.read_u8()?,
			filter: reader.read_u8()?,
			interlace: reader.read_u8()?,
		})
	}

	pub fn write<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), PngError> {
		writer.write_u32::<BigEndian>(self.width)?;
		writer.write_u32::<BigEndian>(self.height)?;
		writer.write_u8(self.bpp)?;
		writer.write_u8(self.format as u8)?;
		writer.write_u8(self.compression)?;
		writer.write_u8(self.filter)?;
		writer.write_u8(self.interlace)?;
		Ok(())
	}
}

fn read_chunk_data<T: ReadBytesExt>(reader: &mut T, chunk_header: &ChunkHeader) -> Result<Vec<u8>, PngError> {
	let mut chunk_bytes = vec![0u8; chunk_header.size as usize];
	reader.read_exact(&mut chunk_bytes)?;

	let mut hasher = crc32fast::Hasher::new();
	hasher.write(&chunk_header.name);
	hasher.write(&chunk_bytes);
	let actual_checksum = hasher.finalize();
	let expected_checksum = reader.read_u32::<BigEndian>()?;
	if actual_checksum != expected_checksum {
		return Err(PngError::BadFile(format!("Chunk checksum verification failed for chunk {:?}", chunk_header)));
	}
	Ok(chunk_bytes)
}

fn write_chunk<T: WriteBytesExt>(writer: &mut T, chunk_header: &ChunkHeader, data: &[u8]) -> Result<(), PngError> {
	let mut hasher = crc32fast::Hasher::new();
	hasher.write(&chunk_header.name);
	hasher.write(&data);
	let checksum = hasher.finalize();

	chunk_header.write(writer)?;
	writer.write(data)?;
	writer.write_u32::<BigEndian>(checksum)?;

	Ok(())
}

fn find_chunk<T: ReadBytesExt>(reader: &mut T, chunk_name: [u8; 4]) -> Result<ChunkHeader, PngError> {
	loop {
		let chunk_header = match ChunkHeader::read(reader) {
			Ok(chunk_header) => chunk_header,
			Err(err) => return Err(err),
		};

		if chunk_header.name == chunk_name {
			return Ok(chunk_header);
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Filter {
	None = 0,
	Sub = 1,
	Up = 2,
	Average = 3,
	Paeth = 4,
}

impl Filter {
	pub fn from(value: u8) -> Result<Self, PngError> {
		use Filter::*;
		match value {
			0 => Ok(None),
			1 => Ok(Sub),
			2 => Ok(Up),
			3 => Ok(Average),
			4 => Ok(Paeth),
			_ => Err(PngError::UnsupportedFilter(value)),
		}
	}
}

trait ScanlinePixelConverter<PixelType: Pixel> {
	fn read_pixel(&mut self, x: usize, palette: &Option<Palette>) -> Result<PixelType, PngError>;
	fn write_pixel(&mut self, x: usize, pixel: PixelType) -> Result<(), PngError>;
}

struct ScanlineBuffer {
	format: ColorFormat,
	stride: usize,
	bpp: usize,
	y: usize,
	height: usize,
	current: Vec<u8>,
	previous: Vec<u8>,
}

impl ScanlineBuffer {
	pub fn new(ihdr: &ImageHeaderChunk) -> Result<Self, PngError> {
		let bpp = match ihdr.format {
			ColorFormat::IndexedColor => 1,
			ColorFormat::RGB => 3,
			ColorFormat::RGBA => 4,
			_ => return Err(PngError::BadFile(format!("Unsupported color format: {:?}", ihdr.format))),
		};
		let stride = ihdr.width as usize * bpp;
		Ok(ScanlineBuffer {
			format: ihdr.format,
			stride,
			bpp,
			y: 0,
			height: ihdr.height as usize,
			current: vec![0u8; stride],
			previous: vec![0u8; stride],
		})
	}

	fn decode_byte(&mut self, filter: Filter, byte: u8, x: usize, y: usize) -> u8 {
		match filter {
			Filter::None => byte,
			Filter::Sub => byte.wrapping_add(if x < self.bpp { 0 } else { self.current[x - self.bpp] }), // unsigned arithmetic modulo 256
			Filter::Up => byte.wrapping_add(if y < 1 { 0 } else { self.previous[x] }), // unsigned arithmetic modulo 256
			Filter::Average => {
				let a = if x < self.bpp { 0 } else { self.current[x - self.bpp] } as i16;
				let b = if y < 1 { 0 } else { self.previous[x] } as i16;
				// unsigned arithmetic modulo 256 *except* for the average calculation itself which must not overflow!
				let average = (a + b) / 2;
				byte.wrapping_add(average as u8)
			},
			Filter::Paeth => {
				let a = if x < self.bpp { 0 } else { self.current[x - self.bpp] } as i16;
				let b = if y < 1 { 0 } else { self.previous[x] } as i16;
				let c = if x >= self.bpp && y >= 1 { self.previous[x - self.bpp] } else { 0 } as i16;
				let p = a + b - c;
				let pa = (p - a).abs();
				let pb = (p - b).abs();
				let pc = (p - c).abs();
				let value = if pa <= pb && pa <= pc {
					a as u8
				} else if pb <= pc {
					b as u8
				} else {
					c as u8
				};
				// all of the above must not overflow. however, this last part is unsigned arithmetic modulo 256
				byte.wrapping_add(value)
			},
		}
	}

	fn encode_byte(&mut self, filter: Filter, byte: u8, _x: usize, _y: usize) -> u8 {
		match filter {
			Filter::None => byte,
			_ => 0, // leaving out the rest for now. we hardcode usage of Filter::None when saving PNGs currently
		}
	}

	pub fn read_line<T: ReadBytesExt>(&mut self, reader: &mut T) -> Result<(), PngError> {
		if self.y >= self.height {
			return Err(PngError::IOError(io::Error::from(io::ErrorKind::UnexpectedEof)));
		} else if self.y >= 1 {
			self.previous.copy_from_slice(&self.current);
		}
		let y = self.y;
		self.y += 1;

		let filter = Filter::from(reader.read_u8()?)?;
		for x in 0..self.stride {
			let byte = reader.read_u8()?;
			let decoded = self.decode_byte(filter, byte, x, y);
			self.current[x] = decoded;
		}

		Ok(())
	}

	pub fn write_line<T: WriteBytesExt>(&mut self, filter: Filter, writer: &mut T) -> Result<(), PngError> {
		if self.y >= self.height {
			return Err(PngError::IOError(io::Error::from(io::ErrorKind::UnexpectedEof)));
		} else if self.y >= 1 {
			self.previous.copy_from_slice(&self.current);
		}
		let y = self.y;
		self.y += 1;

		writer.write_u8(filter as u8)?;
		for x in 0..self.stride {
			let byte = self.current[x];
			let encoded = self.encode_byte(filter, byte, x, y);
			writer.write_u8(encoded)?;
			self.current[x] = encoded;
		}

		Ok(())
	}
}

impl ScanlinePixelConverter<u8> for ScanlineBuffer {
	fn read_pixel(&mut self, x: usize, _palette: &Option<Palette>) -> Result<u8, PngError> {
		let offset = x * self.bpp;
		match self.format {
			ColorFormat::IndexedColor => {
				Ok(self.current[offset])
			},
			_ => return Err(PngError::BadFile(format!("Unsupported color format for this PixelReader: {:?}", self.format))),
		}
	}

	fn write_pixel(&mut self, x: usize, pixel: u8) -> Result<(), PngError> {
		let offset = x * self.bpp;
		match self.format {
			ColorFormat::IndexedColor => {
				self.current[offset] = pixel;
				Ok(())
			},
			_ => return Err(PngError::BadFile(format!("Unsupported color format for this PixelReader: {:?}", self.format))),
		}
	}
}

impl ScanlinePixelConverter<u32> for ScanlineBuffer {
	fn read_pixel(&mut self, x: usize, palette: &Option<Palette>) -> Result<u32, PngError> {
		let offset = x * self.bpp;
		match self.format {
			ColorFormat::IndexedColor => {
				let color = self.current[offset];
				if let Some(palette) = palette {
					Ok(palette[color])
				} else {
					return Err(PngError::BadFile(String::from("No palette to map indexed-color format pixels to RGBA format destination")));
				}
			},
			ColorFormat::RGB => {
				let r = self.current[offset];
				let g = self.current[offset + 1];
				let b = self.current[offset + 2];
				Ok(to_rgb32(r, g, b))
			},
			ColorFormat::RGBA => {
				let r = self.current[offset];
				let g = self.current[offset + 1];
				let b = self.current[offset + 2];
				let a = self.current[offset + 3];
				Ok(to_argb32(a, r, g, b))
			},
			_ => return Err(PngError::BadFile(format!("Unsupported color format for this PixelReader: {:?}", self.format))),
		}
	}

	fn write_pixel(&mut self, x: usize, pixel: u32) -> Result<(), PngError> {
		let offset = x * self.bpp;
		match self.format {
			ColorFormat::RGB => {
				let (r, g, b) = from_rgb32(pixel);
				self.current[offset] = r;
				self.current[offset + 1] = g;
				self.current[offset + 2] = b;
				Ok(())
			},
			ColorFormat::RGBA => {
				let (a, r, g, b) = from_argb32(pixel);
				self.current[offset] = r;
				self.current[offset + 1] = g;
				self.current[offset + 2] = b;
				self.current[offset + 3] = a;
				Ok(())
			},
			_ => return Err(PngError::BadFile(format!("Unsupported color format for this PixelReader: {:?}", self.format))),
		}
	}
}


fn load_png_bytes<Reader, PixelType>(
	reader: &mut Reader
) -> Result<(Bitmap<PixelType>, Option<Palette>), PngError>
where
	Reader: ReadBytesExt,
	PixelType: Pixel,
	ScanlineBuffer: ScanlinePixelConverter<PixelType>
{
	let header: [u8; 8] = reader.read_bytes()?;
	if header != PNG_HEADER {
		return Err(PngError::BadFile(String::from("Unexpected 8-byte header, probably not a PNG file")));
	}

	// get the IHDR chunk first

	let chunk_header = match find_chunk(reader, *b"IHDR") {
		Ok(header) => header,
		Err(PngError::IOError(io_error)) if io_error.kind() == io::ErrorKind::UnexpectedEof => {
			return Err(PngError::BadFile(String::from("No IHDR chunk found, probably not a PNG file")));
		}
		Err(err) => return Err(err),
	};
	let chunk_bytes = read_chunk_data(reader, &chunk_header)?;
	let ihdr = ImageHeaderChunk::read(&mut chunk_bytes.as_slice())?;

	// file format validations based on the limited subset of PNGs we will be supporting

	if ihdr.bpp != 8 {
		return Err(PngError::BadFile(String::from("Unsupported color bit depth.")));
	}
	if ihdr.format != ColorFormat::IndexedColor
		&& ihdr.format != ColorFormat::RGB
		&& ihdr.format != ColorFormat::RGBA {
		return Err(PngError::BadFile(String::from("Unsupported pixel color format.")));
	}
	if ihdr.compression != 0 {
		return Err(PngError::BadFile(String::from("Unsupported compression method.")));
	}
	if ihdr.filter != 0 {
		return Err(PngError::BadFile(String::from("Unsupported filter method.")));
	}
	if ihdr.interlace != 0 {
		return Err(PngError::BadFile(String::from("Interlaced images are not supported.")));
	}

	// if this is an indexed-color PNG, we expect to find a PLTE chunk next (or at least before the IDAT chunks)

	let palette = if ihdr.format == ColorFormat::IndexedColor {
		let chunk_header = match find_chunk(reader, *b"PLTE") {
			Ok(header) => header,
			Err(PngError::IOError(io_error)) if io_error.kind() == io::ErrorKind::UnexpectedEof => {
				return Err(PngError::BadFile(String::from("No PLTE chunk found in an indexed-color PNG")));
			}
			Err(err) => return Err(err),
		};

		let chunk_bytes = read_chunk_data(reader, &chunk_header)?;
		let num_colors = (chunk_header.size / 3) as usize;
		Some(Palette::load_num_colors_from_bytes(
			&mut chunk_bytes.as_slice(),
			PaletteFormat::Normal,
			num_colors,
		)?)
	} else {
		None
	};

	// now we're just looking for IDAT chunks. keep reading these chunks only, ignoring all others.
	// TODO: some way to read and decompress this data on the fly chunk-by-chunk, without first needing to
	//       read in ALL of the chunks into a combined buffer? it looks like chunk boundaries just
	//       arbitrarily cut off the deflate stream (that is, each chunk is NOT a separate deflate stream
	//       with just more data). so we'd need some deflate decompressor that can stream its input
	//       (compressed) byte stream too ...

	let mut compressed_data = Vec::new();
	loop {
		let chunk_header = match find_chunk(reader, *b"IDAT") {
			Ok(header) => header,
			Err(PngError::IOError(io_error)) if io_error.kind() == io::ErrorKind::UnexpectedEof => break,
			Err(err) => return Err(err),
		};

		compressed_data.append(&mut read_chunk_data(reader, &chunk_header)?);
	}

	let mut scanline_buffer = ScanlineBuffer::new(&ihdr)?;

	let mut output = Bitmap::internal_new(ihdr.width, ihdr.height).unwrap();
	let mut deflater = flate2::read::ZlibDecoder::<&[u8]>::new(&compressed_data);

	for y in 0..ihdr.height as usize {
		scanline_buffer.read_line(&mut deflater)?;
		for x in 0..ihdr.width as usize {
			let pixel = scanline_buffer.read_pixel(x, &palette)?;
			unsafe { output.set_pixel_unchecked(x as i32, y as i32, pixel); }
		}
	}

	Ok((output, palette))
}

fn write_png_bytes<Writer, PixelType>(
	writer: &mut Writer,
	bitmap: &Bitmap<PixelType>,
	palette: Option<&Palette>,
) -> Result<(), PngError>
where
	Writer: WriteBytesExt,
	PixelType: Pixel,
	ScanlineBuffer: ScanlinePixelConverter<PixelType>,
{
	let format = if Bitmap::<PixelType>::PIXEL_SIZE == 1 {
		ColorFormat::IndexedColor
	} else {
		ColorFormat::RGBA
	};

	// magic PNG header

	writer.write_all(&PNG_HEADER)?;

	// write IHDR chunk

	let ihdr = ImageHeaderChunk {
		width: bitmap.width(),
		height: bitmap.height(),
		bpp: 8,
		format,
		compression: 0,
		filter: 0,
		interlace: 0,
	};
	let mut chunk_bytes = Vec::new();
	ihdr.write(&mut chunk_bytes)?;
	let chunk_header = ChunkHeader { name: *b"IHDR", size: chunk_bytes.len() as u32 };
	write_chunk(writer, &chunk_header, &chunk_bytes)?;

	// if there is a palette, write it out in a PLTE chunk

	if let Some(palette) = palette {
		let mut chunk_bytes = Vec::new();
		palette.to_bytes(&mut chunk_bytes, PaletteFormat::Normal)?;
		let chunk_header = ChunkHeader { name: *b"PLTE", size: 768 };
		write_chunk(writer, &chunk_header, &chunk_bytes)?;
	}

	// now write out the raw pixel data as IDAT chunk(s)

	// convert the bitmap pixels into png scanline format and compress via deflate

	let mut scanline_buffer = ScanlineBuffer::new(&ihdr)?;
	let mut inflater = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());

	for y in 0..ihdr.height as usize {
		for x in 0..ihdr.width as usize {
			let pixel = unsafe { bitmap.get_pixel_unchecked(x as i32, y as i32) };
			scanline_buffer.write_pixel(x, pixel)?;
		}
		scanline_buffer.write_line(Filter::None, &mut inflater)?;
	}
	let chunk_bytes = inflater.finish()?;

	// write out IDAT chunks, splitting the compressed data to be written into multiple IDAT chunks.

	for sub_chunk_bytes in chunk_bytes.chunks(PNG_WRITE_IDAT_CHUNK_SIZE) {
		let chunk_header = ChunkHeader { name: *b"IDAT", size: sub_chunk_bytes.len() as u32 };

		let mut hasher = crc32fast::Hasher::new();
		hasher.write(&chunk_header.name);
		hasher.write(&sub_chunk_bytes);
		let checksum = hasher.finalize();

		chunk_header.write(writer)?;
		writer.write(sub_chunk_bytes)?;
		writer.write_u32::<BigEndian>(checksum)?;
	}

	// all done, write the IEND chunk

	let chunk_header = ChunkHeader { name: *b"IEND", size: 0 };
	let checksum = crc32fast::hash(&chunk_header.name);

	chunk_header.write(writer)?;
	writer.write_u32::<BigEndian>(checksum)?;

	Ok(())
}

impl IndexedBitmap {
	pub fn load_png_bytes<T: ReadBytesExt>(
		reader: &mut T,
	) -> Result<(IndexedBitmap, Option<Palette>), PngError> {
		load_png_bytes(reader)
	}

	pub fn load_png_file(path: &Path) -> Result<(IndexedBitmap, Option<Palette>), PngError> {
		let f = File::open(path)?;
		let mut reader = BufReader::new(f);
		Self::load_png_bytes(&mut reader)
	}

	pub fn to_png_bytes<T: WriteBytesExt>(
		&self,
		writer: &mut T,
		palette: &Palette,
	) -> Result<(), PngError> {
		write_png_bytes(writer, &self, Some(palette))
	}

	pub fn to_png_file(&self, path: &Path, palette: &Palette) -> Result<(), PngError> {
		let f = File::create(path)?;
		let mut writer = BufWriter::new(f);
		self.to_png_bytes(&mut writer, palette)
	}
}

impl RgbaBitmap {
	pub fn load_png_bytes<T: ReadBytesExt>(
		reader: &mut T,
	) -> Result<(RgbaBitmap, Option<Palette>), PngError> {
		load_png_bytes(reader)
	}

	pub fn load_png_file(path: &Path) -> Result<(RgbaBitmap, Option<Palette>), PngError> {
		let f = File::open(path)?;
		let mut reader = BufReader::new(f);
		Self::load_png_bytes(&mut reader)
	}

	pub fn to_png_bytes<T: WriteBytesExt>(
		&self,
		writer: &mut T,
	) -> Result<(), PngError> {
		write_png_bytes(writer, &self, None)
	}

	pub fn to_png_file(&self, path: &Path) -> Result<(), PngError> {
		let f = File::create(path)?;
		let mut writer = BufWriter::new(f);
		self.to_png_bytes(&mut writer)
	}
}

#[cfg(test)]
pub mod tests {
	use std::path::PathBuf;

	use claim::*;
	use tempfile::TempDir;

	use crate::tests::{load_raw_argb, load_raw_indexed};

	use super::*;

	const BASE_PATH: &str = "./test-assets/png/";

	fn test_file(file: &Path) -> PathBuf {
		PathBuf::from(BASE_PATH).join(file)
	}

	#[test]
	pub fn loads_indexed_256_color() -> Result<(), PngError> {
		let ref_bytes = load_raw_indexed(test_file(Path::new("indexed_8.bin")).as_path())?;
		let (bmp, palette) = IndexedBitmap::load_png_file(test_file(Path::new("indexed_8.png")).as_path())?;
		assert!(palette.is_some());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_indexed_256_color_to_rgba_destination() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(test_file(Path::new("indexed_8_rgba.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("indexed_8.png")).as_path())?;
		assert!(palette.is_some());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_rgb_color() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(test_file(Path::new("rgb.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_rgba_color() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(test_file(Path::new("rgba.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("rgba.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_filter_0() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(test_file(Path::new("filter_0_rgb.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("filter_0_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_filter_1() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(test_file(Path::new("filter_1_rgb.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("filter_1_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_filter_2() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(test_file(Path::new("filter_2_rgb.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("filter_2_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_filter_3() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(test_file(Path::new("filter_3_rgb.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("filter_3_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_filter_4() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(test_file(Path::new("filter_4_rgb.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("filter_4_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_larger_indexed_256color_images() -> Result<(), PngError> {
		let ref_bytes = load_raw_indexed(test_file(Path::new("large_1_indexed.bin")).as_path())?;
		let (bmp, palette) = IndexedBitmap::load_png_file(test_file(Path::new("large_1_indexed.png")).as_path())?;
		assert!(palette.is_some());
		assert_eq!(ref_bytes, bmp.pixels);

		let ref_bytes = load_raw_indexed(test_file(Path::new("large_2_indexed.bin")).as_path())?;
		let (bmp, palette) = IndexedBitmap::load_png_file(test_file(Path::new("large_2_indexed.png")).as_path())?;
		assert!(palette.is_some());
		assert_eq!(ref_bytes, bmp.pixels);

		Ok(())
	}

	#[test]
	pub fn loads_larger_rgb_images() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(test_file(Path::new("large_1_rgba.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("large_1_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);

		let ref_bytes = load_raw_argb(test_file(Path::new("large_2_rgba.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("large_2_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);

		Ok(())
	}

	#[test]
	pub fn load_and_save_indexed_256_color() -> Result<(), PngError> {
		let tmp_dir = TempDir::new()?;

		let ref_bytes = load_raw_indexed(test_file(Path::new("indexed_8.bin")).as_path())?;

		let (bmp, palette) = IndexedBitmap::load_png_file(test_file(Path::new("indexed_8.png")).as_path())?;
		assert_eq!(32, bmp.width());
		assert_eq!(32, bmp.height());
		assert_eq!(bmp.pixels, ref_bytes);
		assert!(palette.is_some());

		let save_path = tmp_dir.path().join("test_save.png");
		bmp.to_png_file(&save_path, palette.as_ref().unwrap())?;
		let (reloaded_bmp, reloaded_palette) = IndexedBitmap::load_png_file(&save_path)?;
		assert_eq!(32, reloaded_bmp.width());
		assert_eq!(32, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels, bmp.pixels);
		assert_eq!(reloaded_palette, palette);

		Ok(())
	}

	#[test]
	pub fn load_and_save_indexed_256_color_larger_image() -> Result<(), PngError> {
		let tmp_dir = TempDir::new()?;

		// first image

		let ref_bytes = load_raw_indexed(test_file(Path::new("large_1_indexed.bin")).as_path())?;

		let (bmp, palette) = IndexedBitmap::load_png_file(test_file(Path::new("large_1_indexed.png")).as_path())?;
		assert_eq!(320, bmp.width());
		assert_eq!(200, bmp.height());
		assert_eq!(bmp.pixels, ref_bytes);
		assert!(palette.is_some());

		let save_path = tmp_dir.path().join("test_save.png");
		bmp.to_png_file(&save_path, palette.as_ref().unwrap())?;
		let (reloaded_bmp, reloaded_palette) = IndexedBitmap::load_png_file(&save_path)?;
		assert_eq!(320, reloaded_bmp.width());
		assert_eq!(200, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels, bmp.pixels);
		assert_eq!(reloaded_palette, palette);

		// second image

		let ref_bytes = load_raw_indexed(test_file(Path::new("large_2_indexed.bin")).as_path())?;

		let (bmp, palette) = IndexedBitmap::load_png_file(test_file(Path::new("large_2_indexed.png")).as_path())?;
		assert_eq!(320, bmp.width());
		assert_eq!(200, bmp.height());
		assert_eq!(bmp.pixels, ref_bytes);
		assert!(palette.is_some());

		let save_path = tmp_dir.path().join("test_save.png");
		bmp.to_png_file(&save_path, palette.as_ref().unwrap())?;
		let (reloaded_bmp, reloaded_palette) = IndexedBitmap::load_png_file(&save_path)?;
		assert_eq!(320, reloaded_bmp.width());
		assert_eq!(200, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels, bmp.pixels);
		assert_eq!(reloaded_palette, palette);

		Ok(())
	}

	#[test]
	pub fn load_and_save_rgb_color() -> Result<(), PngError> {
		let tmp_dir = TempDir::new()?;

		let ref_bytes = load_raw_argb(test_file(Path::new("rgb.bin")).as_path())?;

		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("rgb.png")).as_path())?;
		assert_eq!(32, bmp.width());
		assert_eq!(32, bmp.height());
		assert_eq!(bmp.pixels, ref_bytes);
		assert!(palette.is_none());

		let save_path = tmp_dir.path().join("test_save.png");
		bmp.to_png_file(&save_path)?;
		let (reloaded_bmp, reloaded_palette) = RgbaBitmap::load_png_file(&save_path)?;
		assert_eq!(32, reloaded_bmp.width());
		assert_eq!(32, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels, bmp.pixels);
		assert!(reloaded_palette.is_none());

		Ok(())
	}

	#[test]
	pub fn load_and_save_rgb_color_larger_image() -> Result<(), PngError> {
		let tmp_dir = TempDir::new()?;

		// first image

		let ref_bytes = load_raw_argb(test_file(Path::new("large_1_rgba.bin")).as_path())?;

		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("large_1_rgb.png")).as_path())?;
		assert_eq!(320, bmp.width());
		assert_eq!(200, bmp.height());
		assert_eq!(bmp.pixels, ref_bytes);
		assert!(palette.is_none());

		let save_path = tmp_dir.path().join("test_save.png");
		bmp.to_png_file(&save_path)?;
		let (reloaded_bmp, reloaded_palette) = RgbaBitmap::load_png_file(&save_path)?;
		assert_eq!(320, reloaded_bmp.width());
		assert_eq!(200, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels, bmp.pixels);
		assert!(reloaded_palette.is_none());

		// second image

		let ref_bytes = load_raw_argb(test_file(Path::new("large_2_rgba.bin")).as_path())?;

		let (bmp, palette) = RgbaBitmap::load_png_file(test_file(Path::new("large_2_rgb.png")).as_path())?;
		assert_eq!(320, bmp.width());
		assert_eq!(200, bmp.height());
		assert_eq!(bmp.pixels, ref_bytes);
		assert!(palette.is_none());

		let save_path = tmp_dir.path().join("test_save.png");
		bmp.to_png_file(&save_path)?;
		let (reloaded_bmp, reloaded_palette) = RgbaBitmap::load_png_file(&save_path)?;
		assert_eq!(320, reloaded_bmp.width());
		assert_eq!(200, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels, bmp.pixels);
		assert!(reloaded_palette.is_none());

		Ok(())
	}

	#[test]
	pub fn load_fails_on_unsupported_formats() -> Result<(), PngError> {
		assert_matches!(
			RgbaBitmap::load_png_file(test_file(Path::new("unsupported_alpha_8bit.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			RgbaBitmap::load_png_file(test_file(Path::new("unsupported_greyscale_8bit.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			RgbaBitmap::load_png_file(test_file(Path::new("unsupported_indexed_16col.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			RgbaBitmap::load_png_file(test_file(Path::new("unsupported_rgb_16bit.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			RgbaBitmap::load_png_file(test_file(Path::new("unsupported_rgba_16bit.png")).as_path()),
			Err(PngError::BadFile(..))
		);

		assert_matches!(
			IndexedBitmap::load_png_file(test_file(Path::new("unsupported_alpha_8bit.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			IndexedBitmap::load_png_file(test_file(Path::new("unsupported_greyscale_8bit.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			IndexedBitmap::load_png_file(test_file(Path::new("unsupported_indexed_16col.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			IndexedBitmap::load_png_file(test_file(Path::new("unsupported_rgb_16bit.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			IndexedBitmap::load_png_file(test_file(Path::new("unsupported_rgba_16bit.png")).as_path()),
			Err(PngError::BadFile(..))
		);

		// also test the extra formats that IndexedBitmap does not support which RgbaBitmap does
		// (anything not 256-color indexed basically ...)
		assert_matches!(
			IndexedBitmap::load_png_file(test_file(Path::new("rgb.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			IndexedBitmap::load_png_file(test_file(Path::new("rgba.png")).as_path()),
			Err(PngError::BadFile(..))
		);

		Ok(())
	}
}