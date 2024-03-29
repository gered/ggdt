use std::mem::swap;

use crate::graphics::{Bitmap, Character, Font, FontRenderOpts, Pixel};
use crate::math::{lerp, Rect};

impl<PixelType: Pixel> Bitmap<PixelType> {
	/// Fills the entire bitmap with the given color.
	pub fn clear(&mut self, color: PixelType) {
		self.pixels.fill(color);
	}

	/// Sets the pixel at the given coordinates to the color specified. If the coordinates lie
	/// outside of the bitmaps clipping region, no pixels will be changed.
	#[inline]
	pub fn set_pixel(&mut self, x: i32, y: i32, color: PixelType) {
		if let Some(pixels) = self.pixels_at_mut(x, y) {
			pixels[0] = color;
		}
	}

	/// Sets the pixel at the given coordinates to the color returned by the given function. The
	/// given function is one that accepts a color value that corresponds to the current pixel
	/// at the given coordinates. If the coordinates lie outside of the bitmaps clipping region,
	/// no pixels will be changed and the given pixel function will not be called.
	#[inline]
	pub fn set_custom_pixel(&mut self, x: i32, y: i32, pixel_fn: impl Fn(PixelType) -> PixelType) {
		unsafe {
			if let Some(pixels) = self.pixels_at_mut_ptr(x, y) {
				*pixels = pixel_fn(*pixels);
			}
		}
	}

	/// Sets the pixel at the given coordinates to the color specified. The coordinates are not
	/// checked for validity, so it is up to you to ensure they lie within the bounds of the
	/// bitmap.
	#[inline]
	pub unsafe fn set_pixel_unchecked(&mut self, x: i32, y: i32, color: PixelType) {
		let p = self.pixels_at_mut_ptr_unchecked(x, y);
		*p = color;
	}

	/// Sets the pixel at the given coordinates to the color returned by the given function. The
	/// given function is one that accepts a color value that corresponds to the current pixel at
	/// the given coordinates. The coordinates are not checked for validity, so it is up to you to
	/// ensure they lie within the bounds of the bitmap.
	#[inline]
	pub unsafe fn set_custom_pixel_unchecked(&mut self, x: i32, y: i32, pixel_fn: impl Fn(PixelType) -> PixelType) {
		let p = self.pixels_at_mut_ptr_unchecked(x, y);
		*p = pixel_fn(*p);
	}

	/// Gets the pixel at the given coordinates. If the coordinates lie outside of the bitmaps
	/// clipping region, None is returned.
	#[inline]
	pub fn get_pixel(&self, x: i32, y: i32) -> Option<PixelType> {
		self.pixels_at(x, y).map(|pixels| pixels[0])
	}

	/// Gets the pixel at the given coordinates. The coordinates are not checked for validity, so
	/// it is up to you to ensure they lie within the bounds of the bitmap.
	#[inline]
	pub unsafe fn get_pixel_unchecked(&self, x: i32, y: i32) -> PixelType {
		*(self.pixels_at_ptr_unchecked(x, y))
	}

	#[inline]
	pub fn sample_at(&self, u: f32, v: f32) -> PixelType {
		// HACK: the 0.00001 shit. there is some weird and classic "off-by-1" issue happening SOMEWHERE in the
		//       textured 2d triangle rendering and i cannot find it. every other article i've read on the subject
		//       doesn't seem to do anything special here so either i'm doing something wrong or no one else tests
		//       their shit either. i just don't know which it is! but this hack fix definitely is all kinds of shit.
		//       i have no doubt that this "Fix" will probably be the source of OTHER issues too that i've just not
		//       seen yet in my testing. ugh.
		let x = lerp(0.0, self.width as f32 - 0.00001, u) as i32 % self.width as i32;
		let y = lerp(0.0, self.height as f32 - 0.00001, v) as i32 % self.height as i32;
		unsafe { self.get_pixel_unchecked(x, y) }
	}

	/// Renders a single character using the font given.
	#[inline]
	pub fn print_char<T: Font>(&mut self, ch: char, x: i32, y: i32, opts: FontRenderOpts<PixelType>, font: &T) {
		font.character(ch) //
			.draw(self, x, y, opts);
	}

	/// Renders the string of text using the font given.
	pub fn print_string<T: Font>(&mut self, text: &str, x: i32, y: i32, opts: FontRenderOpts<PixelType>, font: &T) {
		let mut current_x = x;
		let mut current_y = y;
		for ch in text.chars() {
			match ch {
				' ' => current_x += font.space_width() as i32,
				'\n' => {
					current_x = x;
					current_y += font.line_height() as i32
				}
				'\r' => (),
				otherwise => {
					self.print_char(otherwise, current_x, current_y, opts, font);
					current_x += font.character(otherwise).bounds().width as i32;
				}
			}
		}
	}

	/// Draws a line from x1,y1 to x2,y2.
	pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: PixelType) {
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
				*dest = color;
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
						*dest = color;
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
						*dest = color;
					}
				}
			}
		}
	}

	pub fn line_custom(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, pixel_fn: impl Fn(PixelType) -> PixelType) {
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
				*dest = pixel_fn(*dest);
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
						*dest = pixel_fn(*dest);
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
						*dest = pixel_fn(*dest);
					}
				}
			}
		}
	}

	/// Draws a horizontal line from x1,y to x2,y.
	pub fn horiz_line(&mut self, x1: i32, x2: i32, y: i32, color: PixelType) {
		let mut region = Rect::from_coords(x1, y, x2, y);
		if region.clamp_to(&self.clip_region) {
			unsafe {
				let dest = &mut self.pixels_at_mut_unchecked(region.x, region.y)[0..region.width as usize];
				dest.fill(color);
			}
		}
	}

	pub fn horiz_line_custom(&mut self, x1: i32, x2: i32, y: i32, pixel_fn: impl Fn(PixelType) -> PixelType) {
		let mut region = Rect::from_coords(x1, y, x2, y);
		if region.clamp_to(&self.clip_region) {
			unsafe {
				let dest = &mut self.pixels_at_mut_unchecked(region.x, region.y)[0..region.width as usize];
				for pixel in dest.iter_mut() {
					*pixel = pixel_fn(*pixel);
				}
			}
		}
	}

	/// Draws a vertical line from x,y1 to x,y2.
	pub fn vert_line(&mut self, x: i32, y1: i32, y2: i32, color: PixelType) {
		let mut region = Rect::from_coords(x, y1, x, y2);
		if region.clamp_to(&self.clip_region) {
			unsafe {
				let mut dest = self.pixels_at_mut_ptr_unchecked(region.x, region.y);
				for _ in 0..region.height {
					*dest = color;
					dest = dest.add(self.width as usize);
				}
			}
		}
	}

	pub fn vert_line_custom(&mut self, x: i32, y1: i32, y2: i32, pixel_fn: impl Fn(PixelType) -> PixelType) {
		let mut region = Rect::from_coords(x, y1, x, y2);
		if region.clamp_to(&self.clip_region) {
			unsafe {
				let mut dest = self.pixels_at_mut_ptr_unchecked(region.x, region.y);
				for _ in 0..region.height {
					*dest = pixel_fn(*dest);
					dest = dest.add(self.width as usize);
				}
			}
		}
	}

	/// Draws an empty box (rectangle) using the points x1,y1 and x2,y2 to form the box to be
	/// drawn, assuming they are specifying the top-left and bottom-right corners respectively.
	pub fn rect(&mut self, mut x1: i32, mut y1: i32, mut x2: i32, mut y2: i32, color: PixelType) {
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
			x: x1, //
			y: y1,
			width: (x2 - x1 + 1) as u32,
			height: (y2 - y1 + 1) as u32,
		};
		if !region.clamp_to(&self.clip_region) {
			return;
		}

		// top line, only if y1 was originally within bounds
		if y1 == region.y {
			unsafe {
				let dest = &mut self.pixels_at_mut_unchecked(region.x, region.y)[0..region.width as usize];
				dest.fill(color);
			}
		}

		// bottom line, only if y2 was originally within bounds
		if y2 == region.bottom() {
			unsafe {
				let dest = &mut self.pixels_at_mut_unchecked(region.x, region.bottom())[0..region.width as usize];
				dest.fill(color);
			}
		}

		// left line, only if x1 was originally within bounds
		if x1 == region.x {
			unsafe {
				let mut dest = self.pixels_at_mut_ptr_unchecked(region.x, region.y);
				for _ in 0..region.height {
					*dest = color;
					dest = dest.add(self.width as usize);
				}
			}
		}

		// right line, only if x2 was originally within bounds
		if x2 == region.right() {
			unsafe {
				let mut dest = self.pixels_at_mut_ptr_unchecked(region.right(), region.y);
				for _ in 0..region.height {
					*dest = color;
					dest = dest.add(self.width as usize);
				}
			}
		}
	}

	pub fn rect_custom(
		&mut self,
		mut x1: i32,
		mut y1: i32,
		mut x2: i32,
		mut y2: i32,
		pixel_fn: impl Fn(PixelType) -> PixelType,
	) {
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
			x: x1, //
			y: y1,
			width: (x2 - x1 + 1) as u32,
			height: (y2 - y1 + 1) as u32,
		};
		if !region.clamp_to(&self.clip_region) {
			return;
		}

		// we want to draw the top and bottom lines 1 pixel shorter at both ends to avoid overdrawing the
		// corners of the rect. this is mostly important for blended drawing operations.
		// if the region's left and/or right x coordinate was NOT clipped, we chop off 1 pixel from either/both
		// to avoid corner overdraw. if either/both of these x coordinates WAS clipped, we don't need to adjust that
		// side, as no corner overdraw is possible in that case.
		let mut horiz_draw_x = region.x;
		let mut horiz_draw_width = region.width;
		if x1 == region.x {
			horiz_draw_x += 1;
			horiz_draw_width -= 1;
		}
		if x2 == region.right() {
			horiz_draw_width -= 1;
		}

		// top line, only if y1 was originally within bounds
		if y1 == region.y {
			unsafe {
				let dest = &mut self.pixels_at_mut_unchecked(horiz_draw_x, region.y)[0..horiz_draw_width as usize];
				for pixel in dest.iter_mut() {
					*pixel = pixel_fn(*pixel);
				}
			}
		}

		// bottom line, only if y2 was originally within bounds
		if y2 == region.bottom() {
			unsafe {
				let dest =
					&mut self.pixels_at_mut_unchecked(horiz_draw_x, region.bottom())[0..horiz_draw_width as usize];
				for pixel in dest.iter_mut() {
					*pixel = pixel_fn(*pixel);
				}
			}
		}

		// left line, only if x1 was originally within bounds
		if x1 == region.x {
			unsafe {
				let mut dest = self.pixels_at_mut_ptr_unchecked(region.x, region.y);
				for _ in 0..region.height {
					*dest = pixel_fn(*dest);
					dest = dest.add(self.width as usize);
				}
			}
		}

		// right line, only if x2 was originally within bounds
		if x2 == region.right() {
			unsafe {
				let mut dest = self.pixels_at_mut_ptr_unchecked(region.right(), region.y);
				for _ in 0..region.height {
					*dest = pixel_fn(*dest);
					dest = dest.add(self.width as usize);
				}
			}
		}
	}

	/// Draws a filled box (rectangle) using the points x1,y1 and x2,y2 to form the box to be
	/// drawn, assuming they are specifying the top-left and bottom-right corners respectively.
	pub fn filled_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: PixelType) {
		let mut region = Rect::from_coords(x1, y1, x2, y2);
		if region.clamp_to(&self.clip_region) {
			unsafe {
				let mut dest = self.pixels_at_mut_ptr_unchecked(region.x, region.y);
				for _ in 0..region.height {
					// write the pixels horizontally for the entire width
					let row_pixels = std::slice::from_raw_parts_mut(dest, region.width as usize);
					row_pixels.fill(color);
					// move original pointer to the next row
					dest = dest.add(self.width as usize);
				}
			}
		}
	}

	pub fn filled_rect_custom(
		&mut self,
		x1: i32,
		y1: i32,
		x2: i32,
		y2: i32,
		pixel_fn: impl Fn(PixelType) -> PixelType,
	) {
		let mut region = Rect::from_coords(x1, y1, x2, y2);
		if region.clamp_to(&self.clip_region) {
			unsafe {
				let mut dest = self.pixels_at_mut_ptr_unchecked(region.x, region.y);
				for _ in 0..region.height {
					// write the pixels horizontally for the entire width
					let row_pixels = std::slice::from_raw_parts_mut(dest, region.width as usize);
					for pixel in row_pixels.iter_mut() {
						*pixel = pixel_fn(*pixel);
					}
					// move original pointer to the next row
					dest = dest.add(self.width as usize);
				}
			}
		}
	}

	/// Draws the outline of a circle formed by the center point and radius given.
	pub fn circle(&mut self, center_x: i32, center_y: i32, radius: u32, color: PixelType) {
		// TODO: optimize
		let mut x = 0;
		let mut y = radius as i32;
		let mut m = 5 - 4 * radius as i32;

		while x <= y {
			self.set_pixel(center_x + x, center_y + y, color);
			self.set_pixel(center_x + x, center_y - y, color);
			self.set_pixel(center_x - x, center_y + y, color);
			self.set_pixel(center_x - x, center_y - y, color);
			self.set_pixel(center_x + y, center_y + x, color);
			self.set_pixel(center_x + y, center_y - x, color);
			self.set_pixel(center_x - y, center_y + x, color);
			self.set_pixel(center_x - y, center_y - x, color);

			if m > 0 {
				y -= 1;
				m -= 8 * y;
			}

			x += 1;
			m += 8 * x + 4;
		}
	}

	/// Draws a filled circle formed by the center point and radius given.
	pub fn filled_circle(&mut self, center_x: i32, center_y: i32, radius: u32, color: PixelType) {
		// TODO: optimize
		let mut x = 0;
		let mut y = radius as i32;
		let mut m = 5 - 4 * radius as i32;

		while x <= y {
			self.horiz_line(center_x - x, center_x + x, center_y - y, color);
			self.horiz_line(center_x - y, center_x + y, center_y - x, color);
			self.horiz_line(center_x - y, center_x + y, center_y + x, color);
			self.horiz_line(center_x - x, center_x + x, center_y + y, color);

			if m > 0 {
				y -= 1;
				m -= 8 * y;
			}

			x += 1;
			m += 8 * x + 4;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[rustfmt::skip]
	#[test]
	pub fn set_and_get_pixel() {
		let mut bmp = Bitmap::<u8>::new(8, 8).unwrap();

		assert_eq!(None, bmp.get_pixel(-1, -1));

		assert_eq!(0, bmp.get_pixel(0, 0).unwrap());
		bmp.set_pixel(0, 0, 7);
		assert_eq!(7, bmp.get_pixel(0, 0).unwrap());

		assert_eq!(
			bmp.pixels(),
			&[
				7, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
			]
		);

		assert_eq!(0, bmp.get_pixel(2, 4).unwrap());
		bmp.set_pixel(2, 4, 5);
		assert_eq!(5, bmp.get_pixel(2, 4).unwrap());

		assert_eq!(
			bmp.pixels(),
			&[
				7, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 5, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
			]
		);
	}

	#[rustfmt::skip]
	#[test]
	pub fn set_and_get_pixel_unchecked() {
		let mut bmp = Bitmap::<u8>::new(8, 8).unwrap();

		assert_eq!(0, unsafe { bmp.get_pixel_unchecked(0, 0) });
		unsafe { bmp.set_pixel_unchecked(0, 0, 7) };
		assert_eq!(7, unsafe { bmp.get_pixel_unchecked(0, 0) });

		assert_eq!(
			bmp.pixels(),
			&[
				7, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
			]
		);

		assert_eq!(0, unsafe { bmp.get_pixel_unchecked(2, 4) });
		unsafe { bmp.set_pixel_unchecked(2, 4, 5) };
		assert_eq!(5, unsafe { bmp.get_pixel_unchecked(2, 4) });

		assert_eq!(
			bmp.pixels(),
			&[
				7, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 5, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
			]
		);
	}
}
