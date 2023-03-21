use std::path::Path;

use crate::graphics::bitmap::{Bitmap, BitmapError};
use crate::graphics::palette::Palette;

pub mod blit;

pub type RgbaBitmap = Bitmap<u32>;

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
		Self::internal_new(width, height)
	}

	pub fn load_file(path: &Path) -> Result<(Self, Option<Palette>), BitmapError> {
		if let Some(extension) = path.extension() {
			let extension = extension.to_ascii_lowercase();
			match extension.to_str() {
				Some("png") => Ok(Self::load_png_file(path)?),
				Some("pcx") => {
					let (bmp, palette) = Self::load_pcx_file(path)?;
					Ok((bmp, Some(palette)))
				},
				Some("gif") => {
					let (bmp, palette) = Self::load_gif_file(path)?;
					Ok((bmp, Some(palette)))
				},
				Some("iff") | Some("lbm") | Some("pbm") | Some("bbm") => {
					let (bmp, palette) = Self::load_iff_file(path)?;
					Ok((bmp, Some(palette)))
				}
				_ => Err(BitmapError::UnknownFileType(String::from(
					"Unrecognized file extension",
				))),
			}
		} else {
			Err(BitmapError::UnknownFileType(String::from(
				"No file extension",
			)))
		}
	}
}
