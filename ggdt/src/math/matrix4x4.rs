use std::ops::{Mul, MulAssign};

use crate::math::{nearly_equal, Vector3, Vector4};

/// Represents a 4x4 column-major matrix and provides common methods for matrix math.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Matrix4x4 {
	pub m: [f32; 16],
}

impl Matrix4x4 {
	pub const M11: usize = 0;
	pub const M12: usize = 4;
	pub const M13: usize = 8;
	pub const M14: usize = 12;
	pub const M21: usize = 1;
	pub const M22: usize = 5;
	pub const M23: usize = 9;
	pub const M24: usize = 13;
	pub const M31: usize = 2;
	pub const M32: usize = 6;
	pub const M33: usize = 10;
	pub const M34: usize = 14;
	pub const M41: usize = 3;
	pub const M42: usize = 7;
	pub const M43: usize = 11;
	pub const M44: usize = 15;

	#[rustfmt::skip]
	pub const IDENTITY: Matrix4x4 = Matrix4x4 {
		m: [
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0
		],
	};

	/// Returns a new identity matrix.
	#[inline]
	pub fn identity() -> Matrix4x4 {
		Matrix4x4::IDENTITY
	}

	/// Creates a new matrix with the specified elements.
	#[allow(clippy::too_many_arguments)]
	#[rustfmt::skip]
	#[inline]
	pub fn new(
		m11: f32, m12: f32, m13: f32, m14: f32,
		m21: f32, m22: f32, m23: f32, m24: f32,
		m31: f32, m32: f32, m33: f32, m34: f32,
		m41: f32, m42: f32, m43: f32, m44: f32,
	) -> Matrix4x4 {
		Matrix4x4 {
			m: [
				m11, m21, m31, m41,
				m12, m22, m32, m42,
				m13, m23, m33, m43,
				m14, m24, m34, m44,
			]
		}
	}

	/// Creates a new rotation matrix from a set of euler angles.
	///
	/// # Arguments
	///
	/// * `x`: the x angle (in radians)
	/// * `y`: the y angle (in radians)
	/// * `z`: the z angle (in radians)
	pub fn from_euler_angles(x: f32, y: f32, z: f32) -> Matrix4x4 {
		let rotate_z = Matrix4x4::new_rotation_z(z);
		let rotate_y = Matrix4x4::new_rotation_y(y);
		let rotate_x = Matrix4x4::new_rotation_x(x);

		// "right-to-left" column-major matrix concatenation
		rotate_z * rotate_y * rotate_x
	}

	/// Creates a new rotation matrix for rotation around the x axis.
	///
	/// # Arguments
	///
	/// * `radians`: angle to rotate the x axis around (in radians)
	#[rustfmt::skip]
	#[inline]
	pub fn new_rotation_x(radians: f32) -> Matrix4x4 {
		let (s, c) = radians.sin_cos();
		Matrix4x4::new(
			1.0, 0.0, 0.0, 0.0,
			0.0, c, -s, 0.0,
			0.0, s, c, 0.0,
			0.0, 0.0, 0.0, 1.0
		)
	}

	/// Creates a new rotation matrix for rotation around the y axis.
	///
	/// # Arguments
	///
	/// * `radians`: angle to rotate the y axis around (in radians)
	#[rustfmt::skip]
	#[inline]
	pub fn new_rotation_y(radians: f32) -> Matrix4x4 {
		let (s, c) = radians.sin_cos();
		Matrix4x4::new(
			c, 0.0, s, 0.0,
			0.0, 1.0, 0.0, 0.0,
			-s, 0.0, c, 0.0,
			0.0, 0.0, 0.0, 1.0
		)
	}

	/// Creates a new rotation matrix for rotation around the z axis.
	///
	/// # Arguments
	///
	/// * `radians`: angle to rotate the z axis around (in radians)
	#[rustfmt::skip]
	#[inline]
	pub fn new_rotation_z(radians: f32) -> Matrix4x4 {
		let (s, c) = radians.sin_cos();
		Matrix4x4::new(
			c, -s, 0.0, 0.0,
			s, c, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0,
		)
	}

	/// Creates a translation matrix.
	///
	/// # Arguments
	///
	/// * `x`: the amount to translate on the x axis
	/// * `y`: the amount to translate on the y axis
	/// * `z`: the amount to translate on the z axis
	#[rustfmt::skip]
	#[inline]
	pub fn new_translation(x: f32, y: f32, z: f32) -> Matrix4x4 {
		Matrix4x4::new(
			1.0, 0.0, 0.0, x,
			0.0, 1.0, 0.0, y,
			0.0, 0.0, 1.0, z,
			0.0, 0.0, 0.0, 1.0,
		)
	}

	/// Creates a scaling matrix from scaling factors for each axis.
	///
	/// # Arguments
	///
	/// * `x`: the scale factor for the x axis
	/// * `y`: the scale factor for the y axis
	/// * `z`: the scale factor for the z axis
	#[rustfmt::skip]
	#[inline]
	pub fn new_scaling(x: f32, y: f32, z: f32) -> Matrix4x4 {
		Matrix4x4::new(
			x, 0.0, 0.0, 0.0,
			0.0, y, 0.0, 0.0,
			0.0, 0.0, z, 0.0,
			0.0, 0.0, 0.0, 1.0,
		)
	}

	/// Calculates the determinant of this matrix.
	#[rustfmt::skip]
	#[inline]
	pub fn determinant(&self) -> f32 {
		(self.m[Matrix4x4::M11] * self.m[Matrix4x4::M22] - self.m[Matrix4x4::M21] * self.m[Matrix4x4::M12]) *
			(self.m[Matrix4x4::M33] * self.m[Matrix4x4::M44] - self.m[Matrix4x4::M43] * self.m[Matrix4x4::M34]) -
			(self.m[Matrix4x4::M11] * self.m[Matrix4x4::M32] - self.m[Matrix4x4::M31] * self.m[Matrix4x4::M12]) *
			(self.m[Matrix4x4::M23] * self.m[Matrix4x4::M44] - self.m[Matrix4x4::M43] * self.m[Matrix4x4::M24]) +
			(self.m[Matrix4x4::M11] * self.m[Matrix4x4::M42] - self.m[Matrix4x4::M41] * self.m[Matrix4x4::M12]) *
			(self.m[Matrix4x4::M23] * self.m[Matrix4x4::M34] - self.m[Matrix4x4::M33] * self.m[Matrix4x4::M24]) +
			(self.m[Matrix4x4::M21] * self.m[Matrix4x4::M32] - self.m[Matrix4x4::M31] * self.m[Matrix4x4::M22]) *
			(self.m[Matrix4x4::M13] * self.m[Matrix4x4::M44] - self.m[Matrix4x4::M43] * self.m[Matrix4x4::M14]) -
			(self.m[Matrix4x4::M21] * self.m[Matrix4x4::M42] - self.m[Matrix4x4::M41] * self.m[Matrix4x4::M22]) *
			(self.m[Matrix4x4::M13] * self.m[Matrix4x4::M34] - self.m[Matrix4x4::M33] * self.m[Matrix4x4::M14]) +
			(self.m[Matrix4x4::M31] * self.m[Matrix4x4::M42] - self.m[Matrix4x4::M41] * self.m[Matrix4x4::M32]) *
			(self.m[Matrix4x4::M13] * self.m[Matrix4x4::M24] - self.m[Matrix4x4::M23] * self.m[Matrix4x4::M14])
	}

	/// Calculates the inverse of this matrix.
	#[rustfmt::skip]
	pub fn invert(&self) -> Matrix4x4 {
		let d = self.determinant();
		if nearly_equal(d, 0.0, 0.000001) {
			Matrix4x4::IDENTITY
		} else {
			let d = 1.0 / d;
			Matrix4x4 {
				m: [
					d * (self.m[Matrix4x4::M22] * (self.m[Matrix4x4::M33] * self.m[Matrix4x4::M44] - self.m[Matrix4x4::M43] * self.m[Matrix4x4::M34]) + self.m[Matrix4x4::M32] * (self.m[Matrix4x4::M43] * self.m[Matrix4x4::M24] - self.m[Matrix4x4::M23] * self.m[Matrix4x4::M44]) + self.m[Matrix4x4::M42] * (self.m[Matrix4x4::M23] * self.m[Matrix4x4::M34] - self.m[Matrix4x4::M33] * self.m[Matrix4x4::M24])),
					d * (self.m[Matrix4x4::M23] * (self.m[Matrix4x4::M31] * self.m[Matrix4x4::M44] - self.m[Matrix4x4::M41] * self.m[Matrix4x4::M34]) + self.m[Matrix4x4::M33] * (self.m[Matrix4x4::M41] * self.m[Matrix4x4::M24] - self.m[Matrix4x4::M21] * self.m[Matrix4x4::M44]) + self.m[Matrix4x4::M43] * (self.m[Matrix4x4::M21] * self.m[Matrix4x4::M34] - self.m[Matrix4x4::M31] * self.m[Matrix4x4::M24])),
					d * (self.m[Matrix4x4::M24] * (self.m[Matrix4x4::M31] * self.m[Matrix4x4::M42] - self.m[Matrix4x4::M41] * self.m[Matrix4x4::M32]) + self.m[Matrix4x4::M34] * (self.m[Matrix4x4::M41] * self.m[Matrix4x4::M22] - self.m[Matrix4x4::M21] * self.m[Matrix4x4::M42]) + self.m[Matrix4x4::M44] * (self.m[Matrix4x4::M21] * self.m[Matrix4x4::M32] - self.m[Matrix4x4::M31] * self.m[Matrix4x4::M22])),
					d * (self.m[Matrix4x4::M21] * (self.m[Matrix4x4::M42] * self.m[Matrix4x4::M33] - self.m[Matrix4x4::M32] * self.m[Matrix4x4::M43]) + self.m[Matrix4x4::M31] * (self.m[Matrix4x4::M22] * self.m[Matrix4x4::M43] - self.m[Matrix4x4::M42] * self.m[Matrix4x4::M23]) + self.m[Matrix4x4::M41] * (self.m[Matrix4x4::M32] * self.m[Matrix4x4::M23] - self.m[Matrix4x4::M22] * self.m[Matrix4x4::M33])),
					d * (self.m[Matrix4x4::M32] * (self.m[Matrix4x4::M13] * self.m[Matrix4x4::M44] - self.m[Matrix4x4::M43] * self.m[Matrix4x4::M14]) + self.m[Matrix4x4::M42] * (self.m[Matrix4x4::M33] * self.m[Matrix4x4::M14] - self.m[Matrix4x4::M13] * self.m[Matrix4x4::M34]) + self.m[Matrix4x4::M12] * (self.m[Matrix4x4::M43] * self.m[Matrix4x4::M34] - self.m[Matrix4x4::M33] * self.m[Matrix4x4::M44])),
					d * (self.m[Matrix4x4::M33] * (self.m[Matrix4x4::M11] * self.m[Matrix4x4::M44] - self.m[Matrix4x4::M41] * self.m[Matrix4x4::M14]) + self.m[Matrix4x4::M43] * (self.m[Matrix4x4::M31] * self.m[Matrix4x4::M14] - self.m[Matrix4x4::M11] * self.m[Matrix4x4::M34]) + self.m[Matrix4x4::M13] * (self.m[Matrix4x4::M41] * self.m[Matrix4x4::M34] - self.m[Matrix4x4::M31] * self.m[Matrix4x4::M44])),
					d * (self.m[Matrix4x4::M34] * (self.m[Matrix4x4::M11] * self.m[Matrix4x4::M42] - self.m[Matrix4x4::M41] * self.m[Matrix4x4::M12]) + self.m[Matrix4x4::M44] * (self.m[Matrix4x4::M31] * self.m[Matrix4x4::M12] - self.m[Matrix4x4::M11] * self.m[Matrix4x4::M32]) + self.m[Matrix4x4::M14] * (self.m[Matrix4x4::M41] * self.m[Matrix4x4::M32] - self.m[Matrix4x4::M31] * self.m[Matrix4x4::M42])),
					d * (self.m[Matrix4x4::M31] * (self.m[Matrix4x4::M42] * self.m[Matrix4x4::M13] - self.m[Matrix4x4::M12] * self.m[Matrix4x4::M43]) + self.m[Matrix4x4::M41] * (self.m[Matrix4x4::M12] * self.m[Matrix4x4::M33] - self.m[Matrix4x4::M32] * self.m[Matrix4x4::M13]) + self.m[Matrix4x4::M11] * (self.m[Matrix4x4::M32] * self.m[Matrix4x4::M43] - self.m[Matrix4x4::M42] * self.m[Matrix4x4::M33])),
					d * (self.m[Matrix4x4::M42] * (self.m[Matrix4x4::M13] * self.m[Matrix4x4::M24] - self.m[Matrix4x4::M23] * self.m[Matrix4x4::M14]) + self.m[Matrix4x4::M12] * (self.m[Matrix4x4::M23] * self.m[Matrix4x4::M44] - self.m[Matrix4x4::M43] * self.m[Matrix4x4::M24]) + self.m[Matrix4x4::M22] * (self.m[Matrix4x4::M43] * self.m[Matrix4x4::M14] - self.m[Matrix4x4::M13] * self.m[Matrix4x4::M44])),
					d * (self.m[Matrix4x4::M43] * (self.m[Matrix4x4::M11] * self.m[Matrix4x4::M24] - self.m[Matrix4x4::M21] * self.m[Matrix4x4::M14]) + self.m[Matrix4x4::M13] * (self.m[Matrix4x4::M21] * self.m[Matrix4x4::M44] - self.m[Matrix4x4::M41] * self.m[Matrix4x4::M24]) + self.m[Matrix4x4::M23] * (self.m[Matrix4x4::M41] * self.m[Matrix4x4::M14] - self.m[Matrix4x4::M11] * self.m[Matrix4x4::M44])),
					d * (self.m[Matrix4x4::M44] * (self.m[Matrix4x4::M11] * self.m[Matrix4x4::M22] - self.m[Matrix4x4::M21] * self.m[Matrix4x4::M12]) + self.m[Matrix4x4::M14] * (self.m[Matrix4x4::M21] * self.m[Matrix4x4::M42] - self.m[Matrix4x4::M41] * self.m[Matrix4x4::M22]) + self.m[Matrix4x4::M24] * (self.m[Matrix4x4::M41] * self.m[Matrix4x4::M12] - self.m[Matrix4x4::M11] * self.m[Matrix4x4::M42])),
					d * (self.m[Matrix4x4::M41] * (self.m[Matrix4x4::M22] * self.m[Matrix4x4::M13] - self.m[Matrix4x4::M12] * self.m[Matrix4x4::M23]) + self.m[Matrix4x4::M11] * (self.m[Matrix4x4::M42] * self.m[Matrix4x4::M23] - self.m[Matrix4x4::M22] * self.m[Matrix4x4::M43]) + self.m[Matrix4x4::M21] * (self.m[Matrix4x4::M12] * self.m[Matrix4x4::M43] - self.m[Matrix4x4::M42] * self.m[Matrix4x4::M13])),
					d * (self.m[Matrix4x4::M12] * (self.m[Matrix4x4::M33] * self.m[Matrix4x4::M24] - self.m[Matrix4x4::M23] * self.m[Matrix4x4::M34]) + self.m[Matrix4x4::M22] * (self.m[Matrix4x4::M13] * self.m[Matrix4x4::M34] - self.m[Matrix4x4::M33] * self.m[Matrix4x4::M14]) + self.m[Matrix4x4::M32] * (self.m[Matrix4x4::M23] * self.m[Matrix4x4::M14] - self.m[Matrix4x4::M13] * self.m[Matrix4x4::M24])),
					d * (self.m[Matrix4x4::M13] * (self.m[Matrix4x4::M31] * self.m[Matrix4x4::M24] - self.m[Matrix4x4::M21] * self.m[Matrix4x4::M34]) + self.m[Matrix4x4::M23] * (self.m[Matrix4x4::M11] * self.m[Matrix4x4::M34] - self.m[Matrix4x4::M31] * self.m[Matrix4x4::M14]) + self.m[Matrix4x4::M33] * (self.m[Matrix4x4::M21] * self.m[Matrix4x4::M14] - self.m[Matrix4x4::M11] * self.m[Matrix4x4::M24])),
					d * (self.m[Matrix4x4::M14] * (self.m[Matrix4x4::M31] * self.m[Matrix4x4::M22] - self.m[Matrix4x4::M21] * self.m[Matrix4x4::M32]) + self.m[Matrix4x4::M24] * (self.m[Matrix4x4::M11] * self.m[Matrix4x4::M32] - self.m[Matrix4x4::M31] * self.m[Matrix4x4::M12]) + self.m[Matrix4x4::M34] * (self.m[Matrix4x4::M21] * self.m[Matrix4x4::M12] - self.m[Matrix4x4::M11] * self.m[Matrix4x4::M22])),
					d * (self.m[Matrix4x4::M11] * (self.m[Matrix4x4::M22] * self.m[Matrix4x4::M33] - self.m[Matrix4x4::M32] * self.m[Matrix4x4::M23]) + self.m[Matrix4x4::M21] * (self.m[Matrix4x4::M32] * self.m[Matrix4x4::M13] - self.m[Matrix4x4::M12] * self.m[Matrix4x4::M33]) + self.m[Matrix4x4::M31] * (self.m[Matrix4x4::M12] * self.m[Matrix4x4::M23] - self.m[Matrix4x4::M22] * self.m[Matrix4x4::M13])),
				]
			}
		}
	}

	/// Calculates the transpose of this matrix.
	#[rustfmt::skip]
	pub fn transpose(&self) -> Matrix4x4 {
		Matrix4x4::new(
			self.m[Matrix4x4::M11], self.m[Matrix4x4::M21], self.m[Matrix4x4::M31], self.m[Matrix4x4::M41],
			self.m[Matrix4x4::M12], self.m[Matrix4x4::M22], self.m[Matrix4x4::M32], self.m[Matrix4x4::M42],
			self.m[Matrix4x4::M13], self.m[Matrix4x4::M23], self.m[Matrix4x4::M33], self.m[Matrix4x4::M43],
			self.m[Matrix4x4::M14], self.m[Matrix4x4::M24], self.m[Matrix4x4::M34], self.m[Matrix4x4::M44],
		)
	}

	/// Sets all of the elements of this matrix.
	#[allow(clippy::too_many_arguments)]
	#[rustfmt::skip]
	#[inline]
	pub fn set(
		&mut self,
		m11: f32, m12: f32, m13: f32, m14: f32,
		m21: f32, m22: f32, m23: f32, m24: f32,
		m31: f32, m32: f32, m33: f32, m34: f32,
		m41: f32, m42: f32, m43: f32, m44: f32,
	) {
		self.m = [
			m11, m21, m31, m41,
			m12, m22, m32, m42,
			m13, m23, m33, m43,
			m14, m24, m34, m44
		];
	}

	/// Constructs a camera modelview matrix. This is equivalent to the OpenGL function `gluLookAt`.
	///
	/// # Arguments
	///
	/// * `position`: The position of the camera (eye).
	/// * `target`: The direction that the camera is pointing (center).
	/// * `up`: The up direction, relative to the camera.
	#[rustfmt::skip]
	pub fn new_look_at(position: &Vector3, target: &Vector3, up: &Vector3) -> Matrix4x4 {
		let forward = (*target - *position).normalize();
		let left = forward.cross(up).normalize();
		let up = left.cross(&forward);

		let out = Matrix4x4::new(
			left.x,     left.y,     left.z,     0.0,
			up.x,       up.y,       up.z,       0.0,
			-forward.x, -forward.y, -forward.z, 0.0,
			0.0,        0.0,        0.0,        1.0,
		);

		out * Matrix4x4::new_translation(position.x, position.y, position.z)
	}

	/// Constructs an orthogonal projection matrix. This is equivalent to the OpenGL function
	/// `glOrtho`.
	///
	/// # Arguments
	///
	/// * `left`: Coordinate of the left vertical clipping plane.
	/// * `right`: Coordinate of the right vertical clipping plane.
	/// * `bottom`: Coordinate of the bottom horizontal clipping plane.
	/// * `top`: Coordinate of the top horizontal clipping plane.
	/// * `near`: Near clipping plane distance.
	/// * `far`: Far clipping plane distance.
	#[rustfmt::skip]
	pub fn new_orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Matrix4x4 {
		Matrix4x4::new(
			2.0 / (right - left), 0.0, 0.0, -((right + left) / (right - left)),
			0.0, 2.0 / (top - bottom), 0.0, -((top + bottom) / (top - bottom)),
			0.0, 0.0, -2.0 / (far - near), -((far + near) / (far - near)),
			0.0, 0.0, 0.0, 1.0,
		)
	}

	/// Constructs a perspective projection matrix. This is equivalent to a matrix created by the
	/// OpenGL function `glFrustum`.
	///
	/// # Arguments
	///
	/// * `left`: Coordinate of the left vertical clipping plane.
	/// * `right`: Coordinate of the right vertical clipping plane.
	/// * `bottom`: Coordinate of the bottom horizontal clipping plane.
	/// * `top`: Coordinate of the top horizontal clipping plane.
	/// * `near`: Near clipping plane distance.
	/// * `far`: Far clipping plane distance.
	#[rustfmt::skip]
	pub fn new_perspective(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Matrix4x4 {
		Matrix4x4::new(
			(2.0 * near) / (right - left), 0.0, (right + left) / (right - left), 0.0,
			0.0, (2.0 * near) / (top - bottom), (top + bottom) / (top - bottom), 0.0,
			0.0, 0.0, -((far + near) / (far - near)), -((2.0 * far * near) / (far - near)),
			0.0, 0.0, -1.0, 0.0,
		)
	}

	/// Constructs a perspective projection matrix. This is equivalent to a matrix created by the
	/// OpenGL function `gluPerspective`.
	///
	/// # Arguments
	///
	/// * `fov`: Angle specifying the field of view (in radians).
	/// * `aspect_ratio`: The aspect ratio of the viewport.
	/// * `near`: Near clipping plane distance.
	/// * `far`: Far clipping plane distance.
	#[rustfmt::skip]
	pub fn new_perspective_fov(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Matrix4x4 {
		let f = 1.0 / (fov / 2.0).tan();

		Matrix4x4::new(
			f / aspect_ratio, 0.0, 0.0, 0.0,
			0.0, f, 0.0, 0.0,
			0.0, 0.0, (far + near) / (near - far), (2.0 * far * near) / (near - far),
			0.0, 0.0, -1.0, 0.0,
		)
	}
}

#[rustfmt::skip]
#[inline]
fn matrix4x4_mul(left: Matrix4x4, right: Matrix4x4) -> [f32; 16] {
	[
		left.m[Matrix4x4::M11] * right.m[Matrix4x4::M11] + left.m[Matrix4x4::M12] * right.m[Matrix4x4::M21] + left.m[Matrix4x4::M13] * right.m[Matrix4x4::M31] + left.m[Matrix4x4::M14] * right.m[Matrix4x4::M41],
		left.m[Matrix4x4::M21] * right.m[Matrix4x4::M11] + left.m[Matrix4x4::M22] * right.m[Matrix4x4::M21] + left.m[Matrix4x4::M23] * right.m[Matrix4x4::M31] + left.m[Matrix4x4::M24] * right.m[Matrix4x4::M41],
		left.m[Matrix4x4::M31] * right.m[Matrix4x4::M11] + left.m[Matrix4x4::M32] * right.m[Matrix4x4::M21] + left.m[Matrix4x4::M33] * right.m[Matrix4x4::M31] + left.m[Matrix4x4::M34] * right.m[Matrix4x4::M41],
		left.m[Matrix4x4::M41] * right.m[Matrix4x4::M11] + left.m[Matrix4x4::M42] * right.m[Matrix4x4::M21] + left.m[Matrix4x4::M43] * right.m[Matrix4x4::M31] + left.m[Matrix4x4::M44] * right.m[Matrix4x4::M41],
		left.m[Matrix4x4::M11] * right.m[Matrix4x4::M12] + left.m[Matrix4x4::M12] * right.m[Matrix4x4::M22] + left.m[Matrix4x4::M13] * right.m[Matrix4x4::M32] + left.m[Matrix4x4::M14] * right.m[Matrix4x4::M42],
		left.m[Matrix4x4::M21] * right.m[Matrix4x4::M12] + left.m[Matrix4x4::M22] * right.m[Matrix4x4::M22] + left.m[Matrix4x4::M23] * right.m[Matrix4x4::M32] + left.m[Matrix4x4::M24] * right.m[Matrix4x4::M42],
		left.m[Matrix4x4::M31] * right.m[Matrix4x4::M12] + left.m[Matrix4x4::M32] * right.m[Matrix4x4::M22] + left.m[Matrix4x4::M33] * right.m[Matrix4x4::M32] + left.m[Matrix4x4::M34] * right.m[Matrix4x4::M42],
		left.m[Matrix4x4::M41] * right.m[Matrix4x4::M12] + left.m[Matrix4x4::M42] * right.m[Matrix4x4::M22] + left.m[Matrix4x4::M43] * right.m[Matrix4x4::M32] + left.m[Matrix4x4::M44] * right.m[Matrix4x4::M42],
		left.m[Matrix4x4::M11] * right.m[Matrix4x4::M13] + left.m[Matrix4x4::M12] * right.m[Matrix4x4::M23] + left.m[Matrix4x4::M13] * right.m[Matrix4x4::M33] + left.m[Matrix4x4::M14] * right.m[Matrix4x4::M43],
		left.m[Matrix4x4::M21] * right.m[Matrix4x4::M13] + left.m[Matrix4x4::M22] * right.m[Matrix4x4::M23] + left.m[Matrix4x4::M23] * right.m[Matrix4x4::M33] + left.m[Matrix4x4::M24] * right.m[Matrix4x4::M43],
		left.m[Matrix4x4::M31] * right.m[Matrix4x4::M13] + left.m[Matrix4x4::M32] * right.m[Matrix4x4::M23] + left.m[Matrix4x4::M33] * right.m[Matrix4x4::M33] + left.m[Matrix4x4::M34] * right.m[Matrix4x4::M43],
		left.m[Matrix4x4::M41] * right.m[Matrix4x4::M13] + left.m[Matrix4x4::M42] * right.m[Matrix4x4::M23] + left.m[Matrix4x4::M43] * right.m[Matrix4x4::M33] + left.m[Matrix4x4::M44] * right.m[Matrix4x4::M43],
		left.m[Matrix4x4::M11] * right.m[Matrix4x4::M14] + left.m[Matrix4x4::M12] * right.m[Matrix4x4::M24] + left.m[Matrix4x4::M13] * right.m[Matrix4x4::M34] + left.m[Matrix4x4::M14] * right.m[Matrix4x4::M44],
		left.m[Matrix4x4::M21] * right.m[Matrix4x4::M14] + left.m[Matrix4x4::M22] * right.m[Matrix4x4::M24] + left.m[Matrix4x4::M23] * right.m[Matrix4x4::M34] + left.m[Matrix4x4::M24] * right.m[Matrix4x4::M44],
		left.m[Matrix4x4::M31] * right.m[Matrix4x4::M14] + left.m[Matrix4x4::M32] * right.m[Matrix4x4::M24] + left.m[Matrix4x4::M33] * right.m[Matrix4x4::M34] + left.m[Matrix4x4::M34] * right.m[Matrix4x4::M44],
		left.m[Matrix4x4::M41] * right.m[Matrix4x4::M14] + left.m[Matrix4x4::M42] * right.m[Matrix4x4::M24] + left.m[Matrix4x4::M43] * right.m[Matrix4x4::M34] + left.m[Matrix4x4::M44] * right.m[Matrix4x4::M44],
	]
}

impl Mul for Matrix4x4 {
	type Output = Self;

	#[inline]
	fn mul(self, rhs: Self) -> Self::Output {
		Matrix4x4 { m: matrix4x4_mul(self, rhs) }
	}
}

impl MulAssign for Matrix4x4 {
	#[inline]
	fn mul_assign(&mut self, rhs: Self) {
		self.m = matrix4x4_mul(*self, rhs);
	}
}

impl Mul<Vector3> for Matrix4x4 {
	type Output = Vector3;

	#[rustfmt::skip]
	#[inline]
	fn mul(self, rhs: Vector3) -> Self::Output {
		Vector3 {
			x: rhs.x * self.m[Matrix4x4::M11] + rhs.y * self.m[Matrix4x4::M12] + rhs.z * self.m[Matrix4x4::M13] + self.m[Matrix4x4::M14],
			y: rhs.x * self.m[Matrix4x4::M21] + rhs.y * self.m[Matrix4x4::M22] + rhs.z * self.m[Matrix4x4::M23] + self.m[Matrix4x4::M24],
			z: rhs.x * self.m[Matrix4x4::M31] + rhs.y * self.m[Matrix4x4::M32] + rhs.z * self.m[Matrix4x4::M33] + self.m[Matrix4x4::M34],
		}
	}
}

impl Mul<Vector4> for Matrix4x4 {
	type Output = Vector4;

	#[rustfmt::skip]
	#[inline]
	fn mul(self, rhs: Vector4) -> Self::Output {
		Vector4 {
			x: rhs.x * self.m[Matrix4x4::M11] + rhs.y * self.m[Matrix4x4::M12] + rhs.z * self.m[Matrix4x4::M13] + rhs.w * self.m[Matrix4x4::M14],
			y: rhs.x * self.m[Matrix4x4::M21] + rhs.y * self.m[Matrix4x4::M22] + rhs.z * self.m[Matrix4x4::M23] + rhs.w * self.m[Matrix4x4::M24],
			z: rhs.x * self.m[Matrix4x4::M31] + rhs.y * self.m[Matrix4x4::M32] + rhs.z * self.m[Matrix4x4::M33] + rhs.w * self.m[Matrix4x4::M34],
			w: rhs.x * self.m[Matrix4x4::M41] + rhs.y * self.m[Matrix4x4::M42] + rhs.z * self.m[Matrix4x4::M43] + rhs.w * self.m[Matrix4x4::M44],
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::math::*;

	use super::*;

	#[test]
	pub fn test_new() {
		let m = Matrix4x4::new(
			1.1, 1.2, 1.3, 1.4, //
			2.1, 2.2, 2.3, 2.4, //
			3.1, 3.2, 3.3, 3.4, //
			4.1, 4.2, 4.3, 4.4,
		);
		assert_eq!(1.1, m.m[Matrix4x4::M11]);
		assert_eq!(1.2, m.m[Matrix4x4::M12]);
		assert_eq!(1.3, m.m[Matrix4x4::M13]);
		assert_eq!(1.4, m.m[Matrix4x4::M14]);
		assert_eq!(2.1, m.m[Matrix4x4::M21]);
		assert_eq!(2.2, m.m[Matrix4x4::M22]);
		assert_eq!(2.3, m.m[Matrix4x4::M23]);
		assert_eq!(2.4, m.m[Matrix4x4::M24]);
		assert_eq!(3.1, m.m[Matrix4x4::M31]);
		assert_eq!(3.2, m.m[Matrix4x4::M32]);
		assert_eq!(3.3, m.m[Matrix4x4::M33]);
		assert_eq!(3.4, m.m[Matrix4x4::M34]);
		assert_eq!(4.1, m.m[Matrix4x4::M41]);
		assert_eq!(4.2, m.m[Matrix4x4::M42]);
		assert_eq!(4.3, m.m[Matrix4x4::M43]);
		assert_eq!(4.4, m.m[Matrix4x4::M44]);
	}

	#[test]
	pub fn test_set() {
		let mut m = Matrix4x4 { m: [0.0; 16] };
		m.set(
			1.1, 1.2, 1.3, 1.4, //
			2.1, 2.2, 2.3, 2.4, //
			3.1, 3.2, 3.3, 3.4, //
			4.1, 4.2, 4.3, 4.4,
		);
		assert_eq!(1.1, m.m[Matrix4x4::M11]);
		assert_eq!(1.2, m.m[Matrix4x4::M12]);
		assert_eq!(1.3, m.m[Matrix4x4::M13]);
		assert_eq!(1.4, m.m[Matrix4x4::M14]);
		assert_eq!(2.1, m.m[Matrix4x4::M21]);
		assert_eq!(2.2, m.m[Matrix4x4::M22]);
		assert_eq!(2.3, m.m[Matrix4x4::M23]);
		assert_eq!(2.4, m.m[Matrix4x4::M24]);
		assert_eq!(3.1, m.m[Matrix4x4::M31]);
		assert_eq!(3.2, m.m[Matrix4x4::M32]);
		assert_eq!(3.3, m.m[Matrix4x4::M33]);
		assert_eq!(3.4, m.m[Matrix4x4::M34]);
		assert_eq!(4.1, m.m[Matrix4x4::M41]);
		assert_eq!(4.2, m.m[Matrix4x4::M42]);
		assert_eq!(4.3, m.m[Matrix4x4::M43]);
		assert_eq!(4.4, m.m[Matrix4x4::M44]);
	}

	#[test]
	pub fn test_identity() {
		let m = Matrix4x4::identity();
		assert_eq!(1.0, m.m[Matrix4x4::M11]);
		assert_eq!(0.0, m.m[Matrix4x4::M12]);
		assert_eq!(0.0, m.m[Matrix4x4::M13]);
		assert_eq!(0.0, m.m[Matrix4x4::M14]);
		assert_eq!(0.0, m.m[Matrix4x4::M21]);
		assert_eq!(1.0, m.m[Matrix4x4::M22]);
		assert_eq!(0.0, m.m[Matrix4x4::M23]);
		assert_eq!(0.0, m.m[Matrix4x4::M24]);
		assert_eq!(0.0, m.m[Matrix4x4::M31]);
		assert_eq!(0.0, m.m[Matrix4x4::M32]);
		assert_eq!(1.0, m.m[Matrix4x4::M33]);
		assert_eq!(0.0, m.m[Matrix4x4::M34]);
		assert_eq!(0.0, m.m[Matrix4x4::M41]);
		assert_eq!(0.0, m.m[Matrix4x4::M42]);
		assert_eq!(0.0, m.m[Matrix4x4::M43]);
		assert_eq!(1.0, m.m[Matrix4x4::M44]);
	}

	#[rustfmt::skip]
	#[test]
	pub fn test_transpose() {
		let m = Matrix4x4::new(
			1.0,  2.0,  3.0,  4.0,
			5.0,  6.0,  7.0,  8.0,
			9.0,  10.0, 11.0, 12.0,
			13.0, 14.0, 15.0, 16.0,
		);
		let t = m.transpose();
		assert_eq!(1.0, t.m[Matrix4x4::M11]);
		assert_eq!(5.0, t.m[Matrix4x4::M12]);
		assert_eq!(9.0, t.m[Matrix4x4::M13]);
		assert_eq!(13.0, t.m[Matrix4x4::M14]);
		assert_eq!(2.0, t.m[Matrix4x4::M21]);
		assert_eq!(6.0, t.m[Matrix4x4::M22]);
		assert_eq!(10.0, t.m[Matrix4x4::M23]);
		assert_eq!(14.0, t.m[Matrix4x4::M24]);
		assert_eq!(3.0, t.m[Matrix4x4::M31]);
		assert_eq!(7.0, t.m[Matrix4x4::M32]);
		assert_eq!(11.0, t.m[Matrix4x4::M33]);
		assert_eq!(15.0, t.m[Matrix4x4::M34]);
		assert_eq!(4.0, t.m[Matrix4x4::M41]);
		assert_eq!(8.0, t.m[Matrix4x4::M42]);
		assert_eq!(12.0, t.m[Matrix4x4::M43]);
		assert_eq!(16.0, t.m[Matrix4x4::M44]);
	}

	#[rustfmt::skip]
	#[test]
	pub fn test_mul() {
		let a = Matrix4x4::new(
			5.0, 7.0, 9.0, 10.0, 
			2.0, 3.0, 3.0, 8.0, 
			8.0, 10.0, 2.0, 3.0, 
			3.0, 3.0, 4.0, 8.0
		);
		let b = Matrix4x4::new(
			3.0, 10.0, 12.0, 18.0,
			12.0, 1.0, 4.0, 9.0,
			9.0, 10.0, 12.0, 2.0,
			3.0, 12.0, 4.0, 10.0
		);
		
		let c = a * b;
		assert!(nearly_equal(210.0, c.m[Matrix4x4::M11], 0.001));
		assert!(nearly_equal(267.0, c.m[Matrix4x4::M12], 0.001));
		assert!(nearly_equal(236.0, c.m[Matrix4x4::M13], 0.001));
		assert!(nearly_equal(271.0, c.m[Matrix4x4::M14], 0.001));
		assert!(nearly_equal(93.0, c.m[Matrix4x4::M21], 0.001));
		assert!(nearly_equal(149.0, c.m[Matrix4x4::M22], 0.001));
		assert!(nearly_equal(104.0, c.m[Matrix4x4::M23], 0.001));
		assert!(nearly_equal(149.0, c.m[Matrix4x4::M24], 0.001));
		assert!(nearly_equal(171.0, c.m[Matrix4x4::M31], 0.001));
		assert!(nearly_equal(146.0, c.m[Matrix4x4::M32], 0.001));
		assert!(nearly_equal(172.0, c.m[Matrix4x4::M33], 0.001));
		assert!(nearly_equal(268.0, c.m[Matrix4x4::M34], 0.001));
		assert!(nearly_equal(105.0, c.m[Matrix4x4::M41], 0.001));
		assert!(nearly_equal(169.0, c.m[Matrix4x4::M42], 0.001));
		assert!(nearly_equal(128.0, c.m[Matrix4x4::M43], 0.001));
		assert!(nearly_equal(169.0, c.m[Matrix4x4::M44], 0.001));
		
		let c = b * a;
		assert!(nearly_equal(185.0, c.m[Matrix4x4::M11], 0.001));
		assert!(nearly_equal(225.0, c.m[Matrix4x4::M12], 0.001));
		assert!(nearly_equal(153.0, c.m[Matrix4x4::M13], 0.001));
		assert!(nearly_equal(290.0, c.m[Matrix4x4::M14], 0.001));
		assert!(nearly_equal(121.0, c.m[Matrix4x4::M21], 0.001));
		assert!(nearly_equal(154.0, c.m[Matrix4x4::M22], 0.001));
		assert!(nearly_equal(155.0, c.m[Matrix4x4::M23], 0.001));
		assert!(nearly_equal(212.0, c.m[Matrix4x4::M24], 0.001));
		assert!(nearly_equal(167.0, c.m[Matrix4x4::M31], 0.001));
		assert!(nearly_equal(219.0, c.m[Matrix4x4::M32], 0.001));
		assert!(nearly_equal(143.0, c.m[Matrix4x4::M33], 0.001));
		assert!(nearly_equal(222.0, c.m[Matrix4x4::M34], 0.001));
		assert!(nearly_equal(101.0, c.m[Matrix4x4::M41], 0.001));
		assert!(nearly_equal(127.0, c.m[Matrix4x4::M42], 0.001));
		assert!(nearly_equal(111.0, c.m[Matrix4x4::M43], 0.001));
		assert!(nearly_equal(218.0, c.m[Matrix4x4::M44], 0.001));
	}

	#[rustfmt::skip]
	#[test]
	pub fn test_mul_assign() {
		let mut a = Matrix4x4::new(
			5.0, 7.0, 9.0, 10.0,
			2.0, 3.0, 3.0, 8.0,
			8.0, 10.0, 2.0, 3.0,
			3.0, 3.0, 4.0, 8.0
		);
		let b = Matrix4x4::new(
			3.0, 10.0, 12.0, 18.0,
			12.0, 1.0, 4.0, 9.0,
			9.0, 10.0, 12.0, 2.0,
			3.0, 12.0, 4.0, 10.0
		);

		a *= b;
		assert!(nearly_equal(210.0, a.m[Matrix4x4::M11], 0.001));
		assert!(nearly_equal(267.0, a.m[Matrix4x4::M12], 0.001));
		assert!(nearly_equal(236.0, a.m[Matrix4x4::M13], 0.001));
		assert!(nearly_equal(271.0, a.m[Matrix4x4::M14], 0.001));
		assert!(nearly_equal(93.0, a.m[Matrix4x4::M21], 0.001));
		assert!(nearly_equal(149.0, a.m[Matrix4x4::M22], 0.001));
		assert!(nearly_equal(104.0, a.m[Matrix4x4::M23], 0.001));
		assert!(nearly_equal(149.0, a.m[Matrix4x4::M24], 0.001));
		assert!(nearly_equal(171.0, a.m[Matrix4x4::M31], 0.001));
		assert!(nearly_equal(146.0, a.m[Matrix4x4::M32], 0.001));
		assert!(nearly_equal(172.0, a.m[Matrix4x4::M33], 0.001));
		assert!(nearly_equal(268.0, a.m[Matrix4x4::M34], 0.001));
		assert!(nearly_equal(105.0, a.m[Matrix4x4::M41], 0.001));
		assert!(nearly_equal(169.0, a.m[Matrix4x4::M42], 0.001));
		assert!(nearly_equal(128.0, a.m[Matrix4x4::M43], 0.001));
		assert!(nearly_equal(169.0, a.m[Matrix4x4::M44], 0.001));

		let a = Matrix4x4::new(
			5.0, 7.0, 9.0, 10.0,
			2.0, 3.0, 3.0, 8.0,
			8.0, 10.0, 2.0, 3.0,
			3.0, 3.0, 4.0, 8.0
		);
		let mut b = Matrix4x4::new(
			3.0, 10.0, 12.0, 18.0,
			12.0, 1.0, 4.0, 9.0,
			9.0, 10.0, 12.0, 2.0,
			3.0, 12.0, 4.0, 10.0
		);
		
		b *= a;
		assert!(nearly_equal(185.0, b.m[Matrix4x4::M11], 0.001));
		assert!(nearly_equal(225.0, b.m[Matrix4x4::M12], 0.001));
		assert!(nearly_equal(153.0, b.m[Matrix4x4::M13], 0.001));
		assert!(nearly_equal(290.0, b.m[Matrix4x4::M14], 0.001));
		assert!(nearly_equal(121.0, b.m[Matrix4x4::M21], 0.001));
		assert!(nearly_equal(154.0, b.m[Matrix4x4::M22], 0.001));
		assert!(nearly_equal(155.0, b.m[Matrix4x4::M23], 0.001));
		assert!(nearly_equal(212.0, b.m[Matrix4x4::M24], 0.001));
		assert!(nearly_equal(167.0, b.m[Matrix4x4::M31], 0.001));
		assert!(nearly_equal(219.0, b.m[Matrix4x4::M32], 0.001));
		assert!(nearly_equal(143.0, b.m[Matrix4x4::M33], 0.001));
		assert!(nearly_equal(222.0, b.m[Matrix4x4::M34], 0.001));
		assert!(nearly_equal(101.0, b.m[Matrix4x4::M41], 0.001));
		assert!(nearly_equal(127.0, b.m[Matrix4x4::M42], 0.001));
		assert!(nearly_equal(111.0, b.m[Matrix4x4::M43], 0.001));
		assert!(nearly_equal(218.0, b.m[Matrix4x4::M44], 0.001));
	}

	#[rustfmt::skip]
	#[test]
	pub fn test_vector4_mul() {
		let v = Vector4::new(1.0, 2.0, 3.0, 4.0);
		let m = Matrix4x4::new(
			1.0, 2.0, 3.0, 4.0,
			5.0, 6.0, 7.0, 8.0,
			9.0, 10.0, 11.0, 12.0,
			13.0, 14.0, 15.0, 16.0,
		);
		let t = m * v;
		assert!(nearly_equal(t.x, 30.0, 0.0001));
		assert!(nearly_equal(t.y, 70.0, 0.0001));
		assert!(nearly_equal(t.z, 110.0, 0.0001));
		assert!(nearly_equal(t.w, 150.0, 0.0001));
	}

	#[test]
	pub fn test_translation() {
		let v = Vector3::new(10.2, 5.7, 1.8);
		let m = Matrix4x4::new_translation(2.0, 3.0, -1.7);
		let t = m * v;
		assert!(nearly_equal(12.2, t.x, 0.001));
		assert!(nearly_equal(8.7, t.y, 0.001));
		assert!(nearly_equal(0.1, t.z, 0.001));
	}

	#[test]
	pub fn test_scaling() {
		let v = Vector3::new(10.2, 5.7, 1.8);
		let m = Matrix4x4::new_scaling(3.0, 4.0, 0.5);
		let t = m * v;
		assert!(nearly_equal(30.6, t.x, 0.001));
		assert!(nearly_equal(22.8, t.y, 0.001));
		assert!(nearly_equal(0.9, t.z, 0.001));
	}

	#[test]
	pub fn test_rotation_x() {
		let v = Vector3::new(10.2, 5.7, 1.8);
		let m = Matrix4x4::new_rotation_x(45.0f32.to_radians());
		let t = m * v;
		assert!(nearly_equal(10.2, t.x, 0.001));
		assert!(nearly_equal(2.757, t.y, 0.001));
		assert!(nearly_equal(5.303, t.z, 0.001));
	}

	#[test]
	pub fn test_rotation_y() {
		let v = Vector3::new(10.2, 5.7, 1.8);
		let m = Matrix4x4::new_rotation_y(45.0f32.to_radians());
		let t = m * v;
		assert!(nearly_equal(8.485, t.x, 0.001));
		assert!(nearly_equal(5.7, t.y, 0.001));
		assert!(nearly_equal(-5.939, t.z, 0.001));
	}

	#[test]
	pub fn test_rotation_z() {
		let v = Vector3::new(10.2, 5.7, 1.8);
		let m = Matrix4x4::new_rotation_z(45.0f32.to_radians());
		let t = m * v;
		assert!(nearly_equal(3.181, t.x, 0.001));
		assert!(nearly_equal(11.243, t.y, 0.001));
		assert!(nearly_equal(1.8, t.z, 0.001));
	}

	#[rustfmt::skip]
	#[test]
	pub fn test_determinant() {
		let m = Matrix4x4::new(
			1.0, 2.0, 6.0, 6.0,
			4.0, 7.0, 3.0, 2.0,
			0.0, 0.0, 0.0, 0.0,
			1.0, 2.0, 2.0, 9.0
		);
		let d = m.determinant();
		assert!(nearly_equal(0.0, d, 0.000001));

		let m = Matrix4x4::new(
			5.0, 7.0, 9.0, 10.0,
			2.0, 3.0, 3.0, 8.0,
			8.0, 10.0, 2.0, 3.0,
			3.0, 3.0, 4.0, 8.0
		);
		let d = m.determinant();
		assert!(nearly_equal(-361.0, d, 0.001));
	}

	#[rustfmt::skip]
	#[test]
	pub fn test_invert() {
		let m = Matrix4x4::new(
			1.0, 2.0, 6.0, 6.0,
			4.0, 7.0, 3.0, 2.0,
			0.0, 0.0, 0.0, 0.0,
			1.0, 2.0, 2.0, 9.0
		);
		let inv = m.invert();
		assert!(nearly_equal(1.0, inv.m[Matrix4x4::M11], 0.000001));
		assert!(nearly_equal(0.0, inv.m[Matrix4x4::M12], 0.000001));
		assert!(nearly_equal(0.0, inv.m[Matrix4x4::M13], 0.000001));
		assert!(nearly_equal(0.0, inv.m[Matrix4x4::M14], 0.000001));
		assert!(nearly_equal(0.0, inv.m[Matrix4x4::M21], 0.000001));
		assert!(nearly_equal(1.0, inv.m[Matrix4x4::M22], 0.000001));
		assert!(nearly_equal(0.0, inv.m[Matrix4x4::M23], 0.000001));
		assert!(nearly_equal(0.0, inv.m[Matrix4x4::M24], 0.000001));
		assert!(nearly_equal(0.0, inv.m[Matrix4x4::M31], 0.000001));
		assert!(nearly_equal(0.0, inv.m[Matrix4x4::M32], 0.000001));
		assert!(nearly_equal(1.0, inv.m[Matrix4x4::M33], 0.000001));
		assert!(nearly_equal(0.0, inv.m[Matrix4x4::M34], 0.000001));
		assert!(nearly_equal(0.0, inv.m[Matrix4x4::M41], 0.000001));
		assert!(nearly_equal(0.0, inv.m[Matrix4x4::M42], 0.000001));
		assert!(nearly_equal(0.0, inv.m[Matrix4x4::M43], 0.000001));
		assert!(nearly_equal(1.0, inv.m[Matrix4x4::M44], 0.000001));

		let m = Matrix4x4::new(
			5.0, 7.0, 9.0, 10.0,
			2.0, 3.0, 3.0, 8.0,
			8.0, 10.0, 2.0, 3.0,
			3.0, 3.0, 4.0, 8.0
		);
		let inv = m.invert();
		assert!(nearly_equal(-0.196676, inv.m[Matrix4x4::M11], 0.000001));
		assert!(nearly_equal(-0.750693, inv.m[Matrix4x4::M12], 0.000001));
		assert!(nearly_equal( 0.072022, inv.m[Matrix4x4::M13], 0.000001));
		assert!(nearly_equal( 0.969529, inv.m[Matrix4x4::M14], 0.000001));
		assert!(nearly_equal( 0.141274, inv.m[Matrix4x4::M21], 0.000001));
		assert!(nearly_equal( 0.595568, inv.m[Matrix4x4::M22], 0.000001));
		assert!(nearly_equal( 0.060941, inv.m[Matrix4x4::M23], 0.000001));
		assert!(nearly_equal(-0.795014, inv.m[Matrix4x4::M24], 0.000001));
		assert!(nearly_equal( 0.196676, inv.m[Matrix4x4::M31], 0.000001));
		assert!(nearly_equal(-0.249307, inv.m[Matrix4x4::M32], 0.000001));
		assert!(nearly_equal(-0.072022, inv.m[Matrix4x4::M33], 0.000001));
		assert!(nearly_equal( 0.030471, inv.m[Matrix4x4::M34], 0.000001));
		assert!(nearly_equal(-0.077562, inv.m[Matrix4x4::M41], 0.000001));
		assert!(nearly_equal( 0.182825, inv.m[Matrix4x4::M42], 0.000001));
		assert!(nearly_equal(-0.013850, inv.m[Matrix4x4::M43], 0.000001));
		assert!(nearly_equal( 0.044321, inv.m[Matrix4x4::M44], 0.000001));
	}
}
