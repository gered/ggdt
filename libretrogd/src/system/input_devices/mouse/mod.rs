use crate::graphics::*;
use crate::math::*;
use crate::system::MouseEvent;

use super::*;

pub use self::buttons::*;

pub mod buttons;

const MAX_BUTTONS: usize = 32;

const DEFAULT_MOUSE_CURSOR_HOTSPOT_X: u32 = 0;
const DEFAULT_MOUSE_CURSOR_HOTSPOT_Y: u32 = 0;
const DEFAULT_MOUSE_CURSOR_WIDTH: usize = 16;
const DEFAULT_MOUSE_CURSOR_HEIGHT: usize = 16;

#[rustfmt::skip]
const DEFAULT_MOUSE_CURSOR: [u8; DEFAULT_MOUSE_CURSOR_WIDTH * DEFAULT_MOUSE_CURSOR_HEIGHT] = [
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

/// Holds the current state of the mouse.
///
/// Must be explicitly updated each frame by calling `handle_event` each frame for all SDL2 events
/// received, as well as calling `do_events` once each frame. Usually, you would accomplish all
/// this house-keeping by simply calling [`System`]'s `do_events` method once per frame.
///
/// [`System`]: crate::System
#[derive(Debug)]
pub struct Mouse {
	x: i32,
	y: i32,
	x_delta: i32,
	y_delta: i32,
	buttons: [ButtonState; MAX_BUTTONS],
	cursor: Bitmap,
	cursor_background: Bitmap,
	cursor_hotspot_x: u32,
	cursor_hotspot_y: u32,
	cursor_enabled: bool,
}

impl Mouse {
	pub fn new() -> Mouse {
		let (cursor, cursor_background, cursor_hotspot_x, cursor_hotspot_y) =
			Self::get_default_mouse_cursor();

		Mouse {
			x: 0,
			y: 0,
			x_delta: 0,
			y_delta: 0,
			buttons: [ButtonState::Idle; MAX_BUTTONS],
			cursor,
			cursor_background,
			cursor_hotspot_x,
			cursor_hotspot_y,
			cursor_enabled: false,
		}
	}

	/// Returns the current x coordinate of the mouse cursor.
	#[inline]
	pub fn x(&self) -> i32 {
		self.x
	}

	/// Returns the current y coordinate of the mouse cursor.
	#[inline]
	pub fn y(&self) -> i32 {
		self.y
	}

	/// Returns the amount of pixels along the x-axis that the mouse cursor moved since the last
	/// time that the mouse state was updated.
	#[inline]
	pub fn x_delta(&self) -> i32 {
		self.x_delta
	}

	/// Returns the amount of pixels along the y-axis that the mouse cursor moved since the last
	/// time that the mouse state was updated.
	#[inline]
	pub fn y_delta(&self) -> i32 {
		self.y_delta
	}

	/// Returns true if the given button was just pressed or is being held down.
	#[inline]
	pub fn is_button_down(&self, button: usize) -> bool {
		matches!(
            self.buttons[button],
            ButtonState::Pressed | ButtonState::Held
        )
	}

	/// Returns true if the given button was not just pressed and is not being held down.
	#[inline]
	pub fn is_button_up(&self, button: usize) -> bool {
		matches!(
            self.buttons[button],
            ButtonState::Released | ButtonState::Idle
        )
	}

	/// Returns true if the given button was just pressed (not being held down, yet).
	#[inline]
	pub fn is_button_pressed(&self, button: usize) -> bool {
		self.buttons[button] == ButtonState::Pressed
	}

	/// Returns true if the given button was just released.
	#[inline]
	pub fn is_button_released(&self, button: usize) -> bool {
		self.buttons[button] == ButtonState::Released
	}

	/// Returns a reference to the current mouse cursor bitmap.
	#[inline]
	pub fn cursor_bitmap(&self) -> &Bitmap {
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
	pub fn set_mouse_cursor(&mut self, cursor: Bitmap, hotspot_x: u32, hotspot_y: u32) {
		self.cursor = cursor;
		self.cursor_background = Bitmap::new(self.cursor.width(), self.cursor.height()).unwrap();
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

	fn get_default_mouse_cursor() -> (Bitmap, Bitmap, u32, u32) {
		let mut cursor = Bitmap::new(
			DEFAULT_MOUSE_CURSOR_WIDTH as u32,
			DEFAULT_MOUSE_CURSOR_HEIGHT as u32,
		)
			.unwrap();
		cursor.pixels_mut().copy_from_slice(&DEFAULT_MOUSE_CURSOR);

		let cursor_background = Bitmap::new(cursor.width(), cursor.height()).unwrap();

		(
			cursor,
			cursor_background,
			DEFAULT_MOUSE_CURSOR_HOTSPOT_X,
			DEFAULT_MOUSE_CURSOR_HOTSPOT_Y,
		)
	}

	#[inline]
	fn get_cursor_render_position(&self) -> (i32, i32) {
		(
			self.x - self.cursor_hotspot_x as i32,
			self.y - self.cursor_hotspot_y as i32,
		)
	}

	/// Renders the mouse cursor bitmap onto the destination bitmap at the mouse's current
	/// position. The destination bitmap specified is assumed to be the [`System`]'s video
	/// backbuffer bitmap. The background on the destination bitmap is saved internally and a
	/// subsequent call to [`Mouse::hide_cursor`] will restore the background.
	///
	/// If mouse cursor rendering is not currently enabled, this method does nothing.
	///
	/// Applications will not normally need to call this method, as if mouse cursor rendering is
	/// enabled, this will be automatically handled by [`System::display`].
	///
	/// [`System`]: crate::System
	/// [`System::display`]: crate::System::display
	pub fn render_cursor(&mut self, dest: &mut Bitmap) {
		if !self.cursor_enabled {
			return;
		}

		let (x, y) = self.get_cursor_render_position();

		// preserve existing background first
		self.cursor_background.blit_region(
			BlitMethod::Solid,
			&dest,
			&Rect::new(x, y, self.cursor.width(), self.cursor.height()),
			0,
			0,
		);

		dest.blit(BlitMethod::Transparent(255), &self.cursor, x, y);
	}

	/// Restores the original destination bitmap contents where the mouse cursor bitmap was
	/// rendered to during the previous call to [`Mouse::render_cursor`]. The destination bitmap
	/// specified is assumed to be the [`System`]'s video backbuffer bitmap.
	///
	/// If mouse cursor rendering is not currently enabled, this method does nothing.
	///
	/// Applications will not normally need to call this method, as if mouse cursor rendering is
	/// enabled, this will be automatically handled by [`System::display`].
	///
	/// [`System`]: crate::System
	/// [`System::display`]: crate::System::display
	pub fn hide_cursor(&mut self, dest: &mut Bitmap) {
		if !self.cursor_enabled {
			return;
		}

		let (x, y) = self.get_cursor_render_position();
		dest.blit(BlitMethod::Solid, &self.cursor_background, x, y);
	}

	fn update_button_state(&mut self, button: u32, is_pressed: bool) {
		let button_state = &mut self.buttons[button as usize];
		*button_state = if is_pressed {
			match *button_state {
				ButtonState::Pressed => ButtonState::Held,
				ButtonState::Held => ButtonState::Held,
				_ => ButtonState::Pressed,
			}
		} else {
			match *button_state {
				ButtonState::Pressed | ButtonState::Held => ButtonState::Released,
				ButtonState::Released => ButtonState::Idle,
				ButtonState::Idle => ButtonState::Idle,
			}
		}
	}
}

impl InputDevice for Mouse {
	fn update(&mut self) {
		self.x_delta = 0;
		self.y_delta = 0;
		for state in self.buttons.iter_mut() {
			*state = match *state {
				ButtonState::Pressed => ButtonState::Held,
				ButtonState::Released => ButtonState::Idle,
				otherwise => otherwise,
			}
		}
	}
}

impl SystemEventHandler for Mouse {
	fn handle_event(&mut self, event: &SystemEvent) -> bool {
		match event {
			SystemEvent::Mouse(MouseEvent::MouseMotion {
				                   x,
				                   y,
				                   x_delta,
				                   y_delta,
				                   buttons,
			                   }) => {
				self.x = *x;
				self.y = *y;
				self.x_delta = *x_delta;
				self.y_delta = *y_delta;

				self.update_button_state(MouseButton::Left as u32, buttons.contains(MouseButtons::LEFT_BUTTON));
				self.update_button_state(MouseButton::Middle as u32, buttons.contains(MouseButtons::MIDDLE_BUTTON));
				self.update_button_state(MouseButton::Right as u32, buttons.contains(MouseButtons::RIGHT_BUTTON));
				self.update_button_state(MouseButton::X1 as u32, buttons.contains(MouseButtons::X1));
				self.update_button_state(MouseButton::X2 as u32, buttons.contains(MouseButtons::X2));
				true
			}
			SystemEvent::Mouse(MouseEvent::MouseButtonDown { button, .. }) => {
				self.update_button_state(*button as u32, true);
				true
			}
			SystemEvent::Mouse(MouseEvent::MouseButtonUp { button, .. }) => {
				self.update_button_state(*button as u32, false);
				true
			}
			_ => false,
		}
	}
}