use crate::graphics::{BlendMap, IndexedBitmap};

impl IndexedBitmap {
	/// Sets the pixel at the given coordinates using a blended color via the specified blend map,
	/// or using the color specified if the blend map does not include the given color. If the
	/// coordinates lie outside of the bitmaps clipping region, no pixels will be changed.
	#[inline]
	pub fn set_blended_pixel(&mut self, x: i32, y: i32, color: u8, blend_map: &BlendMap) {
		self.set_custom_pixel(
			x, //
			y,
			|dest_color| {
				if let Some(blended_color) = blend_map.blend(color, dest_color) {
					blended_color
				} else {
					color
				}
			},
		);
	}

	/// Sets the pixel at the given coordinates using a blended color via the specified blend map,
	/// or using the color specified if the blend map does not include the given color. The
	/// coordinates are not checked for validity, so it is up to you to ensure they lie within the
	/// bounds of the bitmap.
	#[inline]
	pub unsafe fn set_blended_pixel_unchecked(&mut self, x: i32, y: i32, color: u8, blend_map: &BlendMap) {
		self.set_custom_pixel_unchecked(
			x, //
			y,
			|dest_color| {
				if let Some(blended_color) = blend_map.blend(color, dest_color) {
					blended_color
				} else {
					color
				}
			},
		);
	}

	/// Draws a line from x1,y1 to x2,y2 by blending the drawn pixels using the given blend map,
	/// or the color specified if the blend map does not include this color.
	pub fn blended_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u8, blend_map: &BlendMap) {
		if let Some(blend_mapping) = blend_map.get_mapping(color) {
			self.line_custom(
				x1, //
				y1,
				x2,
				y2,
				|dest_color| blend_mapping[dest_color as usize],
			);
		} else {
			self.line(x1, y1, x2, y2, color);
		}
	}

	/// Draws a horizontal line from x1,y to x2,y by blending the drawn pixels using the given
	/// blend map, or the color specified if the blend map does not include this color.
	pub fn blended_horiz_line(&mut self, x1: i32, x2: i32, y: i32, color: u8, blend_map: &BlendMap) {
		if let Some(blend_mapping) = blend_map.get_mapping(color) {
			self.horiz_line_custom(
				x1, //
				x2,
				y,
				|dest_color| blend_mapping[dest_color as usize],
			);
		} else {
			self.horiz_line(x1, x2, y, color);
		}
	}

	/// Draws a vertical line from x,y1 to x,y2 by blending the drawn pixels using the given blend
	/// map, or the color specified if the blend map does not include this color.
	pub fn blended_vert_line(&mut self, x: i32, y1: i32, y2: i32, color: u8, blend_map: &BlendMap) {
		if let Some(blend_mapping) = blend_map.get_mapping(color) {
			self.vert_line_custom(
				x, //
				y1,
				y2,
				|dest_color| blend_mapping[dest_color as usize],
			);
		} else {
			self.vert_line(x, y1, y2, color);
		}
	}

	/// Draws an empty box (rectangle) using the points x1,y1 and x2,y2 to form the box to be
	/// drawn, assuming they are specifying the top-left and bottom-right corners respectively.
	/// The box is drawn by blending the drawn pixels using the given blend map, or the color
	/// specified if the blend map does not include this color.
	pub fn blended_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u8, blend_map: &BlendMap) {
		if let Some(blend_mapping) = blend_map.get_mapping(color) {
			self.rect_custom(
				x1, //
				y1,
				x2,
				y2,
				|dest_color| blend_mapping[dest_color as usize],
			);
		} else {
			self.rect(x1, y1, x2, y2, color);
		}
	}

	/// Draws a filled box (rectangle) using the points x1,y1 and x2,y2 to form the box to be
	/// drawn, assuming they are specifying the top-left and bottom-right corners respectively. The
	/// filled box is draw by blending the drawn pixels using the given blend map, or the color
	/// specified if the blend map does not include this color.
	pub fn blended_filled_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u8, blend_map: &BlendMap) {
		if let Some(blend_mapping) = blend_map.get_mapping(color) {
			self.filled_rect_custom(
				x1, //
				y1,
				x2,
				y2,
				|dest_color| blend_mapping[dest_color as usize],
			);
		} else {
			self.filled_rect(x1, y1, x2, y2, color);
		}
	}
}
