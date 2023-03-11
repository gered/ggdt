use crate::graphics::bitmap::blit::clip_blit;
use crate::graphics::bitmap::rgb::RgbaBitmap;
use crate::graphics::bitmapatlas::BitmapAtlas;
use crate::math::rect::Rect;

#[derive(Clone, PartialEq)]
pub enum RgbaBlitMethod {
	/// Solid blit, no transparency or other per-pixel adjustments.
	Solid,
	/// Same as [RgbaBlitMethod::Solid] but the drawn image can also be flipped horizontally
	/// and/or vertically.
	SolidFlipped {
		horizontal_flip: bool,
		vertical_flip: bool,
	},
	/// Transparent blit, the specified source color pixels are skipped.
	Transparent(u32),
	/// Same as [RgbaBlitMethod::Transparent] but the drawn image can also be flipped horizontally
	/// and/or vertically.
	TransparentFlipped {
		transparent_color: u32,
		horizontal_flip: bool,
		vertical_flip: bool,
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
	/// Same as [RgbaBlitMethod::RotoZoom] except that the specified source color pixels are skipped.
	RotoZoomTransparent {
		angle: f32,
		scale_x: f32,
		scale_y: f32,
		transparent_color: u32,
	},
}

impl RgbaBitmap {
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
			RotoZoomTransparent { .. } => {}

			// set axis flip arguments
			SolidFlipped { horizontal_flip, vertical_flip, .. } |
			TransparentFlipped { horizontal_flip, vertical_flip, .. } |
			TransparentFlippedSingle { horizontal_flip, vertical_flip, .. } => {
				if !clip_blit(
					self.clip_region(),
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
					self.clip_region(),
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
			SolidFlipped { horizontal_flip, vertical_flip } => {
				self.solid_flipped_blit(src, src_region, dest_x, dest_y, horizontal_flip, vertical_flip)
			}
			Transparent(transparent_color) => {
				self.transparent_blit(src, src_region, dest_x, dest_y, transparent_color)
			}
			TransparentFlipped { transparent_color, horizontal_flip, vertical_flip } => {
				self.transparent_flipped_blit(src, src_region, dest_x, dest_y, transparent_color, horizontal_flip, vertical_flip)
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
			RotoZoomTransparent { angle, scale_x, scale_y, transparent_color } => {
				self.rotozoom_transparent_blit(src, src_region, dest_x, dest_y, angle, scale_x, scale_y, transparent_color)
			}
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
	pub unsafe fn blit_atlas_unchecked(&mut self, method: RgbaBlitMethod, src: &BitmapAtlas<Self>, index: usize, x: i32, y: i32) {
		if let Some(src_region) = src.get(index) {
			self.blit_region_unchecked(method, src.bitmap(), &src_region, x, y);
		}
	}
}
