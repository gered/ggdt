use thiserror::Error;

use crate::graphics::Pixel;
use crate::math::rect::Rect;

pub mod blit;
pub mod general;
pub mod gif;
pub mod iff;
pub mod indexed;
pub mod pcx;
pub mod primitives;
pub mod rgb;

#[derive(Error, Debug)]
pub enum BitmapError {
	#[error("Invalid bitmap dimensions")]
	InvalidDimensions,

	#[error("Region is not fully within bitmap boundaries")]
	OutOfBounds,

	#[error("Unknown bitmap file type: {0}")]
	UnknownFileType(String),

	#[error("Bitmap IFF file error")]
	IffError(#[from] iff::IffError),

	#[error("Bitmap PCX file error")]
	PcxError(#[from] pcx::PcxError),

	#[error("Bitmap GIF file error")]
	GifError(#[from] gif::GifError),
}

/// Container for 256 color 2D pixel/image data that can be rendered to the screen. Pixel data
/// is stored as contiguous bytes, where each pixel is an index into a separate 256 color palette
/// stored independently of the bitmap. The pixel data is not padded in any way, so the stride from
/// one row to the next is always exactly equal to the bitmap width. Rendering operations provided
/// here are done with respect to the bitmaps clipping region, where rendering outside of the
/// clipping region is simply not performed / stops at the clipping boundary.
#[derive(Clone, Eq, PartialEq)]
pub struct Bitmap<PixelType: Pixel> {
	width: u32,
	height: u32,
	pixels: Box<[PixelType]>,
	clip_region: Rect,
}

impl<PixelType: Pixel> std::fmt::Debug for Bitmap<PixelType> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Bitmap")
			.field("width", &self.width)
			.field("height", &self.height)
			.field("clip_region", &self.clip_region)
			.finish_non_exhaustive()
	}
}

impl<PixelType: Pixel> Bitmap<PixelType> {
	pub const PIXEL_SIZE: usize = std::mem::size_of::<PixelType>();

	/// Creates a new Bitmap with the specified dimensions.
	///
	/// # Arguments
	///
	/// * `width`: the width of the bitmap in pixels
	/// * `height`: the height of the bitmap in pixels
	///
	/// returns: `Result<Bitmap, BitmapError>`
	pub fn new(width: u32, height: u32) -> Result<Self, BitmapError> {
		if width == 0 || height == 0 {
			return Err(BitmapError::InvalidDimensions);
		}

		Ok(Bitmap {
			width,
			height,
			pixels: vec![Default::default(); (width * height) as usize].into_boxed_slice(),
			clip_region: Rect {
				x: 0,
				y: 0,
				width,
				height,
			},
		})
	}

	/// Creates a new Bitmap, copying the pixel data from a sub-region of another source Bitmap.
	/// The resulting bitmap will have dimensions equal to that of the region specified.
	///
	/// # Arguments
	///
	/// * `source`: the source bitmap to copy from
	/// * `region`: the region on the source bitmap to copy from
	///
	/// returns: `Result<Bitmap, BitmapError>`
	pub fn from(source: &Self, region: &Rect) -> Result<Self, BitmapError> {
		if !source.full_bounds().contains_rect(region) {
			return Err(BitmapError::OutOfBounds);
		}

		let mut bmp = Self::new(region.width, region.height)?;
		unsafe { bmp.solid_blit(source, region, 0, 0) };
		Ok(bmp)
	}

	/// Returns the width of the bitmap in pixels.
	#[inline]
	pub fn width(&self) -> u32 {
		self.width
	}

	/// Returns the height of the bitmap in pixels.
	#[inline]
	pub fn height(&self) -> u32 {
		self.height
	}

	/// Returns the right x coordinate of the bitmap.
	#[inline]
	pub fn right(&self) -> u32 {
		self.width - 1
	}

	/// Returns the bottom x coordinate of the bitmap.
	#[inline]
	pub fn bottom(&self) -> u32 {
		self.height - 1
	}

	/// Returns the current clipping region set on this bitmap.
	#[inline]
	pub fn clip_region(&self) -> &Rect {
		&self.clip_region
	}

	/// Returns a rect representing the full bitmap boundaries, ignoring the current clipping
	/// region set on this bitmap.
	#[inline]
	pub fn full_bounds(&self) -> Rect {
		Rect {
			x: 0,
			y: 0,
			width: self.width,
			height: self.height,
		}
	}

	/// Sets a new clipping region on this bitmap. The region will be automatically clamped to
	/// the maximum bitmap boundaries if the supplied region extends beyond it.
	///
	/// # Arguments
	///
	/// * `region`: the new clipping region
	pub fn set_clip_region(&mut self, region: &Rect) {
		self.clip_region = *region;
		self.clip_region.clamp_to(&self.full_bounds());
	}

	/// Resets the bitmaps clipping region back to the default (full boundaries of the bitmap).
	pub fn reset_clip_region(&mut self) {
		self.clip_region = self.full_bounds();
	}

	/// Returns a reference to the raw pixels in this bitmap.
	#[inline]
	pub fn pixels(&self) -> &[PixelType] {
		&self.pixels
	}

	/// Returns a mutable reference to the raw pixels in this bitmap.
	#[inline]
	pub fn pixels_mut(&mut self) -> &mut [PixelType] {
		&mut self.pixels
	}

	/// Returns a reference to the subset of the raw pixels in this bitmap beginning at the
	/// given coordinates and extending to the end of the bitmap. If the coordinates given are
	/// outside the bitmap's current clipping region, None is returned.
	#[inline]
	pub fn pixels_at(&self, x: i32, y: i32) -> Option<&[PixelType]> {
		if self.is_xy_visible(x, y) {
			let offset = self.get_offset_to_xy(x, y);
			Some(&self.pixels[offset..])
		} else {
			None
		}
	}

	/// Returns a mutable reference to the subset of the raw pixels in this bitmap beginning at the
	/// given coordinates and extending to the end of the bitmap. If the coordinates given are
	/// outside the bitmap's current clipping region, None is returned.
	#[inline]
	pub fn pixels_at_mut(&mut self, x: i32, y: i32) -> Option<&mut [PixelType]> {
		if self.is_xy_visible(x, y) {
			let offset = self.get_offset_to_xy(x, y);
			Some(&mut self.pixels[offset..])
		} else {
			None
		}
	}

	/// Returns an unsafe reference to the subset of the raw pixels in this bitmap beginning at the
	/// given coordinates and extending to the end of the bitmap. The coordinates are not checked
	/// for validity, so it is up to you to ensure they lie within the bounds of the bitmap.
	#[inline]
	pub unsafe fn pixels_at_unchecked(&self, x: i32, y: i32) -> &[PixelType] {
		let offset = self.get_offset_to_xy(x, y);
		std::slice::from_raw_parts(self.pixels.as_ptr().add(offset), self.pixels.len() - offset)
	}

	/// Returns a mutable unsafe reference to the subset of the raw pixels in this bitmap beginning
	/// at the given coordinates and extending to the end of the bitmap. The coordinates are not
	/// checked for validity, so it is up to you to ensure they lie within the bounds of the bitmap.
	#[inline]
	pub unsafe fn pixels_at_mut_unchecked(&mut self, x: i32, y: i32) -> &mut [PixelType] {
		let offset = self.get_offset_to_xy(x, y);
		std::slice::from_raw_parts_mut(
			self.pixels.as_mut_ptr().add(offset),
			self.pixels.len() - offset,
		)
	}

	/// Returns a pointer to the subset of the raw pixels in this bitmap beginning at the given
	/// coordinates. If the coordinates given are outside the bitmap's current clipping region,
	/// None is returned.
	#[inline]
	pub unsafe fn pixels_at_ptr(&self, x: i32, y: i32) -> Option<*const PixelType> {
		if self.is_xy_visible(x, y) {
			let offset = self.get_offset_to_xy(x, y);
			Some(self.pixels.as_ptr().add(offset))
		} else {
			None
		}
	}

	/// Returns a mutable pointer to the subset of the raw pixels in this bitmap beginning at the
	/// given coordinates. If the coordinates given are outside the bitmap's current clipping
	/// region, None is returned.
	#[inline]
	pub unsafe fn pixels_at_mut_ptr(&mut self, x: i32, y: i32) -> Option<*mut PixelType> {
		if self.is_xy_visible(x, y) {
			let offset = self.get_offset_to_xy(x, y);
			Some(self.pixels.as_mut_ptr().add(offset))
		} else {
			None
		}
	}

	/// Returns an unsafe pointer to the subset of the raw pixels in this bitmap beginning at the
	/// given coordinates. The coordinates are not checked for validity, so it is up to you to
	/// ensure they lie within the bounds of the bitmap.
	#[inline]
	pub unsafe fn pixels_at_ptr_unchecked(&self, x: i32, y: i32) -> *const PixelType {
		let offset = self.get_offset_to_xy(x, y);
		self.pixels.as_ptr().add(offset)
	}

	/// Returns a mutable unsafe pointer to the subset of the raw pixels in this bitmap beginning
	/// at the given coordinates. The coordinates are not checked for validity, so it is up to you
	/// to ensure they lie within the bounds of the bitmap.
	#[inline]
	pub unsafe fn pixels_at_mut_ptr_unchecked(&mut self, x: i32, y: i32) -> *mut PixelType {
		let offset = self.get_offset_to_xy(x, y);
		self.pixels.as_mut_ptr().add(offset)
	}

	/// Returns an offset corresponding to the coordinates of the pixel given on this bitmap that
	/// can be used with a reference to the raw pixels in this bitmap to access that pixel. The
	/// coordinates given are not checked for validity.
	#[inline]
	pub fn get_offset_to_xy(&self, x: i32, y: i32) -> usize {
		((y * self.width as i32) + x) as usize
	}

	/// Returns true if the coordinates given lie within the bitmaps clipping region.
	#[inline]
	pub fn is_xy_visible(&self, x: i32, y: i32) -> bool {
		(x >= self.clip_region.x)
			&& (y >= self.clip_region.y)
			&& (x <= self.clip_region.right())
			&& (y <= self.clip_region.bottom())
	}
}

#[cfg(test)]
pub mod tests {
	use claim::assert_matches;

	use super::*;

	#[rustfmt::skip]
	static RAW_BMP_PIXELS: &[u8] = &[
		0, 0, 0, 0, 0, 0, 0, 0,
		0, 1, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 2,
	];

	#[rustfmt::skip]
	static RAW_BMP_PIXELS_SUBSET: &[u8] = &[
		0, 0, 0, 0,
		0, 0, 0, 0,
		0, 0, 0, 0,
		0, 0, 0, 2,
	];

	#[test]
	pub fn creation_and_sizing() {
		assert_matches!(Bitmap::<u8>::new(0, 0), Err(BitmapError::InvalidDimensions));
		assert_matches!(Bitmap::<u8>::new(16, 0), Err(BitmapError::InvalidDimensions));
		assert_matches!(Bitmap::<u8>::new(0, 32), Err(BitmapError::InvalidDimensions));
		let bmp = Bitmap::<u8>::new(16, 32).unwrap();
		assert_eq!(16, bmp.width());
		assert_eq!(32, bmp.height());
		assert_eq!(15, bmp.right());
		assert_eq!(31, bmp.bottom());
		assert_eq!(
			Rect {
				x: 0,
				y: 0,
				width: 16,
				height: 32,
			},
			bmp.full_bounds()
		);
		assert_eq!(
			Rect {
				x: 0,
				y: 0,
				width: 16,
				height: 32,
			},
			*bmp.clip_region()
		);
	}

	#[test]
	pub fn copy_from() {
		let mut bmp = Bitmap::<u8>::new(8, 8).unwrap();
		bmp.pixels_mut().copy_from_slice(RAW_BMP_PIXELS);

		assert_matches!(
            Bitmap::<u8>::from(&bmp, &Rect::new(0, 0, 16, 16)),
            Err(BitmapError::OutOfBounds)
        );

		let copy = Bitmap::<u8>::from(&bmp, &Rect::new(0, 0, 8, 8)).unwrap();
		assert_eq!(bmp.pixels(), copy.pixels());

		let copy = Bitmap::<u8>::from(&bmp, &Rect::new(4, 4, 4, 4)).unwrap();
		assert_eq!(RAW_BMP_PIXELS_SUBSET, copy.pixels());
	}

	#[test]
	pub fn xy_offset_calculation() {
		let bmp = Bitmap::<u8>::new(20, 15).unwrap();
		assert_eq!(0, bmp.get_offset_to_xy(0, 0));
		assert_eq!(19, bmp.get_offset_to_xy(19, 0));
		assert_eq!(20, bmp.get_offset_to_xy(0, 1));
		assert_eq!(280, bmp.get_offset_to_xy(0, 14));
		assert_eq!(299, bmp.get_offset_to_xy(19, 14));
		assert_eq!(227, bmp.get_offset_to_xy(7, 11));
	}

	#[test]
	pub fn bounds_testing_and_clip_region() {
		let mut bmp = Bitmap::<u8>::new(16, 8).unwrap();
		assert!(bmp.is_xy_visible(0, 0));
		assert!(bmp.is_xy_visible(15, 0));
		assert!(bmp.is_xy_visible(0, 7));
		assert!(bmp.is_xy_visible(15, 7));
		assert!(!bmp.is_xy_visible(-1, -1));
		assert!(!bmp.is_xy_visible(16, 8));
		assert!(!bmp.is_xy_visible(4, -2));
		assert!(!bmp.is_xy_visible(11, 8));
		assert!(!bmp.is_xy_visible(-1, 3));
		assert!(!bmp.is_xy_visible(16, 6));

		let new_clip_region = Rect::from_coords(4, 2, 12, 6);
		bmp.set_clip_region(&new_clip_region);
		assert_eq!(
			Rect {
				x: 0,
				y: 0,
				width: 16,
				height: 8,
			},
			bmp.full_bounds()
		);
		assert_eq!(new_clip_region, *bmp.clip_region());
		assert!(bmp.is_xy_visible(4, 2));
		assert!(bmp.is_xy_visible(12, 2));
		assert!(bmp.is_xy_visible(4, 6));
		assert!(bmp.is_xy_visible(12, 6));
		assert!(!bmp.is_xy_visible(3, 1));
		assert!(!bmp.is_xy_visible(13, 7));
		assert!(!bmp.is_xy_visible(5, 1));
		assert!(!bmp.is_xy_visible(10, 7));
		assert!(!bmp.is_xy_visible(3, 4));
		assert!(!bmp.is_xy_visible(13, 5));

		assert!(!bmp.is_xy_visible(0, 0));
		assert!(!bmp.is_xy_visible(15, 0));
		assert!(!bmp.is_xy_visible(0, 7));
		assert!(!bmp.is_xy_visible(15, 7));
		bmp.reset_clip_region();
		assert!(bmp.is_xy_visible(0, 0));
		assert!(bmp.is_xy_visible(15, 0));
		assert!(bmp.is_xy_visible(0, 7));
		assert!(bmp.is_xy_visible(15, 7));
		assert!(!bmp.is_xy_visible(-1, -1));
		assert!(!bmp.is_xy_visible(16, 8));
	}

	#[test]
	pub fn pixels_at() {
		let mut bmp = Bitmap::<u8>::new(8, 8).unwrap();
		bmp.pixels_mut().copy_from_slice(RAW_BMP_PIXELS);

		assert_eq!(None, bmp.pixels_at(-1, -1));

		let offset = bmp.get_offset_to_xy(1, 1);
		let pixels = bmp.pixels_at(0, 0).unwrap();
		assert_eq!(64, pixels.len());
		assert_eq!(0, pixels[0]);
		assert_eq!(1, pixels[offset]);
		assert_eq!(2, pixels[63]);

		let pixels = bmp.pixels_at(1, 1).unwrap();
		assert_eq!(55, pixels.len());
		assert_eq!(1, pixels[0]);
		assert_eq!(2, pixels[54]);
	}

	#[test]
	pub fn pixels_at_mut() {
		let mut bmp = Bitmap::<u8>::new(8, 8).unwrap();
		bmp.pixels_mut().copy_from_slice(RAW_BMP_PIXELS);

		assert_eq!(None, bmp.pixels_at_mut(-1, -1));

		let offset = bmp.get_offset_to_xy(1, 1);
		let pixels = bmp.pixels_at_mut(0, 0).unwrap();
		assert_eq!(64, pixels.len());
		assert_eq!(0, pixels[0]);
		assert_eq!(1, pixels[offset]);
		assert_eq!(2, pixels[63]);

		let pixels = bmp.pixels_at_mut(1, 1).unwrap();
		assert_eq!(55, pixels.len());
		assert_eq!(1, pixels[0]);
		assert_eq!(2, pixels[54]);
	}

	#[test]
	pub fn pixels_at_unchecked() {
		let mut bmp = Bitmap::<u8>::new(8, 8).unwrap();
		bmp.pixels_mut().copy_from_slice(RAW_BMP_PIXELS);

		let offset = bmp.get_offset_to_xy(1, 1);
		let pixels = unsafe { bmp.pixels_at_unchecked(0, 0) };
		assert_eq!(64, pixels.len());
		assert_eq!(0, pixels[0]);
		assert_eq!(1, pixels[offset]);
		assert_eq!(2, pixels[63]);

		let pixels = unsafe { bmp.pixels_at_unchecked(1, 1) };
		assert_eq!(55, pixels.len());
		assert_eq!(1, pixels[0]);
		assert_eq!(2, pixels[54]);
	}

	#[test]
	pub fn pixels_at_mut_unchecked() {
		let mut bmp = Bitmap::<u8>::new(8, 8).unwrap();
		bmp.pixels_mut().copy_from_slice(RAW_BMP_PIXELS);

		let offset = bmp.get_offset_to_xy(1, 1);
		let pixels = unsafe { bmp.pixels_at_mut_unchecked(0, 0) };
		assert_eq!(64, pixels.len());
		assert_eq!(0, pixels[0]);
		assert_eq!(1, pixels[offset]);
		assert_eq!(2, pixels[63]);

		let pixels = unsafe { bmp.pixels_at_mut_unchecked(1, 1) };
		assert_eq!(55, pixels.len());
		assert_eq!(1, pixels[0]);
		assert_eq!(2, pixels[54]);
	}

	#[test]
	pub fn pixels_at_ptr() {
		let mut bmp = Bitmap::<u8>::new(8, 8).unwrap();
		bmp.pixels_mut().copy_from_slice(RAW_BMP_PIXELS);

		assert_eq!(None, unsafe { bmp.pixels_at_ptr(-1, -1) });

		let offset = bmp.get_offset_to_xy(1, 1);
		let pixels = unsafe { bmp.pixels_at_ptr(0, 0).unwrap() };
		assert_eq!(0, unsafe { *pixels });
		assert_eq!(1, unsafe { *(pixels.add(offset)) });
		assert_eq!(2, unsafe { *(pixels.add(63)) });

		let pixels = unsafe { bmp.pixels_at_ptr(1, 1).unwrap() };
		assert_eq!(1, unsafe { *pixels });
		assert_eq!(2, unsafe { *(pixels.add(54)) });
	}

	#[test]
	pub fn pixels_at_mut_ptr() {
		let mut bmp = Bitmap::<u8>::new(8, 8).unwrap();
		bmp.pixels_mut().copy_from_slice(RAW_BMP_PIXELS);

		assert_eq!(None, unsafe { bmp.pixels_at_mut_ptr(-1, -1) });

		let offset = bmp.get_offset_to_xy(1, 1);
		let pixels = unsafe { bmp.pixels_at_mut_ptr(0, 0).unwrap() };
		assert_eq!(0, unsafe { *pixels });
		assert_eq!(1, unsafe { *(pixels.add(offset)) });
		assert_eq!(2, unsafe { *(pixels.add(63)) });

		let pixels = unsafe { bmp.pixels_at_mut_ptr(1, 1).unwrap() };
		assert_eq!(1, unsafe { *pixels });
		assert_eq!(2, unsafe { *(pixels.add(54)) });
	}

	#[test]
	pub fn pixels_at_ptr_unchecked() {
		let mut bmp = Bitmap::<u8>::new(8, 8).unwrap();
		bmp.pixels_mut().copy_from_slice(RAW_BMP_PIXELS);

		let offset = bmp.get_offset_to_xy(1, 1);
		let pixels = unsafe { bmp.pixels_at_ptr_unchecked(0, 0) };
		assert_eq!(0, unsafe { *pixels });
		assert_eq!(1, unsafe { *(pixels.add(offset)) });
		assert_eq!(2, unsafe { *(pixels.add(63)) });

		let pixels = unsafe { bmp.pixels_at_ptr_unchecked(1, 1) };
		assert_eq!(1, unsafe { *pixels });
		assert_eq!(2, unsafe { *(pixels.add(54)) });
	}

	#[test]
	pub fn pixels_at_mut_ptr_unchecked() {
		let mut bmp = Bitmap::<u8>::new(8, 8).unwrap();
		bmp.pixels_mut().copy_from_slice(RAW_BMP_PIXELS);

		let offset = bmp.get_offset_to_xy(1, 1);
		let pixels = unsafe { bmp.pixels_at_mut_ptr_unchecked(0, 0) };
		assert_eq!(0, unsafe { *pixels });
		assert_eq!(1, unsafe { *(pixels.add(offset)) });
		assert_eq!(2, unsafe { *(pixels.add(63)) });

		let pixels = unsafe { bmp.pixels_at_mut_ptr_unchecked(1, 1) };
		assert_eq!(1, unsafe { *pixels });
		assert_eq!(2, unsafe { *(pixels.add(54)) });
	}
}
