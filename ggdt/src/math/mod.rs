use std::ops::{Add, Div, Mul, Sub};

pub mod circle;
pub mod matrix3x3;
pub mod rect;
pub mod vector2;

pub mod prelude;

pub const PI: f32 = std::f32::consts::PI;
// 180 degrees
pub const HALF_PI: f32 = PI / 2.0;
// 90 degrees
pub const QUARTER_PI: f32 = PI / 4.0;
// 45 degrees
pub const TWO_PI: f32 = PI * 2.0; // 360 degrees

pub const RADIANS_0: f32 = 0.0;
pub const RADIANS_45: f32 = PI / 4.0;
pub const RADIANS_90: f32 = PI / 2.0;
pub const RADIANS_135: f32 = (3.0 * PI) / 4.0;
pub const RADIANS_180: f32 = PI;
pub const RADIANS_225: f32 = (5.0 * PI) / 4.0;
pub const RADIANS_270: f32 = (3.0 * PI) / 2.0;
pub const RADIANS_315: f32 = (7.0 * PI) / 4.0;
pub const RADIANS_360: f32 = PI * 2.0;

pub const PI_OVER_180: f32 = PI / 180.0;
pub const INVERSE_PI_OVER_180: f32 = 180.0 / PI;

// 2d directions. intended to be usable in a 2d screen-space where the origin 0,0 is at the
// top-left of the screen, where moving up is accomplished by decrementing Y.
// TODO: this is not a perfect solution and does pose some problems in various math calculations ...
pub const UP: f32 = -RADIANS_90;
pub const DOWN: f32 = RADIANS_90;
pub const LEFT: f32 = RADIANS_180;
pub const RIGHT: f32 = RADIANS_0;

/// Returns true if the two f32 values are "close enough" to be considered equal using the
/// precision of the provided epsilon value.
#[inline]
pub fn nearly_equal(a: f32, b: f32, epsilon: f32) -> bool {
	// this could still be improved
	a == b || (a - b).abs() <= epsilon
}

/// Linearly interpolates between two values.
///
/// # Arguments
///
/// * `a`: first value (low end of range)
/// * `b`: second value (high end of range)
/// * `t`: the amount to interpolate between the two values, specified as a fraction
#[inline]
pub fn lerp<N>(a: N, b: N, t: f32) -> N
where
	N: Copy + Add<Output = N> + Sub<Output = N> + Mul<f32, Output = N>,
{
	a + (b - a) * t
}

/// Given a linearly interpolated value and the original range (high and low) of the linear
/// interpolation, this will return the original interpolation factor (as a fraction)
///
/// # Arguments
///
/// * `a`: first value (low end of range)
/// * `b`: second value (high end of range)
/// * `lerp_result`: the interpolated value between the range given
#[inline]
pub fn inverse_lerp<N>(a: N, b: N, lerp_result: N) -> f32
where
	N: Copy + Sub<Output = N> + Div<N, Output = f32>,
{
	(lerp_result - a) / (b - a)
}

/// Interpolates between two values using a cubic equation.
///
/// # Arguments
///
/// * `a`: first value (low end of range)
/// * `b`: second value (high end of range)
/// * `t`: the amount to interpolate between the two values, specified as a fraction
#[inline]
pub fn smooth_lerp<N>(a: N, b: N, t: f32) -> N
where
	N: Copy + Add<Output = N> + Sub<Output = N> + Mul<f32, Output = N>,
{
	let t = t.clamp(0.0, 1.0);
	lerp(a, b, (t * t) * (3.0 - (2.0 * t)))
}

/// Re-scales a given value from an old min/max range to a new and different min/max range such
/// that the returned value is approximately at the same relative position within the new min/max
/// range.
///
/// # Arguments
///
/// * `value`: the value to be re-scaled which is currently between `old_min` and `old_max`
/// * `old_min`: original min value (low end of range)
/// * `old_max`: original max value (high end of range)
/// * `new_min`: new min value (low end of range)
/// * `new_max`: new max value (high end of range)
#[inline]
pub fn scale_range<N>(value: N, old_min: N, old_max: N, new_min: N, new_max: N) -> N
where
	N: Copy + Add<Output = N> + Sub<Output = N> + Mul<Output = N> + Div<Output = N>,
{
	(new_max - new_min) * (value - old_min) / (old_max - old_min) + new_min
}

/// Calculates the angle (in radians) between the two points.
#[inline]
pub fn angle_between(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
	let delta_x = x2 - x1;
	let delta_y = y2 - y1;
	delta_y.atan2(delta_x)
}

/// Returns the X and Y point of a normalized 2D vector that points in the same direction as
/// the given angle.
#[inline]
pub fn angle_to_direction(radians: f32) -> (f32, f32) {
	let x = radians.cos();
	let y = radians.sin();
	(x, y)
}

/// Calculates the squared distance between two 2D points.
#[inline]
pub fn distance_squared_between(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
	(x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)
}

/// Calculates the distance between two 2D points.
#[inline]
pub fn distance_between(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
	distance_squared_between(x1, y1, x2, y2).sqrt()
}

pub trait NearlyEqual {
	type Output;

	/// Returns true if the two f32 values are "close enough" to be considered equal using the
	/// precision of the provided epsilon value.
	fn nearly_equal(self, other: Self::Output, epsilon: f32) -> bool;
}

impl NearlyEqual for f32 {
	type Output = f32;

	#[inline]
	fn nearly_equal(self, other: Self::Output, epsilon: f32) -> bool {
		nearly_equal(self, other, epsilon)
	}
}

pub trait WrappingRadians {
	type Type;

	/// Adds two angle values in radians together returning the result. The addition will wrap
	/// around so that the returned result always lies within 0 -> 2π radians (0 -> 360 degrees).
	fn wrapping_radians_add(self, other: Self::Type) -> Self::Type;

	/// Subtracts two angle values in radians returning the result. The subtraction will wrap
	/// around so that the returned result always lies within 0 -> 2π radians (0 -> 360 degrees).
	fn wrapping_radians_sub(self, other: Self::Type) -> Self::Type;
}

impl WrappingRadians for f32 {
	type Type = f32;

	#[inline]
	fn wrapping_radians_add(self, other: Self::Type) -> Self::Type {
		let result = self + other;
		if result < RADIANS_0 {
			result + RADIANS_360
		} else if result >= RADIANS_360 {
			result - RADIANS_360
		} else {
			result
		}
	}

	#[inline]
	fn wrapping_radians_sub(self, other: Self::Type) -> Self::Type {
		let result = self - other;
		if result < RADIANS_0 {
			result + RADIANS_360
		} else if result >= RADIANS_360 {
			result - RADIANS_360
		} else {
			result
		}
	}
}

#[cfg(test)]
pub mod tests {
	use super::*;

	#[test]
	pub fn test_nearly_equal() {
		assert!(nearly_equal(4.0, 4.0, 0.1));
		assert!(4.0f32.nearly_equal(4.0, 0.1));
		assert!(nearly_equal(0.1 + 0.2, 0.3, 0.01));
		assert!(!nearly_equal(1.0001, 1.0005, 0.0001));
	}

	#[test]
	pub fn test_lerp() {
		assert!(nearly_equal(15.0, lerp(10.0, 20.0, 0.5), 0.0001));
		assert!(nearly_equal(10.0, lerp(10.0, 20.0, 0.0), 0.0001));
		assert!(nearly_equal(20.0, lerp(10.0, 20.0, 1.0), 0.0001));
	}

	#[test]
	pub fn test_inverse_lerp() {
		assert_eq!(0.5, inverse_lerp(10.0, 20.0, 15.0f32))
	}

	#[test]
	pub fn test_angle_between() {
		let angle = angle_between(20.0, 20.0, 10.0, 10.0);
		assert!(nearly_equal(-RADIANS_135, angle, 0.0001));
		let angle = angle_between(0.0, 0.0, 10.0, 10.0);
		assert!(nearly_equal(RADIANS_45, angle, 0.0001));
		let angle = angle_between(5.0, 5.0, 5.0, 5.0);
		assert!(nearly_equal(0.0, angle, 0.0001));
	}

	#[test]
	pub fn test_angle_to_direction() {
		let (x, y) = angle_to_direction(RADIANS_0);
		assert!(nearly_equal(x, 1.0, 0.000001));
		assert!(nearly_equal(y, 0.0, 0.000001));
		let (x, y) = angle_to_direction(RADIANS_45);
		assert!(nearly_equal(x, 0.707106, 0.000001));
		assert!(nearly_equal(y, 0.707106, 0.000001));
		let (x, y) = angle_to_direction(RADIANS_225);
		assert!(nearly_equal(x, -0.707106, 0.000001));
		assert!(nearly_equal(y, -0.707106, 0.000001));

		let (x, y) = angle_to_direction(UP);
		assert!(nearly_equal(x, 0.0, 0.000001));
		assert!(nearly_equal(y, -1.0, 0.000001));
		let (x, y) = angle_to_direction(DOWN);
		assert!(nearly_equal(x, 0.0, 0.000001));
		assert!(nearly_equal(y, 1.0, 0.000001));
		let (x, y) = angle_to_direction(LEFT);
		assert!(nearly_equal(x, -1.0, 0.000001));
		assert!(nearly_equal(y, 0.0, 0.000001));
		let (x, y) = angle_to_direction(RIGHT);
		assert!(nearly_equal(x, 1.0, 0.000001));
		assert!(nearly_equal(y, 0.0, 0.000001));
	}

	#[test]
	pub fn test_distance_between() {
		let x1 = -2.0;
		let y1 = 1.0;
		let x2 = 4.0;
		let y2 = 3.0;
		let distance_squared = distance_squared_between(x1, y1, x2, y2);
		let distance = distance_between(x1, y1, x2, y2);
		assert!(nearly_equal(distance_squared, 40.0000, 0.0001));
		assert!(nearly_equal(distance, 6.3245, 0.0001));
	}

	#[test]
	pub fn test_wrapping_radians() {
		assert!(nearly_equal(RADIANS_90, RADIANS_45.wrapping_radians_add(RADIANS_45), 0.0001));
		assert!(nearly_equal(RADIANS_90, RADIANS_180.wrapping_radians_sub(RADIANS_90), 0.0001));

		assert!(nearly_equal(RADIANS_45, RADIANS_315.wrapping_radians_add(RADIANS_90), 0.0001));
		assert!(nearly_equal(RADIANS_315, RADIANS_90.wrapping_radians_sub(RADIANS_135), 0.0001));
	}
}
