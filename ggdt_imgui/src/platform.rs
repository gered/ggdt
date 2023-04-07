use ggdt::system::event::{KeyModifiers, KeyboardEvent, MouseEvent, SystemEvent};
use ggdt::system::input_devices::keyboard::scancodes::Scancode;
use ggdt::system::input_devices::mouse::buttons::MouseButton;
use ggdt::system::res::standard::Standard;
use ggdt::system::res::SystemResources;
use ggdt::system::System;
use std::time::Instant;

fn handle_key(io: &mut imgui::Io, key: Scancode, down: bool) {
	let key = match key {
		Scancode::A => imgui::Key::A,
		Scancode::B => imgui::Key::B,
		Scancode::C => imgui::Key::C,
		Scancode::D => imgui::Key::D,
		Scancode::E => imgui::Key::E,
		Scancode::F => imgui::Key::F,
		Scancode::G => imgui::Key::G,
		Scancode::H => imgui::Key::H,
		Scancode::I => imgui::Key::I,
		Scancode::J => imgui::Key::J,
		Scancode::K => imgui::Key::K,
		Scancode::L => imgui::Key::L,
		Scancode::M => imgui::Key::M,
		Scancode::N => imgui::Key::N,
		Scancode::O => imgui::Key::O,
		Scancode::P => imgui::Key::P,
		Scancode::Q => imgui::Key::Q,
		Scancode::R => imgui::Key::R,
		Scancode::S => imgui::Key::S,
		Scancode::T => imgui::Key::T,
		Scancode::U => imgui::Key::U,
		Scancode::V => imgui::Key::V,
		Scancode::W => imgui::Key::W,
		Scancode::X => imgui::Key::X,
		Scancode::Y => imgui::Key::Y,
		Scancode::Z => imgui::Key::Z,
		Scancode::Num1 => imgui::Key::Keypad1,
		Scancode::Num2 => imgui::Key::Keypad2,
		Scancode::Num3 => imgui::Key::Keypad3,
		Scancode::Num4 => imgui::Key::Keypad4,
		Scancode::Num5 => imgui::Key::Keypad5,
		Scancode::Num6 => imgui::Key::Keypad6,
		Scancode::Num7 => imgui::Key::Keypad7,
		Scancode::Num8 => imgui::Key::Keypad8,
		Scancode::Num9 => imgui::Key::Keypad9,
		Scancode::Num0 => imgui::Key::Keypad0,
		Scancode::Return => imgui::Key::Enter,
		Scancode::Escape => imgui::Key::Escape,
		Scancode::Backspace => imgui::Key::Backspace,
		Scancode::Tab => imgui::Key::Tab,
		Scancode::Space => imgui::Key::Space,
		Scancode::Minus => imgui::Key::Minus,
		Scancode::Equals => imgui::Key::Equal,
		Scancode::LeftBracket => imgui::Key::LeftBracket,
		Scancode::RightBracket => imgui::Key::RightBracket,
		Scancode::Backslash => imgui::Key::Backslash,
		Scancode::Semicolon => imgui::Key::Semicolon,
		Scancode::Apostrophe => imgui::Key::Apostrophe,
		Scancode::Grave => imgui::Key::GraveAccent,
		Scancode::Comma => imgui::Key::Comma,
		Scancode::Period => imgui::Key::Period,
		Scancode::Slash => imgui::Key::Slash,
		Scancode::CapsLock => imgui::Key::CapsLock,
		Scancode::F1 => imgui::Key::F1,
		Scancode::F2 => imgui::Key::F2,
		Scancode::F3 => imgui::Key::F3,
		Scancode::F4 => imgui::Key::F4,
		Scancode::F5 => imgui::Key::F5,
		Scancode::F6 => imgui::Key::F6,
		Scancode::F7 => imgui::Key::F7,
		Scancode::F8 => imgui::Key::F8,
		Scancode::F9 => imgui::Key::F9,
		Scancode::F10 => imgui::Key::F10,
		Scancode::F11 => imgui::Key::F11,
		Scancode::F12 => imgui::Key::F12,
		Scancode::PrintScreen => imgui::Key::PrintScreen,
		Scancode::ScrollLock => imgui::Key::ScrollLock,
		Scancode::Pause => imgui::Key::Pause,
		Scancode::Insert => imgui::Key::Insert,
		Scancode::Home => imgui::Key::Home,
		Scancode::PageUp => imgui::Key::PageUp,
		Scancode::Delete => imgui::Key::Delete,
		Scancode::End => imgui::Key::End,
		Scancode::PageDown => imgui::Key::PageDown,
		Scancode::Right => imgui::Key::RightArrow,
		Scancode::Left => imgui::Key::LeftArrow,
		Scancode::Down => imgui::Key::DownArrow,
		Scancode::Up => imgui::Key::UpArrow,
		Scancode::KpDivide => imgui::Key::KeypadDivide,
		Scancode::KpMultiply => imgui::Key::KeypadMultiply,
		Scancode::KpMinus => imgui::Key::KeypadSubtract,
		Scancode::KpPlus => imgui::Key::KeypadAdd,
		Scancode::KpEnter => imgui::Key::KeypadEnter,
		Scancode::Kp1 => imgui::Key::Keypad1,
		Scancode::Kp2 => imgui::Key::Keypad2,
		Scancode::Kp3 => imgui::Key::Keypad3,
		Scancode::Kp4 => imgui::Key::Keypad4,
		Scancode::Kp5 => imgui::Key::Keypad5,
		Scancode::Kp6 => imgui::Key::Keypad6,
		Scancode::Kp7 => imgui::Key::Keypad7,
		Scancode::Kp8 => imgui::Key::Keypad8,
		Scancode::Kp9 => imgui::Key::Keypad9,
		Scancode::Kp0 => imgui::Key::Keypad0,
		Scancode::KpPeriod => imgui::Key::KeypadDecimal,
		Scancode::Application => imgui::Key::Menu,
		Scancode::KpEquals => imgui::Key::KeypadEqual,
		Scancode::Menu => imgui::Key::Menu,
		Scancode::LCtrl => imgui::Key::LeftCtrl,
		Scancode::LShift => imgui::Key::LeftShift,
		Scancode::LAlt => imgui::Key::LeftAlt,
		Scancode::LGui => imgui::Key::LeftSuper,
		Scancode::RCtrl => imgui::Key::RightCtrl,
		Scancode::RShift => imgui::Key::RightShift,
		Scancode::RAlt => imgui::Key::RightAlt,
		Scancode::RGui => imgui::Key::RightSuper,
		_ => return,
	};
	io.add_key_event(key, down);
}

fn handle_key_modifier(io: &mut imgui::Io, keymod: KeyModifiers) {
	io.add_key_event(imgui::Key::ModShift, keymod.intersects(KeyModifiers::LSHIFTMOD | KeyModifiers::RSHIFTMOD));
	io.add_key_event(imgui::Key::ModCtrl, keymod.intersects(KeyModifiers::LCTRLMOD | KeyModifiers::RCTRLMOD));
	io.add_key_event(imgui::Key::ModAlt, keymod.intersects(KeyModifiers::LALTMOD | KeyModifiers::RALTMOD));
	io.add_key_event(imgui::Key::ModSuper, keymod.intersects(KeyModifiers::LGUIMOD | KeyModifiers::RGUIMOD));
}

fn handle_mouse_button_event(io: &mut imgui::Io, button: MouseButton, down: bool) {
	match button {
		MouseButton::Left => io.add_mouse_button_event(imgui::MouseButton::Left, down),
		MouseButton::Right => io.add_mouse_button_event(imgui::MouseButton::Right, down),
		MouseButton::Middle => io.add_mouse_button_event(imgui::MouseButton::Middle, down),
		MouseButton::X1 => io.add_mouse_button_event(imgui::MouseButton::Extra1, down),
		MouseButton::X2 => io.add_mouse_button_event(imgui::MouseButton::Extra2, down),
		_ => {}
	}
}

pub struct Platform {
	last_frame: Instant,
}

impl Platform {
	pub fn new(imgui: &mut imgui::Context) -> Self {
		imgui.set_ini_filename(None);
		imgui.set_platform_name(Some(String::from("ggdt")));

		imgui.style_mut().anti_aliased_lines = false;
		imgui.style_mut().anti_aliased_lines_use_tex = false;

		Platform { last_frame: Instant::now() }
	}

	pub fn handle_event(&mut self, context: &mut imgui::Context, event: &SystemEvent) -> bool {
		let io = context.io_mut();

		match *event {
			SystemEvent::Mouse(MouseEvent::MouseMotion { x, y, .. }) => {
				io.add_mouse_pos_event([x as f32, y as f32]);
				true
			}
			SystemEvent::Mouse(MouseEvent::MouseButtonUp { button, .. }) => {
				handle_mouse_button_event(io, button, false);
				true
			}
			SystemEvent::Mouse(MouseEvent::MouseButtonDown { button, .. }) => {
				handle_mouse_button_event(io, button, true);
				true
			}
			SystemEvent::Keyboard(KeyboardEvent::KeyUp { scancode: Some(scancode), keymod, .. }) => {
				handle_key_modifier(io, keymod);
				handle_key(io, scancode, false);
				true
			}
			SystemEvent::Keyboard(KeyboardEvent::KeyDown { scancode: Some(scancode), keymod, .. }) => {
				handle_key_modifier(io, keymod);
				handle_key(io, scancode, true);
				true
			}
			SystemEvent::Keyboard(KeyboardEvent::TextInput { ref text }) => {
				for ch in text.chars() {
					io.add_input_character(ch);
				}
				true
			}
			_ => false,
		}
	}

	pub fn prepare_frame(&mut self, context: &mut imgui::Context, system: &System<Standard>) {
		let io = context.io_mut();

		let now = Instant::now();
		io.update_delta_time(now.duration_since(self.last_frame));
		self.last_frame = now;

		io.display_size = [system.res.width() as f32, system.res.height() as f32];
		io.display_framebuffer_scale = [1.0, 1.0];
	}
}
