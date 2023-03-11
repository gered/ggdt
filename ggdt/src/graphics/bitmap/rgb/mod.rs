use crate::graphics::bitmap::{Bitmap, BitmapError};

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
}
