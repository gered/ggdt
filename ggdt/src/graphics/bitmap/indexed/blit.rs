use std::rc::Rc;

use crate::graphics::{
	clip_blit, per_pixel_blit, per_pixel_flipped_blit, per_pixel_rotozoom_blit, BitmapAtlas, BlendMap, IndexedBitmap,
};
use crate::math::Rect;

#[derive(Clone, PartialEq)]
pub enum IndexedBlitMethod {
	/// Solid blit, no transparency or other per-pixel adjustments.
	Solid,
	SolidBlended {
		blend_map: Rc<BlendMap>,
	},
	/// Same as [IndexedBlitMethod::Solid] but the drawn image can also be flipped horizontally
	/// and/or vertically.
	SolidFlipped {
		horizontal_flip: bool,
		vertical_flip: bool,
	},
	SolidFlippedBlended {
		horizontal_flip: bool,
		vertical_flip: bool,
		blend_map: Rc<BlendMap>,
	},
	/// Same as [IndexedBlitMethod::Solid] except that the drawn pixels have their color indices offset
	/// by the amount given.
	SolidOffset(u8),
	/// Combination of [IndexedBlitMethod::SolidFlipped] and [IndexedBlitMethod::SolidOffset].
	SolidFlippedOffset {
		horizontal_flip: bool,
		vertical_flip: bool,
		offset: u8,
	},
	/// Transparent blit, the specified source color pixels are skipped.
	Transparent(u8),
	TransparentBlended {
		transparent_color: u8,
		blend_map: Rc<BlendMap>,
	},
	/// Same as [IndexedBlitMethod::Transparent] but the drawn image can also be flipped horizontally
	/// and/or vertically.
	TransparentFlipped {
		transparent_color: u8,
		horizontal_flip: bool,
		vertical_flip: bool,
	},
	TransparentFlippedBlended {
		transparent_color: u8,
		horizontal_flip: bool,
		vertical_flip: bool,
		blend_map: Rc<BlendMap>,
	},
	/// Same as [IndexedBlitMethod::Transparent] except that the visible pixels on the destination are all
	/// drawn using the same color.
	TransparentSingle {
		transparent_color: u8,
		draw_color: u8,
	},
	/// Combination of [IndexedBlitMethod::TransparentFlipped] and [IndexedBlitMethod::TransparentSingle].
	TransparentFlippedSingle {
		transparent_color: u8,
		horizontal_flip: bool,
		vertical_flip: bool,
		draw_color: u8,
	},
	/// Same as [IndexedBlitMethod::Transparent] except that the drawn pixels have their color indices
	/// offset by the amount given. The transparent color check is not affected by the offset and
	/// is always treated as an absolute palette color index.
	TransparentOffset {
		transparent_color: u8,
		offset: u8,
	},
	/// Combination of [IndexedBlitMethod::TransparentFlipped] and [IndexedBlitMethod::TransparentOffset].
	TransparentFlippedOffset {
		transparent_color: u8,
		horizontal_flip: bool,
		vertical_flip: bool,
		offset: u8,
	},
	/// Rotozoom blit, works the same as [IndexedBlitMethod::Solid] except that rotation and scaling is
	/// performed.
	RotoZoom {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
	},
	RotoZoomBlended {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		blend_map: Rc<BlendMap>,
	},
	/// Same as [IndexedBlitMethod::RotoZoom] except that the specified source color pixels are skipped.
	RotoZoomTransparent {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		transparent_color: u8,
	},
	RotoZoomTransparentBlended {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		transparent_color: u8,
		blend_map: Rc<BlendMap>,
	},
	/// Same as [IndexedBlitMethod::RotoZoom] except that the drawn pixels have their color indices
	/// offset by the amount given.
	RotoZoomOffset {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		offset: u8,
	},
	/// Same as [IndexedBlitMethod::RotoZoomTransparent] except that the drawn pixels have their color
	/// indices offset by the amount given. The transparent color check is not affected by the
	/// offset and is always treated as an absolute palette color index.
	RotoZoomTransparentOffset {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		transparent_color: u8,
		offset: u8,
	},
}

impl IndexedBitmap {
	pub unsafe fn solid_blended_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		blend_map: Rc<BlendMap>,
	) {
		per_pixel_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			|src_pixels, dest_pixels| {
				if let Some(blended_pixel) = blend_map.blend(*src_pixels, *dest_pixels) {
					*dest_pixels = blended_pixel;
				} else {
					*dest_pixels = *src_pixels;
				}
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
		blend_map: Rc<BlendMap>,
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
				if let Some(blended_pixel) = blend_map.blend(*src_pixels, *dest_pixels) {
					*dest_pixels = blended_pixel;
				} else {
					*dest_pixels = *src_pixels;
				}
			},
		);
	}

	pub unsafe fn solid_palette_offset_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		offset: u8,
	) {
		per_pixel_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			|src_pixels, dest_pixels| {
				*dest_pixels = (*src_pixels).wrapping_add(offset);
			},
		);
	}

	pub unsafe fn solid_flipped_palette_offset_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		horizontal_flip: bool,
		vertical_flip: bool,
		offset: u8,
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
				*dest_pixels = (*src_pixels).wrapping_add(offset);
			},
		);
	}

	pub unsafe fn transparent_blended_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		transparent_color: u8,
		blend_map: Rc<BlendMap>,
	) {
		per_pixel_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			|src_pixels, dest_pixels| {
				if *src_pixels != transparent_color {
					if let Some(blended_pixel) = blend_map.blend(*src_pixels, *dest_pixels) {
						*dest_pixels = blended_pixel;
					} else {
						*dest_pixels = *src_pixels;
					}
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
		transparent_color: u8,
		horizontal_flip: bool,
		vertical_flip: bool,
		blend_map: Rc<BlendMap>,
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
					if let Some(blended_pixel) = blend_map.blend(*src_pixels, *dest_pixels) {
						*dest_pixels = blended_pixel;
					} else {
						*dest_pixels = *src_pixels;
					}
				}
			},
		);
	}

	pub unsafe fn transparent_palette_offset_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		transparent_color: u8,
		offset: u8,
	) {
		per_pixel_blit(
			self, //
			src,
			src_region,
			dest_x,
			dest_y,
			|src_pixels, dest_pixels| {
				if *src_pixels != transparent_color {
					*dest_pixels = (*src_pixels).wrapping_add(offset);
				}
			},
		);
	}

	pub unsafe fn transparent_flipped_palette_offset_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		transparent_color: u8,
		horizontal_flip: bool,
		vertical_flip: bool,
		offset: u8,
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
					*dest_pixels = (*src_pixels).wrapping_add(offset);
				}
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
		blend_map: Rc<BlendMap>,
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
					let draw_pixel = if let Some(blended_pixel) = blend_map.blend(src_pixel, dest_pixel) {
						blended_pixel
					} else {
						src_pixel
					};
					dest_bitmap.set_pixel(draw_x, draw_y, draw_pixel);
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
		transparent_color: u8,
		blend_map: Rc<BlendMap>,
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
						let draw_pixel = if let Some(blended_pixel) = blend_map.blend(src_pixel, dest_pixel) {
							blended_pixel
						} else {
							src_pixel
						};
						dest_bitmap.set_pixel(draw_x, draw_y, draw_pixel);
					}
				}
			},
		);
	}

	pub unsafe fn rotozoom_palette_offset_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		offset: u8,
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
				let src_pixel = src_pixel.wrapping_add(offset);
				dest_bitmap.set_pixel(draw_x, draw_y, src_pixel);
			},
		);
	}

	pub unsafe fn rotozoom_transparent_palette_offset_blit(
		&mut self,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		transparent_color: u8,
		offset: u8,
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
					let src_pixel = src_pixel.wrapping_add(offset);
					dest_bitmap.set_pixel(draw_x, draw_y, src_pixel);
				}
			},
		);
	}

	pub fn blit_region(
		&mut self,
		method: IndexedBlitMethod,
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
		use IndexedBlitMethod::*;
		match method {
			// rotozoom blits internally clip per-pixel right now ... and regardless, the normal
			// clip_blit() function wouldn't handle a rotozoom blit destination region anyway ...
			RotoZoom { .. } => {}
			RotoZoomBlended { .. } => {}
			RotoZoomOffset { .. } => {}
			RotoZoomTransparent { .. } => {}
			RotoZoomTransparentBlended { .. } => {}
			RotoZoomTransparentOffset { .. } => {}

			// set axis flip arguments
			SolidFlipped { horizontal_flip, vertical_flip, .. }
			| SolidFlippedBlended { horizontal_flip, vertical_flip, .. }
			| SolidFlippedOffset { horizontal_flip, vertical_flip, .. }
			| TransparentFlipped { horizontal_flip, vertical_flip, .. }
			| TransparentFlippedBlended { horizontal_flip, vertical_flip, .. }
			| TransparentFlippedSingle { horizontal_flip, vertical_flip, .. }
			| TransparentFlippedOffset { horizontal_flip, vertical_flip, .. } => {
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
		method: IndexedBlitMethod,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
	) {
		use IndexedBlitMethod::*;
		match method {
			Solid => self.solid_blit(src, src_region, dest_x, dest_y),
			SolidFlipped { horizontal_flip, vertical_flip } => {
				self.solid_flipped_blit(src, src_region, dest_x, dest_y, horizontal_flip, vertical_flip)
			}
			SolidOffset(offset) => self.solid_palette_offset_blit(src, src_region, dest_x, dest_y, offset),
			SolidFlippedOffset { horizontal_flip, vertical_flip, offset } => {
				self.solid_flipped_palette_offset_blit(src, src_region, dest_x, dest_y, horizontal_flip, vertical_flip, offset)
			}
			Transparent(transparent_color) => {
				self.transparent_blit(src, src_region, dest_x, dest_y, transparent_color)
			}
			TransparentFlipped { transparent_color, horizontal_flip, vertical_flip } => {
				self.transparent_flipped_blit(src, src_region, dest_x, dest_y, transparent_color, horizontal_flip, vertical_flip)
			}
			TransparentOffset { transparent_color, offset } => {
				self.transparent_palette_offset_blit(src, src_region, dest_x, dest_y, transparent_color, offset)
			}
			TransparentFlippedOffset { transparent_color, horizontal_flip, vertical_flip, offset } => {
				self.transparent_flipped_palette_offset_blit(src, src_region, dest_x, dest_y, transparent_color, horizontal_flip, vertical_flip, offset)
			}
			TransparentSingle { transparent_color, draw_color } => {
				self.transparent_single_color_blit(src, src_region, dest_x, dest_y, transparent_color, draw_color)
			}
			TransparentFlippedSingle { transparent_color, horizontal_flip, vertical_flip, draw_color } => {
				self.transparent_flipped_single_color_blit(src, src_region, dest_x, dest_y, transparent_color, horizontal_flip, vertical_flip, draw_color)
			}
			RotoZoom { angle, scale_x, scale_y } => {
				self.rotozoom_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y)
			}
			RotoZoomOffset { angle, scale_x, scale_y, offset } => {
				self.rotozoom_palette_offset_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y, offset)
			}
			RotoZoomTransparent { angle, scale_x, scale_y, transparent_color } => {
				self.rotozoom_transparent_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y, transparent_color)
			}
			RotoZoomTransparentOffset { angle, scale_x, scale_y, transparent_color, offset } => {
				self.rotozoom_transparent_palette_offset_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y, transparent_color, offset)
			}
			SolidBlended { blend_map } => {
				self.solid_blended_blit(src, src_region, dest_x, dest_y, blend_map)
			}
			SolidFlippedBlended { horizontal_flip, vertical_flip, blend_map } => {
				self.solid_flipped_blended_blit(src, src_region, dest_x, dest_y, horizontal_flip, vertical_flip, blend_map)
			}
			TransparentBlended { transparent_color, blend_map } => {
				self.transparent_blended_blit(src, src_region, dest_x, dest_y, transparent_color, blend_map)
			}
			TransparentFlippedBlended { transparent_color, horizontal_flip, vertical_flip, blend_map } => {
				self.transparent_flipped_blended_blit(src, src_region, dest_x, dest_y, transparent_color, horizontal_flip, vertical_flip, blend_map)
			}
			RotoZoomBlended { angle, scale_x, scale_y, blend_map } => {
				self.rotozoom_blended_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y, blend_map)
			}
			RotoZoomTransparentBlended { angle, scale_x, scale_y, transparent_color, blend_map } => {
				self.rotozoom_transparent_blended_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y, transparent_color, blend_map)
			}
		}
	}

	#[inline]
	pub fn blit(&mut self, method: IndexedBlitMethod, src: &Self, x: i32, y: i32) {
		let src_region = Rect::new(0, 0, src.width, src.height);
		self.blit_region(method, src, &src_region, x, y);
	}

	#[inline]
	pub fn blit_atlas(&mut self, method: IndexedBlitMethod, src: &BitmapAtlas<Self>, index: usize, x: i32, y: i32) {
		if let Some(src_region) = src.get(index) {
			self.blit_region(method, src.bitmap(), src_region, x, y);
		}
	}

	#[inline]
	pub unsafe fn blit_unchecked(&mut self, method: IndexedBlitMethod, src: &Self, x: i32, y: i32) {
		let src_region = Rect::new(0, 0, src.width, src.height);
		self.blit_region_unchecked(method, src, &src_region, x, y);
	}

	#[inline]
	pub unsafe fn blit_atlas_unchecked(
		&mut self,
		method: IndexedBlitMethod,
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
