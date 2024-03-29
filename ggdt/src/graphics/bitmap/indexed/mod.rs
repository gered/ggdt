use std::path::Path;

use crate::graphics::{Bitmap, BitmapError, Palette, RgbaBitmap, RGBA};

mod blit;
mod primitives;
mod triangles;

pub use blit::*;
pub use primitives::*;
pub use triangles::*;

pub type IndexedBitmap = Bitmap<u8>;

impl IndexedBitmap {
	/// Creates a new Bitmap with the specified dimensions.
	///
	/// # Arguments
	///
	/// * `width`: the width of the bitmap in pixels
	/// * `height`: the height of the bitmap in pixels
	///
	/// returns: `Result<Bitmap, BitmapError>`
	pub fn new(width: u32, height: u32) -> Result<Self, BitmapError> {
		Self::internal_new(width, height, 0)
	}

	pub fn load_file(path: &Path) -> Result<(Self, Palette), BitmapError> {
		if let Some(extension) = path.extension() {
			let extension = extension.to_ascii_lowercase();
			match extension.to_str() {
				Some("png") => {
					let (bmp, palette) = Self::load_png_file(path)?;
					Ok((bmp, palette.expect("Indexed color PNG loaded and should have returned a Palette")))
				}
				Some("pcx") => Ok(Self::load_pcx_file(path)?),
				Some("gif") => Ok(Self::load_gif_file(path)?),
				Some("iff") | Some("lbm") | Some("pbm") | Some("bbm") => Ok(Self::load_iff_file(path)?),
				_ => Err(BitmapError::UnknownFileType(String::from("Unrecognized file extension"))),
			}
		} else {
			Err(BitmapError::UnknownFileType(String::from("No file extension")))
		}
	}

	/// Copies and converts the entire pixel data from this bitmap to a destination expecting
	/// 32-bit RGBA-format pixel data. This can be used to display the contents of the bitmap
	/// on-screen by using an SDL Surface, OpenGL texture, etc as the destination.
	///
	/// # Arguments
	///
	/// * `dest`: destination 32-bit RGBA pixel buffer to copy converted pixels to
	/// * `palette`: the 256 colour palette to use during pixel conversion
	pub fn copy_as_rgba_to(&self, dest: &mut [RGBA], palette: &Palette) {
		for (src, dest) in self.pixels().iter().zip(dest.iter_mut()) {
			*dest = palette[*src];
		}
	}

	/// Makes a [`RgbaBitmap`] copy of this bitmap, using the specified 256 colour palette during
	/// the pixel format conversion.
	///
	/// # Arguments
	///
	/// * `palette`: the 256 colour palette to use during pixel conversion
	///
	/// returns: `RgbaBitmap`
	pub fn to_rgba(&self, palette: &Palette) -> RgbaBitmap {
		let mut output = RgbaBitmap::new(self.width, self.height).unwrap();
		self.copy_as_rgba_to(output.pixels_mut(), palette);
		output
	}
}
