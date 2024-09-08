use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use super::{nearly_equal, NearlyEqual};

/// Represents a 4D vector and provides common methods for vector math.
/// Uses a right-handed 3D coordinate system.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector4 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub w: f32,
}

impl Vector4 {
	pub const ZERO: Vector4 = Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };

	/// Creates a new vector with the specified X, Y, Z and W components.
	#[inline]
	pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
		Vector4 { x, y, z, w }
	}

	/// Calculates the distance between this and another vector.
	#[inline]
	pub fn distance(&self, other: &Vector4) -> f32 {
		self.distance_squared(other).sqrt()
	}

	/// Calculates the squared distance between this and another vector.
	#[inline]
	pub fn distance_squared(&self, other: &Vector4) -> f32 {
		((other.x - self.x) * (other.x - self.x))
			+ ((other.y - self.y) * (other.y - self.y))
			+ ((other.z - self.z) * (other.z - self.z))
			+ ((other.w - self.w) * (other.w - self.w))
	}

	/// Calculates the dot product of this and another vector.
	#[inline]
	pub fn dot(&self, other: &Vector4) -> f32 {
		(self.x * other.x) + (self.y * other.y) + (self.z * other.z) + (self.w * other.w)
	}

	/// Calculates the length (a.k.a. magnitude) of this vector.
	#[inline]
	pub fn length(&self) -> f32 {
		self.length_squared().sqrt()
	}

	/// Calculates the squared length of this vector.
	#[inline]
	pub fn length_squared(&self) -> f32 {
		(self.x * self.x) + (self.y * self.y) + (self.z * self.z) + (self.w * self.w)
	}

	/// Returns a normalized vector from this vector.
	#[inline]
	pub fn normalize(&self) -> Vector4 {
		let inverse_length = 1.0 / self.length();
		Vector4 {
			x: self.x * inverse_length, //
			y: self.y * inverse_length,
			z: self.z * inverse_length,
			w: self.w * inverse_length,
		}
	}

	/// Returns an extended (or shrunk) vector from this vector, where the returned vector will
	/// have a length exactly matching the specified length, but will retain the same
	/// direction.
	#[inline]
	pub fn extend(&self, length: f32) -> Vector4 {
		*self * (length / self.length())
	}
}

impl Neg for Vector4 {
	type Output = Self;

	#[inline]
	fn neg(self) -> Self::Output {
		Vector4 {
			x: -self.x, //
			y: -self.y,
			z: -self.z,
			w: -self.w,
		}
	}
}

impl Add for Vector4 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Vector4 {
			x: self.x + rhs.x, //
			y: self.y + rhs.y,
			z: self.z + rhs.z,
			w: self.w + rhs.w,
		}
	}
}

impl AddAssign for Vector4 {
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
		self.w += rhs.w;
	}
}

impl Sub for Vector4 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Vector4 {
			x: self.x - rhs.x, //
			y: self.y - rhs.y,
			z: self.z - rhs.z,
			w: self.w - rhs.w,
		}
	}
}

impl SubAssign for Vector4 {
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
		self.w -= rhs.w;
	}
}

impl Mul for Vector4 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Vector4 {
			x: self.x * rhs.x, //
			y: self.y * rhs.y,
			z: self.z * rhs.z,
			w: self.w * rhs.w,
		}
	}
}

impl MulAssign for Vector4 {
	fn mul_assign(&mut self, rhs: Self) {
		self.x *= rhs.x;
		self.y *= rhs.y;
		self.z *= rhs.z;
		self.w *= rhs.w;
	}
}

impl Div for Vector4 {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		Vector4 {
			x: self.x / rhs.x, //
			y: self.y / rhs.y,
			z: self.z / rhs.z,
			w: self.w / rhs.w,
		}
	}
}

impl DivAssign for Vector4 {
	fn div_assign(&mut self, rhs: Self) {
		self.x /= rhs.x;
		self.y /= rhs.y;
		self.z /= rhs.z;
		self.w /= rhs.w;
	}
}

impl Mul<f32> for Vector4 {
	type Output = Self;

	fn mul(self, rhs: f32) -> Self::Output {
		Vector4 {
			x: self.x * rhs, //
			y: self.y * rhs,
			z: self.z * rhs,
			w: self.w * rhs,
		}
	}
}

impl MulAssign<f32> for Vector4 {
	fn mul_assign(&mut self, rhs: f32) {
		self.x *= rhs;
		self.y *= rhs;
		self.z *= rhs;
		self.w *= rhs;
	}
}

impl Div<f32> for Vector4 {
	type Output = Self;

	fn div(self, rhs: f32) -> Self::Output {
		Vector4 {
			x: self.x / rhs, //
			y: self.y / rhs,
			z: self.z / rhs,
			w: self.w / rhs,
		}
	}
}

impl DivAssign<f32> for Vector4 {
	fn div_assign(&mut self, rhs: f32) {
		self.x /= rhs;
		self.y /= rhs;
		self.z /= rhs;
		self.w /= rhs;
	}
}

impl NearlyEqual for Vector4 {
	type Output = Self;

	fn nearly_equal(self, other: Self::Output, epsilon: f32) -> bool {
		nearly_equal(self.x, other.x, epsilon)
			&& nearly_equal(self.y, other.y, epsilon)
			&& nearly_equal(self.z, other.z, epsilon)
			&& nearly_equal(self.w, other.w, epsilon)
	}
}

#[cfg(test)]
mod tests {
	use crate::math::*;

	use super::*;

	#[test]
	pub fn test_new() {
		let v = Vector4::new(3.0, 7.0, -1.4, 2.1);
		assert!(nearly_equal(v.x, 3.0, 0.0001));
		assert!(nearly_equal(v.y, 7.0, 0.0001));
		assert!(nearly_equal(v.z, -1.4, 0.0001));
		assert!(nearly_equal(v.w, 2.1, 0.0001));
	}

	#[test]
	pub fn test_neg() {
		let v = Vector4 { x: 1.0, y: 2.0, z: -3.0, w: 4.0 };
		let neg = -v;
		assert!(nearly_equal(neg.x, -1.0, 0.0001));
		assert!(nearly_equal(neg.y, -2.0, 0.0001));
		assert!(nearly_equal(neg.z, 3.0, 0.0001));
		assert!(nearly_equal(neg.w, -4.0, 0.0001));
	}

	#[test]
	pub fn test_add() {
		let a = Vector4 { x: 3.0, y: 4.0, z: 6.0, w: 7.0 };
		let b = Vector4 { x: 1.0, y: 2.0, z: 5.0, w: 8.0 };
		let c = a + b;
		assert!(nearly_equal(c.x, 4.0, 0.0001));
		assert!(nearly_equal(c.y, 6.0, 0.0001));
		assert!(nearly_equal(c.z, 11.0, 0.0001));
		assert!(nearly_equal(c.w, 15.0, 0.0001));

		let mut a = Vector4 { x: 3.0, y: 4.0, z: 6.0, w: 7.0 };
		let b = Vector4 { x: 1.0, y: 2.0, z: 5.0, w: 8.0 };
		a += b;
		assert!(nearly_equal(a.x, 4.0, 0.0001));
		assert!(nearly_equal(a.y, 6.0, 0.0001));
		assert!(nearly_equal(a.z, 11.0, 0.0001));
		assert!(nearly_equal(a.w, 15.0, 0.0001));
	}

	#[test]
	pub fn test_sub() {
		let a = Vector4 { x: 3.0, y: 4.0, z: 6.0, w: 7.0 };
		let b = Vector4 { x: 1.0, y: 2.0, z: 5.0, w: 8.0 };
		let c = a - b;
		assert!(nearly_equal(c.x, 2.0, 0.0001));
		assert!(nearly_equal(c.y, 2.0, 0.0001));
		assert!(nearly_equal(c.z, 1.0, 0.0001));
		assert!(nearly_equal(c.w, -1.0, 0.0001));

		let mut a = Vector4 { x: 3.0, y: 4.0, z: 6.0, w: 7.0 };
		let b = Vector4 { x: 1.0, y: 2.0, z: 5.0, w: 8.0 };
		a -= b;
		assert!(nearly_equal(a.x, 2.0, 0.0001));
		assert!(nearly_equal(a.y, 2.0, 0.0001));
		assert!(nearly_equal(a.z, 1.0, 0.0001));
		assert!(nearly_equal(a.w, -1.0, 0.0001));
	}

	#[test]
	pub fn test_mul() {
		let a = Vector4 { x: 2.5, y: 6.0, z: 3.0, w: -4.1 };
		let b = Vector4 { x: 1.25, y: 2.0, z: 2.5, w: 2.0 };
		let c = a * b;
		assert!(nearly_equal(c.x, 3.125, 0.0001));
		assert!(nearly_equal(c.y, 12.0, 0.0001));
		assert!(nearly_equal(c.z, 7.5, 0.0001));
		assert!(nearly_equal(c.w, -8.2, 0.0001));

		let mut a = Vector4 { x: 2.5, y: 6.0, z: 3.0, w: -4.1 };
		let b = Vector4 { x: 1.25, y: 2.0, z: 2.5, w: 2.0 };
		a *= b;
		assert!(nearly_equal(a.x, 3.125, 0.0001));
		assert!(nearly_equal(a.y, 12.0, 0.0001));
		assert!(nearly_equal(a.z, 7.5, 0.0001));
		assert!(nearly_equal(a.w, -8.2, 0.0001));
	}

	#[test]
	pub fn test_div() {
		let a = Vector4 { x: 2.5, y: 6.0, z: 3.0, w: -4.1 };
		let b = Vector4 { x: 1.25, y: 2.0, z: 2.5, w: 2.0 };
		let c = a / b;
		assert!(nearly_equal(c.x, 2.0, 0.0001));
		assert!(nearly_equal(c.y, 3.0, 0.0001));
		assert!(nearly_equal(c.z, 1.2, 0.0001));
		assert!(nearly_equal(c.w, -2.05, 0.0001));

		let mut a = Vector4 { x: 2.5, y: 6.0, z: 3.0, w: -4.1 };
		let b = Vector4 { x: 1.25, y: 2.0, z: 2.5, w: 2.0 };
		a /= b;
		assert!(nearly_equal(a.x, 2.0, 0.0001));
		assert!(nearly_equal(a.y, 3.0, 0.0001));
		assert!(nearly_equal(a.z, 1.2, 0.0001));
		assert!(nearly_equal(a.w, -2.05, 0.0001));
	}

	#[test]
	pub fn test_scalar_mul() {
		let a = Vector4 { x: 1.0, y: 2.0, z: -3.0, w: 4.0 };
		let b = a * 2.0;
		assert!(nearly_equal(b.x, 2.0, 0.0001));
		assert!(nearly_equal(b.y, 4.0, 0.0001));
		assert!(nearly_equal(b.z, -6.0, 0.0001));
		assert!(nearly_equal(b.w, 8.0, 0.0001));

		let mut a = Vector4 { x: 1.0, y: 2.0, z: -3.0, w: 4.0 };
		a *= 2.0;
		assert!(nearly_equal(b.x, 2.0, 0.0001));
		assert!(nearly_equal(b.y, 4.0, 0.0001));
		assert!(nearly_equal(b.z, -6.0, 0.0001));
		assert!(nearly_equal(b.w, 8.0, 0.0001));
	}

	#[test]
	pub fn test_scalar_div() {
		let a = Vector4 { x: 1.0, y: 2.0, z: -3.0, w: 4.0 };
		let b = a / 2.0;
		assert!(nearly_equal(b.x, 0.5, 0.0001));
		assert!(nearly_equal(b.y, 1.0, 0.0001));
		assert!(nearly_equal(b.z, -1.5, 0.0001));
		assert!(nearly_equal(b.w, 2.0, 0.0001));

		let mut a = Vector4 { x: 1.0, y: 2.0, z: -3.0, w: 4.0 };
		a /= 2.0;
		assert!(nearly_equal(b.x, 0.5, 0.0001));
		assert!(nearly_equal(b.y, 1.0, 0.0001));
		assert!(nearly_equal(b.z, -1.5, 0.0001));
		assert!(nearly_equal(b.w, 2.0, 0.0001));
	}

	#[test]
	pub fn test_nearly_equal() {
		let a = Vector4 { x: 3.5, y: -7.1, z: 1.8, w: 2.1 };
		let b = Vector4 { x: 3.5, y: -7.1, z: 1.8, w: 2.2 };
		assert!(!a.nearly_equal(b, 0.0001));

		let a = Vector4 { x: 2.0, y: 4.0, z: 6.1, w: 3.5 };
		let b = Vector4 { x: 2.0, y: 4.0, z: 6.1, w: 3.5 };
		assert!(a.nearly_equal(b, 0.0001));
	}

	#[test]
	pub fn test_length() {
		let v = Vector4 { x: 3.0, y: -2.0, z: 1.0, w: 4.0 };
		let length_squared = v.length_squared();
		let length = v.length();
		assert!(nearly_equal(length_squared, 30.0, 0.0001));
		assert!(nearly_equal(length, 5.4772, 0.0001));

		let v = Vector4 { x: 11.6, y: 0.0, z: 4.0, w: 1.2 };
		let length_squared = v.length_squared();
		let length = v.length();
		assert!(nearly_equal(length_squared, 152.0001, 0.0001));
		assert!(nearly_equal(length, 12.3288, 0.0001));
	}

	#[test]
	pub fn test_dot() {
		let a = Vector4 { x: -1.0, y: 2.0, z: 3.0, w: 4.0 };
		let b = Vector4 { x: 0.0, y: 5.0, z: 1.0, w: 6.0 };
		let dot = a.dot(&b);
		assert!(nearly_equal(dot, 37.0, 0.0001));

		let a = Vector4 { x: 4.0, y: 0.0, z: -3.0, w: 1.0 };
		let b = Vector4 { x: 0.0, y: -2.0, z: 0.0, w: 0.0 };
		let dot = a.dot(&b);
		assert!(nearly_equal(dot, 0.0, 0.0001));
	}

	#[test]
	pub fn test_distance() {
		let a = Vector4 { x: 6.0, y: 2.5, z: -3.0, w: 1.0 };
		let b = Vector4 { x: 1.1, y: -4.0, z: 5.0, w: 2.0 };
		let distance_squared = a.distance_squared(&b);
		let distance = a.distance(&b);
		assert!(nearly_equal(distance_squared, 131.26, 0.0001));
		assert!(nearly_equal(distance, 11.4569, 0.0001));

		let a = Vector4 { x: 7.1, y: 4.0, z: 3.0, w: 2.0 };
		let b = Vector4 { x: 17.0, y: -6.0, z: 2.5, w: 1.0 };
		let distance_squared = a.distance_squared(&b);
		let distance = a.distance(&b);
		assert!(nearly_equal(distance_squared, 199.26, 0.0001));
		assert!(nearly_equal(distance, 14.1159, 0.0001));
	}

	#[test]
	pub fn test_normalize() {
		let v = Vector4 { x: 3.0, y: 1.0, z: 2.0, w: 4.0 };
		let normalized = v.normalize();
		assert!(nearly_equal(normalized.x, 0.5477, 0.0001));
		assert!(nearly_equal(normalized.y, 0.1826, 0.0001));
		assert!(nearly_equal(normalized.z, 0.3651, 0.0001));
		assert!(nearly_equal(normalized.w, 0.7303, 0.0001));
	}

	#[test]
	pub fn test_extend() {
		let v = Vector4 { x: 3.0, y: -2.0, z: 1.0, w: 4.0 };
		let extended = v.extend(10.9545);
		assert!(nearly_equal(extended.x, 6.0, 0.0001));
		assert!(nearly_equal(extended.y, -4.0, 0.0001));
		assert!(nearly_equal(extended.z, 2.0, 0.0001));
		assert!(nearly_equal(extended.w, 8.0, 0.0001));
	}

	#[test]
	pub fn test_lerp() {
		let a = Vector4 { x: 5.0, y: 1.0, z: 3.0, w: 1.2 };
		let b = Vector4 { x: 10.0, y: 2.0, z: 6.0, w: 2.4 };
		let c = lerp(a, b, 0.5);
		assert!(nearly_equal(c.x, 7.5, 0.0001));
		assert!(nearly_equal(c.y, 1.5, 0.0001));
		assert!(nearly_equal(c.z, 4.5, 0.0001));
		assert!(nearly_equal(c.w, 1.8, 0.0001));
	}
}
