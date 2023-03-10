//! This module and all sub-modules contain graphics functionality that uses indexed colours. That is, each pixel
//! is a `u8` and treated as index into a [`Palette`], so 256 maximum colours are possible.

pub mod bitmap;
pub mod blendmap;
pub mod palette;

pub mod prelude;