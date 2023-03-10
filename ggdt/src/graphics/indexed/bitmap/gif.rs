use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::graphics::indexed::bitmap::Bitmap;
use crate::graphics::indexed::palette::{Palette, PaletteError, PaletteFormat};
use crate::utils::lzwgif::{lzw_decode, lzw_encode, LzwError};

const BITS_FOR_256_COLORS: u32 = 7;   // formula is `2 ^ (bits + 1) = num_colors`

fn bits_to_num_colors(bits: u32) -> u32 {
	1_u32.wrapping_shl(bits + 1)
}

fn read_raw_sub_block_data<T: Read>(reader: &mut T) -> Result<Box<[u8]>, GifError> {
	let mut data = Vec::new();
	let mut count = reader.read_u8()?;
	while count > 0 {
		let mut sub_block = vec![0u8; count as usize];
		reader.read_exact(&mut sub_block)?;
		data.append(&mut sub_block);
		// read next sub block data size (or 0 if this is the end)
		count = reader.read_u8()?;
	}
	Ok(data.into_boxed_slice())
}

fn write_raw_sub_block_data<T: Write>(data: &[u8], writer: &mut T) -> Result<(), GifError> {
	let mut bytes_left = data.len();
	let mut pos = 0;
	while bytes_left > 0 {
		let sub_block_length = if bytes_left >= 255 { 255 } else { bytes_left };
		writer.write_u8(sub_block_length as u8)?;
		let sub_block = &data[pos..sub_block_length];
		writer.write_all(sub_block)?;
		pos += sub_block_length;
		bytes_left -= sub_block_length;
	}
	// terminator (sub block of zero length)
	writer.write_u8(0)?;
	Ok(())
}

#[derive(Error, Debug)]
pub enum GifError {
	#[error("Bad or unsupported GIF file: {0}")]
	BadFile(String),

	#[error("GIF palette data error")]
	BadPalette(#[from] PaletteError),

	#[error("Unknown extension block: {0}")]
	UnknownExtension(u8),

	#[error("LZW encoding/decoding error")]
	LzwError(#[from] LzwError),

	#[error("")]
	IOError(#[from] std::io::Error),
}

pub enum GifSettings {
	Default,
	TransparentColor(u8),
}

#[derive(Debug, Copy, Clone)]
struct GifHeader {
	signature: [u8; 3],
	version: [u8; 3],
	screen_width: u16,
	screen_height: u16,
	flags: u8,
	background_color: u8,
	aspect_ratio: u8,
}

#[allow(dead_code)]
impl GifHeader {
	pub fn has_global_color_table(&self) -> bool {
		self.flags & 0b10000000 != 0
	}

	pub fn set_global_color_table(&mut self, value: bool) {
		self.flags |= (value as u8).wrapping_shl(7);
	}

	pub fn color_resolution_bits(&self) -> u8 {
		(self.flags & 0b01110000).wrapping_shr(4)
	}

	pub fn set_color_resolution_bits(&mut self, value: u8) {
		self.flags |= (value & 0b111).wrapping_shl(4);
	}

	pub fn is_color_table_entries_sorted(&self) -> bool {
		self.flags & 0b00001000 != 0
	}

	pub fn set_color_table_entries_sorted(&mut self, value: bool) {
		self.flags |= (value as u8).wrapping_shl(3);
	}

	pub fn global_color_table_bits(&self) -> u8 {
		self.flags & 0b00000111
	}

	pub fn set_global_color_table_bits(&mut self, value: u8) {
		self.flags |= value & 0b111;
	}

	pub fn read<T: Read>(reader: &mut T) -> Result<Self, GifError> {
		let mut signature = [0u8; 3];
		reader.read_exact(&mut signature)?;
		let mut version = [0u8; 3];
		reader.read_exact(&mut version)?;
		Ok(GifHeader {
			signature,
			version,
			screen_width: reader.read_u16::<LittleEndian>()?,
			screen_height: reader.read_u16::<LittleEndian>()?,
			flags: reader.read_u8()?,
			background_color: reader.read_u8()?,
			aspect_ratio: reader.read_u8()?,
		})
	}

	pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), GifError> {
		writer.write_all(&self.signature)?;
		writer.write_all(&self.version)?;
		writer.write_u16::<LittleEndian>(self.screen_width)?;
		writer.write_u16::<LittleEndian>(self.screen_height)?;
		writer.write_u8(self.flags)?;
		writer.write_u8(self.background_color)?;
		writer.write_u8(self.aspect_ratio)?;
		Ok(())
	}
}

const GIF_TRAILER: u8 = 0x3b;
const EXTENSION_INTRODUCER: u8 = 0x21;
const IMAGE_DESCRIPTOR_SEPARATOR: u8 = 0x2c;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GifExtensionLabel {
	GraphicControl = 0xf9,
	PlainText = 0x01,
	Application = 0xff,
	Comment = 0xfe,
}

impl GifExtensionLabel {
	pub fn from(value: u8) -> Result<Self, GifError> {
		use GifExtensionLabel::*;
		match value {
			0xf9 => Ok(GraphicControl),
			0x01 => Ok(PlainText),
			0xff => Ok(Application),
			0xfe => Ok(Comment),
			_ => Err(GifError::UnknownExtension(value))
		}
	}
}

#[derive(Debug, Copy, Clone)]
struct GraphicControlExtension {
	block_size: u8,
	flags: u8,
	delay: u16,
	transparent_color: u8,
	terminator: u8,
}

impl GraphicControlExtension {
	pub fn read<T: Read>(reader: &mut T) -> Result<Self, GifError> {
		Ok(GraphicControlExtension {
			block_size: reader.read_u8()?,
			flags: reader.read_u8()?,
			delay: reader.read_u16::<LittleEndian>()?,
			transparent_color: reader.read_u8()?,
			terminator: reader.read_u8()?,
		})
	}

	pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), GifError> {
		writer.write_u8(self.block_size)?;
		writer.write_u8(self.flags)?;
		writer.write_u16::<LittleEndian>(self.delay)?;
		writer.write_u8(self.transparent_color)?;
		writer.write_u8(self.terminator)?;
		Ok(())
	}
}

#[derive(Debug, Clone)]
struct PlainTextExtension {
	block_size: u8,
	text_x: u16,
	text_y: u16,
	text_width: u16,
	text_height: u16,
	cell_width: u8,
	cell_height: u8,
	foreground_color: u8,
	background_color: u8,
	data: Box<[u8]>,
}

#[allow(dead_code)]
impl PlainTextExtension {
	pub fn read<T: Read>(reader: &mut T) -> Result<Self, GifError> {
		Ok(PlainTextExtension {
			block_size: reader.read_u8()?,
			text_x: reader.read_u16::<LittleEndian>()?,
			text_y: reader.read_u16::<LittleEndian>()?,
			text_width: reader.read_u16::<LittleEndian>()?,
			text_height: reader.read_u16::<LittleEndian>()?,
			cell_width: reader.read_u8()?,
			cell_height: reader.read_u8()?,
			foreground_color: reader.read_u8()?,
			background_color: reader.read_u8()?,
			data: read_raw_sub_block_data(reader)?,
		})
	}

	pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), GifError> {
		writer.write_u8(self.block_size)?;
		writer.write_u16::<LittleEndian>(self.text_x)?;
		writer.write_u16::<LittleEndian>(self.text_y)?;
		writer.write_u16::<LittleEndian>(self.text_width)?;
		writer.write_u16::<LittleEndian>(self.text_height)?;
		writer.write_u8(self.cell_width)?;
		writer.write_u8(self.cell_height)?;
		writer.write_u8(self.foreground_color)?;
		writer.write_u8(self.background_color)?;
		write_raw_sub_block_data(&self.data, writer)?;
		Ok(())
	}
}

#[derive(Debug, Clone)]
struct ApplicationExtension {
	block_size: u8,
	identifier: [u8; 8],
	authentication_code: [u8; 3],
	data: Box<[u8]>,
}

#[allow(dead_code)]
impl ApplicationExtension {
	pub fn read<T: Read>(reader: &mut T) -> Result<Self, GifError> {
		let block_size = reader.read_u8()?;
		let mut identifier = [0u8; 8];
		reader.read_exact(&mut identifier)?;
		let mut authentication_code = [0u8; 3];
		reader.read_exact(&mut authentication_code)?;
		Ok(ApplicationExtension {
			block_size,
			identifier,
			authentication_code,
			data: read_raw_sub_block_data(reader)?,
		})
	}

	pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), GifError> {
		writer.write_u8(self.block_size)?;
		writer.write_all(&self.identifier)?;
		writer.write_all(&self.authentication_code)?;
		write_raw_sub_block_data(&self.data, writer)?;
		Ok(())
	}
}

#[derive(Debug, Clone)]
struct CommentExtension {
	data: Box<[u8]>,
}

#[allow(dead_code)]
impl CommentExtension {
	pub fn read<T: Read>(reader: &mut T) -> Result<Self, GifError> {
		Ok(CommentExtension {
			data: read_raw_sub_block_data(reader)?,
		})
	}

	pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), GifError> {
		write_raw_sub_block_data(&self.data, writer)?;
		Ok(())
	}
}

#[derive(Debug, Clone)]
struct LocalImageDescriptor {
	x: u16,
	y: u16,
	width: u16,
	height: u16,
	flags: u8,
}

#[allow(dead_code)]
impl LocalImageDescriptor {
	pub fn has_local_color_table(&self) -> bool {
		self.flags & 0b10000000 != 0
	}

	pub fn set_local_color_table(&mut self, value: bool) {
		self.flags |= (value as u8).wrapping_shl(7);
	}

	pub fn is_color_table_entries_sorted(&self) -> bool {
		self.flags & 0b00100000 != 0
	}

	pub fn set_color_table_entries_sorted(&mut self, value: bool) {
		self.flags |= (value as u8).wrapping_shl(5);
	}

	pub fn local_color_table_bits(&self) -> u8 {
		self.flags & 0b00000111
	}

	pub fn set_local_color_table_bits(&mut self, value: u8) {
		self.flags |= value & 0b111;
	}

	pub fn read<T: Read>(reader: &mut T) -> Result<Self, GifError> {
		Ok(LocalImageDescriptor {
			x: reader.read_u16::<LittleEndian>()?,
			y: reader.read_u16::<LittleEndian>()?,
			width: reader.read_u16::<LittleEndian>()?,
			height: reader.read_u16::<LittleEndian>()?,
			flags: reader.read_u8()?,
		})
	}

	pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), GifError> {
		writer.write_u16::<LittleEndian>(self.x)?;
		writer.write_u16::<LittleEndian>(self.y)?;
		writer.write_u16::<LittleEndian>(self.width)?;
		writer.write_u16::<LittleEndian>(self.height)?;
		writer.write_u8(self.flags)?;
		Ok(())
	}
}

fn load_image_section<T: ReadBytesExt>(
	reader: &mut T,
	gif_header: &GifHeader,
	_graphic_control: &Option<GraphicControlExtension>,
) -> Result<(Bitmap, Option<Palette>), GifError> {
	let descriptor = LocalImageDescriptor::read(reader)?;

	let palette: Option<Palette>;
	if descriptor.has_local_color_table() {
		let num_colors = bits_to_num_colors(descriptor.local_color_table_bits() as u32) as usize;
		palette = Some(Palette::load_num_colors_from_bytes(
			reader,
			PaletteFormat::Normal,
			num_colors,
		)?);
	} else {
		palette = None; // we expect that there was a global color table previously
	}

	let mut bitmap = Bitmap::new(gif_header.screen_width as u32, gif_header.screen_height as u32).unwrap();
	let mut writer = bitmap.pixels_mut();
	lzw_decode(reader, &mut writer)?;

	Ok((bitmap, palette))
}

fn save_image_section<T: WriteBytesExt>(
	writer: &mut T,
	bitmap: &Bitmap,
) -> Result<(), GifError> {
	writer.write_u8(IMAGE_DESCRIPTOR_SEPARATOR)?;
	let image_descriptor = LocalImageDescriptor {
		x: 0,
		y: 0,
		width: bitmap.width as u16,
		height: bitmap.height as u16,
		flags: 0,  // again, we're not using local color tables, so no flags to set here
	};
	image_descriptor.write(writer)?;

	// todo: allow this to changed based on the input palette, if/when we allow gifs to be
	//       saved with smaller than 256 colour palettes
	let lzw_minimum_code_size = 8;

	let mut reader = bitmap.pixels();
	lzw_encode(&mut reader, writer, lzw_minimum_code_size as usize)?;

	Ok(())
}

impl Bitmap {
	pub fn load_gif_bytes<T: ReadBytesExt>(
		reader: &mut T,
	) -> Result<(Bitmap, Palette), GifError> {
		let header = GifHeader::read(reader)?;
		if header.signature != *b"GIF" || header.version != *b"89a" {
			return Err(GifError::BadFile(String::from("Expected GIF89a header signature")));
		}

		// note that we might later overwrite this with a local color table (if this gif has one)
		let mut palette: Option<Palette>;
		if header.has_global_color_table() {
			let num_colors = bits_to_num_colors(header.global_color_table_bits() as u32) as usize;
			palette = Some(Palette::load_num_colors_from_bytes(
				reader,
				PaletteFormat::Normal,
				num_colors,
			)?);
		} else {
			palette = None; // we expect to find a local color table later
		}

		let mut bitmap: Option<Bitmap> = None;
		let mut current_graphic_control: Option<GraphicControlExtension> = None;

		loop {
			let current_byte = reader.read_u8()?;

			// check for eof via the gif's "trailer" block ...
			if current_byte == 0x3b {
				break;
			}
			// if we have already successfully read a bitmap and palette from this file, we can
			// stop reading the rest. we only care about the first frame (if there are multiple)
			// and palette we find
			if bitmap.is_some() && palette.is_some() {
				break;
			}

			match current_byte {
				GIF_TRAILER => break,
				IMAGE_DESCRIPTOR_SEPARATOR => {
					let (frame_bitmap, frame_palette) = load_image_section(reader, &header, &current_graphic_control)?;
					bitmap = Some(frame_bitmap);
					if frame_palette.is_some() {
						palette = frame_palette;
					}
				}
				EXTENSION_INTRODUCER => {
					let label = GifExtensionLabel::from(reader.read_u8()?)?;
					match label {
						GifExtensionLabel::GraphicControl => {
							current_graphic_control = Some(GraphicControlExtension::read(reader)?);
						}
						GifExtensionLabel::PlainText => {
							let _plain_text = PlainTextExtension::read(reader)?;
							// todo: do something with this maybe
						}
						GifExtensionLabel::Application => {
							let _application = ApplicationExtension::read(reader)?;
							// todo: do something with this maybe
						}
						GifExtensionLabel::Comment => {
							let _comment = CommentExtension::read(reader)?;
							// todo: do something with this maybe
						}
					}
				}
				_ => {
					return Err(GifError::BadFile(format!("Unexpected byte found {} not a file trailer, image separator or extension introducer", current_byte)));
				}
			}
		}

		if bitmap.is_none() {
			return Err(GifError::BadFile(String::from("No image data was found")));
		}
		if palette.is_none() {
			return Err(GifError::BadFile(String::from("No palette data was found")));
		}

		Ok((bitmap.unwrap(), palette.unwrap()))
	}

	pub fn load_gif_file(path: &Path) -> Result<(Bitmap, Palette), GifError> {
		let f = File::open(path)?;
		let mut reader = BufReader::new(f);
		Self::load_gif_bytes(&mut reader)
	}

	pub fn to_gif_bytes<T: WriteBytesExt>(
		&self,
		writer: &mut T,
		palette: &Palette,
		settings: GifSettings,
	) -> Result<(), GifError> {
		let mut header = GifHeader {
			signature: *b"GIF",
			version: *b"89a",
			screen_width: self.width as u16,
			screen_height: self.height as u16,
			flags: 0,
			background_color: 0,
			aspect_ratio: 0,
		};
		header.set_global_color_table(true);
		header.set_global_color_table_bits(BITS_FOR_256_COLORS as u8);
		header.set_color_resolution_bits(BITS_FOR_256_COLORS as u8);
		header.write(writer)?;

		// write the provided palette out as the global color table. we will not be providing any
		// local color tables.
		palette.to_bytes(writer, PaletteFormat::Normal)?;

		let transparent_color: u8;
		match settings {
			GifSettings::Default => {
				transparent_color = 0;
			}
			GifSettings::TransparentColor(color) => {
				transparent_color = color;
			}
		}

		writer.write_u8(EXTENSION_INTRODUCER)?;
		writer.write_u8(GifExtensionLabel::GraphicControl as u8)?;
		let graphic_control = GraphicControlExtension {
			block_size: 4,
			flags: 0,
			delay: 0,
			transparent_color,
			terminator: 0,
		};
		graphic_control.write(writer)?;

		save_image_section(writer, &self)?;

		writer.write_u8(GIF_TRAILER)?;
		Ok(())
	}

	pub fn to_gif_file(
		&self,
		path: &Path,
		palette: &Palette,
		settings: GifSettings,
	) -> Result<(), GifError> {
		let f = File::create(path)?;
		let mut writer = BufWriter::new(f);
		self.to_gif_bytes(&mut writer, palette, settings)
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
	fn load_and_save() -> Result<(), GifError> {
		let dp2_palette =
			Palette::load_from_file(Path::new("./test-assets/dp2.pal"), PaletteFormat::Normal)
				.unwrap();
		let tmp_dir = TempDir::new()?;

		let (bmp, palette) = Bitmap::load_gif_file(Path::new("./test-assets/test.gif"))?;
		assert_eq!(16, bmp.width());
		assert_eq!(16, bmp.height());
		assert_eq!(bmp.pixels(), TEST_BMP_PIXELS_RAW);
		assert_eq!(palette, dp2_palette);

		let save_path = tmp_dir.path().join("test_save.gif");
		bmp.to_gif_file(&save_path, &palette, GifSettings::Default)?;
		let (reloaded_bmp, reloaded_palette) = Bitmap::load_gif_file(&save_path)?;
		assert_eq!(16, reloaded_bmp.width());
		assert_eq!(16, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels(), TEST_BMP_PIXELS_RAW);
		assert_eq!(reloaded_palette, dp2_palette);

		Ok(())
	}

	#[test]
	fn load_and_save_larger_image() -> Result<(), GifError> {
		// this test is mostly useful to get a LZW decode and encode that includes at least one
		// "clear code" and accompanying table reset

		let tmp_dir = TempDir::new()?;

		// first image

		let (bmp, palette) = Bitmap::load_gif_file(Path::new("./test-assets/test_image.gif"))?;
		assert_eq!(320, bmp.width());
		assert_eq!(200, bmp.height());
		assert_eq!(bmp.pixels(), TEST_LARGE_BMP_PIXELS_RAW);

		let save_path = tmp_dir.path().join("test_save.gif");
		bmp.to_gif_file(&save_path, &palette, GifSettings::Default)?;
		let (reloaded_bmp, _) = Bitmap::load_gif_file(&save_path)?;
		assert_eq!(320, reloaded_bmp.width());
		assert_eq!(200, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels(), TEST_LARGE_BMP_PIXELS_RAW);

		// second image

		let (bmp, palette) = Bitmap::load_gif_file(Path::new("./test-assets/test_image2.gif"))?;
		assert_eq!(320, bmp.width());
		assert_eq!(200, bmp.height());
		assert_eq!(bmp.pixels(), TEST_LARGE_BMP_PIXELS_RAW_2);

		let save_path = tmp_dir.path().join("test_save_2.gif");
		bmp.to_gif_file(&save_path, &palette, GifSettings::Default)?;
		let (reloaded_bmp, _) = Bitmap::load_gif_file(&save_path)?;
		assert_eq!(320, reloaded_bmp.width());
		assert_eq!(200, reloaded_bmp.height());
		assert_eq!(reloaded_bmp.pixels(), TEST_LARGE_BMP_PIXELS_RAW_2);

		Ok(())
	}
}
