use crate::graphics::{Bitmap, Pixel};
use crate::math::Rect;

/// Clips the region for a source bitmap to be used in a subsequent blit operation. The source
/// region will be clipped against the clipping region given for the destination bitmap. The
/// top-left coordinates of the location to blit to on the destination bitmap are also adjusted
/// only if necessary based on the clipping performed.
///
/// # Arguments
///
/// * `dest_clip_region`: the clipping region for the destination bitmap
/// * `src_blit_region`: the region on the source bitmap that is to be blitted, which may be
///   clipped if necessary to at least partially fit into the destination clipping region given
/// * `dest_x`: the x (left) coordinate of the location on the destination bitmap to blit the
///   source to, which may be adjusted as necessary during clipping
/// * `dest_y`: the y (top) coordinate of the location on the destination bitmap to blit the source
///   to, which may be adjusted as necessary during clipping
/// * `horizontal_flip`: whether the blit is supposed to flip the source image horizontally
/// * `vertical_flip`: whether the blit is supposed to flip the source image vertically
///
/// returns: true if the results of the clip is partially or entirely visible on the destination
/// bitmap, or false if the blit is entirely outside of the destination bitmap (and so no blit
/// needs to occur)
pub fn clip_blit(
	dest_clip_region: &Rect,
	src_blit_region: &mut Rect,
	dest_x: &mut i32,
	dest_y: &mut i32,
	horizontal_flip: bool,
	vertical_flip: bool,
) -> bool {
	// off the left edge?
	if *dest_x < dest_clip_region.x {
		// completely off the left edge?
		if (*dest_x + src_blit_region.width as i32 - 1) < dest_clip_region.x {
			return false;
		}

		let offset = dest_clip_region.x - *dest_x;
		if !horizontal_flip {
			src_blit_region.x += offset;
		}
		src_blit_region.width = (src_blit_region.width as i32 - offset) as u32;
		*dest_x = dest_clip_region.x;
	}

	// off the right edge?
	if *dest_x > dest_clip_region.width as i32 - src_blit_region.width as i32 {
		// completely off the right edge?
		if *dest_x > dest_clip_region.right() {
			return false;
		}

		let offset = *dest_x + src_blit_region.width as i32 - dest_clip_region.width as i32;
		if horizontal_flip {
			src_blit_region.x += offset;
		}
		src_blit_region.width = (src_blit_region.width as i32 - offset) as u32;
	}

	// off the top edge?
	if *dest_y < dest_clip_region.y {
		// completely off the top edge?
		if (*dest_y + src_blit_region.height as i32 - 1) < dest_clip_region.y {
			return false;
		}

		let offset = dest_clip_region.y - *dest_y;
		if !vertical_flip {
			src_blit_region.y += offset;
		}
		src_blit_region.height = (src_blit_region.height as i32 - offset) as u32;
		*dest_y = dest_clip_region.y;
	}

	// off the bottom edge?
	if *dest_y > dest_clip_region.height as i32 - src_blit_region.height as i32 {
		// completely off the bottom edge?
		if *dest_y > dest_clip_region.bottom() {
			return false;
		}

		let offset = *dest_y + src_blit_region.height as i32 - dest_clip_region.height as i32;
		if vertical_flip {
			src_blit_region.y += offset;
		}
		src_blit_region.height = (src_blit_region.height as i32 - offset) as u32;
	}

	true
}

#[inline]
fn get_flipped_blit_properties<PixelType: Pixel>(
	src: &Bitmap<PixelType>,
	src_region: &Rect,
	horizontal_flip: bool,
	vertical_flip: bool,
) -> (isize, i32, i32, isize) {
	let x_inc;
	let src_start_x;
	let src_start_y;
	let src_next_row_inc;

	if !horizontal_flip && !vertical_flip {
		x_inc = 1;
		src_start_x = src_region.x;
		src_start_y = src_region.y;
		src_next_row_inc = (src.width - src_region.width) as isize;
	} else if horizontal_flip && !vertical_flip {
		x_inc = -1;
		src_start_x = src_region.right();
		src_start_y = src_region.y;
		src_next_row_inc = (src.width + src_region.width) as isize;
	} else if !horizontal_flip && vertical_flip {
		x_inc = 1;
		src_start_x = src_region.x;
		src_start_y = src_region.bottom();
		src_next_row_inc = -((src.width + src_region.width) as isize);
	} else {
		x_inc = -1;
		src_start_x = src_region.right();
		src_start_y = src_region.bottom();
		src_next_row_inc = -((src.width - src_region.width) as isize);
	}

	(x_inc, src_start_x, src_start_y, src_next_row_inc)
}

#[inline]
pub unsafe fn per_pixel_blit<PixelType: Pixel>(
	dest: &mut Bitmap<PixelType>,
	src: &Bitmap<PixelType>,
	src_region: &Rect,
	dest_x: i32,
	dest_y: i32,
	pixel_fn: impl Fn(*const PixelType, *mut PixelType),
) {
	let src_next_row_inc = (src.width - src_region.width) as usize;
	let dest_next_row_inc = (dest.width - src_region.width) as usize;
	let mut src_pixels = src.pixels_at_ptr_unchecked(src_region.x, src_region.y);
	let mut dest_pixels = dest.pixels_at_mut_ptr_unchecked(dest_x, dest_y);

	for _ in 0..src_region.height {
		for _ in 0..src_region.width {
			pixel_fn(src_pixels, dest_pixels);
			src_pixels = src_pixels.add(1);
			dest_pixels = dest_pixels.add(1);
		}

		src_pixels = src_pixels.add(src_next_row_inc);
		dest_pixels = dest_pixels.add(dest_next_row_inc);
	}
}

#[inline]
pub unsafe fn per_pixel_flipped_blit<PixelType: Pixel>(
	dest: &mut Bitmap<PixelType>,
	src: &Bitmap<PixelType>,
	src_region: &Rect,
	dest_x: i32,
	dest_y: i32,
	horizontal_flip: bool,
	vertical_flip: bool,
	pixel_fn: impl Fn(*const PixelType, *mut PixelType),
) {
	let dest_next_row_inc = (dest.width - src_region.width) as usize;
	let (x_inc, src_start_x, src_start_y, src_next_row_inc) =
		get_flipped_blit_properties(src, src_region, horizontal_flip, vertical_flip);

	let mut src_pixels = src.pixels_at_ptr_unchecked(src_start_x, src_start_y);
	let mut dest_pixels = dest.pixels_at_mut_ptr_unchecked(dest_x, dest_y);

	for _ in 0..src_region.height {
		for _ in 0..src_region.width {
			pixel_fn(src_pixels, dest_pixels);
			src_pixels = src_pixels.offset(x_inc);
			dest_pixels = dest_pixels.add(1);
		}

		src_pixels = src_pixels.offset(src_next_row_inc);
		dest_pixels = dest_pixels.add(dest_next_row_inc);
	}
}

#[inline]
pub unsafe fn per_pixel_rotozoom_blit<PixelType: Pixel>(
	dest: &mut Bitmap<PixelType>,
	src: &Bitmap<PixelType>,
	src_region: &Rect,
	dest_x: i32,
	dest_y: i32,
	angle: f32,
	scale_x: f32,
	scale_y: f32,
	pixel_fn: impl Fn(PixelType, &mut Bitmap<PixelType>, i32, i32),
) {
	let dest_width = src_region.width as f32 * scale_x;
	let dest_height = src_region.height as f32 * scale_y;

	let half_src_width = src_region.width as f32 * 0.5;
	let half_src_height = src_region.height as f32 * 0.5;
	let half_dest_width = dest_width * 0.5;
	let half_dest_height = dest_height * 0.5;

	// calculate the destination bitmap axis-aligned bounding box of the region we're drawing to
	// based on the source bitmap bounds when rotated and scaled. this is to prevent potentially
	// cutting off the corners of the drawn bitmap depending on the exact rotation angle, since
	// dest_width and dest_height can only really be used (by themselves) to calculate bounding box
	// extents for 90-degree angle rotations. this feels kinda ugly to me, but not sure what other
	// clever way to calculate this that there might be (if any).

	let sin = angle.sin();
	let cos = angle.cos();

	let left = -half_dest_width * cos - half_dest_height * sin;
	let top = -half_dest_width * sin + half_dest_height * cos;
	let right = half_dest_width * cos - half_dest_height * sin;
	let bottom = half_dest_width * sin + half_dest_height * cos;

	let (top_left_x, top_left_y) = (left + half_dest_width, top + half_dest_height);
	let (top_right_x, top_right_y) = (right + half_dest_width, bottom + half_dest_height);
	let (bottom_left_x, bottom_left_y) = (-left + half_dest_width, -top + half_dest_height);
	let (bottom_right_x, bottom_right_y) = (-right + half_dest_width, -bottom + half_dest_height);

	// HACK: -/+ 1's because this seems to fix some destination area accidental clipping for _some_
	//       rotation angles ... ? i guess some other math is probably wrong somewhere or some
	//       floating point rounding fun perhaps?
	let dest_region = Rect::from_coords(
		top_left_x.min(bottom_left_x).min(top_right_x).min(bottom_right_x) as i32 - 1,
		top_left_y.min(bottom_left_y).min(top_right_y).min(bottom_right_y) as i32 - 1,
		top_left_x.max(bottom_left_x).max(top_right_x).max(bottom_right_x) as i32 + 1,
		top_left_y.max(bottom_left_y).max(top_right_y).max(bottom_right_y) as i32 + 1,
	);

	// now we're ready to draw. we'll be iterating through each pixel on the area we calculated
	// just above -- that is (x1,y1)-(x2,y2) -- on the DESTINATION bitmap and for each of these
	// x/y coordinates we'll sample the source bitmap after applying a reverse rotation/scale to get
	// the equivalent source bitmap x/y pixel coordinate to be drawn. this is to ensure we don't
	// end up with any "gap" pixels which would likely result if we instead simply iterated through
	// the source bitmap pixels and only drew the resulting pixels.

	let sin = -angle.sin();
	let cos = angle.cos();

	let scale_x = 1.0 / scale_x;
	let scale_y = 1.0 / scale_y;

	for y in dest_region.y..=dest_region.bottom() {
		for x in dest_region.x..=dest_region.right() {
			// map the destination bitmap x/y coordinate we're currently at to it's source bitmap
			// x/y coordinate by applying a reverse rotation/scale.
			// note that for these transformations, we're doing a "weird" thing by utilizing the
			// destination bitmap's center point as the origin _except_ for the final post-transform
			// offset where we instead use the source bitmap's center point to re-translate the
			// coordinates back. this is necessary because of the (potential) scale differences!
			let src_x = ((x as f32 - half_dest_width) * cos * scale_x)
				- ((y as f32 - half_dest_height) * sin * scale_x)
				+ half_src_width;
			let src_y = ((x as f32 - half_dest_width) * sin * scale_y)
				+ ((y as f32 - half_dest_height) * cos * scale_y)
				+ half_src_height;

			// ensure the source x,y is in bounds, as it very well might not be depending on exactly
			// where we are inside the destination area currently. also, we're not interested in
			// wrapping of course, since we just want to draw a single instance of this source
			// bitmap.
			if src_x >= 0.0
				&& (src_x as i32) < (src_region.width as i32)
				&& src_y >= 0.0 && (src_y as i32) < (src_region.height as i32)
			{
				let pixel = src.get_pixel_unchecked(src_x as i32 + src_region.x, src_y as i32 + src_region.y);

				let draw_x = x + dest_x;
				let draw_y = y + dest_y;
				pixel_fn(pixel, dest, draw_x, draw_y);
			}
		}
	}
}

impl<PixelType: Pixel> Bitmap<PixelType> {
	pub unsafe fn solid_blit(&mut self, src: &Self, src_region: &Rect, dest_x: i32, dest_y: i32) {
		let src_row_length = src_region.width as usize;
		let src_pitch = src.width as usize;
		let dest_pitch = self.width as usize;
		let mut src_pixels = src.pixels_at_ptr_unchecked(src_region.x, src_region.y);
		let mut dest_pixels = self.pixels_at_mut_ptr_unchecked(dest_x, dest_y);

		for _ in 0..src_region.height {
			dest_pixels.copy_from(src_pixels, src_row_length);
			src_pixels = src_pixels.add(src_pitch);
			dest_pixels = dest_pixels.add(dest_pitch);
		}
	}

	pub unsafe fn solid_flipped_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		horizontal_flip: bool,
		vertical_flip: bool,
	) {
		per_pixel_flipped_blit(
			self,
			src,
			src_region,
			dest_x,
			dest_y,
			horizontal_flip,
			vertical_flip,
			|src_pixels, dest_pixels| {
				*dest_pixels = *src_pixels;
			},
		);
	}

	pub unsafe fn transparent_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		transparent_color: PixelType,
	) {
		per_pixel_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			|src_pixels, dest_pixels| {
				if *src_pixels != transparent_color {
					*dest_pixels = *src_pixels;
				}
			},
		);
	}

	pub unsafe fn transparent_flipped_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		transparent_color: PixelType,
		horizontal_flip: bool,
		vertical_flip: bool,
	) {
		per_pixel_flipped_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			horizontal_flip,
			vertical_flip,
			|src_pixels, dest_pixels| {
				if *src_pixels != transparent_color {
					*dest_pixels = *src_pixels;
				}
			},
		);
	}

	pub unsafe fn transparent_single_color_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		transparent_color: PixelType,
		draw_color: PixelType,
	) {
		per_pixel_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			|src_pixels, dest_pixels| {
				if *src_pixels != transparent_color {
					*dest_pixels = draw_color;
				}
			},
		);
	}

	pub unsafe fn transparent_flipped_single_color_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		transparent_color: PixelType,
		horizontal_flip: bool,
		vertical_flip: bool,
		draw_color: PixelType,
	) {
		per_pixel_flipped_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			horizontal_flip,
			vertical_flip,
			|src_pixels, dest_pixels| {
				if *src_pixels != transparent_color {
					*dest_pixels = draw_color;
				}
			},
		);
	}

	pub unsafe fn rotozoom_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		angle: f32,
		scale_x: f32,
		scale_y: f32,
	) {
		per_pixel_rotozoom_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			angle,
			scale_x,
			scale_y,
			|src_pixel, dest_bitmap, draw_x, draw_y| {
				dest_bitmap.set_pixel(draw_x, draw_y, src_pixel);
			},
		);
	}

	pub unsafe fn rotozoom_transparent_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		transparent_color: PixelType,
	) {
		per_pixel_rotozoom_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			angle,
			scale_x,
			scale_y,
			|src_pixel, dest_bitmap, draw_x, draw_y| {
				if transparent_color != src_pixel {
					dest_bitmap.set_pixel(draw_x, draw_y, src_pixel);
				}
			},
		);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	pub fn clip_blit_regions() {
		let dest = Rect::new(0, 0, 320, 240);

		let mut src: Rect;
		let mut x: i32;
		let mut y: i32;

		src = Rect::new(0, 0, 16, 16);
		x = 10;
		y = 10;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(0, 0, 16, 16));
		assert_eq!(10, x);
		assert_eq!(10, y);

		// left edge

		src = Rect::new(0, 0, 16, 16);
		x = 0;
		y = 10;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(0, 0, 16, 16));
		assert_eq!(0, x);
		assert_eq!(10, y);

		src = Rect::new(0, 0, 16, 16);
		x = -5;
		y = 10;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(5, 0, 11, 16));
		assert_eq!(0, x);
		assert_eq!(10, y);

		src = Rect::new(0, 0, 16, 16);
		x = -16;
		y = 10;
		assert!(!clip_blit(&dest, &mut src, &mut x, &mut y, false, false));

		// right edge

		src = Rect::new(0, 0, 16, 16);
		x = 304;
		y = 10;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(0, 0, 16, 16));
		assert_eq!(304, x);
		assert_eq!(10, y);

		src = Rect::new(0, 0, 16, 16);
		x = 310;
		y = 10;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(0, 0, 10, 16));
		assert_eq!(310, x);
		assert_eq!(10, y);

		src = Rect::new(0, 0, 16, 16);
		x = 320;
		y = 10;
		assert!(!clip_blit(&dest, &mut src, &mut x, &mut y, false, false));

		// top edge

		src = Rect::new(0, 0, 16, 16);
		x = 10;
		y = 0;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(0, 0, 16, 16));
		assert_eq!(10, x);
		assert_eq!(0, y);

		src = Rect::new(0, 0, 16, 16);
		x = 10;
		y = -5;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(0, 5, 16, 11));
		assert_eq!(10, x);
		assert_eq!(0, y);

		src = Rect::new(0, 0, 16, 16);
		x = 10;
		y = -16;
		assert!(!clip_blit(&dest, &mut src, &mut x, &mut y, false, false));

		// bottom edge

		src = Rect::new(0, 0, 16, 16);
		x = 10;
		y = 224;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(0, 0, 16, 16));
		assert_eq!(10, x);
		assert_eq!(224, y);

		src = Rect::new(0, 0, 16, 16);
		x = 10;
		y = 229;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(0, 0, 16, 11));
		assert_eq!(10, x);
		assert_eq!(229, y);

		src = Rect::new(0, 0, 16, 16);
		x = 10;
		y = 240;
		assert!(!clip_blit(&dest, &mut src, &mut x, &mut y, false, false));

		src = Rect::new(16, 16, 16, 16);
		x = -1;
		y = 112;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(17, 16, 15, 16));
		assert_eq!(0, x);
		assert_eq!(112, y);
	}

	#[test]
	pub fn clip_blit_regions_flipped() {
		let dest = Rect::new(0, 0, 320, 240);

		let mut src: Rect;
		let mut x: i32;
		let mut y: i32;

		// left edge

		src = Rect::new(0, 0, 16, 16);
		x = -6;
		y = 10;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, true, false));
		assert_eq!(src, Rect::new(0, 0, 10, 16));
		assert_eq!(0, x);
		assert_eq!(10, y);

		src = Rect::new(0, 0, 16, 16);
		x = -6;
		y = 10;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, true, true));
		assert_eq!(src, Rect::new(0, 0, 10, 16));
		assert_eq!(0, x);
		assert_eq!(10, y);

		// right edge

		src = Rect::new(0, 0, 16, 16);
		x = 312;
		y = 10;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, true, false));
		assert_eq!(src, Rect::new(8, 0, 8, 16));
		assert_eq!(312, x);
		assert_eq!(10, y);

		src = Rect::new(0, 0, 16, 16);
		x = 312;
		y = 10;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, true, true));
		assert_eq!(src, Rect::new(8, 0, 8, 16));
		assert_eq!(312, x);
		assert_eq!(10, y);

		// top edge

		src = Rect::new(0, 0, 16, 16);
		x = 10;
		y = -2;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, true));
		assert_eq!(src, Rect::new(0, 0, 16, 14));
		assert_eq!(10, x);
		assert_eq!(0, y);

		src = Rect::new(0, 0, 16, 16);
		x = 10;
		y = -2;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, true, true));
		assert_eq!(src, Rect::new(0, 0, 16, 14));
		assert_eq!(10, x);
		assert_eq!(0, y);

		// bottom edge

		src = Rect::new(0, 0, 16, 16);
		x = 10;
		y = 235;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, true));
		assert_eq!(src, Rect::new(0, 11, 16, 5));
		assert_eq!(10, x);
		assert_eq!(235, y);

		src = Rect::new(0, 0, 16, 16);
		x = 10;
		y = 235;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, true, true));
		assert_eq!(src, Rect::new(0, 11, 16, 5));
		assert_eq!(10, x);
		assert_eq!(235, y);

		// top-left edge

		src = Rect::new(0, 0, 16, 16);
		x = -2;
		y = -6;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, true, true));
		assert_eq!(src, Rect::new(0, 0, 14, 10));
		assert_eq!(0, x);
		assert_eq!(0, y);

		// top-right edge

		src = Rect::new(0, 0, 16, 16);
		x = 311;
		y = -12;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, true, true));
		assert_eq!(src, Rect::new(7, 0, 9, 4));
		assert_eq!(311, x);
		assert_eq!(0, y);

		// bottom-left edge

		src = Rect::new(0, 0, 16, 16);
		x = -1;
		y = 232;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, true, true));
		assert_eq!(src, Rect::new(0, 8, 15, 8));
		assert_eq!(0, x);
		assert_eq!(232, y);

		// bottom-right edge

		src = Rect::new(0, 0, 16, 16);
		x = 314;
		y = 238;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, true, true));
		assert_eq!(src, Rect::new(10, 14, 6, 2));
		assert_eq!(314, x);
		assert_eq!(238, y);
	}

	#[test]
	pub fn clip_blit_regions_large_source() {
		let dest = Rect::new(0, 0, 64, 64);

		let mut src: Rect;
		let mut x: i32;
		let mut y: i32;

		src = Rect::new(0, 0, 128, 128);
		x = 0;
		y = 0;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(0, 0, 64, 64));
		assert_eq!(0, x);
		assert_eq!(0, y);

		src = Rect::new(0, 0, 128, 128);
		x = -16;
		y = -24;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(16, 24, 64, 64));
		assert_eq!(0, x);
		assert_eq!(0, y);

		src = Rect::new(0, 0, 32, 128);
		x = 10;
		y = -20;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(0, 20, 32, 64));
		assert_eq!(10, x);
		assert_eq!(0, y);

		src = Rect::new(0, 0, 128, 32);
		x = -20;
		y = 10;
		assert!(clip_blit(&dest, &mut src, &mut x, &mut y, false, false));
		assert_eq!(src, Rect::new(20, 0, 64, 32));
		assert_eq!(0, x);
		assert_eq!(10, y);
	}
}
