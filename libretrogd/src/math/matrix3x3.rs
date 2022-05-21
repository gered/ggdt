use std::ops::{Mul, MulAssign};

use crate::math::*;

/// Represents a 3x3 column-major matrix and provides common methods for matrix math.
#[derive(Debug, Copy, Clone)]
pub struct Matrix3x3 {
    pub m: [f32; 9],
}

impl Matrix3x3 {
    pub const M11: usize = 0;
    pub const M12: usize = 3;
    pub const M13: usize = 6;
    pub const M21: usize = 1;
    pub const M22: usize = 4;
    pub const M23: usize = 7;
    pub const M31: usize = 2;
    pub const M32: usize = 5;
    pub const M33: usize = 8;

    pub const IDENTITY: Matrix3x3 = Matrix3x3 {
        m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    };

    /// Returns a new identity matrix.
    #[inline]
    pub fn identity() -> Matrix3x3 {
        Matrix3x3::IDENTITY
    }

    /// Creates a new matrix with the specified elements.
    #[rustfmt::skip]
    #[inline]
    pub fn new(
        m11: f32, m12: f32, m13: f32,
        m21: f32, m22: f32, m23: f32,
        m31: f32, m32: f32, m33: f32,
    ) -> Matrix3x3 {
        Matrix3x3 {
            m: [
                m11, m21, m31,
                m12, m22, m32,
                m13, m23, m33
            ],
        }
    }

    /// Creates a new rotation matrix from a set of euler angles.
    ///
    /// # Arguments
    ///
    /// * `x`: the x angle (in radians)
    /// * `y`: the y angle (in radians)
    /// * `z`: the z angle (in radians)
    pub fn from_euler_angles(x: f32, y: f32, z: f32) -> Matrix3x3 {
        let rotate_z = Matrix3x3::new_rotation_z(z);
        let rotate_y = Matrix3x3::new_rotation_y(y);
        let rotate_x = Matrix3x3::new_rotation_x(x);

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
    pub fn new_rotation_x(radians: f32) -> Matrix3x3 {
        let (s, c) = radians.sin_cos();
        Matrix3x3::new(
            1.0, 0.0, 0.0,
            0.0, c, -s,
            0.0, s, c
        )
    }

    /// Creates a new rotation matrix for rotation around the y axis.
    ///
    /// # Arguments
    ///
    /// * `radians`: angle to rotate the y axis around (in radians)
    #[rustfmt::skip]
    #[inline]
    pub fn new_rotation_y(radians: f32) -> Matrix3x3 {
        let (s, c) = radians.sin_cos();
        Matrix3x3::new(
            c, 0.0, s,
            0.0, 1.0, 0.0,
            -s, 0.0, c
        )
    }

    /// Creates a new rotation matrix for rotation around the z axis.
    ///
    /// # Arguments
    ///
    /// * `radians`: angle to rotate the z axis around (in radians)
    #[rustfmt::skip]
    #[inline]
    pub fn new_rotation_z(radians: f32) -> Matrix3x3 {
        let (s, c) = radians.sin_cos();
        Matrix3x3::new(
            c, -s, 0.0,
            s, c, 0.0,
            0.0, 0.0, 1.0
        )
    }

    /// Creates a translation matrix. For use with 2D coordinates only.
    ///
    /// # Arguments
    ///
    /// * `x`: the amount to translate on the x axis
    /// * `y`: the amount to translate on the y axis
    #[rustfmt::skip]
    #[inline]
    pub fn new_2d_translation(x: f32, y: f32) -> Matrix3x3 {
        Matrix3x3::new(
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            x, y, 1.0
        )
    }

    /// Creates a scaling matrix from scaling factors for each axis. For use with 2D coordinates
    /// only.
    ///
    /// # Arguments
    ///
    /// * `x`: the scale factor for the x axis
    /// * `y`: the scale factor for the y axis
    #[rustfmt::skip]
    #[inline]
    pub fn new_2d_scaling(x: f32, y: f32) -> Matrix3x3 {
        Matrix3x3::new(
            x, 0.0, 0.0,
            0.0, y, 0.0,
            0.0, 0.0, 1.0
        )
    }

    /// Creates a new rotation matrix. For use with 2D coordinates only.
    ///
    /// # Arguments
    ///
    /// * `radians`: angle to rotate by (in radians)
    #[inline(always)]
    pub fn new_2d_rotation(radians: f32) -> Matrix3x3 {
        Matrix3x3::new_rotation_z(radians)
    }

    /// Calculates the determinant of this matrix.
    #[rustfmt::skip]
    #[inline]
    pub fn determinant(&self) -> f32 {
        self.m[Matrix3x3::M11] * self.m[Matrix3x3::M22] * self.m[Matrix3x3::M33] +
        self.m[Matrix3x3::M12] * self.m[Matrix3x3::M23] * self.m[Matrix3x3::M31] +
        self.m[Matrix3x3::M13] * self.m[Matrix3x3::M21] * self.m[Matrix3x3::M32] -
        self.m[Matrix3x3::M11] * self.m[Matrix3x3::M23] * self.m[Matrix3x3::M32] -
        self.m[Matrix3x3::M12] * self.m[Matrix3x3::M21] * self.m[Matrix3x3::M33] -
        self.m[Matrix3x3::M13] * self.m[Matrix3x3::M22] * self.m[Matrix3x3::M31]
    }

    /// Calculates the inverse of this matrix.
    #[rustfmt::skip]
    pub fn invert(&self) -> Matrix3x3 {
        let d = self.determinant();
        if nearly_equal(d, 0.0, 0.000001) {
            Matrix3x3::IDENTITY
        } else {
            let d = 1.0 / d;
            Matrix3x3 {
                m: [
                    d * (self.m[Matrix3x3::M22] * self.m[Matrix3x3::M33] - self.m[Matrix3x3::M32] * self.m[Matrix3x3::M23]),
                    d * (self.m[Matrix3x3::M31] * self.m[Matrix3x3::M23] - self.m[Matrix3x3::M21] * self.m[Matrix3x3::M33]),
                    d * (self.m[Matrix3x3::M21] * self.m[Matrix3x3::M32] - self.m[Matrix3x3::M31] * self.m[Matrix3x3::M22]),
                    d * (self.m[Matrix3x3::M32] * self.m[Matrix3x3::M13] - self.m[Matrix3x3::M12] * self.m[Matrix3x3::M33]),
                    d * (self.m[Matrix3x3::M11] * self.m[Matrix3x3::M33] - self.m[Matrix3x3::M31] * self.m[Matrix3x3::M13]),
                    d * (self.m[Matrix3x3::M31] * self.m[Matrix3x3::M12] - self.m[Matrix3x3::M11] * self.m[Matrix3x3::M32]),
                    d * (self.m[Matrix3x3::M12] * self.m[Matrix3x3::M23] - self.m[Matrix3x3::M22] * self.m[Matrix3x3::M13]),
                    d * (self.m[Matrix3x3::M21] * self.m[Matrix3x3::M13] - self.m[Matrix3x3::M11] * self.m[Matrix3x3::M23]),
                    d * (self.m[Matrix3x3::M11] * self.m[Matrix3x3::M22] - self.m[Matrix3x3::M21] * self.m[Matrix3x3::M12]),
                ]
            }
        }
    }

    /// Calculates the transpose of this matrix.
    #[inline]
    pub fn transpose(&self) -> Matrix3x3 {
        Matrix3x3::new(
            self.m[Matrix3x3::M11],
            self.m[Matrix3x3::M21],
            self.m[Matrix3x3::M31],
            self.m[Matrix3x3::M12],
            self.m[Matrix3x3::M22],
            self.m[Matrix3x3::M32],
            self.m[Matrix3x3::M13],
            self.m[Matrix3x3::M23],
            self.m[Matrix3x3::M33],
        )
    }

    /// Sets all of the elements of this matrix.
    #[inline]
    pub fn set(
        &mut self,
        m11: f32,
        m12: f32,
        m13: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m31: f32,
        m32: f32,
        m33: f32,
    ) {
        self.m[Matrix3x3::M11] = m11;
        self.m[Matrix3x3::M12] = m12;
        self.m[Matrix3x3::M13] = m13;
        self.m[Matrix3x3::M21] = m21;
        self.m[Matrix3x3::M22] = m22;
        self.m[Matrix3x3::M23] = m23;
        self.m[Matrix3x3::M31] = m31;
        self.m[Matrix3x3::M32] = m32;
        self.m[Matrix3x3::M33] = m33;
    }
}

impl Mul for Matrix3x3 {
    type Output = Self;

    #[rustfmt::skip]
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Matrix3x3::new(
            self.m[Matrix3x3::M11] * rhs.m[Matrix3x3::M11] + self.m[Matrix3x3::M12] * rhs.m[Matrix3x3::M21] + self.m[Matrix3x3::M13] * rhs.m[Matrix3x3::M31],
            self.m[Matrix3x3::M11] * rhs.m[Matrix3x3::M12] + self.m[Matrix3x3::M12] * rhs.m[Matrix3x3::M22] + self.m[Matrix3x3::M13] * rhs.m[Matrix3x3::M32],
            self.m[Matrix3x3::M11] * rhs.m[Matrix3x3::M13] + self.m[Matrix3x3::M12] * rhs.m[Matrix3x3::M23] + self.m[Matrix3x3::M13] * rhs.m[Matrix3x3::M33],
            self.m[Matrix3x3::M21] * rhs.m[Matrix3x3::M11] + self.m[Matrix3x3::M22] * rhs.m[Matrix3x3::M21] + self.m[Matrix3x3::M23] * rhs.m[Matrix3x3::M31],
            self.m[Matrix3x3::M21] * rhs.m[Matrix3x3::M12] + self.m[Matrix3x3::M22] * rhs.m[Matrix3x3::M22] + self.m[Matrix3x3::M23] * rhs.m[Matrix3x3::M32],
            self.m[Matrix3x3::M21] * rhs.m[Matrix3x3::M13] + self.m[Matrix3x3::M22] * rhs.m[Matrix3x3::M23] + self.m[Matrix3x3::M23] * rhs.m[Matrix3x3::M33],
            self.m[Matrix3x3::M31] * rhs.m[Matrix3x3::M11] + self.m[Matrix3x3::M32] * rhs.m[Matrix3x3::M21] + self.m[Matrix3x3::M33] * rhs.m[Matrix3x3::M31],
            self.m[Matrix3x3::M31] * rhs.m[Matrix3x3::M12] + self.m[Matrix3x3::M32] * rhs.m[Matrix3x3::M22] + self.m[Matrix3x3::M33] * rhs.m[Matrix3x3::M32],
            self.m[Matrix3x3::M31] * rhs.m[Matrix3x3::M13] + self.m[Matrix3x3::M32] * rhs.m[Matrix3x3::M23] + self.m[Matrix3x3::M33] * rhs.m[Matrix3x3::M33]
        )
    }
}

impl MulAssign for Matrix3x3 {
    #[rustfmt::skip]
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.set(
            self.m[Matrix3x3::M11] * rhs.m[Matrix3x3::M11] + self.m[Matrix3x3::M12] * rhs.m[Matrix3x3::M21] + self.m[Matrix3x3::M13] * rhs.m[Matrix3x3::M31],
            self.m[Matrix3x3::M11] * rhs.m[Matrix3x3::M12] + self.m[Matrix3x3::M12] * rhs.m[Matrix3x3::M22] + self.m[Matrix3x3::M13] * rhs.m[Matrix3x3::M32],
            self.m[Matrix3x3::M11] * rhs.m[Matrix3x3::M13] + self.m[Matrix3x3::M12] * rhs.m[Matrix3x3::M23] + self.m[Matrix3x3::M13] * rhs.m[Matrix3x3::M33],
            self.m[Matrix3x3::M21] * rhs.m[Matrix3x3::M11] + self.m[Matrix3x3::M22] * rhs.m[Matrix3x3::M21] + self.m[Matrix3x3::M23] * rhs.m[Matrix3x3::M31],
            self.m[Matrix3x3::M21] * rhs.m[Matrix3x3::M12] + self.m[Matrix3x3::M22] * rhs.m[Matrix3x3::M22] + self.m[Matrix3x3::M23] * rhs.m[Matrix3x3::M32],
            self.m[Matrix3x3::M21] * rhs.m[Matrix3x3::M13] + self.m[Matrix3x3::M22] * rhs.m[Matrix3x3::M23] + self.m[Matrix3x3::M23] * rhs.m[Matrix3x3::M33],
            self.m[Matrix3x3::M31] * rhs.m[Matrix3x3::M11] + self.m[Matrix3x3::M32] * rhs.m[Matrix3x3::M21] + self.m[Matrix3x3::M33] * rhs.m[Matrix3x3::M31],
            self.m[Matrix3x3::M31] * rhs.m[Matrix3x3::M12] + self.m[Matrix3x3::M32] * rhs.m[Matrix3x3::M22] + self.m[Matrix3x3::M33] * rhs.m[Matrix3x3::M32],
            self.m[Matrix3x3::M31] * rhs.m[Matrix3x3::M13] + self.m[Matrix3x3::M32] * rhs.m[Matrix3x3::M23] + self.m[Matrix3x3::M33] * rhs.m[Matrix3x3::M33]
        )
    }
}

impl Mul<Vector2> for Matrix3x3 {
    type Output = Vector2;

    #[rustfmt::skip]
    #[inline]
    fn mul(self, rhs: Vector2) -> Self::Output {
        Vector2 {
            x: rhs.x * self.m[Matrix3x3::M11] + rhs.y * self.m[Matrix3x3::M12] + self.m[Matrix3x3::M13] + self.m[Matrix3x3::M31],
            y: rhs.x * self.m[Matrix3x3::M21] + rhs.y * self.m[Matrix3x3::M22] + self.m[Matrix3x3::M23] + self.m[Matrix3x3::M32]
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_new() {
        let m = Matrix3x3::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        assert_eq!(1.0, m.m[Matrix3x3::M11]);
        assert_eq!(2.0, m.m[Matrix3x3::M12]);
        assert_eq!(3.0, m.m[Matrix3x3::M13]);
        assert_eq!(4.0, m.m[Matrix3x3::M21]);
        assert_eq!(5.0, m.m[Matrix3x3::M22]);
        assert_eq!(6.0, m.m[Matrix3x3::M23]);
        assert_eq!(7.0, m.m[Matrix3x3::M31]);
        assert_eq!(8.0, m.m[Matrix3x3::M32]);
        assert_eq!(9.0, m.m[Matrix3x3::M33]);
    }

    #[test]
    pub fn test_set() {
        let mut m = Matrix3x3 { m: [0.0; 9] };
        m.set(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        assert_eq!(1.0, m.m[Matrix3x3::M11]);
        assert_eq!(2.0, m.m[Matrix3x3::M12]);
        assert_eq!(3.0, m.m[Matrix3x3::M13]);
        assert_eq!(4.0, m.m[Matrix3x3::M21]);
        assert_eq!(5.0, m.m[Matrix3x3::M22]);
        assert_eq!(6.0, m.m[Matrix3x3::M23]);
        assert_eq!(7.0, m.m[Matrix3x3::M31]);
        assert_eq!(8.0, m.m[Matrix3x3::M32]);
        assert_eq!(9.0, m.m[Matrix3x3::M33]);
    }

    #[test]
    pub fn test_identity() {
        let m = Matrix3x3::identity();
        assert_eq!(1.0, m.m[Matrix3x3::M11]);
        assert_eq!(0.0, m.m[Matrix3x3::M12]);
        assert_eq!(0.0, m.m[Matrix3x3::M13]);
        assert_eq!(0.0, m.m[Matrix3x3::M21]);
        assert_eq!(1.0, m.m[Matrix3x3::M22]);
        assert_eq!(0.0, m.m[Matrix3x3::M23]);
        assert_eq!(0.0, m.m[Matrix3x3::M31]);
        assert_eq!(0.0, m.m[Matrix3x3::M32]);
        assert_eq!(1.0, m.m[Matrix3x3::M33]);
    }

    #[rustfmt::skip]
    #[test]
    pub fn test_transpose() {
        let m = Matrix3x3::new(
            1.0, 2.0, 3.0,
            4.0, 5.0, 6.0,
            7.0, 8.0, 9.0
        );
        let t = m.transpose();
        assert_eq!(1.0, t.m[Matrix3x3::M11]);
        assert_eq!(4.0, t.m[Matrix3x3::M12]);
        assert_eq!(7.0, t.m[Matrix3x3::M13]);
        assert_eq!(2.0, t.m[Matrix3x3::M21]);
        assert_eq!(5.0, t.m[Matrix3x3::M22]);
        assert_eq!(8.0, t.m[Matrix3x3::M23]);
        assert_eq!(3.0, t.m[Matrix3x3::M31]);
        assert_eq!(6.0, t.m[Matrix3x3::M32]);
        assert_eq!(9.0, t.m[Matrix3x3::M33]);
    }

    #[test]
    pub fn test_mul() {
        let a = Matrix3x3::new(12.0, 8.0, 4.0, 3.0, 17.0, 14.0, 9.0, 8.0, 10.0);
        let b = Matrix3x3::new(5.0, 19.0, 3.0, 6.0, 15.0, 9.0, 7.0, 8.0, 16.0);
        let c = a * b;
        assert!(nearly_equal(136.0, c.m[Matrix3x3::M11], 0.001));
        assert!(nearly_equal(380.0, c.m[Matrix3x3::M12], 0.001));
        assert!(nearly_equal(172.0, c.m[Matrix3x3::M13], 0.001));
        assert!(nearly_equal(215.0, c.m[Matrix3x3::M21], 0.001));
        assert!(nearly_equal(424.0, c.m[Matrix3x3::M22], 0.001));
        assert!(nearly_equal(386.0, c.m[Matrix3x3::M23], 0.001));
        assert!(nearly_equal(163.0, c.m[Matrix3x3::M31], 0.001));
        assert!(nearly_equal(371.0, c.m[Matrix3x3::M32], 0.001));
        assert!(nearly_equal(259.0, c.m[Matrix3x3::M33], 0.001));

        let mut a = Matrix3x3::new(12.0, 8.0, 4.0, 3.0, 17.0, 14.0, 9.0, 8.0, 10.0);
        let b = Matrix3x3::new(5.0, 19.0, 3.0, 6.0, 15.0, 9.0, 7.0, 8.0, 16.0);
        a *= b;
        assert!(nearly_equal(136.0, a.m[Matrix3x3::M11], 0.001));
        assert!(nearly_equal(380.0, a.m[Matrix3x3::M12], 0.001));
        assert!(nearly_equal(172.0, a.m[Matrix3x3::M13], 0.001));
        assert!(nearly_equal(215.0, a.m[Matrix3x3::M21], 0.001));
        assert!(nearly_equal(424.0, a.m[Matrix3x3::M22], 0.001));
        assert!(nearly_equal(386.0, a.m[Matrix3x3::M23], 0.001));
        assert!(nearly_equal(163.0, a.m[Matrix3x3::M31], 0.001));
        assert!(nearly_equal(371.0, a.m[Matrix3x3::M32], 0.001));
        assert!(nearly_equal(259.0, a.m[Matrix3x3::M33], 0.001));
    }

    #[test]
    pub fn test_2d_translation() {
        let v = Vector2::new(10.2, 5.7);
        let m = Matrix3x3::new_2d_translation(2.0, 3.0);
        let t = m * v;
        assert!(nearly_equal(12.2, t.x, 0.001));
        assert!(nearly_equal(8.7, t.y, 0.001));
    }

    #[test]
    pub fn test_2d_scaling() {
        let v = Vector2::new(10.2, 5.7);
        let m = Matrix3x3::new_2d_scaling(3.0, 4.0);
        let t = m * v;
        assert!(nearly_equal(30.6, t.x, 0.001));
        assert!(nearly_equal(22.8, t.y, 0.001));
    }

    #[test]
    pub fn test_2d_rotation() {
        let v = Vector2::new(0.0, 5.0);
        let m = Matrix3x3::new_2d_rotation(RADIANS_90);
        let t = m * v;
        assert!(nearly_equal(-5.0, t.x, 0.001));
        assert!(nearly_equal(0.0, t.y, 0.001));
    }

    #[test]
    pub fn test_2d_combined_transform() {
        let a = Matrix3x3::new_2d_translation(-2.0, 0.0);
        let b = Matrix3x3::new_2d_rotation(RADIANS_180);
        let m = a * b;
        let v = Vector2::new(0.0, 5.0);
        let t = m * v;
        assert!(nearly_equal(2.0, t.x, 0.001));
        assert!(nearly_equal(-5.0, t.y, 0.001));
    }
}
