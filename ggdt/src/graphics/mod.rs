use std::fmt::Display;

mod bitmap;
mod bitmapatlas;
mod blendmap;
mod color;
mod font;
mod palette;

pub use bitmap::*;
pub use bitmapatlas::*;
pub use blendmap::*;
pub use color::*;
pub use font::*;
pub use palette::*;

/// Common trait to represent single pixel/colour values.
pub trait Pixel: Default + Display + Eq + Copy + Clone {}
impl<T> Pixel for T where T: Default + Display + Eq + Copy + Clone {}
