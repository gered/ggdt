// The primary reason for these "event" enumerations existing is to allow
// us to *not* expose SDL2 types back to applications, thus preventing them
// from being required to explicitly add SDL2 as a dependency even if they
// never call into SDL2 directly anywhere (the SDL2 dependency can just be
// provided automatically by ggdt).
//
// Also note, that with the intended use-cases (for now) that I have for ggdt,
// I don't really care about all possible SDL2 events that could be raised. Thus,
// I only map the SDL2 events which I care about here. I will extend this in the
// future should I require it.

use bitflags::bitflags;

use crate::system::input_devices::keyboard::codes::Keycode;
use crate::system::input_devices::keyboard::scancodes::Scancode;
use crate::system::input_devices::mouse::buttons::{MouseButton, MouseButtons};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum WindowEvent {
	Shown,
	Hidden,
	Exposed,
	Moved(i32, i32),
	Resized(i32, i32),
	SizeChanged(i32, i32),
	Minimized,
	Maximized,
	Restored,
	Enter,
	Leave,
	FocusGained,
	FocusLost,
	Close,
	// for sdl2::event::WindowEvent enum values we haven't mapped / don't care about (yet?)
	Unimplemented,
}

impl From<sdl2::event::WindowEvent> for WindowEvent {
	fn from(value: sdl2::event::WindowEvent) -> Self {
		match value {
			sdl2::event::WindowEvent::Shown => WindowEvent::Shown,
			sdl2::event::WindowEvent::Hidden => WindowEvent::Hidden,
			sdl2::event::WindowEvent::Exposed => WindowEvent::Exposed,
			sdl2::event::WindowEvent::Moved(x, y) => WindowEvent::Moved(x, y),
			sdl2::event::WindowEvent::Resized(width, height) => WindowEvent::Resized(width, height),
			sdl2::event::WindowEvent::SizeChanged(width, height) => WindowEvent::SizeChanged(width, height),
			sdl2::event::WindowEvent::Minimized => WindowEvent::Minimized,
			sdl2::event::WindowEvent::Maximized => WindowEvent::Maximized,
			sdl2::event::WindowEvent::Restored => WindowEvent::Restored,
			sdl2::event::WindowEvent::Enter => WindowEvent::Enter,
			sdl2::event::WindowEvent::Leave => WindowEvent::Leave,
			sdl2::event::WindowEvent::FocusGained => WindowEvent::FocusGained,
			sdl2::event::WindowEvent::FocusLost => WindowEvent::FocusLost,
			sdl2::event::WindowEvent::Close => WindowEvent::Close,
			_ => WindowEvent::Unimplemented,
		}
	}
}

bitflags! {
	#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
	pub struct KeyModifiers: u16 {
		const NOMOD = 0x0000;
		const LSHIFTMOD = 0x0001;
		const RSHIFTMOD = 0x0002;
		const LCTRLMOD = 0x0040;
		const RCTRLMOD = 0x0080;
		const LALTMOD = 0x0100;
		const RALTMOD = 0x0200;
		const LGUIMOD = 0x0400;
		const RGUIMOD = 0x0800;
		const NUMMOD = 0x1000;
		const CAPSMOD = 0x2000;
		const MODEMOD = 0x4000;
		const RESERVEDMOD = 0x8000;
	}
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum KeyboardEvent {
	KeyUp {
		keycode: Option<Keycode>, //
		scancode: Option<Scancode>,
		keymod: KeyModifiers,
		repeat: bool,
	},
	KeyDown {
		keycode: Option<Keycode>, //
		scancode: Option<Scancode>,
		keymod: KeyModifiers,
		repeat: bool,
	},
	TextInput {
		text: String,
	},
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MouseEvent {
	MouseMotion {
		x: i32, //
		y: i32,
		x_delta: i32,
		y_delta: i32,
		buttons: MouseButtons,
	},
	MouseButtonDown {
		x: i32, //
		y: i32,
		button: MouseButton,
		clicks: u8,
	},
	MouseButtonUp {
		x: i32, //
		y: i32,
		button: MouseButton,
		clicks: u8,
	},
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SystemEvent {
	Quit,
	AppTerminating,
	AppLowMemory,
	AppWillEnterBackground,
	AppDidEnterBackground,
	AppWillEnterForeground,
	AppDidEnterForeground,
	Window(WindowEvent),
	Keyboard(KeyboardEvent),
	Mouse(MouseEvent),
	// for the many sdl2::event::Event enum values that we don't are about quite yet ...
	Unimplemented,
}

impl From<sdl2::event::Event> for SystemEvent {
	fn from(value: sdl2::event::Event) -> Self {
		match value {
			sdl2::event::Event::Quit { .. } => SystemEvent::Quit,
			sdl2::event::Event::AppTerminating { .. } => SystemEvent::AppTerminating,
			sdl2::event::Event::AppLowMemory { .. } => SystemEvent::AppLowMemory,
			sdl2::event::Event::AppWillEnterBackground { .. } => SystemEvent::AppWillEnterBackground,
			sdl2::event::Event::AppDidEnterBackground { .. } => SystemEvent::AppDidEnterBackground,
			sdl2::event::Event::AppWillEnterForeground { .. } => SystemEvent::AppWillEnterForeground,
			sdl2::event::Event::AppDidEnterForeground { .. } => SystemEvent::AppDidEnterForeground,
			sdl2::event::Event::Window { win_event, .. } => SystemEvent::Window(win_event.into()),

			sdl2::event::Event::KeyDown { keycode, scancode, keymod, repeat, .. } => {
				SystemEvent::Keyboard(KeyboardEvent::KeyDown {
					keycode: keycode.map(|keycode| keycode.into()),
					scancode: scancode.map(|scancode| scancode.into()),
					keymod: KeyModifiers::from_bits_truncate(keymod.bits()),
					repeat,
				})
			}
			sdl2::event::Event::KeyUp { keycode, scancode, keymod, repeat, .. } => {
				SystemEvent::Keyboard(KeyboardEvent::KeyUp {
					keycode: keycode.map(|keycode| keycode.into()),
					scancode: scancode.map(|scancode| scancode.into()),
					keymod: KeyModifiers::from_bits_truncate(keymod.bits()),
					repeat,
				})
			}
			sdl2::event::Event::TextInput { text, .. } => SystemEvent::Keyboard(KeyboardEvent::TextInput { text }),
			sdl2::event::Event::MouseMotion { mousestate, x, y, xrel, yrel, .. } => {
				SystemEvent::Mouse(MouseEvent::MouseMotion {
					x,
					y,
					x_delta: xrel,
					y_delta: yrel,
					buttons: MouseButtons::from_bits_truncate(mousestate.to_sdl_state()),
				})
			}
			sdl2::event::Event::MouseButtonDown { mouse_btn, clicks, x, y, .. } => {
				SystemEvent::Mouse(MouseEvent::MouseButtonDown { x, y, clicks, button: mouse_btn.into() })
			}
			sdl2::event::Event::MouseButtonUp { mouse_btn, clicks, x, y, .. } => {
				SystemEvent::Mouse(MouseEvent::MouseButtonUp { x, y, clicks, button: mouse_btn.into() })
			}

			_ => SystemEvent::Unimplemented,
		}
	}
}

/// Common trait for implementing a handler of [`SystemEvent`]s that are polled during the
/// application's main loop.
pub trait SystemEventHandler {
	/// Processes the data from the given [`SystemEvent`]. Returns true if the processing actually
	/// recognized the passed event and handled it, or false if the event was ignored.
	fn handle_event(&mut self, event: &SystemEvent) -> bool;
}

/// An interator for SDL2 system events, polled via [`SystemEventPump`].
pub struct SystemEventIterator<'a> {
	iter: sdl2::event::EventPollIterator<'a>,
}

impl Iterator for SystemEventIterator<'_> {
	type Item = SystemEvent;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|e| e.into())
	}
}

/// Provides an event pump iterator that wraps over SDL2 events, allowing applications to respond
/// to all events each frame as [`SystemEvent`] instances.
pub struct SystemEventPump {
	sdl_event_pump: sdl2::EventPump,
}

impl SystemEventPump {
	pub fn from(pump: sdl2::EventPump) -> Self {
		SystemEventPump { sdl_event_pump: pump }
	}

	/// Returns an iterator over [`SystemEvent`]s that have been generated since the last time
	/// events were polled (usually, in the previous frame).
	pub fn poll_iter(&mut self) -> SystemEventIterator {
		self.sdl_event_pump.pump_events();
		SystemEventIterator { iter: self.sdl_event_pump.poll_iter() }
	}
}
