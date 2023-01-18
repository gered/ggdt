// The primary reason for these "event" enumerations existing is to allow
// us to *not* expose SDL2 types back to applications, thus preventing them
// from being required to explicitly add SDL2 as a dependency even if they
// never call into SDL2 directly anywhere (the SDL2 dependency can just be
// provided automatically by libretrogd).
//
// Also note, that with the intended use-cases (for now) that I have for libretrogd,
// I don't really care about all possible SDL2 events that could be raised. Thus,
// I only map the SDL2 events which I care about here. I will extend this in the
// future should I require it.

use bitflags::bitflags;

use crate::system::{Keycode, MouseButton, MouseButtons, Scancode};
use crate::system::MouseEvent::MouseButtonUp;

#[derive(Debug, Clone, PartialEq)]
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
}

impl TryFrom<sdl2::event::WindowEvent> for WindowEvent {
    type Error = ();

    fn try_from(value: sdl2::event::WindowEvent) -> Result<Self, Self::Error> {
        match value {
            sdl2::event::WindowEvent::Shown => Ok(WindowEvent::Shown),
            sdl2::event::WindowEvent::Hidden => Ok(WindowEvent::Hidden),
            sdl2::event::WindowEvent::Exposed => Ok(WindowEvent::Exposed),
            sdl2::event::WindowEvent::Moved(x, y) => Ok(WindowEvent::Moved(x, y)),
            sdl2::event::WindowEvent::Resized(width, height) => Ok(WindowEvent::Resized(width, height)),
            sdl2::event::WindowEvent::SizeChanged(width, height) => Ok(WindowEvent::SizeChanged(width, height)),
            sdl2::event::WindowEvent::Minimized => Ok(WindowEvent::Minimized),
            sdl2::event::WindowEvent::Maximized => Ok(WindowEvent::Maximized),
            sdl2::event::WindowEvent::Restored => Ok(WindowEvent::Restored),
            sdl2::event::WindowEvent::Enter => Ok(WindowEvent::Enter),
            sdl2::event::WindowEvent::Leave => Ok(WindowEvent::Leave),
            sdl2::event::WindowEvent::FocusGained => Ok(WindowEvent::FocusGained),
            sdl2::event::WindowEvent::FocusLost => Ok(WindowEvent::FocusLost),
            sdl2::event::WindowEvent::Close => Ok(WindowEvent::Close),
            _ => Err(())
        }
    }
}

bitflags! {
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

#[derive(Debug, Clone, PartialEq)]
pub enum KeyboardEvent {
    KeyUp {
        keycode: Option<Keycode>,
        scancode: Option<Scancode>,
        keymod: KeyModifiers,
        repeat: bool,
    },
    KeyDown {
        keycode: Option<Keycode>,
        scancode: Option<Scancode>,
        keymod: KeyModifiers,
        repeat: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseEvent {
    MouseMotion {
        x: i32,
        y: i32,
        x_delta: i32,
        y_delta: i32,
        buttons: MouseButtons,
    },
    MouseButtonDown {
        x: i32,
        y: i32,
        button: MouseButton,
        clicks: u8,
    },
    MouseButtonUp {
        x: i32,
        y: i32,
        button: MouseButton,
        clicks: u8,
    },
}

#[derive(Debug, Clone, PartialEq)]
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
}

impl TryFrom<sdl2::event::Event> for SystemEvent {
    type Error = ();

    fn try_from(value: sdl2::event::Event) -> Result<Self, Self::Error> {
        match value {
            sdl2::event::Event::Quit { .. } => Ok(SystemEvent::Quit),
            sdl2::event::Event::AppTerminating { .. } => Ok(SystemEvent::AppTerminating),
            sdl2::event::Event::AppLowMemory { .. } => Ok(SystemEvent::AppLowMemory),
            sdl2::event::Event::AppWillEnterBackground { .. } => Ok(SystemEvent::AppWillEnterBackground),
            sdl2::event::Event::AppDidEnterBackground { .. } => Ok(SystemEvent::AppDidEnterBackground),
            sdl2::event::Event::AppWillEnterForeground { .. } => Ok(SystemEvent::AppWillEnterForeground),
            sdl2::event::Event::AppDidEnterForeground { .. } => Ok(SystemEvent::AppDidEnterForeground),
            sdl2::event::Event::Window { win_event, .. } => {
                match win_event.try_into() {
                    Ok(window_event) => Ok(SystemEvent::Window(window_event)),
                    Err(e) => Err(e),
                }
            },
            sdl2::event::Event::KeyDown { keycode, scancode, keymod, repeat, .. } => {
                Ok(SystemEvent::Keyboard(KeyboardEvent::KeyDown {
                    keycode: keycode.map(|keycode| keycode.into()),
                    scancode: scancode.map(|scancode| scancode.into()),
                    keymod: KeyModifiers::from_bits_truncate(keymod.bits()),
                    repeat
                }))
            },
            sdl2::event::Event::KeyUp { keycode, scancode, keymod, repeat, .. } => {
                Ok(SystemEvent::Keyboard(KeyboardEvent::KeyUp {
                    keycode: keycode.map(|keycode| keycode.into()),
                    scancode: scancode.map(|scancode| scancode.into()),
                    keymod: KeyModifiers::from_bits_truncate(keymod.bits()),
                    repeat
                }))
            }
            sdl2::event::Event::MouseMotion { mousestate, x, y, xrel, yrel, .. } => {
                Ok(SystemEvent::Mouse(MouseEvent::MouseMotion {
                    x,
                    y,
                    x_delta: xrel,
                    y_delta: yrel,
                    buttons: MouseButtons::from_bits_truncate(mousestate.to_sdl_state()),
                }))
            },
            sdl2::event::Event::MouseButtonDown { mouse_btn, clicks, x, y, .. } => {
                Ok(SystemEvent::Mouse(MouseEvent::MouseButtonDown {
                    x,
                    y,
                    clicks,
                    button: mouse_btn.into(),
                }))
            },
            sdl2::event::Event::MouseButtonUp { mouse_btn, clicks, x, y, .. } => {
                Ok(SystemEvent::Mouse(MouseButtonUp {
                    x,
                    y,
                    clicks,
                    button: mouse_btn.into(),
                }))
            },
            _ => Err(())
        }
    }
}