use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::math::{nearly_equal, NearlyEqual};

/// Represents a 3D vector and provides common methods for vector math.
/// Uses a right-handed 3D coordinate system.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector3 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl Vector3 {
	pub const ZERO: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 0.0 };

	pub const UP: Vector3 = Vector3 { x: 0.0, y: 1.0, z: 0.0 };
	pub const DOWN: Vector3 = Vector3 { x: 0.0, y: -1.0, z: 0.0 };
	pub const FORWARD: Vector3 = Vector3 { x: 0.0, y: 0.0, z: -1.0 };
	pub const BACKWARD: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 1.0 };
	pub const LEFT: Vector3 = Vector3 { x: -1.0, y: 0.0, z: 0.0 };
	pub const RIGHT: Vector3 = Vector3 { x: 1.0, y: 0.0, z: 0.0 };

	pub const X_AXIS: Vector3 = Vector3 { x: 1.0, y: 0.0, z: 0.0 };
	pub const Y_AXIS: Vector3 = Vector3 { x: 0.0, y: 1.0, z: 0.0 };
	pub const Z_AXIS: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 1.0 };

	/// Creates a new vector with the specified X, Y and Z components.
	#[inline]
	pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
		Vector3 { x, y, z }
	}

	/// Returns the cross product of this and another vector.
	#[inline]
	pub fn cross(&self, other: &Vector3) -> Vector3 {
		Vector3 {
			x: (self.y * other.z) - (self.z * other.y),
			y: (self.z * other.x) - (self.x * other.z),
			z: (self.x * other.y) - (self.y * other.x),
		}
	}

	/// Calculates the distance between this and another vector.
	#[inline]
	pub fn distance(&self, other: &Vector3) -> f32 {
		self.distance_squared(other).sqrt()
	}

	/// Calculates the squared distance between this and another vector.
	#[inline]
	pub fn distance_squared(&self, other: &Vector3) -> f32 {
		((other.x - self.x) * (other.x - self.x))
			+ ((other.y - self.y) * (other.y - self.y))
			+ ((other.z - self.z) * (other.z - self.z))
	}

	/// Calculates the dot product of this and another vector.
	#[inline]
	pub fn dot(&self, other: &Vector3) -> f32 {
		(self.x * other.x) + (self.y * other.y) + (self.z * other.z)
	}

	/// Calculate the length (a.k.a. magnitude) of this vector.
	#[inline]
	pub fn length(&self) -> f32 {
		self.length_squared().sqrt()
	}

	/// Calculates the squared length of this vector.
	#[inline]
	pub fn length_squared(&self) -> f32 {
		(self.x * self.x) + (self.y * self.y) + (self.z * self.z)
	}

	/// Returns a normalized vector from this vector.
	#[inline]
	pub fn normalize(&self) -> Vector3 {
		let inverse_length = 1.0 / self.length();
		Vector3 {
			x: self.x * inverse_length, //
			y: self.y * inverse_length,
			z: self.z * inverse_length,
		}
	}

	/// Returns an extended (or shrunk) vector from this vector, where the returned vector will
	/// have a length exactly matching the specified length, but will return the same direction.
	#[inline]
	pub fn extend(&self, length: f32) -> Vector3 {
		*self * (length / self.length())
	}
}

impl Neg for Vector3 {
	type Output = Self;

	#[inline]
	fn neg(self) -> Self::Output {
		Vector3 {
			x: -self.x, //
			y: -self.y,
			z: -self.z,
		}
	}
}

impl Add for Vector3 {
	type Output = Self;

	#[inline]
	fn add(self, rhs: Self) -> Self::Output {
		Vector3 {
			x: self.x + rhs.x, //
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
	}
}

impl AddAssign for Vector3 {
	#[inline]
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
	}
}

impl Sub for Vector3 {
	type Output = Self;

	#[inline]
	fn sub(self, rhs: Self) -> Self::Output {
		Vector3 {
			x: self.x - rhs.x, //
			y: self.y - rhs.y,
			z: self.z - rhs.z,
		}
	}
}

impl SubAssign for Vector3 {
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
	}
}

impl Mul for Vector3 {
	type Output = Self;

	#[inline]
	fn mul(self, rhs: Self) -> Self::Output {
		Vector3 {
			x: self.x * rhs.x, //
			y: self.y * rhs.y,
			z: self.z * rhs.z,
		}
	}
}

impl MulAssign for Vector3 {
	#[inline]
	fn mul_assign(&mut self, rhs: Self) {
		self.x *= rhs.x;
		self.y *= rhs.y;
		self.z *= rhs.z;
	}
}

impl Div for Vector3 {
	type Output = Self;

	#[inline]
	fn div(self, rhs: Self) -> Self::Output {
		Vector3 {
			x: self.x / rhs.x, //
			y: self.y / rhs.y,
			z: self.z / rhs.z,
		}
	}
}

impl DivAssign for Vector3 {
	#[inline]
	fn div_assign(&mut self, rhs: Self) {
		self.x /= rhs.x;
		self.y /= rhs.y;
		self.z /= rhs.z;
	}
}

impl Mul<f32> for Vector3 {
	type Output = Self;

	#[inline]
	fn mul(self, rhs: f32) -> Self::Output {
		Vector3 {
			x: self.x * rhs, //
			y: self.y * rhs,
			z: self.z * rhs,
		}
	}
}

impl MulAssign<f32> for Vector3 {
	#[inline]
	fn mul_assign(&mut self, rhs: f32) {
		self.x *= rhs;
		self.y *= rhs;
		self.z *= rhs;
	}
}

impl Div<f32> for Vector3 {
	type Output = Self;

	#[inline]
	fn div(self, rhs: f32) -> Self::Output {
		Vector3 {
			x: self.x / rhs, //
			y: self.y / rhs,
			z: self.z / rhs,
		}
	}
}

impl DivAssign<f32> for Vector3 {
	#[inline]
	fn div_assign(&mut self, rhs: f32) {
		self.x /= rhs;
		self.y /= rhs;
		self.z /= rhs;
	}
}

impl NearlyEqual for Vector3 {
	type Output = Self;

	#[inline]
	fn nearly_equal(self, other: Self::Output, epsilon: f32) -> bool {
		nearly_equal(self.x, other.x, epsilon)
			&& nearly_equal(self.y, other.y, epsilon)
			&& nearly_equal(self.z, other.z, epsilon)
	}
}

#[cfg(test)]
mod tests {
	use crate::math::*;

	use super::*;

	#[test]
	pub fn test_new() {
		let v = Vector3::new(3.0, 7.0, -1.4);
		assert!(nearly_equal(v.x, 3.0, 0.0001));
		assert!(nearly_equal(v.y, 7.0, 0.0001));
		assert!(nearly_equal(v.z, -1.4, 0.0001));
	}

	#[test]
	pub fn test_neg() {
		let v = Vector3 { x: 1.0, y: 2.0, z: -3.0 };
		let neg = -v;
		assert!(nearly_equal(neg.x, -1.0, 0.0001));
		assert!(nearly_equal(neg.y, -2.0, 0.0001));
		assert!(nearly_equal(neg.z, 3.0, 0.0001));
	}

	#[test]
	pub fn test_add() {
		let a = Vector3 { x: 3.0, y: 4.0, z: 6.0 };
		let b = Vector3 { x: 1.0, y: 2.0, z: 5.0 };
		let c = a + b;
		assert!(nearly_equal(c.x, 4.0, 0.0001));
		assert!(nearly_equal(c.y, 6.0, 0.0001));
		assert!(nearly_equal(c.z, 11.0, 0.0001));

		let mut a = Vector3 { x: 3.0, y: 4.0, z: 6.0 };
		let b = Vector3 { x: 1.0, y: 2.0, z: 5.0 };
		a += b;
		assert!(nearly_equal(a.x, 4.0, 0.0001));
		assert!(nearly_equal(a.y, 6.0, 0.0001));
		assert!(nearly_equal(a.z, 11.0, 0.0001));
	}

	#[test]
	pub fn test_sub() {
		let a = Vector3 { x: 3.0, y: 4.0, z: 6.0 };
		let b = Vector3 { x: 1.0, y: 2.0, z: 5.0 };
		let c = a - b;
		assert!(nearly_equal(c.x, 2.0, 0.0001));
		assert!(nearly_equal(c.y, 2.0, 0.0001));
		assert!(nearly_equal(c.z, 1.0, 0.0001));

		let mut a = Vector3 { x: 3.0, y: 4.0, z: 6.0 };
		let b = Vector3 { x: 1.0, y: 2.0, z: 5.0 };
		a -= b;
		assert!(nearly_equal(a.x, 2.0, 0.0001));
		assert!(nearly_equal(a.y, 2.0, 0.0001));
		assert!(nearly_equal(a.z, 1.0, 0.0001));
	}

	#[test]
	pub fn test_mul() {
		let a = Vector3 { x: 2.5, y: 6.0, z: 3.0 };
		let b = Vector3 { x: 1.25, y: 2.0, z: 2.5 };
		let c = a * b;
		assert!(nearly_equal(c.x, 3.125, 0.0001));
		assert!(nearly_equal(c.y, 12.0, 0.0001));
		assert!(nearly_equal(c.z, 7.5, 0.0001));

		let mut a = Vector3 { x: 2.5, y: 6.0, z: 3.0 };
		let b = Vector3 { x: 1.25, y: 2.0, z: 2.5 };
		a *= b;
		assert!(nearly_equal(a.x, 3.125, 0.0001));
		assert!(nearly_equal(a.y, 12.0, 0.0001));
		assert!(nearly_equal(a.z, 7.5, 0.0001));
	}

	#[test]
	pub fn test_div() {
		let a = Vector3 { x: 2.5, y: 6.0, z: 3.0 };
		let b = Vector3 { x: 1.25, y: 2.0, z: 2.5 };
		let c = a / b;
		assert!(nearly_equal(c.x, 2.0, 0.0001));
		assert!(nearly_equal(c.y, 3.0, 0.0001));
		assert!(nearly_equal(c.z, 1.2, 0.0001));

		let mut a = Vector3 { x: 2.5, y: 6.0, z: 3.0 };
		let b = Vector3 { x: 1.25, y: 2.0, z: 2.5 };
		a /= b;
		assert!(nearly_equal(a.x, 2.0, 0.0001));
		assert!(nearly_equal(a.y, 3.0, 0.0001));
		assert!(nearly_equal(a.z, 1.2, 0.0001));
	}

	#[test]
	pub fn test_scalar_mul() {
		let a = Vector3 { x: 1.0, y: 2.0, z: -3.0 };
		let b = a * 2.0;
		assert!(nearly_equal(b.x, 2.0, 0.0001));
		assert!(nearly_equal(b.y, 4.0, 0.0001));
		assert!(nearly_equal(b.z, -6.0, 0.0001));

		let mut a = Vector3 { x: 1.0, y: 2.0, z: -3.0 };
		a *= 2.0;
		assert!(nearly_equal(b.x, 2.0, 0.0001));
		assert!(nearly_equal(b.y, 4.0, 0.0001));
		assert!(nearly_equal(b.z, -6.0, 0.0001));
	}

	#[test]
	pub fn test_scalar_div() {
		let a = Vector3 { x: 1.0, y: 2.0, z: -3.0 };
		let b = a / 2.0;
		assert!(nearly_equal(b.x, 0.5, 0.0001));
		assert!(nearly_equal(b.y, 1.0, 0.0001));
		assert!(nearly_equal(b.z, -1.5, 0.0001));

		let mut a = Vector3 { x: 1.0, y: 2.0, z: -3.0 };
		a /= 2.0;
		assert!(nearly_equal(b.x, 0.5, 0.0001));
		assert!(nearly_equal(b.y, 1.0, 0.0001));
		assert!(nearly_equal(b.z, -1.5, 0.0001));
	}

	#[test]
	pub fn test_nearly_equal() {
		let a = Vector3 { x: 3.4, y: -7.1, z: 1.8 };
		let b = Vector3 { x: 3.5, y: -7.1, z: 1.8 };
		assert!(!a.nearly_equal(b, 0.0001));

		let a = Vector3 { x: 2.0, y: 4.0, z: 6.1 };
		let b = Vector3 { x: 2.0, y: 4.0, z: 6.1 };
		assert!(a.nearly_equal(b, 0.0001));
	}

	#[test]
	pub fn test_length() {
		let v = Vector3 { x: 3.0, y: -2.0, z: 1.0 };
		let length_squared = v.length_squared();
		let length = v.length();
		assert!(nearly_equal(length_squared, 14.0, 0.0001));
		assert!(nearly_equal(length, 3.7416, 0.0001));

		let v = Vector3 { x: 11.6, y: 0.0, z: 4.0 };
		let length_squared = v.length_squared();
		let length = v.length();
		assert!(nearly_equal(length_squared, 150.56, 0.0001));
		assert!(nearly_equal(length, 12.2702, 0.0001));
	}

	#[test]
	pub fn test_dot() {
		let a = Vector3 { x: -1.0, y: 2.0, z: 3.0 };
		let b = Vector3 { x: 0.0, y: 5.0, z: 1.0 };
		let dot = a.dot(&b);
		assert!(nearly_equal(dot, 13.0, 0.0001));

		let a = Vector3 { x: 4.0, y: 0.0, z: -3.0 };
		let b = Vector3 { x: 0.0, y: -2.0, z: 0.0 };
		let dot = a.dot(&b);
		assert!(nearly_equal(dot, 0.0, 0.0001));
	}

	#[test]
	pub fn test_distance() {
		let a = Vector3 { x: 6.0, y: 2.5, z: -3.0 };
		let b = Vector3 { x: 1.1, y: -4.0, z: 5.0 };
		let distance_squared = a.distance_squared(&b);
		let distance = a.distance(&b);
		assert!(nearly_equal(distance_squared, 130.26, 0.0001));
		assert!(nearly_equal(distance, 11.4131, 0.0001));

		let a = Vector3 { x: 7.1, y: 4.0, z: 3.0 };
		let b = Vector3 { x: 17.0, y: -6.0, z: 2.5 };
		let distance_squared = a.distance_squared(&b);
		let distance = a.distance(&b);
		assert!(nearly_equal(distance_squared, 198.26, 0.0001));
		assert!(nearly_equal(distance, 14.0804, 0.0001));
	}

	#[test]
	pub fn test_normalize() {
		let v = Vector3 { x: 3.0, y: 1.0, z: 2.0 };
		let normalized = v.normalize();
		assert!(nearly_equal(normalized.x, 0.8017, 0.0001));
		assert!(nearly_equal(normalized.y, 0.2672, 0.0001));
		assert!(nearly_equal(normalized.z, 0.5345, 0.0001));
	}

	#[test]
	pub fn test_extend() {
		let v = Vector3 { x: 3.0, y: -2.0, z: 1.0 };
		let extended = v.extend(7.4832);
		assert!(nearly_equal(extended.x, 6.0, 0.0001));
		assert!(nearly_equal(extended.y, -4.0, 0.0001));
		assert!(nearly_equal(extended.z, 2.0, 0.0001));
	}

	#[test]
	pub fn test_lerp() {
		let a = Vector3 { x: 5.0, y: 1.0, z: 3.0 };
		let b = Vector3 { x: 10.0, y: 2.0, z: 6.0 };
		let c = lerp(a, b, 0.5);
		assert!(nearly_equal(c.x, 7.5, 0.0001));
		assert!(nearly_equal(c.y, 1.5, 0.0001));
		assert!(nearly_equal(c.z, 4.5, 0.0001));
	}
}
