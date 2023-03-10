use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Seek, SeekFrom};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::graphics::indexed::bitmap::Bitmap;
use crate::graphics::indexed::palette::{from_rgb32, Palette, PaletteError, PaletteFormat};
use crate::utils::bytes::ReadFixedLengthByteArray;

#[derive(Error, Debug)]
pub enum PcxError {
	#[error("Bad or unsupported PCX file: {0}")]
	BadFile(String),

	#[error("PCX palette data error")]
	BadPalette(#[from] PaletteError),

	#[error("PCX I/O error")]
	IOError(#[from] std::io::Error),
}

#[derive(Debug, Copy, Clone)]
#[repr(packed)]
struct PcxHeader {
	manufacturer: u8,
	version: u8,
	encoding: u8,
	bpp: u8,
	x1: u16,
	y1: u16,
	x2: u16,
	y2: u16,
	horizontal_dpi: u16,
	vertical_dpi: u16,
	ega_palette: [u8; 48],
	reserved: u8,
	num_color_planes: u8,
	bytes_per_line: u16,
	palette_type: u16,
	horizontal_size: u16,
	vertical_size: u16,
	padding: [u8; 54],
}

impl PcxHeader {
	pub fn read<T: ReadBytesExt>(reader: &mut T) -> Result<Self, PcxError> {
		Ok(PcxHeader {
			manufacturer: reader.read_u8()?,
			version: reader.read_u8()?,
			encoding: reader.read_u8()?,
			bpp: reader.read_u8()?,
			x1: reader.read_u16::<LittleEndian>()?,
			y1: reader.read_u16::<LittleEndian>()?,
			x2: reader.read_u16::<LittleEndian>()?,
			y2: reader.read_u16::<LittleEndian>()?,
			horizontal_dpi: reader.read_u16::<LittleEndian>()?,
			vertical_dpi: reader.read_u16::<LittleEndian>()?,
			ega_palette: reader.read_bytes()?,
			reserved: reader.read_u8()?,
			num_color_planes: reader.read_u8()?,
			bytes_per_line: reader.read_u16::<LittleEndian>()?,
			palette_type: reader.read_u16::<LittleEndian>()?,
			horizontal_size: reader.read_u16::<LittleEndian>()?,
			vertical_size: reader.read_u16::<LittleEndian>()?,
			padding: reader.read_bytes()?,
		})
	}

	pub fn write<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), PcxError> {
		writer.write_u8(self.manufacturer)?;
		writer.write_u8(self.version)?;
		writer.write_u8(self.encoding)?;
		writer.write_u8(self.bpp)?;
		writer.write_u16::<LittleEndian>(self.x1)?;
		writer.write_u16::<LittleEndian>(self.y1)?;
		writer.write_u16::<LittleEndian>(self.x2)?;
		writer.write_u16::<LittleEndian>(self.y2)?;
		writer.write_u16::<LittleEndian>(self.horizontal_dpi)?;
		writer.write_u16::<LittleEndian>(self.vertical_dpi)?;
		writer.write_all(&self.ega_palette)?;
		writer.write_u8(self.reserved)?;
		writer.write_u8(self.num_color_planes)?;
		writer.write_u16::<LittleEndian>(self.bytes_per_line)?;
		writer.write_u16::<LittleEndian>(self.palette_type)?;
		writer.write_u16::<LittleEndian>(self.horizontal_size)?;
		writer.write_u16::<LittleEndian>(self.vertical_size)?;
		writer.write_all(&self.padding)?;
		Ok(())
	}
}

fn write_pcx_data<T: WriteBytesExt>(
	writer: &mut T,
	run_count: u8,
	pixel: u8,
) -> Result<(), PcxError> {
	if (run_count > 1) || ((pixel & 0xc0) == 0xc0) {
		writer.write_u8(0xc0 | run_count)?;
	}
	writer.write_u8(pixel)?;
	Ok(())
}

impl Bitmap {
	pub fn load_pcx_bytes<T: ReadBytesExt + Seek>(
		reader: &mut T,
	) -> Result<(Bitmap, Palette), PcxError> {
		let header = PcxHeader::read(reader)?;

		if header.manufacturer != 10 {
			return Err(PcxError::BadFile(String::from(
				"Unexpected header.manufacturer value, probably not a PCX file",
			)));
		}
		if header.version != 5 {
			return Err(PcxError::BadFile(String::from(
				"Only version 5 PCX files are supported",
			)));
		}
		if header.encoding != 1 {
			return Err(PcxError::BadFile(String::from(
				"Only RLE-compressed PCX files are supported",
			)));
		}
		if header.bpp != 8 {
			return Err(PcxError::BadFile(String::from(
				"Only 8-bit indexed (256 color palette) PCX files are supported",
			)));
		}
		if header.x2 == 0 || header.y2 == 0 {
			return Err(PcxError::BadFile(String::from(
				"Invalid PCX image dimensions",
			)));
		}

		// read the PCX file's pixel data into a bitmap

		let width = (header.x2 + 1) as u32;
		let height = (header.y2 + 1) as u32;
		let mut bmp = Bitmap::new(width, height).unwrap();
		let mut writer = Cursor::new(bmp.pixels_mut());

		for _y in 0..height {
			// read the next scanline's worth of pixels from the PCX file
			let mut x: u32 = 0;
			while x < (header.bytes_per_line as u32) {
				let mut data: u8;
				let mut count: u32;

				// read pixel or RLE count
				data = reader.read_u8()?;

				if (data & 0xc0) == 0xc0 {
					// it was an RLE count, actual pixel is the next byte ...
					count = (data & 0x3f) as u32;
					data = reader.read_u8()?;
				} else {
					// it was just a single pixel
					count = 1;
				}

				// write the current pixel value 'data' to the bitmap 'count' number of times
				while count > 0 {
					if x <= width {
						writer.write_u8(data)?;
					} else {
						writer.seek(SeekFrom::Current(1))?;
					}

					x += 1;
					count -= 1;
				}
			}
		}

		// now read the palette data located at the end of the PCX file
		// palette data should be for 256 colors, 3 bytes per color = 768 bytes
		// the palette is preceded by a single byte, 0x0c, which we will also validate

		reader.seek(SeekFrom::End(-769))?;

		let palette_marker = reader.read_u8()?;
		if palette_marker != 0x0c {
			return Err(PcxError::BadFile(String::from(
				"Palette not found at end of file",
			)));
		}

		let palette = Palette::load_from_bytes(reader, PaletteFormat::Normal)?;

		Ok((bmp, palette))
	}

	pub fn load_pcx_file(path: &Path) -> Result<(Bitmap, Palette), PcxError> {
		let f = File::open(path)?;
		let mut reader = BufReader::new(f);
		Self::load_pcx_bytes(&mut reader)
	}

	pub fn to_pcx_bytes<T: WriteBytesExt>(
		&self,
		writer: &mut T,
		palette: &Palette,
	) -> Result<(), PcxError> {
		let header = PcxHeader {
			manufacturer: 10,
			version: 5,
			encoding: 1,
			bpp: 8,
			x1: 0,
			y1: 0,
			x2: self.right() as u16,
			y2: self.bottom() as u16,
			horizontal_dpi: 320,
			vertical_dpi: 200,
			ega_palette: [0u8; 48],
			reserved: 0,
			num_color_planes: 1,
			bytes_per_line: self.width() as u16,
			palette_type: 1,
			horizontal_size: self.width() as u16,
			vertical_size: self.height() as u16,
			padding: [0u8; 54],
		};
		header.write(writer)?;

		let pixels = self.pixels();
		let mut i = 0;

		for _y in 0..=self.bottom() {
			// write one scanline at a time. breaking runs that could have continued across
			// scanlines in the process, as per the pcx standard

			let mut run_count = 0;
			let mut run_pixel = 0;

			for _x in 0..=self.right() {
				let pixel = pixels[i];
				i += 1;

				if run_count == 0 {
					run_count = 1;
					run_pixel = pixel;
				} else {
					if (pixel != run_pixel) || (run_count >= 63) {
						write_pcx_data(writer, run_count, run_pixel)?;
						run_count = 1;
						run_pixel = pixel;
					} else {
						run_count += 1;
					}
				}
			}

			// end the scanline, writing out whatever run we might have had going
			write_pcx_data(writer, run_count, run_pixel)?;
		}

		// marker for beginning of palette data
		writer.write_u8(0xc)?;

		for i in 0..=255 {
			let argb = palette[i];
			let (r, g, b) = from_rgb32(argb);
			writer.write_u8(r)?;
			writer.write_u8(g)?;
			writer.write_u8(b)?;
		}

		Ok(())
	}

	pub fn to_pcx_file(&self, path: &Path, palette: &Palette) -> Result<(), PcxError> {
		let f = File::create(path)?;
		let mut writer = BufWriter::new(f);
		self.to_pcx_bytes(&mut writer, palette)
	}
}

#[cfg(test)]
pub mod tests {
	use tempfile::TempDir;

	use super::*;

	pub static TEST_BMP_PIXELS_RAW: &[u8] = include_bytes!("../../../../test-assets/test_bmp_pixels_raw.bin");
	pub static TEST_LARGE_BMP_PIXELS_RAW: &[u8] = include_bytes!("../../../../test-assets/test_large_bmp_pixels_raw.bin");
	pub static TEST_LARGE_BMP_PIXELS_RAW_2: &[u8] = include_bytes!("../../../../test-assets/test_large_bmp_pixels_raw2.bin");

	#[test]
	pub fn load_and_save() -> Result<(), PcxError> {
		let dp2_palette =
			Palette::load_from_file(Path::new("./test-assets/dp2.pal"), PaletteFormat::Normal)
				.unwrap();
		let tmp_dir = TempDir::new()?;

		let (bmp, palette) = Bitmap::load_pcx_file(Path::new("./test-assets/test.pcx"))?;
		assert_eq!(16, bmp.width());
		assert_eq!(16, bmp.height());
		assert_eq!(bmp.pixels(), TEST_BMP_PIXELS_RAW);
		assert_eq!(palette, dp2_palette);

		let save_path = tmp_dir.path().join("test_save.pcx");
		bmp.to_pcx_file(&save_path, &palette)?;
		let (reloaded_bmp, reloaded_palette) = Bitmap::load_pcx_file(&save_path)?;
		assert_eq!(16, reloaded_bmp.width());
		assert_eq!(16, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels(), TEST_BMP_PIXELS_RAW);
		assert_eq!(reloaded_palette, dp2_palette);

		Ok(())
	}

	#[test]
	pub fn load_and_save_larger_image() -> Result<(), PcxError> {
		let tmp_dir = TempDir::new()?;

		// first image

		let (bmp, palette) = Bitmap::load_pcx_file(Path::new("./test-assets/test_image.pcx"))?;
		assert_eq!(320, bmp.width());
		assert_eq!(200, bmp.height());
		assert_eq!(bmp.pixels(), TEST_LARGE_BMP_PIXELS_RAW);

		let save_path = tmp_dir.path().join("test_save.pcx");
		bmp.to_pcx_file(&save_path, &palette)?;
		let (reloaded_bmp, _) = Bitmap::load_pcx_file(&save_path)?;
		assert_eq!(320, reloaded_bmp.width());
		assert_eq!(200, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels(), TEST_LARGE_BMP_PIXELS_RAW);

		// second image

		let (bmp, palette) = Bitmap::load_pcx_file(Path::new("./test-assets/test_image2.pcx"))?;
		assert_eq!(320, bmp.width());
		assert_eq!(200, bmp.height());
		assert_eq!(bmp.pixels(), TEST_LARGE_BMP_PIXELS_RAW_2);

		let save_path = tmp_dir.path().join("test_save_2.pcx");
		bmp.to_pcx_file(&save_path, &palette)?;
		let (reloaded_bmp, _) = Bitmap::load_pcx_file(&save_path)?;
		assert_eq!(320, reloaded_bmp.width());
		assert_eq!(200, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels(), TEST_LARGE_BMP_PIXELS_RAW_2);

		Ok(())
	}
}
