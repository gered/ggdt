use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor};
use std::ops::{Index, IndexMut};
use std::path::Path;

use byteorder::{ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::graphics::{Bitmap, Pixel};
use crate::math::Rect;

pub static VGA_FONT_BYTES: &[u8] = include_bytes!("../../assets/vga.fnt");

pub const NUM_CHARS: usize = 256;
pub const CHAR_HEIGHT: usize = 8;
pub const CHAR_FIXED_WIDTH: usize = 8;

#[derive(Error, Debug)]
pub enum FontError {
	#[error("Invalid font file: {0}")]
	InvalidFile(String),

	#[error("Font I/O error")]
	IOError(#[from] std::io::Error),

	#[error("Invalid character dimensions")]
	InvalidCharacterDimensions,

	#[error("Invalid number of characters")]
	InvalidNumberOfCharacters,

	#[error("Invalid line height")]
	InvalidLineHeight,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FontRenderOpts<PixelType: Pixel> {
	Color(PixelType),
	None,
}

pub trait Character {
	fn bounds(&self) -> &Rect;
	fn draw<PixelType>(&self, dest: &mut Bitmap<PixelType>, x: i32, y: i32, opts: FontRenderOpts<PixelType>)
	where
		PixelType: Pixel;
}

pub trait Font {
	type CharacterType: Character;

	fn character(&self, ch: char) -> &Self::CharacterType;
	fn space_width(&self) -> u8;
	fn line_height(&self) -> u8;
	fn measure<PixelType>(&self, text: &str, opts: FontRenderOpts<PixelType>) -> (u32, u32)
	where
		PixelType: Pixel;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BitmaskCharacter {
	bytes: [u8; CHAR_HEIGHT],
	bounds: Rect,
}

impl Default for BitmaskCharacter {
	fn default() -> Self {
		BitmaskCharacter {
			//
			bytes: [0u8; CHAR_HEIGHT],
			bounds: Rect::new(0, 0, CHAR_FIXED_WIDTH as u32, CHAR_HEIGHT as u32),
		}
	}
}

impl Index<u8> for BitmaskCharacter {
	type Output = u8;

	#[inline]
	fn index(&self, index: u8) -> &Self::Output {
		&self.bytes[index as usize]
	}
}

impl IndexMut<u8> for BitmaskCharacter {
	#[inline]
	fn index_mut(&mut self, index: u8) -> &mut Self::Output {
		&mut self.bytes[index as usize]
	}
}

impl Character for BitmaskCharacter {
	#[inline]
	fn bounds(&self) -> &Rect {
		&self.bounds
	}

	fn draw<PixelType>(&self, dest: &mut Bitmap<PixelType>, x: i32, y: i32, opts: FontRenderOpts<PixelType>)
	where
		PixelType: Pixel,
	{
		// out of bounds check
		if ((x + self.bounds.width as i32) < dest.clip_region().x)
			|| ((y + self.bounds.height as i32) < dest.clip_region().y)
			|| (x >= dest.clip_region().right())
			|| (y >= dest.clip_region().bottom())
		{
			return;
		}

		let color = match opts {
			FontRenderOpts::Color(color) => color,
			// this kind of highlights a weakness of this design i guess. what does it mean to render a BitmaskFont,
			// which has no inherent colour information in it, when there is no specific render colour passed in?
			// TODO: is it better to return an error here? should a BitmaskFont have a "default colour" to fall back to?
			//       or, should a Bitmap have a "default colour" property we could fall back to? not sure!
			_ => return,
		};

		// TODO: i'm sure this can be optimized, lol
		for char_y in 0..self.bounds.height as usize {
			let mut bit_mask = 0x80;
			for char_x in 0..self.bounds.width as usize {
				if self.bytes[char_y] & bit_mask > 0 {
					dest.set_pixel(x + char_x as i32, y + char_y as i32, color);
				}
				bit_mask >>= 1;
			}
		}
	}
}

impl BitmaskCharacter {
	pub fn new(bytes: [u8; CHAR_HEIGHT], width: usize) -> Result<BitmaskCharacter, FontError> {
		if width < 1 || width > CHAR_FIXED_WIDTH {
			return Err(FontError::InvalidCharacterDimensions);
		}

		Ok(BitmaskCharacter { bytes, bounds: Rect::new(0, 0, width as u32, CHAR_HEIGHT as u32) })
	}

	#[inline]
	pub fn width(&self) -> u8 {
		self.bounds.width as u8
	}

	pub fn set_width(&mut self, width: u8) -> Result<(), FontError> {
		if width < 1 || width > CHAR_FIXED_WIDTH as u8 {
			return Err(FontError::InvalidCharacterDimensions);
		}
		self.bounds.width = width as u32;
		Ok(())
	}

	#[inline]
	pub fn height(&self) -> u8 {
		self.bounds.height as u8
	}

	pub fn set_height(&mut self, height: u8) -> Result<(), FontError> {
		if height < 1 || height > CHAR_HEIGHT as u8 {
			return Err(FontError::InvalidCharacterDimensions);
		}
		self.bounds.height = height as u32;
		Ok(())
	}
}

#[derive(Clone, Eq, PartialEq)]
pub struct BitmaskFont {
	characters: Box<[BitmaskCharacter]>,
	line_height: u8,
	space_width: u8,
}

impl std::fmt::Debug for BitmaskFont {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("BitmaskFont") //
			.field("line_height", &self.line_height)
			.field("space_width", &self.space_width)
			.field("characters.len()", &self.characters.len())
			.finish()
	}
}

impl BitmaskFont {
	pub fn new_vga_font() -> Result<BitmaskFont, FontError> {
		BitmaskFont::load_from_bytes(&mut Cursor::new(VGA_FONT_BYTES))
	}

	pub fn new(characters: &[BitmaskCharacter], line_height: usize) -> Result<BitmaskFont, FontError> {
		if characters.len() != NUM_CHARS {
			return Err(FontError::InvalidNumberOfCharacters);
		}
		if line_height < 1 || line_height > CHAR_HEIGHT {
			return Err(FontError::InvalidLineHeight);
		}

		let mut font = BitmaskFont {
			characters: Box::from(characters),
			line_height: line_height as u8,
			space_width: characters[' ' as usize].bounds.width as u8,
		};

		for i in 0..NUM_CHARS {
			font.characters[i].bounds.height = line_height as u32;
		}

		Ok(font)
	}

	pub fn load_from_file(path: &Path) -> Result<BitmaskFont, FontError> {
		let f = File::open(path)?;
		let mut reader = BufReader::new(f);

		BitmaskFont::load_from_bytes(&mut reader)
	}

	pub fn load_from_bytes<T: ReadBytesExt>(reader: &mut T) -> Result<BitmaskFont, FontError> {
		let mut characters: Vec<BitmaskCharacter> = Vec::with_capacity(NUM_CHARS);

		// read character bitmap data
		for _ in 0..NUM_CHARS {
			let mut buffer = [0u8; CHAR_HEIGHT];
			reader.read_exact(&mut buffer)?;
			let character = BitmaskCharacter {
				bytes: buffer,
				// bounds are filled in below. ugh.
				bounds: Rect { x: 0, y: 0, width: 0, height: 0 },
			};
			characters.push(character);
		}

		// read character widths (used for rendering)
		for i in 0..NUM_CHARS {
			characters[i].bounds.width = reader.read_u8()? as u32;
		}

		// read global font height (used for rendering)
		let line_height = reader.read_u8()?;

		Self::new(&characters, line_height as usize)
	}

	pub fn to_file(&self, path: &Path) -> Result<(), FontError> {
		let f = File::create(path)?;
		let mut writer = BufWriter::new(f);
		self.to_bytes(&mut writer)
	}

	pub fn to_bytes<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), FontError> {
		// write character bitmap data
		for i in 0..NUM_CHARS {
			writer.write_all(&self.characters[i].bytes)?;
		}

		// write character widths
		for i in 0..NUM_CHARS {
			writer.write_u8(self.characters[i].bounds.width as u8)?;
		}

		// write global font height
		writer.write_u8(self.line_height)?;

		Ok(())
	}
}

impl Font for BitmaskFont {
	type CharacterType = BitmaskCharacter;

	#[inline]
	fn character(&self, ch: char) -> &Self::CharacterType {
		&self.characters[ch as usize]
	}

	#[inline]
	fn space_width(&self) -> u8 {
		self.space_width
	}

	#[inline]
	fn line_height(&self) -> u8 {
		self.line_height
	}

	fn measure<PixelType>(&self, text: &str, _opts: FontRenderOpts<PixelType>) -> (u32, u32)
	where
		PixelType: Pixel,
	{
		if text.is_empty() {
			return (0, 0);
		}
		let mut height = 0;
		let mut width = 0;
		let mut x = 0;
		// trimming whitespace off the end because it won't be rendered (since it's whitespace)
		// and thus, won't contribute to visible rendered output (what we're measuring)
		for ch in text.trim_end().chars() {
			match ch {
				'\n' => {
					if x == 0 {
						height += self.line_height as u32;
					}
					width = std::cmp::max(width, x);
					x = 0;
				}
				'\r' => (),
				ch => {
					if x == 0 {
						height += self.line_height as u32;
					}
					x += self.character(ch).bounds().width;
				}
			}
		}
		width = std::cmp::max(width, x);
		(width, height)
	}
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use super::*;

	const BASE_PATH: &str = "./test-assets/font/";

	fn test_file(file: &Path) -> PathBuf {
		PathBuf::from(BASE_PATH).join(file)
	}

	#[test]
	pub fn load_font() -> Result<(), FontError> {
		let font = BitmaskFont::load_from_file(test_file(Path::new("vga.fnt")).as_path())?;
		assert_eq!(256, font.characters.len());
		assert_eq!(CHAR_FIXED_WIDTH as u8, font.space_width);
		for character in font.characters.iter() {
			assert_eq!(CHAR_FIXED_WIDTH as u8, character.bounds.width as u8);
			assert_eq!(CHAR_HEIGHT, character.bytes.len());
		}

		Ok(())
	}

	#[test]
	pub fn measure_text() -> Result<(), FontError> {
		{
			let font = BitmaskFont::load_from_file(test_file(Path::new("vga.fnt")).as_path())?;

			assert_eq!((40, 8), font.measure("Hello", FontRenderOpts::<u8>::None));
			assert_eq!((40, 16), font.measure("Hello\nthere", FontRenderOpts::<u8>::None));
			assert_eq!((88, 24), font.measure("longer line\nshort\nthe end", FontRenderOpts::<u8>::None));
			assert_eq!((0, 0), font.measure("", FontRenderOpts::<u8>::None));
			assert_eq!((0, 0), font.measure(" ", FontRenderOpts::<u8>::None));
			assert_eq!((40, 16), font.measure("\nhello", FontRenderOpts::<u8>::None));
			assert_eq!((0, 0), font.measure("\n", FontRenderOpts::<u8>::None));
			assert_eq!((40, 8), font.measure("hello\n", FontRenderOpts::<u8>::None));
			assert_eq!((40, 24), font.measure("hello\n\nthere", FontRenderOpts::<u8>::None));
		}

		{
			let font = BitmaskFont::load_from_file(test_file(Path::new("small.fnt")).as_path())?;

			assert_eq!((22, 7), font.measure("Hello", FontRenderOpts::<u8>::None));
			assert_eq!((24, 14), font.measure("Hello\nthere", FontRenderOpts::<u8>::None));
			assert_eq!((50, 21), font.measure("longer line\nshort\nthe end", FontRenderOpts::<u8>::None));
			assert_eq!((0, 0), font.measure("", FontRenderOpts::<u8>::None));
			assert_eq!((0, 0), font.measure(" ", FontRenderOpts::<u8>::None));
			assert_eq!((21, 14), font.measure("\nhello", FontRenderOpts::<u8>::None));
			assert_eq!((0, 0), font.measure("\n", FontRenderOpts::<u8>::None));
			assert_eq!((21, 7), font.measure("hello\n", FontRenderOpts::<u8>::None));
			assert_eq!((24, 21), font.measure("hello\n\nthere", FontRenderOpts::<u8>::None));
		}

		Ok(())
	}
}
