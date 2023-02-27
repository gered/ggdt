use crate::system::{SystemEvent, SystemEventHandler};

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
}

/// Container for all available input devices available for applications to use.
pub struct InputDevices {
    pub keyboard: keyboard::Keyboard,
    pub mouse: mouse::Mouse,
}

impl InputDevice for InputDevices {
    fn update(&mut self) {
        self.keyboard.update();
        self.mouse.update();
    }
}

impl SystemEventHandler for InputDevices {
    fn handle_event(&mut self, event: &SystemEvent) -> bool {
        if self.keyboard.handle_event(event) {
            return true;
        }
        if self.mouse.handle_event(event) {
            return true;
        }
        false
    }
}