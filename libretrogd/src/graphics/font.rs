use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor};
use std::path::Path;

use byteorder::{ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::{Bitmap, Rect};

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
}

#[derive(Debug)]
pub enum FontRenderOpts {
    Color(u8),
    None,
}

pub trait Character {
    fn bounds(&self) -> &Rect;
    fn draw(&self, dest: &mut Bitmap, x: i32, y: i32, opts: FontRenderOpts);
}

pub trait Font {
    type CharacterType: Character;

    fn character(&self, ch: char) -> &Self::CharacterType;
    fn space_width(&self) -> u8;
    fn line_height(&self) -> u8;
}

#[derive(Debug)]
pub struct BitmaskCharacter {
    bytes: [u8; CHAR_HEIGHT],
    bounds: Rect,
}

impl Character for BitmaskCharacter {
    #[inline]
    fn bounds(&self) -> &Rect {
        &self.bounds
    }

    fn draw(&self, dest: &mut Bitmap, x: i32, y: i32, opts: FontRenderOpts) {
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
            _ => 0,
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

#[derive(Debug)]
pub struct BitmaskFont {
    characters: Box<[BitmaskCharacter]>,
    line_height: u8,
    space_width: u8,
}

impl BitmaskFont {
    pub fn new_vga_font() -> Result<BitmaskFont, FontError> {
        BitmaskFont::load_from_bytes(&mut Cursor::new(VGA_FONT_BYTES))
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
                bounds: Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
            };
            characters.push(character);
        }

        // read character widths (used for rendering)
        for i in 0..NUM_CHARS {
            characters[i].bounds.width = reader.read_u8()? as u32;
        }

        // read global font height (used for rendering)
        let line_height = reader.read_u8()?;
        for i in 0..NUM_CHARS {
            characters[i].bounds.height = line_height as u32;
        }

        let space_width = characters[' ' as usize].bounds.width as u8;

        Ok(BitmaskFont {
            characters: characters.into_boxed_slice(),
            line_height,
            space_width,
        })
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
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn load_font() -> Result<(), FontError> {
        let font = BitmaskFont::load_from_file(Path::new("./assets/vga.fnt"))?;
        assert_eq!(256, font.characters.len());
        assert_eq!(CHAR_FIXED_WIDTH as u8, font.space_width);
        for character in font.characters.iter() {
            assert_eq!(CHAR_FIXED_WIDTH as u8, character.bounds.width as u8);
            assert_eq!(CHAR_HEIGHT, character.bytes.len());
        }

        Ok(())
    }
}
