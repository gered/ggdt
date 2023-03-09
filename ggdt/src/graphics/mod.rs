use num_traits::{PrimInt, Unsigned};

pub use self::bitmap::*;
pub use self::bitmapatlas::*;
pub use self::font::*;

pub mod bitmap;
pub mod bitmapatlas;
pub mod font;
pub mod indexed;
pub mod rgb;

/// Common trait to represent single pixel/colour values.
pub trait Pixel: PrimInt + Unsigned {}
impl<T> Pixel for T where T: PrimInt + Unsigned {}
