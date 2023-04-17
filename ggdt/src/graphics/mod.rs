use num_traits::{PrimInt, Unsigned};
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
pub trait Pixel: PrimInt + Unsigned + Default + Display {}
impl<T> Pixel for T where T: PrimInt + Unsigned + Default + Display {}
