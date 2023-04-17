mod keyboard;
mod mouse;

pub use keyboard::*;
pub use mouse::*;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ButtonState {
	Idle,
	Pressed,
	Held,
	Released,
}

/// Common trait for input device implementations.
pub trait InputDevice {
	/// Performs internal house-keeping necessary for properly reporting the current state of this
	/// input device. Normally this should be called on the device before all of this frame's
	/// input events have been processed via `handle_event`.
	fn update(&mut self);
}
