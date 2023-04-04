use num_traits::{PrimInt, Unsigned};
use std::fmt::Display;

pub mod bitmap;
pub mod bitmapatlas;
pub mod blendmap;
pub mod color;
pub mod font;
pub mod palette;

pub mod prelude;

/// Common trait to represent single pixel/colour values.
pub trait Pixel: PrimInt + Unsigned + Default + Display {}
impl<T> Pixel for T where T: PrimInt + Unsigned + Default + Display {}
