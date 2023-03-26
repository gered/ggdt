use crate::graphics::bitmap::rgb::RgbaBitmap;
use crate::graphics::color::BlendFunction;

impl RgbaBitmap {
	/// Sets the pixel at the given coordinates using a blended color via the specified blend function
	/// If the coordinates lie outside of the bitmaps clipping region, no pixels will be changed.
	#[inline]
	pub fn set_blended_pixel(&mut self, x: i32, y: i32, color: u32, blend: BlendFunction) {
		self.set_custom_pixel(
			x, y,
			|dest_color| {
				blend.blend(color, dest_color)
			}
		);
	}

	/// Sets the pixel at the given coordinates using a blended color via the specified blend function,
	/// The coordinates are not checked for validity, so it is up to you to ensure they lie within the
	/// bounds of the bitmap.
	#[inline]
	pub unsafe fn set_blended_pixel_unchecked(&mut self, x: i32, y: i32, color: u32, blend: BlendFunction) {
		self.set_custom_pixel_unchecked(
			x, y,
			|dest_color| {
				blend.blend(color, dest_color)
			}
		);
	}

	/// Draws a line from x1,y1 to x2,y2 by blending the drawn pixels using the given blend function.
	pub fn blended_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u32, blend: BlendFunction) {
		self.line_custom(
			x1, y1, x2, y2,
			|dest_color| {
				blend.blend(color, dest_color)
			}
		);
	}

	/// Draws a horizontal line from x1,y to x2,y by blending the drawn pixels using the given
	/// blend function.
	pub fn blended_horiz_line(&mut self, x1: i32, x2: i32, y: i32, color: u32, blend: BlendFunction) {
		self.horiz_line_custom(
			x1, x2, y,
			|dest_color| {
				blend.blend(color, dest_color)
			}
		);
	}

	/// Draws a vertical line from x,y1 to x,y2 by blending the drawn pixels using the given blend
	/// function.
	pub fn blended_vert_line(&mut self, x: i32, y1: i32, y2: i32, color: u32, blend: BlendFunction) {
		self.vert_line_custom(
			x, y1, y2,
			|dest_color| {
				blend.blend(color, dest_color)
			}
		);
	}

	/// Draws an empty box (rectangle) using the points x1,y1 and x2,y2 to form the box to be
	/// drawn, assuming they are specifying the top-left and bottom-right corners respectively.
	/// The box is drawn by blending the drawn pixels using the given blend function.
	pub fn blended_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u32, blend: BlendFunction) {
		self.rect_custom(
			x1, y1, x2, y2,
			|dest_color| {
				blend.blend(color, dest_color)
			}
		);
	}

	/// Draws a filled box (rectangle) using the points x1,y1 and x2,y2 to form the box to be
	/// drawn, assuming they are specifying the top-left and bottom-right corners respectively. The
	/// filled box is draw by blending the drawn pixels using the given blend function.
	pub fn blended_filled_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u32, blend: BlendFunction) {
		self.filled_rect_custom(
			x1, y1, x2, y2,
			|dest_color| {
				blend.blend(color, dest_color)
			}
		);
	}
}