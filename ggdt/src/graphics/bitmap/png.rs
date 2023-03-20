use std::fs::File;
use std::hash::Hasher;
use std::io;
use std::io::{BufReader, BufWriter, Cursor, Seek};
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::graphics::bitmap::Bitmap;
use crate::graphics::bitmap::indexed::IndexedBitmap;
use crate::graphics::bitmap::rgb::RgbaBitmap;
use crate::graphics::palette::Palette;
use crate::graphics::Pixel;
use crate::prelude::{PaletteError, PaletteFormat, to_argb32, to_rgb32};
use crate::utils::bytes::ReadFixedLengthByteArray;

const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

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

	pub fn decode(&self, x: usize, y: usize, stride: usize, bpp: usize, data: &[u8]) -> u8 {
		use Filter::*;
		let index = (y * stride) + x;
		match self {
			None => data[index],
			Sub => data[index].wrapping_add(if x < bpp { 0 } else { data[index - bpp] }), // unsigned arithmetic modulo 256
			Up => data[index].wrapping_add(if y < 1 { 0 } else { data[index - stride] }), // unsigned arithmetic modulo 256
			Average => {
				let a = if x < bpp { 0 } else { data[index - bpp] } as i16;
				let b = if y < 1 { 0 } else { data[index - stride] } as i16;
				// unsigned arithmetic modulo 256 *except* for the average calculation itself which must not overflow!
				let average = (a + b) / 2;
				data[index].wrapping_add(average as u8)
			},
			Paeth => {
				let a = if x < bpp { 0 } else { data[index - bpp] } as i16;
				let b = if y < 1 { 0 } else { data[index - stride] } as i16;
				let c = if x >= bpp && y >= 1 { data[index - stride - bpp] } else { 0 } as i16;
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
				data[index].wrapping_add(value)
			}
		}
	}
}

trait PixelReader<PixelType: Pixel> {
	fn next_pixel<T: ReadBytesExt>(&mut self, reader: &mut T) -> Result<PixelType, PngError>;
}

struct PixelDecoder<PixelType: Pixel> {
	bitmap: Bitmap<PixelType>,
	header: ImageHeaderChunk,
	palette: Option<Palette>,
	x: u32,
	y: u32,
	num_pixels_read: usize,
}

impl PixelReader<u8> for PixelDecoder<u8> {
	fn next_pixel<T: ReadBytesExt>(&mut self, reader: &mut T) -> Result<u8, PngError> {
		match self.header.format {
			ColorFormat::IndexedColor => {
				Ok(reader.read_u8()?)
			}
			_ => return Err(PngError::BadFile(format!("Unsupported color format for this PixelReader: {:?}", self.header.format))),
		}
	}
}

impl PixelReader<u32> for PixelDecoder<u32> {
	fn next_pixel<T: ReadBytesExt>(&mut self, reader: &mut T) -> Result<u32, PngError> {
		match self.header.format {
			ColorFormat::IndexedColor => {
				let color = reader.read_u8()?;
				if let Some(palette) = &self.palette {
					Ok(palette[color])
				} else {
					return Err(PngError::BadFile(String::from("No palette to map indexed-color format pixels to RGBA format destination")));
				}
			}
			ColorFormat::RGB => {
				let r = reader.read_u8()?;
				let g = reader.read_u8()?;
				let b = reader.read_u8()?;
				Ok(to_rgb32(r, g, b))
			}
			ColorFormat::RGBA => {
				let r = reader.read_u8()?;
				let g = reader.read_u8()?;
				let b = reader.read_u8()?;
				let a = reader.read_u8()?;
				Ok(to_argb32(a, r, g, b))
			}
			_ => return Err(PngError::BadFile(format!("Unsupported color format for this PixelReader: {:?}", self.header.format))),
		}
	}
}

impl<PixelType> PixelDecoder<PixelType>
where
	Self: PixelReader<PixelType>,
	PixelType: Pixel
{
	pub fn new(header: ImageHeaderChunk, palette: Option<Palette>) -> Self {
		PixelDecoder {
			bitmap: Bitmap::internal_new(header.width, header.height).unwrap(),
			header,
			palette,
			x: 0,
			y: 0,
			num_pixels_read: 0,
		}
	}

	pub fn decode(&mut self, data: &[u8]) -> Result<(), PngError> {
		// TODO: this entire function is kinda gross. bleh. but i just wanted to get it working first

		let mut decoder = flate2::read::ZlibDecoder::new(data);

		let bpp = match self.header.format {
			ColorFormat::IndexedColor => 1,
			ColorFormat::RGB => 3,
			ColorFormat::RGBA => 4,
			_ => return Err(PngError::BadFile(format!("Unsupported color format: {:?}", self.header.format))),
		};
		let stride = self.header.width as usize * bpp;
		let mut decoded_bytes = vec![0u8; stride * self.header.height as usize];
		let mut idx = 0;

		for y in 0..self.bitmap.height as usize {
			let filter = Filter::from(decoder.read_u8()?)?;
			for x in 0..stride as usize {
				decoded_bytes[idx] = decoder.read_u8()?;
				let decoded = filter.decode(x, y, stride, bpp, &decoded_bytes);
				decoded_bytes[idx] = decoded;
				idx += 1;
			}
		}

		let mut decoded_reader = Cursor::new(decoded_bytes);
		while self.y < self.bitmap.height {
			while self.x < self.bitmap.width {
				let pixel = self.next_pixel(&mut decoded_reader)?;
				// TODO: we can make this a bit more efficient ...
				unsafe { self.bitmap.set_pixel_unchecked(self.x as i32, self.y as i32, pixel); }
				self.num_pixels_read += 1;

				self.x += 1;
			}
			self.x = 0;
			self.y += 1;
		}

		Ok(())
	}

	pub fn finalize(self) -> Result<(Bitmap<PixelType>, Option<Palette>), PngError> {
		if self.num_pixels_read != self.bitmap.pixels.len() {
			return Err(PngError::BadFile(String::from("PNG file did not contain enough pixel data for the full image. Possibly corrupt or truncated?")));
		} else {
			Ok((self.bitmap, self.palette))
		}
	}
}

fn load_png_bytes<Reader, PixelType>(
	reader: &mut Reader
) -> Result<(Bitmap<PixelType>, Option<Palette>), PngError>
where
	Reader: ReadBytesExt + Seek,
	PixelType: Pixel,
	PixelDecoder<PixelType>: PixelReader<PixelType>
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

	let mut pixel_decoder = PixelDecoder::new(ihdr, palette);
	let mut buffer = Vec::new();
	loop {
		let chunk_header = match find_chunk(reader, *b"IDAT") {
			Ok(header) => header,
			Err(PngError::IOError(io_error)) if io_error.kind() == io::ErrorKind::UnexpectedEof => break,
			Err(err) => return Err(err),
		};

		buffer.append(&mut read_chunk_data(reader, &chunk_header)?);
	}

	pixel_decoder.decode(&buffer)?;
	Ok(pixel_decoder.finalize()?)
}

impl IndexedBitmap {
	pub fn load_png_bytes<T: ReadBytesExt + Seek>(
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
		todo!()
	}

	pub fn to_png_file(&self, path: &Path, palette: &Palette) -> Result<(), PngError> {
		let f = File::create(path)?;
		let mut writer = BufWriter::new(f);
		self.to_png_bytes(&mut writer, palette)
	}
}

impl RgbaBitmap {
	pub fn load_png_bytes<T: ReadBytesExt + Seek>(
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
		todo!()
	}

	pub fn to_png_file(&self, path: &Path) -> Result<(), PngError> {
		let f = File::create(path)?;
		let mut writer = BufWriter::new(f);
		self.to_png_bytes(&mut writer)
	}
}

#[cfg(test)]
pub mod tests {
	use std::io::Read;
	use std::path::PathBuf;

	use byteorder::LittleEndian;
	use claim::*;

	use super::*;

	const BASE_PATH: &str = "./test-assets/png/";

	fn path_to_file(file: &Path) -> PathBuf {
		PathBuf::from(BASE_PATH).join(file)
	}

	fn load_raw_indexed(bin_file: &Path) -> Result<Box<[u8]>, io::Error> {
		let f = File::open(bin_file)?;
		let mut reader = BufReader::new(f);
		let mut buffer = Vec::new();
		reader.read_to_end(&mut buffer)?;
		Ok(buffer.into_boxed_slice())
	}

	fn load_raw_argb(bin_file: &Path) -> Result<Box<[u32]>, io::Error> {
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

	#[test]
	pub fn loads_indexed_256_color() -> Result<(), PngError> {
		let ref_bytes = load_raw_indexed(path_to_file(Path::new("indexed_8.bin")).as_path())?;
		let (bmp, palette) = IndexedBitmap::load_png_file(path_to_file(Path::new("indexed_8.png")).as_path())?;
		assert!(palette.is_some());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_indexed_256_color_to_rgba_destination() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(path_to_file(Path::new("indexed_8_rgba.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(path_to_file(Path::new("indexed_8.png")).as_path())?;
		assert!(palette.is_some());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_rgb_color() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(path_to_file(Path::new("rgb.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(path_to_file(Path::new("rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_rgba_color() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(path_to_file(Path::new("rgba.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(path_to_file(Path::new("rgba.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_filter_0() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(path_to_file(Path::new("filter_0_rgb.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(path_to_file(Path::new("filter_0_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_filter_1() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(path_to_file(Path::new("filter_1_rgb.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(path_to_file(Path::new("filter_1_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_filter_2() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(path_to_file(Path::new("filter_2_rgb.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(path_to_file(Path::new("filter_2_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_filter_3() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(path_to_file(Path::new("filter_3_rgb.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(path_to_file(Path::new("filter_3_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_filter_4() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(path_to_file(Path::new("filter_4_rgb.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(path_to_file(Path::new("filter_4_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);
		Ok(())
	}

	#[test]
	pub fn loads_larger_indexed_256color_images() -> Result<(), PngError> {
		let ref_bytes = load_raw_indexed(path_to_file(Path::new("large_1_indexed.bin")).as_path())?;
		let (bmp, palette) = IndexedBitmap::load_png_file(path_to_file(Path::new("large_1_indexed.png")).as_path())?;
		assert!(palette.is_some());
		assert_eq!(ref_bytes, bmp.pixels);

		let ref_bytes = load_raw_indexed(path_to_file(Path::new("large_2_indexed.bin")).as_path())?;
		let (bmp, palette) = IndexedBitmap::load_png_file(path_to_file(Path::new("large_2_indexed.png")).as_path())?;
		assert!(palette.is_some());
		assert_eq!(ref_bytes, bmp.pixels);

		Ok(())
	}

	#[test]
	pub fn loads_larger_rgb_images() -> Result<(), PngError> {
		let ref_bytes = load_raw_argb(path_to_file(Path::new("large_1_rgba.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(path_to_file(Path::new("large_1_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);

		let ref_bytes = load_raw_argb(path_to_file(Path::new("large_2_rgba.bin")).as_path())?;
		let (bmp, palette) = RgbaBitmap::load_png_file(path_to_file(Path::new("large_2_rgb.png")).as_path())?;
		assert!(palette.is_none());
		assert_eq!(ref_bytes, bmp.pixels);

		Ok(())
	}

	#[test]
	pub fn load_fails_on_unsupported_formats() -> Result<(), PngError> {
		assert_matches!(
			RgbaBitmap::load_png_file(path_to_file(Path::new("unsupported_alpha_8bit.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			RgbaBitmap::load_png_file(path_to_file(Path::new("unsupported_greyscale_8bit.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			RgbaBitmap::load_png_file(path_to_file(Path::new("unsupported_indexed_16col.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			RgbaBitmap::load_png_file(path_to_file(Path::new("unsupported_rgb_16bit.png")).as_path()),
			Err(PngError::BadFile(..))
		);
		assert_matches!(
			RgbaBitmap::load_png_file(path_to_file(Path::new("unsupported_rgba_16bit.png")).as_path()),
			Err(PngError::BadFile(..))
		);

		Ok(())
	}
}