use std::mem::swap;

use crate::graphics::bitmap::indexed::IndexedBitmap;
use crate::graphics::blendmap::BlendMap;
use crate::math::rect::Rect;

impl IndexedBitmap {
	/// Sets the pixel at the given coordinates using a blended color via the specified blend map,
	/// or using the color specified if the blend map does not include the given color. If the
	/// coordinates lie outside of the bitmaps clipping region, no pixels will be changed.
	#[inline]
	pub fn set_blended_pixel(&mut self, x: i32, y: i32, color: u8, blend_map: &BlendMap) {
		if let Some(pixels) = self.pixels_at_mut(x, y) {
			let dest_color = pixels[0];
			if let Some(blended_color) = blend_map.blend(color, dest_color) {
				pixels[0] = blended_color;
			} else {
				pixels[0] = color;
			}
		}
	}

	/// Sets the pixel at the given coordinates using a blended color via the specified blend map,
	/// or using the color specified if the blend map does not include the given color. The
	/// coordinates are not checked for validity, so it is up to you to ensure they lie within the
	/// bounds of the bitmap.
	#[inline]
	pub unsafe fn set_blended_pixel_unchecked(&mut self, x: i32, y: i32, color: u8, blend_map: &BlendMap) {
		let p = self.pixels_at_mut_ptr_unchecked(x, y);
		if let Some(blended_color) = blend_map.blend(color, *p) {
			*p = blended_color;
		} else {
			*p = color;
		}
	}

	/// Draws a line from x1,y1 to x2,y2 by blending the drawn pixels using the given blend map,
	/// or the color specified if the blend map does not include this color.
	pub fn blended_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u8, blend_map: &BlendMap) {
		if let Some(blend_mapping) = blend_map.get_mapping(color) {
			let mut dx = x1;
			let mut dy = y1;
			let delta_x = x2 - x1;
			let delta_y = y2 - y1;
			let delta_x_abs = delta_x.abs();
			let delta_y_abs = delta_y.abs();
			let delta_x_sign = delta_x.signum();
			let delta_y_sign = delta_y.signum();
			let mut x = delta_x_abs / 2;
			let mut y = delta_y_abs / 2;
			let offset_x_inc = delta_x_sign;
			let offset_y_inc = delta_y_sign * self.width as i32;

			unsafe {
				// safety: while we are blindly getting a pointer to this x/y coordinate, we don't
				// write to it unless we know the coordinates are in bounds.
				// TODO: should be ok ... ? or am i making too many assumptions about memory layout?
				let mut dest = self.pixels_at_mut_ptr_unchecked(x1, y1);

				if self.is_xy_visible(dx, dy) {
					*dest = blend_mapping[*dest as usize];
				}

				if delta_x_abs >= delta_y_abs {
					for _ in 0..delta_x_abs {
						y += delta_y_abs;

						if y >= delta_x_abs {
							y -= delta_x_abs;
							dy += delta_y_sign;
							dest = dest.offset(offset_y_inc as isize);
						}

						dx += delta_x_sign;
						dest = dest.offset(offset_x_inc as isize);

						if self.is_xy_visible(dx, dy) {
							*dest = blend_mapping[*dest as usize];
						}
					}
				} else {
					for _ in 0..delta_y_abs {
						x += delta_x_abs;

						if x >= delta_y_abs {
							x -= delta_y_abs;
							dx += delta_x_sign;
							dest = dest.offset(offset_x_inc as isize);
						}

						dy += delta_y_sign;
						dest = dest.offset(offset_y_inc as isize);

						if self.is_xy_visible(dx, dy) {
							*dest = blend_mapping[*dest as usize];
						}
					}
				}
			}
		} else {
			self.line(x1, y1, x2, y2, color);
		}
	}

	/// Draws a horizontal line from x1,y to x2,y by blending the drawn pixels using the given
	/// blend map, or the color specified if the blend map does not include this color.
	pub fn blended_horiz_line(&mut self, x1: i32, x2: i32, y: i32, color: u8, blend_map: &BlendMap) {
		if let Some(blend_mapping) = blend_map.get_mapping(color) {
			let mut region = Rect::from_coords(x1, y, x2, y);
			if region.clamp_to(&self.clip_region) {
				unsafe {
					let dest = self.pixels_at_mut_unchecked(region.x, region.y);
					for x in 0..region.width as usize {
						dest[x] = blend_mapping[dest[x] as usize];
					}
				}
			}
		} else {
			self.horiz_line(x1, x2, y, color);
		}
	}

	/// Draws a vertical line from x,y1 to x,y2 by blending the drawn pixels using the given blend
	/// map, or the color specified if the blend map does not include this color.
	pub fn blended_vert_line(&mut self, x: i32, y1: i32, y2: i32, color: u8, blend_map: &BlendMap) {
		if let Some(blend_mapping) = blend_map.get_mapping(color) {
			let mut region = Rect::from_coords(x, y1, x, y2);
			if region.clamp_to(&self.clip_region) {
				unsafe {
					let mut dest = self.pixels_at_mut_ptr_unchecked(region.x, region.y);
					for _ in 0..region.height {
						*dest = blend_mapping[*dest as usize];
						dest = dest.add(self.width as usize);
					}
				}
			}
		} else {
			self.vert_line(x, y1, y2, color);
		}
	}

	/// Draws an empty box (rectangle) using the points x1,y1 and x2,y2 to form the box to be
	/// drawn, assuming they are specifying the top-left and bottom-right corners respectively.
	/// The box is drawn by blending the drawn pixels using the given blend map, or the color
	/// specified if the blend map does not include this color.
	pub fn blended_rect(&mut self, mut x1: i32, mut y1: i32, mut x2: i32, mut y2: i32, color: u8, blend_map: &BlendMap) {
		if let Some(blend_mapping) = blend_map.get_mapping(color) {
			// note: need to manually do all this instead of just relying on Rect::from_coords (which
			// could otherwise figure all this out for us) mainly just because we need the post-swap
			// x1,y1,x2,y2 values for post-region-clamping comparison purposes ...
			if x2 < x1 {
				swap(&mut x1, &mut x2);
			}
			if y2 < y1 {
				swap(&mut y1, &mut y2);
			}
			let mut region = Rect {
				x: x1,
				y: y1,
				width: (x2 - x1 + 1) as u32,
				height: (y2 - y1 + 1) as u32,
			};
			if !region.clamp_to(&self.clip_region) {
				return;
			}

			// note that since we're performing a blend based on the existing destination pixel,
			// we need to make sure that we don't draw any overlapping corner pixels (where we
			// would end up blending with the edge of a previously drawn line).
			// to solve this issue, we just cut off the left-most and right-most pixels for the
			// two horizontal lines drawn. those corner pixels will be drawn during the vertical
			// line drawing instead.

			// top line, only if y1 was originally within bounds
			if y1 == region.y {
				unsafe {
					let dest = self.pixels_at_mut_unchecked(region.x, region.y);
					for x in 1..(region.width - 1) as usize {
						dest[x] = blend_mapping[dest[x] as usize];
					}
				}
			}

			// bottom line, only if y2 was originally within bounds
			if y2 == region.bottom() {
				unsafe {
					let dest = self.pixels_at_mut_unchecked(region.x, region.bottom());
					for x in 1..(region.width - 1) as usize {
						dest[x] = blend_mapping[dest[x] as usize];
					}
				}
			}

			// left line, only if x1 was originally within bounds
			if x1 == region.x {
				unsafe {
					let mut dest = self.pixels_at_mut_ptr_unchecked(region.x, region.y);
					for _ in 0..region.height {
						*dest = blend_mapping[*dest as usize];
						dest = dest.add(self.width as usize);
					}
				}
			}

			// right line, only if x2 was originally within bounds
			if x2 == region.right() {
				unsafe {
					let mut dest = self.pixels_at_mut_ptr_unchecked(region.right(), region.y);
					for _ in 0..region.height {
						*dest = blend_mapping[*dest as usize];
						dest = dest.add(self.width as usize);
					}
				}
			}
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
			let mut region = Rect::from_coords(x1, y1, x2, y2);
			if region.clamp_to(&self.clip_region) {
				unsafe {
					let mut dest = self.pixels_at_mut_ptr_unchecked(region.x, region.y);
					for _ in 0..region.height {
						for x in 0..region.width as usize {
							let dest_x = dest.offset(x as isize);
							*dest_x = blend_mapping[*dest_x as usize];
						}
						dest = dest.add(self.width as usize);
					}
				}
			}
		} else {
			self.filled_rect(x1, y1, x2, y2, color);
		}
	}
}