extern crate core;
extern crate sdl2;

pub mod audio;
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
