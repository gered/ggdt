use bitflags::bitflags;
use num_derive::FromPrimitive;

// equivalent to SDL's "SDL_BUTTON" macro
#[inline]
const fn button_mask_to_value(mask: u32) -> u32 {
	1 << (mask - 1)
}

bitflags! {
	#[repr(transparent)]
	#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
	pub struct MouseButtons: u32 {
		const LEFT_BUTTON = button_mask_to_value(sdl2::mouse::MouseButton::Left as u32);
		const MIDDLE_BUTTON = button_mask_to_value(sdl2::mouse::MouseButton::Middle as u32);
		const RIGHT_BUTTON = button_mask_to_value(sdl2::mouse::MouseButton::Right as u32);
		const X1 = button_mask_to_value(sdl2::mouse::MouseButton::X1 as u32);
		const X2 = button_mask_to_value(sdl2::mouse::MouseButton::X2 as u32);
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, FromPrimitive)]
#[repr(u8)]
pub enum MouseButton {
	Unknown = 0,
	Left = sdl2::mouse::MouseButton::Left as u8,
	Middle = sdl2::mouse::MouseButton::Middle as u8,
	Right = sdl2::mouse::MouseButton::Right as u8,
	X1 = sdl2::mouse::MouseButton::X1 as u8,
	X2 = sdl2::mouse::MouseButton::X2 as u8,
}

impl From<sdl2::mouse::MouseButton> for MouseButton {
	fn from(value: sdl2::mouse::MouseButton) -> Self {
		match value {
			sdl2::mouse::MouseButton::Unknown => MouseButton::Unknown,
			sdl2::mouse::MouseButton::Left => MouseButton::Left,
			sdl2::mouse::MouseButton::Middle => MouseButton::Middle,
			sdl2::mouse::MouseButton::Right => MouseButton::Right,
			sdl2::mouse::MouseButton::X1 => MouseButton::X1,
			sdl2::mouse::MouseButton::X2 => MouseButton::X2,
		}
	}
}
