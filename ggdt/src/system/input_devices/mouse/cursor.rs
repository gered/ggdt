use crate::graphics::*;
use crate::graphics::indexed::*;
use crate::math::*;

use super::*;

const DEFAULT_MOUSE_CURSOR_HOTSPOT_X: u32 = 0;
const DEFAULT_MOUSE_CURSOR_HOTSPOT_Y: u32 = 0;
const DEFAULT_MOUSE_CURSOR_WIDTH: usize = 16;
const DEFAULT_MOUSE_CURSOR_HEIGHT: usize = 16;

pub trait DefaultMouseCursorBitmaps<BitmapType>
where
	BitmapType: BasicImage
{
	fn get_default() -> BitmapType;
}

/// Provides custom mouse cursor rendering functionality via application provided [`Bitmap`]s (or falling back
/// to a default cursor if none is set by the application). Custom cursor rendering is disabled by default.
#[derive(Debug)]
pub struct CustomMouseCursor<BitmapType>
where
	BitmapType: BasicImage
{
	last_x: i32,
	last_y: i32,
	cursor: BitmapType,
	cursor_background: BitmapType,
	cursor_hotspot_x: u32,
	cursor_hotspot_y: u32,
	cursor_enabled: bool,
}

impl<BitmapType> CustomMouseCursor<BitmapType>
where
	Self: DefaultMouseCursorBitmaps<BitmapType>,
	BitmapType: BasicImage
{
	pub fn new() -> Self {
		let (cursor, cursor_background, cursor_hotspot_x, cursor_hotspot_y) = Self::get_default_mouse_cursor();

		CustomMouseCursor {
			last_x: 0,
			last_y: 0,
			cursor,
			cursor_background,
			cursor_hotspot_x,
			cursor_hotspot_y,
			cursor_enabled: false,
		}
	}

	/// Returns a reference to the current mouse cursor bitmap.
	#[inline]
	pub fn cursor_bitmap(&self) -> &BitmapType {
		&self.cursor
	}

	/// Returns the current mouse cursor's "hotspot" x coordinate.
	#[inline]
	pub fn cursor_hotspot_x(&self) -> u32 {
		self.cursor_hotspot_x
	}

	/// Returns the current mouse cursor's "hotspot" y coordinate.
	#[inline]
	pub fn cursor_hotspot_y(&self) -> u32 {
		self.cursor_hotspot_y
	}

	/// Returns true if mouse cursor bitmap rendering is enabled.
	#[inline]
	pub fn is_cursor_enabled(&self) -> bool {
		self.cursor_enabled
	}

	/// Enables or disables mouse cursor bitmap rendering.
	#[inline]
	pub fn enable_cursor(&mut self, enable: bool) {
		self.cursor_enabled = enable;
	}

	/// Sets the [`Bitmap`] used to display the mouse cursor and the "hotspot" coordinate. The
	/// bitmap provided here should be set up to use color 255 as the transparent color.
	///
	/// # Arguments
	///
	/// * `cursor`: the bitmap to be used to display the mouse cursor on screen
	/// * `hotspot_x`: the "hotspot" x coordinate
	/// * `hotspot_y`: the "hotspot" y coordinate.
	pub fn set_mouse_cursor(&mut self, cursor: BitmapType, hotspot_x: u32, hotspot_y: u32) {
		self.cursor = cursor;
		self.cursor_background = BitmapType::new(self.cursor.width(), self.cursor.height()).unwrap();
		self.cursor_hotspot_x = hotspot_x;
		self.cursor_hotspot_y = hotspot_y;
	}

	/// Resets the mouse cursor bitmap and "hotspot" coordinate back to the default settings.
	pub fn set_default_mouse_cursor(&mut self) {
		let (cursor, background, hotspot_x, hotspot_y) = Self::get_default_mouse_cursor();
		self.cursor = cursor;
		self.cursor_background = background;
		self.cursor_hotspot_x = hotspot_x;
		self.cursor_hotspot_y = hotspot_y;
	}

	#[inline]
	fn get_cursor_render_position(&self) -> (i32, i32) {
		(
			self.last_x - self.cursor_hotspot_x as i32,
			self.last_y - self.cursor_hotspot_y as i32,
		)
	}

	/// Renders the mouse cursor bitmap onto the destination bitmap at the mouse's current
	/// position. The destination bitmap specified is usually the [`SystemResources`]'s video
	/// backbuffer bitmap. The background on the destination bitmap is saved internally and a
	/// subsequent call to [`Self::hide`] will restore the background.
	///
	/// If mouse cursor rendering is not currently enabled, this method does nothing.
	///
	/// Applications will not normally need to call this method, as if mouse cursor rendering is
	/// enabled, this will be automatically handled by [`SystemResources::display`].
	///
	/// [`SystemResources`]: crate::system::SystemResources
	/// [`SystemResources::display`]: crate::system::SystemResources::display
	pub fn render(&mut self, dest: &mut BitmapType) {
		if !self.cursor_enabled {
			return;
		}

		let (x, y) = self.get_cursor_render_position();

		// preserve existing background first
		self.cursor_background.blit_region(
			BasicBlitMethod::Solid,
			&dest,
			&Rect::new(x, y, self.cursor.width(), self.cursor.height()),
			0,
			0,
		);

		let color = 255;
		dest.blit(BasicBlitMethod::Transparent(color), &self.cursor, x, y);
	}

	/// Restores the original destination bitmap contents where the mouse cursor bitmap was
	/// rendered to during the previous call to [`Self::render`]. The destination bitmap
	/// specified is usually the [`SystemResources`]'s video backbuffer bitmap.
	///
	/// If mouse cursor rendering is not currently enabled, this method does nothing.
	///
	/// Applications will not normally need to call this method, as if mouse cursor rendering is
	/// enabled, this will be automatically handled by [`SystemResources::display`].
	///
	/// [`SystemResources`]: crate::system::SystemResources
	/// [`SystemResources::display`]: crate::system::SystemResources::display
	pub fn hide(&mut self, dest: &mut BitmapType) {
		if !self.cursor_enabled {
			return;
		}

		let (x, y) = self.get_cursor_render_position();
		dest.blit(BasicBlitMethod::Solid, &self.cursor_background, x, y);
	}

	/// Updates current state from the given [`Mouse`] device's state, ensuring that subsequent calls to render
	/// a custom mouse cursor reflect the current mouse state. Application's should not normally need to call
	/// this directly as it will be called by [`SystemResources::update`].
	///
	/// [`SystemResources::update`]: crate::system::SystemResources::update
	pub fn update(&mut self, mouse: &Mouse) {
		self.last_x = mouse.x;
		self.last_y = mouse.y;
	}

	fn get_default_mouse_cursor() -> (BitmapType, BitmapType, u32, u32) {
		let cursor = Self::get_default();
		let cursor_background = BitmapType::new(cursor.width(), cursor.height()).unwrap();

		(
			cursor,
			cursor_background,
			DEFAULT_MOUSE_CURSOR_HOTSPOT_X,
			DEFAULT_MOUSE_CURSOR_HOTSPOT_Y,
		)
	}
}

impl DefaultMouseCursorBitmaps<indexed::Bitmap> for CustomMouseCursor<indexed::Bitmap> {
	fn get_default() -> Bitmap {
		#[rustfmt::skip]
		const CURSOR_PIXELS: [u8; DEFAULT_MOUSE_CURSOR_WIDTH * DEFAULT_MOUSE_CURSOR_HEIGHT] = [
			0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0x00, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0x00, 0x0f, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0x00, 0x0f, 0x0f, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0x00, 0x0f, 0x0f, 0x0f, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0x00, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0x00, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0x00, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0x00, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0x00, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff,
			0x00, 0x0f, 0x0f, 0x00, 0x0f, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0x00, 0x0f, 0x00, 0x00, 0x00, 0x0f, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0x00, 0x00, 0xff, 0xff, 0x00, 0x0f, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x0f, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x0f, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
			0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff
		];

		let mut cursor = Bitmap::new(
			DEFAULT_MOUSE_CURSOR_WIDTH as u32,
			DEFAULT_MOUSE_CURSOR_HEIGHT as u32,
		).unwrap();
		cursor.pixels_mut().copy_from_slice(&CURSOR_PIXELS);
		cursor
	}
}

// TODO: other DefaultMouseCursorBitmaps impl's here ... when the other bitmap types are added
