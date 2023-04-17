use crate::system::{ButtonState, InputDevice, MouseEvent, SystemEvent, SystemEventHandler};

mod buttons;
mod cursor;

pub use buttons::*;
pub use cursor::*;

const MAX_BUTTONS: usize = 32;

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
}

impl Mouse {
	pub fn new() -> Mouse {
		Mouse {
			x: 0, //
			y: 0,
			x_delta: 0,
			y_delta: 0,
			buttons: [ButtonState::Idle; MAX_BUTTONS],
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
	pub fn is_button_down(&self, button: MouseButton) -> bool {
		matches!(self.buttons[button as usize], ButtonState::Pressed | ButtonState::Held)
	}

	/// Returns true if the given button was not just pressed and is not being held down.
	#[inline]
	pub fn is_button_up(&self, button: MouseButton) -> bool {
		matches!(self.buttons[button as usize], ButtonState::Released | ButtonState::Idle)
	}

	/// Returns true if the given button was just pressed (not being held down, yet).
	#[inline]
	pub fn is_button_pressed(&self, button: MouseButton) -> bool {
		self.buttons[button as usize] == ButtonState::Pressed
	}

	/// Returns true if the given button was just released.
	#[inline]
	pub fn is_button_released(&self, button: MouseButton) -> bool {
		self.buttons[button as usize] == ButtonState::Released
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
			SystemEvent::Mouse(MouseEvent::MouseMotion { x, y, x_delta, y_delta, buttons }) => {
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
