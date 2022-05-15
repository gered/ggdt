extern crate core;
extern crate sdl2;

pub use crate::graphics::bitmap::*;
pub use crate::graphics::bitmapatlas::*;
pub use crate::graphics::font::*;
pub use crate::graphics::palette::*;
pub use crate::graphics::*;
pub use crate::math::circle::*;
pub use crate::math::matrix3x3::*;
pub use crate::math::rect::*;
pub use crate::math::vector2::*;
pub use crate::math::*;
pub use crate::system::input_devices::keyboard::*;
pub use crate::system::input_devices::mouse::*;
pub use crate::system::input_devices::*;
pub use crate::system::*;

pub mod entities;
pub mod events;
pub mod graphics;
pub mod math;
pub mod states;
pub mod system;
pub mod utils;

pub const LOW_RES: bool = if cfg!(feature = "low_res") {
    true
} else {
    false
};
pub const WIDE_SCREEN: bool = if cfg!(feature = "wide") {
    true
} else {
    false
};

pub const SCREEN_WIDTH: u32 = if cfg!(feature = "low_res") {
    if cfg!(feature = "wide") {
        214
    } else {
        160
    }
} else {
    if cfg!(feature = "wide") {
        428
    } else {
        320
    }
};
pub const SCREEN_HEIGHT: u32 = if cfg!(feature = "low_res") {
    120
} else {
    240
};

pub const SCREEN_TOP: u32 = 0;
pub const SCREEN_LEFT: u32 = 0;
pub const SCREEN_RIGHT: u32 = SCREEN_WIDTH - 1;
pub const SCREEN_BOTTOM: u32 = SCREEN_HEIGHT - 1;

pub const DEFAULT_SCALE_FACTOR: u32 = if cfg!(feature = "low_res") {
    6
} else {
    3
};

pub const NUM_COLORS: usize = 256; // i mean ... the number of colors is really defined by the size of u8 ...
