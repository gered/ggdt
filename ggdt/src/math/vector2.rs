use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::math::matrix3x3::Matrix3x3;
use crate::math::{angle_between, angle_to_direction, nearly_equal, NearlyEqual};

/// Represents a 2D vector and provides common methods for vector math.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector2 {
	pub x: f32,
	pub y: f32,
}

impl Vector2 {
	pub const ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };

	pub const UP: Vector2 = Vector2 { x: 0.0, y: -1.0 };
	pub const DOWN: Vector2 = Vector2 { x: 0.0, y: 1.0 };
	pub const LEFT: Vector2 = Vector2 { x: -1.0, y: 0.0 };
	pub const RIGHT: Vector2 = Vector2 { x: 1.0, y: 0.0 };

	pub const X_AXIS: Vector2 = Vector2 { x: 1.0, y: 0.0 };
	pub const Y_AXIS: Vector2 = Vector2 { x: 0.0, y: 1.0 };

	/// Creates a vector with the specified X and Y components.
	#[inline]
	pub fn new(x: f32, y: f32) -> Vector2 {
		Vector2 { x, y }
	}

	/// Creates a normalized vector that points in the same direction as the given angle.
	#[inline]
	pub fn from_angle(radians: f32) -> Vector2 {
		let (x, y) = angle_to_direction(radians);
		Vector2 { x, y }
	}

	/// Calculates the distance between this and another vector.
	#[inline]
	pub fn distance(&self, other: &Vector2) -> f32 {
		self.distance_squared(other).sqrt()
	}

	/// Calculates the squared distance between this and another vector.
	#[inline]
	pub fn distance_squared(&self, other: &Vector2) -> f32 {
		(other.x - self.x) * (other.x - self.x) + (other.y - self.y) * (other.y - self.y)
	}

	/// Calculates the dot product of this and another vector.
	#[inline]
	pub fn dot(&self, other: &Vector2) -> f32 {
		(self.x * other.x) + (self.y * other.y)
	}

	/// Calculates the length (a.k.a. magnitude) of this vector.
	#[inline]
	pub fn length(&self) -> f32 {
		self.length_squared().sqrt()
	}

	/// Calculates the squared length of this vector.
	#[inline]
	pub fn length_squared(&self) -> f32 {
		(self.x * self.x) + (self.y * self.y)
	}

	/// Returns a normalized vector from this vector.
	pub fn normalize(&self) -> Vector2 {
		let inverse_length = 1.0 / self.length();
		Vector2 {
			x: self.x * inverse_length, //
			y: self.y * inverse_length,
		}
	}

	/// Returns an extended (or shrunk) vector from this vector, where the returned vector will
	/// have a length exactly matching the specified length, but will retain the same direction.
	pub fn extend(&self, length: f32) -> Vector2 {
		*self * (length / self.length())
	}

	/// Returns the angle (in radians) equivalent to the direction of this vector.
	#[inline]
	pub fn angle(&self) -> f32 {
		self.y.atan2(self.x)
	}

	/// Calculates the angle (in radians) between the this and another vector.
	#[inline]
	pub fn angle_between(&self, other: &Vector2) -> f32 {
		angle_between(self.x, self.y, other.x, other.y)
	}

	/// Returns true if this vector is nearly equal to the zero vector (0.0, 0.0).
	#[inline]
	pub fn almost_zero(&self, epsilon: f32) -> bool {
		self.nearly_equal(Vector2::ZERO, epsilon)
	}
}

impl Neg for Vector2 {
	type Output = Self;

	#[inline]
	fn neg(self) -> Self::Output {
		Vector2 {
			x: -self.x, //
			y: -self.y,
		}
	}
}

impl Add for Vector2 {
	type Output = Self;

	#[inline]
	fn add(self, rhs: Self) -> Self::Output {
		Vector2 {
			x: self.x + rhs.x, //
			y: self.y + rhs.y,
		}
	}
}

impl AddAssign for Vector2 {
	#[inline]
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl Sub for Vector2 {
	type Output = Self;

	#[inline]
	fn sub(self, rhs: Self) -> Self::Output {
		Vector2 {
			x: self.x - rhs.x, //
			y: self.y - rhs.y,
		}
	}
}

impl SubAssign for Vector2 {
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
	}
}

impl Mul for Vector2 {
	type Output = Self;

	#[inline]
	fn mul(self, rhs: Self) -> Self::Output {
		Vector2 {
			x: self.x * rhs.x, //
			y: self.y * rhs.y,
		}
	}
}

impl MulAssign for Vector2 {
	#[inline]
	fn mul_assign(&mut self, rhs: Self) {
		self.x *= rhs.x;
		self.y *= rhs.y;
	}
}

impl Div for Vector2 {
	type Output = Self;

	#[inline]
	fn div(self, rhs: Self) -> Self::Output {
		Vector2 {
			x: self.x / rhs.x, //
			y: self.y / rhs.y,
		}
	}
}

impl DivAssign for Vector2 {
	#[inline]
	fn div_assign(&mut self, rhs: Self) {
		self.x /= rhs.x;
		self.y /= rhs.y;
	}
}

impl Mul<f32> for Vector2 {
	type Output = Self;

	#[inline]
	fn mul(self, rhs: f32) -> Self::Output {
		Vector2 {
			x: self.x * rhs, //
			y: self.y * rhs,
		}
	}
}

impl MulAssign<f32> for Vector2 {
	#[inline]
	fn mul_assign(&mut self, rhs: f32) {
		self.x *= rhs;
		self.y *= rhs;
	}
}

impl Div<f32> for Vector2 {
	type Output = Self;

	#[inline]
	fn div(self, rhs: f32) -> Self::Output {
		Vector2 {
			x: self.x / rhs, //
			y: self.y / rhs,
		}
	}
}

impl DivAssign<f32> for Vector2 {
	#[inline]
	fn div_assign(&mut self, rhs: f32) {
		self.x /= rhs;
		self.y /= rhs;
	}
}

impl NearlyEqual for Vector2 {
	type Output = Self;

	#[inline(always)]
	fn nearly_equal(self, other: Self::Output, epsilon: f32) -> bool {
		nearly_equal(self.x, other.x, epsilon) && nearly_equal(self.y, other.y, epsilon)
	}
}

impl MulAssign<Matrix3x3> for Vector2 {
	#[rustfmt::skip]
	#[inline]
	fn mul_assign(&mut self, rhs: Matrix3x3) {
		let x = self.x * rhs.m[Matrix3x3::M11] + self.y * rhs.m[Matrix3x3::M12] + rhs.m[Matrix3x3::M13] + rhs.m[Matrix3x3::M31];
		let y = self.x * rhs.m[Matrix3x3::M21] + self.y * rhs.m[Matrix3x3::M22] + rhs.m[Matrix3x3::M23] + rhs.m[Matrix3x3::M32];
		self.x = x;
		self.y = y;
	}
}

#[cfg(test)]
pub mod tests {
	use crate::math::*;

	use super::*;

	#[test]
	pub fn test_new() {
		let v = Vector2::new(3.0, 7.0);
		assert!(nearly_equal(v.x, 3.0, 0.0001));
		assert!(nearly_equal(v.y, 7.0, 0.0001));
	}

	#[test]
	pub fn test_neg() {
		let v = Vector2 { x: 1.0, y: 2.0 };
		let neg = -v;
		assert!(nearly_equal(neg.x, -1.0, 0.0001));
		assert!(nearly_equal(neg.y, -2.0, 0.0001));
	}

	#[test]
	pub fn test_add() {
		let a = Vector2 { x: 3.0, y: 4.0 };
		let b = Vector2 { x: 1.0, y: 2.0 };
		let c = a + b;
		assert!(nearly_equal(c.x, 4.0, 0.0001));
		assert!(nearly_equal(c.y, 6.0, 0.0001));

		let mut a = Vector2 { x: 3.0, y: 4.0 };
		let b = Vector2 { x: 1.0, y: 2.0 };
		a += b;
		assert!(nearly_equal(a.x, 4.0, 0.0001));
		assert!(nearly_equal(a.y, 6.0, 0.0001));
	}

	#[test]
	pub fn test_sub() {
		let a = Vector2 { x: 3.0, y: 4.0 };
		let b = Vector2 { x: 1.0, y: 2.0 };
		let c = a - b;
		assert!(nearly_equal(c.x, 2.0, 0.0001));
		assert!(nearly_equal(c.y, 2.0, 0.0001));

		let mut a = Vector2 { x: 3.0, y: 4.0 };
		let b = Vector2 { x: 1.0, y: 2.0 };
		a -= b;
		assert!(nearly_equal(a.x, 2.0, 0.0001));
		assert!(nearly_equal(a.y, 2.0, 0.0001));
	}

	#[test]
	pub fn test_mul() {
		let a = Vector2 { x: 2.5, y: 6.0 };
		let b = Vector2 { x: 1.25, y: 2.0 };
		let c = a * b;
		assert!(nearly_equal(c.x, 3.125, 0.0001));
		assert!(nearly_equal(c.y, 12.0, 0.0001));

		let mut a = Vector2 { x: 2.5, y: 6.0 };
		let b = Vector2 { x: 1.25, y: 2.0 };
		a *= b;
		assert!(nearly_equal(a.x, 3.125, 0.0001));
		assert!(nearly_equal(a.y, 12.0, 0.0001));
	}

	#[test]
	pub fn test_div() {
		let a = Vector2 { x: 2.5, y: 6.0 };
		let b = Vector2 { x: 1.25, y: 2.0 };
		let c = a / b;
		assert!(nearly_equal(c.x, 2.0, 0.0001));
		assert!(nearly_equal(c.y, 3.0, 0.0001));

		let mut a = Vector2 { x: 2.5, y: 6.0 };
		let b = Vector2 { x: 1.25, y: 2.0 };
		a /= b;
		assert!(nearly_equal(a.x, 2.0, 0.0001));
		assert!(nearly_equal(a.y, 3.0, 0.0001));
	}

	#[test]
	pub fn test_scalar_mul() {
		let a = Vector2 { x: 1.0, y: 2.0 };
		let b = a * 2.0;
		assert!(nearly_equal(b.x, 2.0, 0.0001));
		assert!(nearly_equal(b.y, 4.0, 0.0001));

		let mut a = Vector2 { x: 1.0, y: 2.0 };
		a *= 2.0;
		assert!(nearly_equal(b.x, 2.0, 0.0001));
		assert!(nearly_equal(b.y, 4.0, 0.0001));
	}

	#[test]
	pub fn test_scalar_div() {
		let a = Vector2 { x: 1.0, y: 2.0 };
		let b = a / 2.0;
		assert!(nearly_equal(b.x, 0.5, 0.0001));
		assert!(nearly_equal(b.y, 1.0, 0.0001));

		let mut a = Vector2 { x: 1.0, y: 2.0 };
		a /= 2.0;
		assert!(nearly_equal(b.x, 0.5, 0.0001));
		assert!(nearly_equal(b.y, 1.0, 0.0001));
	}

	#[test]
	pub fn test_nearly_equal() {
		let a = Vector2 { x: 3.4, y: -7.1 };
		let b = Vector2 { x: 3.5, y: -7.1 };
		assert!(!a.nearly_equal(b, 0.0001));

		let a = Vector2 { x: 2.0, y: 4.0 };
		let b = Vector2 { x: 2.0, y: 4.0 };
		assert!(a.nearly_equal(b, 0.0001));
	}

	#[test]
	pub fn test_length() {
		let v = Vector2 { x: 6.0, y: 8.0 };
		let length_squared = v.length_squared();
		let length = v.length();
		assert!(nearly_equal(length_squared, 100.0, 0.0001));
		assert!(nearly_equal(length, 10.0, 0.0001));
	}

	#[test]
	pub fn test_dot() {
		let a = Vector2 { x: -6.0, y: 8.0 };
		let b = Vector2 { x: 5.0, y: 12.0 };
		let dot = a.dot(&b);
		assert!(nearly_equal(dot, 66.0, 0.0001));

		let a = Vector2 { x: -12.0, y: 16.0 };
		let b = Vector2 { x: 12.0, y: 9.0 };
		let dot = a.dot(&b);
		assert!(nearly_equal(dot, 0.0, 0.0001));
	}

	#[test]
	pub fn test_distance() {
		let a = Vector2 { x: 1.0, y: 1.0 };
		let b = Vector2 { x: 1.0, y: 3.0 };
		let distance_squared = a.distance_squared(&b);
		let distance = a.distance(&b);
		assert!(nearly_equal(distance_squared, 4.0, 0.0001));
		assert!(nearly_equal(distance, 2.0, 0.0001));
	}

	#[test]
	pub fn test_normalize() {
		let v = Vector2 { x: 3.0, y: 4.0 };
		let normalized = v.normalize();
		assert!(nearly_equal(normalized.x, 0.6, 0.0001));
		assert!(nearly_equal(normalized.y, 0.8, 0.0001));
	}

	#[test]
	pub fn test_extend() {
		let v = Vector2 { x: 10.0, y: 1.0 };
		let extended = v.extend(2.0);
		assert!(nearly_equal(extended.x, 1.990, 0.0001));
		assert!(nearly_equal(extended.y, 0.199, 0.0001));
	}

	#[test]
	#[rustfmt::skip]
	pub fn test_angle() {
		assert!(nearly_equal(RADIANS_0, Vector2::new(5.0, 0.0).angle(), 0.0001));
		assert!(nearly_equal(RADIANS_45, Vector2::new(5.0, 5.0).angle(), 0.0001));
		assert!(nearly_equal(RADIANS_90, Vector2::new(0.0, 5.0).angle(), 0.0001));
		assert!(nearly_equal(RADIANS_135, Vector2::new(-5.0, 5.0).angle(), 0.0001));
		assert!(nearly_equal(RADIANS_180, Vector2::new(-5.0, 0.0).angle(), 0.0001));
		assert!(nearly_equal(-RADIANS_135, Vector2::new(-5.0, -5.0).angle(), 0.0001));
		assert!(nearly_equal(-RADIANS_90, Vector2::new(0.0, -5.0).angle(), 0.0001));
		assert!(nearly_equal(-RADIANS_45, Vector2::new(5.0, -5.0).angle(), 0.0001));

		assert!(nearly_equal(crate::math::UP, Vector2::UP.angle(), 0.0001));
		assert!(nearly_equal(crate::math::DOWN, Vector2::DOWN.angle(), 0.0001));
		assert!(nearly_equal(crate::math::LEFT, Vector2::LEFT.angle(), 0.0001));
		assert!(nearly_equal(crate::math::RIGHT, Vector2::RIGHT.angle(), 0.0001));
	}

	#[test]
	pub fn test_from_angle() {
		let v = Vector2::from_angle(RADIANS_0);
		assert!(nearly_equal(v.x, 1.0, 0.000001));
		assert!(nearly_equal(v.y, 0.0, 0.000001));
		let v = Vector2::from_angle(RADIANS_45);
		assert!(nearly_equal(v.x, 0.707106, 0.000001));
		assert!(nearly_equal(v.y, 0.707106, 0.000001));
		let v = Vector2::from_angle(RADIANS_225);
		assert!(nearly_equal(v.x, -0.707106, 0.000001));
		assert!(nearly_equal(v.y, -0.707106, 0.000001));

		let v = Vector2::from_angle(UP);
		assert!(v.nearly_equal(Vector2::UP, 0.000001));
		let v = Vector2::from_angle(DOWN);
		assert!(v.nearly_equal(Vector2::DOWN, 0.000001));
		let v = Vector2::from_angle(LEFT);
		assert!(v.nearly_equal(Vector2::LEFT, 0.000001));
		let v = Vector2::from_angle(RIGHT);
		assert!(v.nearly_equal(Vector2::RIGHT, 0.000001));
	}

	#[test]
	pub fn test_angle_between() {
		let a = Vector2::new(20.0, 20.0);
		let b = Vector2::new(10.0, 10.0);
		let angle = a.angle_between(&b);
		assert!(nearly_equal(-RADIANS_135, angle, 0.0001));

		let a = Vector2::new(0.0, 0.0);
		let b = Vector2::new(10.0, 10.0);
		let angle = a.angle_between(&b);
		assert!(nearly_equal(RADIANS_45, angle, 0.0001));

		let a = Vector2::new(5.0, 5.0);
		let b = Vector2::new(5.0, 5.0);
		let angle = a.angle_between(&b);
		assert!(nearly_equal(0.0, angle, 0.0001));
	}

	#[test]
	pub fn test_lerp() {
		let a = Vector2 { x: 5.0, y: 1.0 };
		let b = Vector2 { x: 10.0, y: 2.0 };
		let c = lerp(a, b, 0.5);
		assert!(nearly_equal(c.x, 7.5, 0.0001));
		assert!(nearly_equal(c.y, 1.5, 0.0001));
	}
}
