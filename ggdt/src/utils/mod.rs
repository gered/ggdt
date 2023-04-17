use std::any::Any;

use num_traits::Unsigned;
use rand::distributions::uniform::SampleUniform;
use rand::Rng;

pub mod bytes;
pub mod io;
pub mod lzwgif;
pub mod packbits;

pub fn rnd_value<N: SampleUniform + PartialOrd>(low: N, high: N) -> N {
	rand::thread_rng().gen_range(low..=high)
}

/// Returns the absolute difference between two unsigned values. This is just here as a temporary
/// alternative to the `abs_diff` method currently provided by Rust but that is marked unstable.
#[inline]
pub fn abs_diff<N: Unsigned + PartialOrd>(a: N, b: N) -> N {
	if a < b {
		b - a
	} else {
		a - b
	}
}

pub trait AsAny {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<A: Any> AsAny for A {
	fn as_any(&self) -> &dyn Any {
		self as &dyn Any
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self as &mut dyn Any
	}
}
