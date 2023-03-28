use crate::graphics::bitmap::blit::{clip_blit, per_pixel_blit, per_pixel_flipped_blit, per_pixel_rotozoom_blit};
use crate::graphics::bitmap::rgb::RgbaBitmap;
use crate::graphics::bitmapatlas::BitmapAtlas;
use crate::graphics::color::{tint_argb32, BlendFunction};
use crate::math::rect::Rect;

#[derive(Clone, PartialEq)]
pub enum RgbaBlitMethod {
	/// Solid blit, no transparency or other per-pixel adjustments.
	Solid,
	SolidTinted(u32),
	SolidBlended(BlendFunction),
	/// Same as [RgbaBlitMethod::Solid] but the drawn image can also be flipped horizontally
	/// and/or vertically.
	SolidFlipped {
		horizontal_flip: bool,
		vertical_flip: bool,
	},
	SolidFlippedTinted {
		horizontal_flip: bool,
		vertical_flip: bool,
		tint_color: u32,
	},
	SolidFlippedBlended {
		horizontal_flip: bool,
		vertical_flip: bool,
		blend: BlendFunction,
	},
	/// Transparent blit, the specified source color pixels are skipped.
	Transparent(u32),
	TransparentTinted {
		transparent_color: u32,
		tint_color: u32,
	},
	TransparentBlended {
		transparent_color: u32,
		blend: BlendFunction,
	},
	/// Same as [RgbaBlitMethod::Transparent] but the drawn image can also be flipped horizontally
	/// and/or vertically.
	TransparentFlipped {
		transparent_color: u32,
		horizontal_flip: bool,
		vertical_flip: bool,
	},
	TransparentFlippedTinted {
		transparent_color: u32,
		horizontal_flip: bool,
		vertical_flip: bool,
		tint_color: u32,
	},
	TransparentFlippedBlended {
		transparent_color: u32,
		horizontal_flip: bool,
		vertical_flip: bool,
		blend: BlendFunction,
	},
	/// Same as [RgbaBlitMethod::Transparent] except that the visible pixels on the destination are all
	/// drawn using the same color.
	TransparentSingle {
		transparent_color: u32,
		draw_color: u32,
	},
	/// Combination of [RgbaBlitMethod::TransparentFlipped] and [RgbaBlitMethod::TransparentSingle].
	TransparentFlippedSingle {
		transparent_color: u32,
		horizontal_flip: bool,
		vertical_flip: bool,
		draw_color: u32,
	},
	/// Rotozoom blit, works the same as [RgbaBlitMethod::Solid] except that rotation and scaling is
	/// performed.
	RotoZoom {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
	},
	RotoZoomTinted {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		tint_color: u32,
	},
	RotoZoomBlended {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		blend: BlendFunction,
	},
	/// Same as [RgbaBlitMethod::RotoZoom] except that the specified source color pixels are skipped.
	RotoZoomTransparent {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		transparent_color: u32,
	},
	RotoZoomTransparentTinted {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		transparent_color: u32,
		tint_color: u32,
	},
	RotoZoomTransparentBlended {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		transparent_color: u32,
		blend: BlendFunction,
	},
}

impl RgbaBitmap {
	pub unsafe fn solid_tinted_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		tint_color: u32,
	) {
		per_pixel_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			|src_pixels, dest_pixels| {
				*dest_pixels = tint_argb32(*src_pixels, tint_color);
			},
		);
	}

	pub unsafe fn solid_blended_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		blend: BlendFunction,
	) {
		per_pixel_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			|src_pixels, dest_pixels| {
				*dest_pixels = blend.blend(*src_pixels, *dest_pixels);
			},
		);
	}

	pub unsafe fn solid_flipped_blended_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		horizontal_flip: bool,
		vertical_flip: bool,
		blend: BlendFunction,
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
				*dest_pixels = blend.blend(*src_pixels, *dest_pixels);
			},
		);
	}

	pub unsafe fn solid_flipped_tinted_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		horizontal_flip: bool,
		vertical_flip: bool,
		tint_color: u32,
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
				*dest_pixels = tint_argb32(*src_pixels, tint_color);
			},
		);
	}

	pub unsafe fn transparent_tinted_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		transparent_color: u32,
		tint_color: u32,
	) {
		per_pixel_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			|src_pixels, dest_pixels| {
				if *src_pixels != transparent_color {
					*dest_pixels = tint_argb32(*src_pixels, tint_color);
				}
			},
		);
	}

	pub unsafe fn transparent_blended_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		transparent_color: u32,
		blend: BlendFunction,
	) {
		per_pixel_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			|src_pixels, dest_pixels| {
				if *src_pixels != transparent_color {
					*dest_pixels = blend.blend(*src_pixels, *dest_pixels);
				}
			},
		);
	}

	pub unsafe fn transparent_flipped_tinted_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		transparent_color: u32,
		horizontal_flip: bool,
		vertical_flip: bool,
		tint_color: u32,
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
					*dest_pixels = tint_argb32(*src_pixels, tint_color);
				}
			},
		);
	}

	pub unsafe fn transparent_flipped_blended_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		transparent_color: u32,
		horizontal_flip: bool,
		vertical_flip: bool,
		blend: BlendFunction,
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
					*dest_pixels = blend.blend(*src_pixels, *dest_pixels);
				}
			},
		);
	}

	pub unsafe fn rotozoom_tinted_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		tint_color: u32,
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
				dest_bitmap.set_pixel(draw_x, draw_y, tint_argb32(src_pixel, tint_color));
			},
		);
	}

	pub unsafe fn rotozoom_blended_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		blend: BlendFunction,
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
				if let Some(dest_pixel) = dest_bitmap.get_pixel(draw_x, draw_y) {
					dest_bitmap.set_pixel(draw_x, draw_y, blend.blend(src_pixel, dest_pixel))
				}
			},
		);
	}

	pub unsafe fn rotozoom_transparent_tinted_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		transparent_color: u32,
		tint_color: u32,
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
					dest_bitmap.set_pixel(draw_x, draw_y, tint_argb32(src_pixel, tint_color));
				}
			},
		);
	}

	pub unsafe fn rotozoom_transparent_blended_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		transparent_color: u32,
		blend: BlendFunction,
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
					if let Some(dest_pixel) = dest_bitmap.get_pixel(draw_x, draw_y) {
						dest_bitmap.set_pixel(draw_x, draw_y, blend.blend(src_pixel, dest_pixel))
					}
				}
			},
		);
	}

	pub fn blit_region(
		&mut self,
		method: RgbaBlitMethod,
		src: &Self,
		src_region: &Rect,
		mut dest_x: i32,
		mut dest_y: i32,
	) {
		// make sure the source region is clipped or even valid at all for the source bitmap given
		let mut src_region = *src_region;
		if !src_region.clamp_to(&src.clip_region) {
			return;
		}

		// some blit methods need to handle clipping a bit differently than others
		use RgbaBlitMethod::*;
		match method {
			// rotozoom blits internally clip per-pixel right now ... and regardless, the normal
			// clip_blit() function wouldn't handle a rotozoom blit destination region anyway ...
			RotoZoom { .. } => {}
			RotoZoomTinted { .. } => {}
			RotoZoomBlended { .. } => {}
			RotoZoomTransparent { .. } => {}
			RotoZoomTransparentTinted { .. } => {}
			RotoZoomTransparentBlended { .. } => {}

			// set axis flip arguments
			SolidFlipped { horizontal_flip, vertical_flip, .. }
			| SolidFlippedTinted { horizontal_flip, vertical_flip, .. }
			| SolidFlippedBlended { horizontal_flip, vertical_flip, .. }
			| TransparentFlipped { horizontal_flip, vertical_flip, .. }
			| TransparentFlippedTinted { horizontal_flip, vertical_flip, .. }
			| TransparentFlippedBlended { horizontal_flip, vertical_flip, .. }
			| TransparentFlippedSingle { horizontal_flip, vertical_flip, .. } => {
				if !clip_blit(
					self.clip_region(), //
					&mut src_region,
					&mut dest_x,
					&mut dest_y,
					horizontal_flip,
					vertical_flip,
				) {
					return;
				}
			}

			// otherwise clip like normal!
			_ => {
				if !clip_blit(
					self.clip_region(), //
					&mut src_region,
					&mut dest_x,
					&mut dest_y,
					false,
					false,
				) {
					return;
				}
			}
		}

		unsafe {
			self.blit_region_unchecked(method, src, &src_region, dest_x, dest_y);
		};
	}

	#[inline]
	#[rustfmt::skip]
	pub unsafe fn blit_region_unchecked(
		&mut self,
		method: RgbaBlitMethod,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
	) {
		use RgbaBlitMethod::*;
		match method {
			Solid => self.solid_blit(src, src_region, dest_x, dest_y),
			SolidTinted(tint_color) => self.solid_tinted_blit(src, src_region, dest_x, dest_y, tint_color),
			SolidBlended(blend) => self.solid_blended_blit(src, src_region, dest_x, dest_y, blend),
			SolidFlipped { horizontal_flip, vertical_flip } => {
				self.solid_flipped_blit(src, src_region, dest_x, dest_y, horizontal_flip, vertical_flip)
			},
			SolidFlippedTinted { horizontal_flip, vertical_flip, tint_color } => {
				self.solid_flipped_tinted_blit(src, src_region, dest_x, dest_y, horizontal_flip, vertical_flip, tint_color)
			},
			SolidFlippedBlended { horizontal_flip, vertical_flip, blend } => {
				self.solid_flipped_blended_blit(src, src_region, dest_x, dest_y, horizontal_flip, vertical_flip, blend)
			},
			Transparent(transparent_color) => {
				self.transparent_blit(src, src_region, dest_x, dest_y, transparent_color)
			},
			TransparentTinted { transparent_color, tint_color } => {
				self.transparent_tinted_blit(src, src_region, dest_x, dest_y, transparent_color, tint_color)
			},
			TransparentBlended { transparent_color, blend } => {
				self.transparent_blended_blit(src, src_region, dest_x, dest_y, transparent_color, blend)
			},
			TransparentFlipped { transparent_color, horizontal_flip, vertical_flip } => {
				self.transparent_flipped_blit(src, src_region, dest_x, dest_y, transparent_color, horizontal_flip, vertical_flip)
			},
			TransparentFlippedTinted { transparent_color, horizontal_flip, vertical_flip, tint_color } => {
				self.transparent_flipped_tinted_blit(src, src_region, dest_x, dest_y, transparent_color, horizontal_flip, vertical_flip, tint_color)
			},
			TransparentFlippedBlended { transparent_color, horizontal_flip, vertical_flip, blend } => {
				self.transparent_flipped_blended_blit(src, src_region, dest_x, dest_y, transparent_color, horizontal_flip, vertical_flip, blend)
			},
			TransparentSingle { transparent_color, draw_color } => {
				self.transparent_single_color_blit(src, src_region, dest_x, dest_y, transparent_color, draw_color)
			},
			TransparentFlippedSingle { transparent_color, horizontal_flip, vertical_flip, draw_color } => {
				self.transparent_flipped_single_color_blit(src, src_region, dest_x, dest_y, transparent_color, horizontal_flip, vertical_flip, draw_color)
			},
			RotoZoom { angle, scale_x, scale_y } => {
				self.rotozoom_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y)
			},
			RotoZoomTinted { angle, scale_x, scale_y, tint_color } => {
				self.rotozoom_tinted_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y, tint_color)
			},
			RotoZoomBlended { angle, scale_x, scale_y, blend } => {
				self.rotozoom_blended_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y, blend)
			},
			RotoZoomTransparent { angle, scale_x, scale_y, transparent_color } => {
				self.rotozoom_transparent_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y, transparent_color)
			},
			RotoZoomTransparentTinted { angle, scale_x, scale_y, transparent_color, tint_color } => {
				self.rotozoom_transparent_tinted_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y, transparent_color, tint_color)
			},
			RotoZoomTransparentBlended { angle, scale_x, scale_y, transparent_color, blend } => {
				self.rotozoom_transparent_blended_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y, transparent_color, blend)
			},
		}
	}

	#[inline]
	pub fn blit(&mut self, method: RgbaBlitMethod, src: &Self, x: i32, y: i32) {
		let src_region = Rect::new(0, 0, src.width, src.height);
		self.blit_region(method, src, &src_region, x, y);
	}

	#[inline]
	pub fn blit_atlas(&mut self, method: RgbaBlitMethod, src: &BitmapAtlas<Self>, index: usize, x: i32, y: i32) {
		if let Some(src_region) = src.get(index) {
			self.blit_region(method, src.bitmap(), src_region, x, y);
		}
	}

	#[inline]
	pub unsafe fn blit_unchecked(&mut self, method: RgbaBlitMethod, src: &Self, x: i32, y: i32) {
		let src_region = Rect::new(0, 0, src.width, src.height);
		self.blit_region_unchecked(method, src, &src_region, x, y);
	}

	#[inline]
	pub unsafe fn blit_atlas_unchecked(
		&mut self,
		method: RgbaBlitMethod,
		src: &BitmapAtlas<Self>,
		index: usize,
		x: i32,
		y: i32,
	) {
		if let Some(src_region) = src.get(index) {
			self.blit_region_unchecked(method, src.bitmap(), src_region, x, y);
		}
	}
}
