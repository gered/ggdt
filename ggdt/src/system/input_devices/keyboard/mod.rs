use crate::system::event::{KeyboardEvent, SystemEvent, SystemEventHandler};
use crate::system::input_devices::keyboard::scancodes::Scancode;
use crate::system::input_devices::{ButtonState, InputDevice};

pub mod codes;
pub mod scancodes;

const MAX_KEYS: usize = 512;

/// Holds the current state of the keyboard.
///
/// Must be explicitly updated each frame by calling `handle_event` each frame for all SDL2 events
/// received, as well as calling `do_events` once each frame. Usually, you would accomplish all
/// this house-keeping by simply calling [`System`]'s `do_events` method once per frame.
///
/// [`System`]: crate::System
#[derive(Debug)]
pub struct Keyboard {
	keyboard: [ButtonState; MAX_KEYS], // Box<[ButtonState]>,
}

impl Keyboard {
	pub fn new() -> Keyboard {
		Keyboard { keyboard: [ButtonState::Idle; MAX_KEYS] }
		/*
		Keyboard {
			keyboard: vec![ButtonState::Idle; 256].into_boxed_slice(),
		}
		 */
	}

	/// Returns true if the given key was just pressed or is being held down.
	#[inline]
	pub fn is_key_down(&self, scancode: Scancode) -> bool {
		matches!(self.keyboard[scancode as usize], ButtonState::Pressed | ButtonState::Held)
	}

	/// Returns true if the given key was not just pressed and is not being held down.
	#[inline]
	pub fn is_key_up(&self, scancode: Scancode) -> bool {
		matches!(self.keyboard[scancode as usize], ButtonState::Released | ButtonState::Idle)
	}

	/// Returns true if the given key was just pressed (not being held down, yet).
	#[inline]
	pub fn is_key_pressed(&self, scancode: Scancode) -> bool {
		self.keyboard[scancode as usize] == ButtonState::Pressed
	}

	/// Returns true if the given key was just released.
	#[inline]
	pub fn is_key_released(&self, scancode: Scancode) -> bool {
		self.keyboard[scancode as usize] == ButtonState::Released
	}
}

impl InputDevice for Keyboard {
	fn update(&mut self) {
		for state in self.keyboard.iter_mut() {
			*state = match *state {
				ButtonState::Pressed => ButtonState::Held,
				ButtonState::Released => ButtonState::Idle,
				otherwise => otherwise,
			};
		}
	}
}

impl SystemEventHandler for Keyboard {
	fn handle_event(&mut self, event: &SystemEvent) -> bool {
		match event {
			SystemEvent::Keyboard(KeyboardEvent::KeyDown { scancode, .. }) => {
				if let Some(scancode) = scancode {
					let state = &mut self.keyboard[*scancode as usize];
					*state = match *state {
						ButtonState::Pressed => ButtonState::Held,
						ButtonState::Held => ButtonState::Held,
						_ => ButtonState::Pressed,
					};
				}
				true
			}
			SystemEvent::Keyboard(KeyboardEvent::KeyUp { scancode, .. }) => {
				if let Some(scancode) = scancode {
					self.keyboard[*scancode as usize] = ButtonState::Released;
				}
				true
			}
			_ => false,
		}
	}
}
