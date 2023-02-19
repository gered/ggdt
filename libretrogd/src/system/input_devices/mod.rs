use crate::system::SystemEvent;

pub mod keyboard;
pub mod mouse;

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

    /// Processes the data from the given [`SystemEvent`] if it is relevant for this input device.
    /// You should pass in all events received every frame and let the input device figure out if it
    /// is relevant to it or not.
    fn handle_event(&mut self, event: &SystemEvent);
}
