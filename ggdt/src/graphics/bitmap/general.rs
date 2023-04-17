//! The purpose of this module is to provide "bit-depth-agnostic" Bitmap drawing capabilities. Basically access to
//! drawing operations that are common or shared across all different Bitmap implementations. This isn't intended to be
//! used all the time by applications, but is useful for certain functionality that we'd like to make generic across
//! all available Bitmap types, where the drawing operations used aren't actually specific to any specific pixel
//! bit-depth.
//!
//! Only a subset of the most common Bitmap drawing operations will be provided here.

use crate::graphics::bitmap::indexed::{IndexedBitmap, IndexedBlitMethod};
use crate::graphics::bitmap::rgb::{RgbaBitmap, RgbaBlitMethod};
use crate::graphics::bitmap::BitmapError;
use crate::graphics::font::{Font, FontRenderOpts};
use crate::graphics::Pixel;
use crate::math::rect::Rect;

#[derive(Clone, PartialEq)]
pub enum GeneralBlitMethod<PixelType: Pixel> {
	Solid,
	Transparent(PixelType),
}

/// Trait that provides "bit-depth-agnostic" access to bitmap drawing operations. This is useful for implementing
/// drawing functionality that is to be made generic across all supported bitmap types and is not specific to
/// any one pixel-depth. Note that this does not provide cross-bit-depth drawing support.
pub trait GeneralBitmap: Sized + Clone {
	type PixelType: Pixel;

	/// Creates a new bitmap with the specified dimensions, in pixels.
	fn new(width: u32, height: u32) -> Result<Self, BitmapError>;

	/// Returns the width of the bitmap in pixels.
	fn width(&self) -> u32;

	/// Returns the height of the bitmap in pixels.
	fn height(&self) -> u32;

	/// Returns the right x coordinate of the bitmap.
	fn right(&self) -> u32;

	/// Returns the bottom x coordinate of the bitmap.
	fn bottom(&self) -> u32;

	/// Returns the current clipping region set on this bitmap.
	fn clip_region(&self) -> &Rect;

	/// Returns a rect representing the full bitmap boundaries, ignoring the current clipping
	/// region set on this bitmap.
	fn full_bounds(&self) -> Rect;

	/// Returns the bit-depth of this bitmap's pixels.
	fn bpp(&self) -> usize {
		std::mem::size_of::<Self::PixelType>() * 8
	}

	/// Fills the entire bitmap with the given color.
	fn clear(&mut self, color: Self::PixelType);

	/// Sets the pixel at the given coordinates to the color specified. If the coordinates lie
	/// outside of the bitmaps clipping region, no pixels will be changed.
	fn set_pixel(&mut self, x: i32, y: i32, color: Self::PixelType);

	/// Gets the pixel at the given coordinates. If the coordinates lie outside of the bitmaps
	/// clipping region, None is returned.
	fn get_pixel(&self, x: i32, y: i32) -> Option<Self::PixelType>;

	/// Draws a line from x1,y1 to x2,y2.
	fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Self::PixelType);

	/// Draws a horizontal line from x1,y to x2,y.
	fn horiz_line(&mut self, x1: i32, x2: i32, y: i32, color: Self::PixelType);

	/// Draws a vertical line from x,y1 to x,y2.
	fn vert_line(&mut self, x: i32, y1: i32, y2: i32, color: Self::PixelType);

	/// Draws an empty box (rectangle) using the points x1,y1 and x2,y2 to form the box to be
	/// drawn, assuming they are specifying the top-left and bottom-right corners respectively.
	fn rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Self::PixelType);

	/// Draws a filled box (rectangle) using the points x1,y1 and x2,y2 to form the box to be
	/// drawn, assuming they are specifying the top-left and bottom-right corners respectively.
	fn filled_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Self::PixelType);

	/// Draws the outline of a circle formed by the center point and radius given.
	fn circle(&mut self, center_x: i32, center_y: i32, radius: u32, color: Self::PixelType);

	/// Draws a filled circle formed by the center point and radius given.
	fn filled_circle(&mut self, center_x: i32, center_y: i32, radius: u32, color: Self::PixelType);

	/// Renders a single character using the font given.
	fn print_char<T: Font>(&mut self, ch: char, x: i32, y: i32, opts: FontRenderOpts<Self::PixelType>, font: &T);

	/// Renders the string of text using the font given.
	fn print_string<T: Font>(&mut self, text: &str, x: i32, y: i32, opts: FontRenderOpts<Self::PixelType>, font: &T);

	fn blit_region(
		&mut self,
		method: GeneralBlitMethod<Self::PixelType>,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
	);

	fn blit(&mut self, method: GeneralBlitMethod<Self::PixelType>, src: &Self, x: i32, y: i32) {
		let src_region = Rect::new(0, 0, src.width(), src.height());
		self.blit_region(method, src, &src_region, x, y);
	}
}

impl GeneralBitmap for IndexedBitmap {
	type PixelType = u8;

	#[inline]
	fn new(width: u32, height: u32) -> Result<Self, BitmapError> {
		Self::new(width, height)
	}

	#[inline]
	fn width(&self) -> u32 {
		self.width()
	}

	#[inline]
	fn height(&self) -> u32 {
		self.height()
	}

	#[inline]
	fn right(&self) -> u32 {
		self.right()
	}

	#[inline]
	fn bottom(&self) -> u32 {
		self.bottom()
	}

	#[inline]
	fn clip_region(&self) -> &Rect {
		self.clip_region()
	}

	#[inline]
	fn full_bounds(&self) -> Rect {
		self.full_bounds()
	}

	#[inline]
	fn clear(&mut self, color: Self::PixelType) {
		self.clear(color)
	}

	#[inline]
	fn set_pixel(&mut self, x: i32, y: i32, color: Self::PixelType) {
		self.set_pixel(x, y, color)
	}

	#[inline]
	fn get_pixel(&self, x: i32, y: i32) -> Option<Self::PixelType> {
		self.get_pixel(x, y)
	}

	#[inline]
	fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Self::PixelType) {
		self.line(x1, y1, x2, y2, color)
	}

	#[inline]
	fn horiz_line(&mut self, x1: i32, x2: i32, y: i32, color: Self::PixelType) {
		self.horiz_line(x1, x2, y, color)
	}

	#[inline]
	fn vert_line(&mut self, x: i32, y1: i32, y2: i32, color: Self::PixelType) {
		self.vert_line(x, y1, y2, color)
	}

	#[inline]
	fn rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Self::PixelType) {
		self.rect(x1, y1, x2, y2, color)
	}

	#[inline]
	fn filled_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Self::PixelType) {
		self.filled_rect(x1, y1, x2, y2, color)
	}

	#[inline]
	fn circle(&mut self, center_x: i32, center_y: i32, radius: u32, color: Self::PixelType) {
		self.circle(center_x, center_y, radius, color)
	}

	#[inline]
	fn filled_circle(&mut self, center_x: i32, center_y: i32, radius: u32, color: Self::PixelType) {
		self.filled_circle(center_x, center_y, radius, color)
	}

	#[inline]
	fn print_char<T: Font>(&mut self, ch: char, x: i32, y: i32, opts: FontRenderOpts<Self::PixelType>, font: &T) {
		self.print_char(ch, x, y, opts, font);
	}

	#[inline]
	fn print_string<T: Font>(&mut self, text: &str, x: i32, y: i32, opts: FontRenderOpts<Self::PixelType>, font: &T) {
		self.print_string(text, x, y, opts, font);
	}

	fn blit_region(
		&mut self,
		method: GeneralBlitMethod<Self::PixelType>,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
	) {
		let blit_method = match method {
			GeneralBlitMethod::Solid => IndexedBlitMethod::Solid,
			GeneralBlitMethod::Transparent(color) => IndexedBlitMethod::Transparent(color),
		};
		self.blit_region(blit_method, src, src_region, dest_x, dest_y)
	}
}

impl GeneralBitmap for RgbaBitmap {
	type PixelType = u32;

	#[inline]
	fn new(width: u32, height: u32) -> Result<Self, BitmapError> {
		Self::new(width, height)
	}

	#[inline]
	fn width(&self) -> u32 {
		self.width()
	}

	#[inline]
	fn height(&self) -> u32 {
		self.height()
	}

	#[inline]
	fn right(&self) -> u32 {
		self.right()
	}

	#[inline]
	fn bottom(&self) -> u32 {
		self.bottom()
	}

	#[inline]
	fn clip_region(&self) -> &Rect {
		self.clip_region()
	}

	#[inline]
	fn full_bounds(&self) -> Rect {
		self.full_bounds()
	}

	#[inline]
	fn clear(&mut self, color: Self::PixelType) {
		self.clear(color)
	}

	#[inline]
	fn set_pixel(&mut self, x: i32, y: i32, color: Self::PixelType) {
		self.set_pixel(x, y, color)
	}

	#[inline]
	fn get_pixel(&self, x: i32, y: i32) -> Option<Self::PixelType> {
		self.get_pixel(x, y)
	}

	#[inline]
	fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Self::PixelType) {
		self.line(x1, y1, x2, y2, color)
	}

	#[inline]
	fn horiz_line(&mut self, x1: i32, x2: i32, y: i32, color: Self::PixelType) {
		self.horiz_line(x1, x2, y, color)
	}

	#[inline]
	fn vert_line(&mut self, x: i32, y1: i32, y2: i32, color: Self::PixelType) {
		self.vert_line(x, y1, y2, color)
	}

	#[inline]
	fn rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Self::PixelType) {
		self.rect(x1, y1, x2, y2, color)
	}

	#[inline]
	fn filled_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Self::PixelType) {
		self.filled_rect(x1, y1, x2, y2, color)
	}

	#[inline]
	fn circle(&mut self, center_x: i32, center_y: i32, radius: u32, color: Self::PixelType) {
		self.circle(center_x, center_y, radius, color)
	}

	#[inline]
	fn filled_circle(&mut self, center_x: i32, center_y: i32, radius: u32, color: Self::PixelType) {
		self.filled_circle(center_x, center_y, radius, color)
	}

	#[inline]
	fn print_char<T: Font>(&mut self, ch: char, x: i32, y: i32, opts: FontRenderOpts<Self::PixelType>, font: &T) {
		self.print_char(ch, x, y, opts, font);
	}

	#[inline]
	fn print_string<T: Font>(&mut self, text: &str, x: i32, y: i32, opts: FontRenderOpts<Self::PixelType>, font: &T) {
		self.print_string(text, x, y, opts, font);
	}

	fn blit_region(
		&mut self,
		method: GeneralBlitMethod<Self::PixelType>,
		src: &Self,
		src_region: &Rect,
		dest_x: i32,
		dest_y: i32,
	) {
		let blit_method = match method {
			GeneralBlitMethod::Solid => RgbaBlitMethod::Solid,
			GeneralBlitMethod::Transparent(color) => RgbaBlitMethod::Transparent(color),
		};
		self.blit_region(blit_method, src, src_region, dest_x, dest_y)
	}
}
