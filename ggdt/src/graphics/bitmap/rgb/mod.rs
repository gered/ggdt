use byteorder::ReadBytesExt;
use std::path::Path;

use crate::graphics::bitmap::{Bitmap, BitmapError};
use crate::graphics::color::{to_argb32, to_rgb32};
use crate::graphics::palette::Palette;

pub mod blit;
pub mod primitives;

pub type RgbaBitmap = Bitmap<u32>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RgbaPixelFormat {
	ARGB,
	RGBA,
}

impl RgbaBitmap {
	/// Creates a new Bitmap with the specified dimensions.
	///
	/// # Arguments
	///
	/// * `width`: the width of the bitmap in pixels
	/// * `height`: the height of the bitmap in pixels
	///
	/// returns: `Result<Bitmap, BitmapError>`
	pub fn new(width: u32, height: u32) -> Result<Self, BitmapError> {
		Self::internal_new(width, height, to_rgb32(0, 0, 0))
	}

	pub fn from_bytes<T: ReadBytesExt>(
		width: u32,
		height: u32,
		format: RgbaPixelFormat,
		reader: &mut T,
	) -> Result<Self, BitmapError> {
		let mut bitmap = Self::internal_new(width, height, 0)?;
		for pixel in bitmap.pixels_mut().iter_mut() {
			*pixel = match format {
				RgbaPixelFormat::RGBA => {
					let r = reader.read_u8()?;
					let g = reader.read_u8()?;
					let b = reader.read_u8()?;
					let a = reader.read_u8()?;
					to_argb32(a, r, g, b)
				}
				RgbaPixelFormat::ARGB => {
					let a = reader.read_u8()?;
					let r = reader.read_u8()?;
					let g = reader.read_u8()?;
					let b = reader.read_u8()?;
					to_argb32(a, r, g, b)
				}
			};
		}
		Ok(bitmap)
	}

	pub fn load_file(path: &Path) -> Result<(Self, Option<Palette>), BitmapError> {
		if let Some(extension) = path.extension() {
			let extension = extension.to_ascii_lowercase();
			match extension.to_str() {
				Some("png") => Ok(Self::load_png_file(path)?),
				Some("pcx") => {
					let (bmp, palette) = Self::load_pcx_file(path)?;
					Ok((bmp, Some(palette)))
				}
				Some("gif") => {
					let (bmp, palette) = Self::load_gif_file(path)?;
					Ok((bmp, Some(palette)))
				}
				Some("iff") | Some("lbm") | Some("pbm") | Some("bbm") => {
					let (bmp, palette) = Self::load_iff_file(path)?;
					Ok((bmp, Some(palette)))
				}
				_ => Err(BitmapError::UnknownFileType(String::from("Unrecognized file extension"))),
			}
		} else {
			Err(BitmapError::UnknownFileType(String::from("No file extension")))
		}
	}
}
