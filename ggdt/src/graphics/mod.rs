use num_traits::{PrimInt, Unsigned};

pub mod bitmap;
pub mod bitmapatlas;
pub mod color;
pub mod font;
pub mod blendmap;
pub mod palette;

pub mod prelude;

/// Common trait to represent single pixel/colour values.
pub trait Pixel: PrimInt + Unsigned + Default {}
impl<T> Pixel for T where T: PrimInt + Unsigned + Default {}
