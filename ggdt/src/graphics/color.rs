use std::ops::{Mul, MulAssign};
use std::simd;

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::utils::{ReadType, WriteType};

// these colours are taken from the default VGA palette

pub const COLOR_BLACK: RGBA = RGBA::from_rgb([0, 0, 0]);
pub const COLOR_BLUE: RGBA = RGBA::from_rgb([0, 0, 170]);
pub const COLOR_GREEN: RGBA = RGBA::from_rgb([0, 170, 0]);
pub const COLOR_CYAN: RGBA = RGBA::from_rgb([0, 170, 170]);
pub const COLOR_RED: RGBA = RGBA::from_rgb([170, 0, 0]);
pub const COLOR_MAGENTA: RGBA = RGBA::from_rgb([170, 0, 170]);
pub const COLOR_BROWN: RGBA = RGBA::from_rgb([170, 85, 0]);
pub const COLOR_LIGHT_GRAY: RGBA = RGBA::from_rgb([170, 170, 170]);
pub const COLOR_DARK_GRAY: RGBA = RGBA::from_rgb([85, 85, 85]);
pub const COLOR_BRIGHT_BLUE: RGBA = RGBA::from_rgb([85, 85, 255]);
pub const COLOR_BRIGHT_GREEN: RGBA = RGBA::from_rgb([85, 255, 85]);
pub const COLOR_BRIGHT_CYAN: RGBA = RGBA::from_rgb([85, 255, 255]);
pub const COLOR_BRIGHT_RED: RGBA = RGBA::from_rgb([255, 85, 85]);
pub const COLOR_BRIGHT_MAGENTA: RGBA = RGBA::from_rgb([255, 85, 255]);
pub const COLOR_BRIGHT_YELLOW: RGBA = RGBA::from_rgb([255, 255, 85]);
pub const COLOR_BRIGHT_WHITE: RGBA = RGBA::from_rgb([255, 255, 255]);

// TODO: probably should name these better, after i do much more reading on the subject :-)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlendFunction {
	Blend,
	BlendSourceWithAlpha(u8),
	TintedBlend(RGBA),
	MultipliedBlend(RGBA),
}

impl BlendFunction {
	#[inline]
	/// Blends the source and destination color together using the function associated with
	/// this enum value.
	///
	/// # Arguments
	///
	/// * `src`: the source color to blend
	/// * `dest`: the destination color to blend the source color over
	///
	/// returns: the blended color
	pub fn blend(&self, src: RGBA, dest: RGBA) -> RGBA {
		use BlendFunction::*;
		match self {
			Blend => src.blend(dest),
			BlendSourceWithAlpha(opacity) => src.blend_with_alpha(dest, *opacity),
			TintedBlend(tint) => src.tint(*tint).blend(dest),
			MultipliedBlend(color) => src.mul(*color).blend(dest),
		}
	}
}

///////////////////////////////////////////////////////////////////////////////

pub trait BytesAsColors<T> {
	/// Casts a slice of bytes to a slice of structured color values.
	///
	/// # Safety
	///
	/// The returned slice may not include all of the original slice's bytes if there is some remainder number of bytes
	/// that is too small to fit into the structured color type.
	unsafe fn as_colors(&self) -> &[T];

	/// Casts a mutable slice of bytes to a mutable slice of structured color values. Changes made to the returned
	/// slice will be reflected in the original slice's bytes.
	///
	/// # Safety
	///
	/// The returned slice may not include all of the original slice's bytes if there is some remainder number of bytes
	/// that is too small to fit into the structured color type.
	unsafe fn as_colors_mut(&mut self) -> &mut [T];
}

pub trait ColorsAsBytes {
	/// Casts a slice of structured color values to a slice of raw bytes that make up those same color values.
	fn as_bytes(&self) -> &[u8];

	/// Casts a mutable slice of structure color values to a mutable slice of raw bytes that make up those same color
	/// values. Changes made to the returned slice will be reflected in the original slice's color values.
	fn as_bytes_mut(&mut self) -> &mut [u8];
}

/// Unpacked 32-bit color represented as individual 8-bit color components where the components are in the
/// order red, green, blue, alpha.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
#[repr(transparent)]
pub struct RGBA(pub simd::u8x4);

impl RGBA {
	pub const SIZE: usize = std::mem::size_of::<Self>();

	/// Returns a color value composed of the provided RGBA color components.
	///
	/// # Arguments
	///
	/// * `rgba`: the 4 color components (0-255) in the order: red, green, blue, alpha
	///
	/// returns: the composed color value
	#[inline]
	pub const fn from_rgba(rgba: [u8; 4]) -> Self {
		RGBA(simd::u8x4::from_array(rgba))
	}

	/// Returns a color value composed of the provided RGB color components. Substitutes a value of 255 for the
	/// missing alpha component.
	///
	/// # Arguments
	///
	/// * `rgb`: the 3 color components (0-255) in the order: red, green, blue
	///
	/// returns: the composed color value
	#[inline]
	pub const fn from_rgb(rgb: [u8; 3]) -> Self {
		RGBA(simd::u8x4::from_array([rgb[0], rgb[1], rgb[2], 255]))
	}

	/// Returns the current red component value (0-255) of this color.
	#[inline]
	pub const fn r(&self) -> u8 {
		self.0.to_array()[0]
	}

	/// Returns the current green component value (0-255) of this color.
	#[inline]
	pub const fn g(&self) -> u8 {
		self.0.to_array()[1]
	}

	/// Returns the current blue component value (0-255) of this color.
	#[inline]
	pub const fn b(&self) -> u8 {
		self.0.to_array()[2]
	}

	/// Returns the current alpha component value (0-255) of this color.
	#[inline]
	pub const fn a(&self) -> u8 {
		self.0.to_array()[3]
	}

	/// Sets the red component value of this color leaving the other components in the color unchanged.
	///
	/// # Arguments
	///
	/// * `value`: the new red component value to be set (0-255)
	#[inline]
	pub fn set_r(&mut self, value: u8) {
		self.0[0] = value
	}

	/// Sets the green component value of this color leaving the other components in the color unchanged.
	///
	/// # Arguments
	///
	/// * `value`: the new green component value to be set (0-255)
	#[inline]
	pub fn set_g(&mut self, value: u8) {
		self.0[1] = value
	}

	/// Sets the blue component value of this color leaving the other components in the color unchanged.
	///
	/// # Arguments
	///
	/// * `value`: the new blue component value to be set (0-255)
	#[inline]
	pub fn set_b(&mut self, value: u8) {
		self.0[2] = value
	}

	/// Sets the alpha component value of this color leaving the other components in the color unchanged.
	///
	/// # Arguments
	///
	/// * `value`: the new alpha component value to be set (0-255)
	#[inline]
	pub fn set_a(&mut self, value: u8) {
		self.0[3] = value
	}

	/// Returns an array with all of this color's RGBA components, in the order: red, green, blue, alpha.
	#[inline]
	pub const fn to_array(&self) -> [u8; 4] {
		self.0.to_array()
	}

	#[inline]
	fn blend_components(strength: u8, src: Self, dest: Self) -> Self {
		let strength = simd::u16x4::splat(strength as u16);
		let max = simd::u16x4::splat(255);
		RGBA((((src.0.cast() * strength) + (dest.0.cast() * (max - strength))) / max).cast())
	}

	/// Alpha blends two colors together, using this color as the source color and the other provided color as the
	/// destination color.
	///
	/// # Arguments
	///
	/// * `dest`: the destination color that this color is being blended into
	///
	/// returns: the blended color result
	#[inline]
	pub fn blend(&self, dest: Self) -> Self {
		RGBA::blend_components(self.a(), *self, dest)
	}

	/// Alpha blends two colors together, where the alpha value used to blend the colors is derived from the given
	/// alpha value multiplied with the source color's alpha component. This allows for more flexibility versus the
	/// [`RGBA::blend`] method allowing direct control over how transparent the source color is when blended over
	/// top of the destination. The blend is performed using this color as the source color and the other provided
	/// color as the destination color.
	///
	/// # Arguments
	///
	/// * `dest`: the destination color that this color is being blended into. the alpha component of this color is
	///           ignored for the purposes of the blending operation.
	/// * `alpha`: the transparency or opacity of the source color over the destination color. this is multiplied
	///            together with the source color's (this color) alpha component to arrive at the final alpha value
	///            used for blending the two color's RGB components together.
	///
	/// returns: the blended color result
	#[inline]
	pub fn blend_with_alpha(&self, dest: Self, alpha: u8) -> Self {
		let alpha = ((alpha as u16 * self.a() as u16) / 255) as u8;
		let mut blended = RGBA::blend_components(alpha, *self, dest);
		blended.set_a(alpha);
		blended
	}

	/// Applies a tint to a color, using the tint color's alpha component as the strength of the tint, where 0 means
	/// no tint, and 255 means full tint. The original color's alpha component is preserved in the result.
	///
	/// # Arguments
	///
	/// * `tint`: the tint color to be applied to this color, where the alpha component represents the strength of
	///           the tint to be applied
	///
	/// returns: the tinted color result
	#[inline]
	pub fn tint(&self, mut tint: Self) -> Self {
		let strength = tint.a();
		tint.set_a(self.a());
		RGBA::blend_components(strength, tint, *self)
	}

	/// Linearly interpolates between this color and another color.
	///
	/// # Arguments
	///
	/// * `other`: the other color to interpolate between, used as the "high" or "end" color value
	/// * `t`: the amount to interpolate between the two values, specified as a fraction
	///
	/// returns: the interpolated color result
	#[inline]
	pub fn lerp(&self, other: Self, t: f32) -> Self {
		RGBA((self.0.cast() + (other.0 - self.0).cast() * simd::f32x4::splat(t)).cast())
	}

	/// Calculates this color's luminance, returned as a value between 0.0 and 1.0.
	#[inline]
	pub fn luminance(&self) -> f32 {
		(LUMINANCE_RED * srgb_to_linearized(self.r()))
			+ (LUMINANCE_GREEN * srgb_to_linearized(self.g()))
			+ (LUMINANCE_BLUE * srgb_to_linearized(self.b()))
	}

	/// Calculates the approximate "brightness" / grey-scale value for this color, returned as a value between
	/// 0 and 255.
	#[inline]
	pub fn greyscale(&self) -> u8 {
		(brightness(self.luminance()) * 255.0) as u8
	}
}

impl Mul for RGBA {
	type Output = RGBA;

	/// Multiplies two colors together, returning the result. The multiplication is performed by individually
	/// multiplying each color component using the formula `(component * component) / 255`.
	#[inline]
	fn mul(self, rhs: Self) -> Self::Output {
		RGBA(((self.0.cast::<u32>() * rhs.0.cast::<u32>()) / simd::u32x4::splat(255)).cast())
	}
}

impl MulAssign for RGBA {
	/// Multiplies two colors together, assigning the result of the multiplication to this color. The multiplication is
	/// performed by individually multiplying each color component using the formula `(component * component) / 255`.
	#[inline]
	fn mul_assign(&mut self, rhs: Self) {
		self.0 = ((self.0.cast::<u32>() * rhs.0.cast::<u32>()) / simd::u32x4::splat(255)).cast()
	}
}

impl From<u32> for RGBA {
	/// Returns a color value constructed by unpacking RGBA color components from the given u32 value. The u32 value
	/// provided is parsed assuming the following locations of each color component: 0xRRGGBBAA.
	#[inline]
	fn from(value: u32) -> Self {
		RGBA::from_rgba([
			((value & 0xff000000) >> 24) as u8, // r
			((value & 0x00ff0000) >> 16) as u8, // g
			((value & 0x0000ff00) >> 8) as u8,  // b
			(value & 0x000000ff) as u8,         // a
		])
	}
}

impl From<RGBA> for u32 {
	/// Returns a u32 containing packed RGBA color components from this color. The returned u32 value contains the
	/// color components packed in format 0xRRGGBBAA.
	#[inline]
	fn from(value: RGBA) -> Self {
		(value.a() as u32) // a
			+ ((value.b() as u32) << 8) // b
			+ ((value.g() as u32) << 16) // g
			+ ((value.r() as u32) << 24) // r
	}
}

impl From<RGBAf> for RGBA {
	/// Converts a [`RGBAf`] color to an equivalent [`RGBA`] color value.
	#[inline]
	fn from(value: RGBAf) -> Self {
		RGBA::from_rgba([
			(value.r() * 255.0) as u8,
			(value.g() * 255.0) as u8,
			(value.b() * 255.0) as u8,
			(value.a() * 255.0) as u8,
		])
	}
}

impl BytesAsColors<RGBA> for [u8] {
	#[inline]
	unsafe fn as_colors(&self) -> &[RGBA] {
		std::slice::from_raw_parts(
			self.as_ptr() as *const RGBA, //
			self.len() / std::mem::size_of::<RGBA>(),
		)
	}

	#[inline]
	unsafe fn as_colors_mut(&mut self) -> &mut [RGBA] {
		std::slice::from_raw_parts_mut(
			self.as_mut_ptr() as *mut RGBA, //
			self.len() / std::mem::size_of::<RGBA>(),
		)
	}
}

impl ColorsAsBytes for [RGBA] {
	#[inline]
	fn as_bytes(&self) -> &[u8] {
		unsafe {
			std::slice::from_raw_parts(
				self.as_ptr() as *const u8, //
				std::mem::size_of_val(self),
			)
		}
	}

	#[inline]
	fn as_bytes_mut(&mut self) -> &mut [u8] {
		unsafe {
			std::slice::from_raw_parts_mut(
				self.as_mut_ptr() as *mut u8, //
				std::mem::size_of_val(self),
			)
		}
	}
}

impl std::fmt::Debug for RGBA {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "0x{:02x}{:02x}{:02x}{:02x}", self.r(), self.g(), self.b(), self.a())
	}
}

impl std::fmt::Display for RGBA {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "0x{:02x}{:02x}{:02x}{:02x}", self.r(), self.g(), self.b(), self.a())
	}
}

impl WriteType for RGBA {
	type ErrorType = std::io::Error;

	#[inline]
	fn write<T: WriteBytesExt>(&self, writer: &mut T) -> Result<(), Self::ErrorType> {
		writer.write_all(self.0.as_array())?;
		Ok(())
	}
}

impl ReadType for RGBA {
	type OutputType = Self;
	type ErrorType = std::io::Error;

	#[inline]
	fn read<T: ReadBytesExt>(reader: &mut T) -> Result<Self::OutputType, Self::ErrorType> {
		Ok(RGBA::from_rgba([reader.read_u8()?, reader.read_u8()?, reader.read_u8()?, reader.read_u8()?]))
	}
}

/// Unpacked 32-bit color represented as individual normalized f32 color components (0.0 to 1.0) where the
/// components are in the order red, green, blue, alpha.
#[derive(Copy, Clone, PartialEq, PartialOrd, Default)]
#[repr(transparent)]
pub struct RGBAf(pub simd::f32x4);

impl RGBAf {
	/// Returns a color value composed of the provided RGBA color components.
	///
	/// # Arguments
	///
	/// * `rgba`: the 4 color components (0.0 to 1.0) in the order: red, green, blue, alpha
	///
	/// returns: the composed color value
	#[inline]
	pub const fn from_rgba(rgba: [f32; 4]) -> Self {
		RGBAf(simd::f32x4::from_array(rgba))
	}

	/// Returns a color value composed of the provided RGB color components. Substitutes a value of 1.0 for the
	/// missing alpha component.
	///
	/// # Arguments
	///
	/// * `rgb`: the 3 color components (0.0 to 1.0) in the order: red, green, blue
	///
	/// returns: the composed color value
	#[inline]
	pub const fn from_rgb(rgb: [f32; 3]) -> Self {
		RGBAf(simd::f32x4::from_array([rgb[0], rgb[1], rgb[2], 1.0]))
	}

	/// Returns the current red component value (0.0 to 1.0) of this color.
	#[inline]
	pub const fn r(&self) -> f32 {
		self.0.to_array()[0]
	}

	/// Returns the current green component value (0.0 to 1.0) of this color.
	#[inline]
	pub const fn g(&self) -> f32 {
		self.0.to_array()[1]
	}

	/// Returns the current blue component value (0.0 to 1.0) of this color.
	#[inline]
	pub const fn b(&self) -> f32 {
		self.0.to_array()[2]
	}

	/// Returns the current alpha component value (0.0 to 1.0) of this color.
	#[inline]
	pub const fn a(&self) -> f32 {
		self.0.to_array()[3]
	}

	/// Sets the red component value of this color leaving the other components in the color unchanged.
	///
	/// # Arguments
	///
	/// * `value`: the new red component value to be set (0.0 to 1.0)
	#[inline]
	pub fn set_r(&mut self, value: f32) {
		self.0[0] = value
	}

	/// Sets the green component value of this color leaving the other components in the color unchanged.
	///
	/// # Arguments
	///
	/// * `value`: the new green component value to be set (0.0 to 1.0)
	#[inline]
	pub fn set_g(&mut self, value: f32) {
		self.0[1] = value
	}

	/// Sets the blue component value of this color leaving the other components in the color unchanged.
	///
	/// # Arguments
	///
	/// * `value`: the new blue component value to be set (0.0 to 1.0)
	#[inline]
	pub fn set_b(&mut self, value: f32) {
		self.0[2] = value
	}

	/// Sets the alpha component value of this color leaving the other components in the color unchanged.
	///
	/// # Arguments
	///
	/// * `value`: the new alpha component value to be set (0.0 to 1.0)
	#[inline]
	pub fn set_a(&mut self, value: f32) {
		self.0[3] = value
	}

	/// Returns an array with all of this color's RGBA components, in the order: red, green, blue, alpha.
	#[inline]
	pub const fn to_array(&self) -> [f32; 4] {
		self.0.to_array()
	}
}

impl From<u32> for RGBAf {
	/// Returns a color value constructed by unpacking RGBA color components from the given u32 value. The u32 value
	/// provided is parsed assuming the following locations of each color component: 0xRRGGBBAA.
	#[inline]
	fn from(value: u32) -> Self {
		RGBAf::from_rgba([
			((value & 0xff000000) >> 24) as f32 / 255.0, // r
			((value & 0x00ff0000) >> 16) as f32 / 255.0, // g
			((value & 0x0000ff00) >> 8) as f32 / 255.0,  // b
			(value & 0x000000ff) as f32 / 255.0,         // a
		])
	}
}

impl From<RGBA> for RGBAf {
	/// Converts a [`RGBAf`] color to an equivalent [`RGBA`] color value.
	#[inline]
	fn from(value: RGBA) -> Self {
		RGBAf::from_rgba([
			value.r() as f32 / 255.0,
			value.g() as f32 / 255.0,
			value.b() as f32 / 255.0,
			value.a() as f32 / 255.0,
		])
	}
}

impl std::fmt::Debug for RGBAf {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "RGBAf({}, {}, {}, {})", self.r(), self.g(), self.b(), self.a())
	}
}

impl std::fmt::Display for RGBAf {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{{R={}, G={}, B={}, A={}}}", self.r(), self.g(), self.b(), self.a())
	}
}

///////////////////////////////////////////////////////////////////////////////

const LUMINANCE_RED: f32 = 0.212655;
const LUMINANCE_GREEN: f32 = 0.715158;
const LUMINANCE_BLUE: f32 = 0.072187;

fn srgb_to_linearized(color_channel: u8) -> f32 {
	let intensity = color_channel as f32 / 255.0;
	if intensity <= 0.04045 {
		intensity / 12.92
	} else {
		((intensity + 0.055) / (1.055)).powf(2.4)
	}
}

/// Calculates the given sRGB color's luminance, returned as a value between 0.0 and 1.0.
pub fn luminance(rgb: [u8; 3]) -> f32 {
	(LUMINANCE_RED * srgb_to_linearized(rgb[0]))
		+ (LUMINANCE_GREEN * srgb_to_linearized(rgb[1]))
		+ (LUMINANCE_BLUE * srgb_to_linearized(rgb[2]))
}

fn brightness(mut luminance: f32) -> f32 {
	if luminance <= 0.0031308 {
		luminance *= 12.92;
	} else {
		luminance = 1.055 * luminance.powf(1.0 / 2.4) - 0.055;
	}
	luminance
}

/// Calculates the approximate "brightness" / grey-scale value for the given sRGB color, returned
/// as a value between 0 and 255.
pub fn greyscale(rgb: [u8; 3]) -> u8 {
	(brightness(luminance(rgb)) * 255.0) as u8
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::math::NearlyEqual;

	#[test]
	fn rgba() {
		let mut color = RGBA(simd::u8x4::from_array([0x11, 0x22, 0x33, 0x44]));
		assert_eq!(color.r(), 0x11);
		assert_eq!(color.g(), 0x22);
		assert_eq!(color.b(), 0x33);
		assert_eq!(color.a(), 0x44);
		assert_eq!(color.to_array(), [0x11, 0x22, 0x33, 0x44]);

		color.set_a(0x55);
		assert_eq!(color.to_array(), [0x11, 0x22, 0x33, 0x55]);
		color.set_r(0x66);
		assert_eq!(color.to_array(), [0x66, 0x22, 0x33, 0x55]);
		color.set_g(0x77);
		assert_eq!(color.to_array(), [0x66, 0x77, 0x33, 0x55]);
		color.set_b(0x88);
		assert_eq!(color.to_array(), [0x66, 0x77, 0x88, 0x55]);

		let color = RGBA::from_rgba([0x11, 0x22, 0x33, 0x44]);
		assert_eq!(color.to_array(), [0x11, 0x22, 0x33, 0x44]);

		let color = RGBA::from_rgb([0x11, 0x22, 0x33]);
		assert_eq!(color.to_array(), [0x11, 0x22, 0x33, 0xff]);

		let color: RGBA = 0x11223344.into();
		assert_eq!(color.to_array(), [0x11, 0x22, 0x33, 0x44]);

		let other = RGBAf::from_rgba([0.1, 0.2, 0.3, 0.5]);
		let color: RGBA = other.into();
		assert_eq!(color.to_array(), [0x19, 0x33, 0x4c, 0x7f]);

		let color = RGBA::from_rgba([0x11, 0x22, 0x33, 0x44]);
		assert_eq!(0x11223344u32, color.into())
	}

	#[test]
	fn rgba_multiplication() {
		assert_eq!([0x11, 0x22, 0x33, 0xff], (RGBA::from(0xffffffff) * RGBA::from(0x112233ff)).to_array());
		assert_eq!([0x11, 0x22, 0x33, 0xff], (RGBA::from(0x112233ff) * RGBA::from(0xffffffff)).to_array());
		assert_eq!([0x03, 0x00, 0x14, 0x7f], (RGBA::from(0x3300667f) * RGBA::from(0x112233ff)).to_array());
		assert_eq!([0x03, 0x00, 0x14, 0x7f], (RGBA::from(0x112233ff) * RGBA::from(0x3300667f)).to_array());

		let mut color = RGBA::from(0xffffffff);
		color *= RGBA::from(0x112233ff);
		assert_eq!([0x11, 0x22, 0x33, 0xff], color.to_array());
		let mut color = RGBA::from(0x112233ff);
		color *= RGBA::from(0xffffffff);
		assert_eq!([0x11, 0x22, 0x33, 0xff], color.to_array());
		let mut color = RGBA::from(0x3300667f);
		color *= RGBA::from(0x112233ff);
		assert_eq!([0x03, 0x00, 0x14, 0x7f], color.to_array());
		let mut color = RGBA::from(0x112233ff);
		color *= RGBA::from(0x3300667f);
		assert_eq!([0x03, 0x00, 0x14, 0x7f], color.to_array());
	}

	#[test]
	fn rgba_lerping() {
		assert_eq!([0x11, 0x22, 0x33, 0x7f], (RGBA::from(0x1122337f).lerp(RGBA::from(0xaabbccff), 0.0).to_array()));
		assert_eq!([0x5d, 0x6e, 0x7f, 0xbf], (RGBA::from(0x1122337f).lerp(RGBA::from(0xaabbccff), 0.5).to_array()));
		assert_eq!([0xaa, 0xbb, 0xcc, 0xff], (RGBA::from(0x1122337f).lerp(RGBA::from(0xaabbccff), 1.0).to_array()));
	}

	#[test]
	#[rustfmt::skip]
	fn rgba_blending() {
		// TODO: for .blend(), is this really the behaviour we want? the output value's alpha
		//       is blended, but the source color's alpha is what is ultimately used to control
		//       the blend operation. what is best here? the output RGB color looks correct at
		//       any rate, just not sure what the proper output alpha component *should* be in
		//       all cases.

		assert_eq!([0x11, 0x22, 0x33, 0xff], RGBA::from(0x112233ff).blend(RGBA::from(0x555555ff)).to_array());
		assert_eq!([0x33, 0x3b, 0x44, 0xbf], RGBA::from(0x1122337f).blend(RGBA::from(0x555555ff)).to_array());
		assert_eq!([0x55, 0x55, 0x55, 0xff], RGBA::from(0x11223300).blend(RGBA::from(0x555555ff)).to_array());

		assert_eq!([0x11, 0x22, 0x33, 0xff], RGBA::from(0x112233ff).blend(RGBA::from(0x5555557f)).to_array());
		assert_eq!([0x33, 0x3b, 0x44, 0x7f], RGBA::from(0x1122337f).blend(RGBA::from(0x5555557f)).to_array());
		assert_eq!([0x55, 0x55, 0x55, 0x7f], RGBA::from(0x11223300).blend(RGBA::from(0x5555557f)).to_array());

		assert_eq!([0x11, 0x22, 0x33, 0xff], RGBA::from(0x112233ff).blend_with_alpha(RGBA::from(0x555555ff), 255).to_array());
		assert_eq!([0x33, 0x3b, 0x44, 0x7f], RGBA::from(0x1122337f).blend_with_alpha(RGBA::from(0x555555ff), 255).to_array());
		assert_eq!([0x55, 0x55, 0x55, 0x00], RGBA::from(0x11223300).blend_with_alpha(RGBA::from(0x555555ff), 255).to_array());

		assert_eq!([0x11, 0x22, 0x33, 0xff], RGBA::from(0x112233ff).blend_with_alpha(RGBA::from(0x5555557f), 255).to_array());
		assert_eq!([0x33, 0x3b, 0x44, 0x7f], RGBA::from(0x1122337f).blend_with_alpha(RGBA::from(0x5555557f), 255).to_array());
		assert_eq!([0x55, 0x55, 0x55, 0x00], RGBA::from(0x11223300).blend_with_alpha(RGBA::from(0x5555557f), 255).to_array());

		assert_eq!([0x32, 0x3b, 0x43, 0x80], RGBA::from(0x112233ff).blend_with_alpha(RGBA::from(0x555555ff), 128).to_array());
		assert_eq!([0x44, 0x48, 0x4c, 0x3f], RGBA::from(0x1122337f).blend_with_alpha(RGBA::from(0x555555ff), 128).to_array());
		assert_eq!([0x55, 0x55, 0x55, 0x00], RGBA::from(0x11223300).blend_with_alpha(RGBA::from(0x555555ff), 128).to_array());

		assert_eq!([0x55, 0x55, 0x55, 0x00], RGBA::from(0x112233ff).blend_with_alpha(RGBA::from(0x555555ff), 0).to_array());
		assert_eq!([0x55, 0x55, 0x55, 0x00], RGBA::from(0x1122337f).blend_with_alpha(RGBA::from(0x555555ff), 0).to_array());
		assert_eq!([0x55, 0x55, 0x55, 0x00], RGBA::from(0x11223300).blend_with_alpha(RGBA::from(0x555555ff), 0).to_array());
	}

	#[test]
	fn rgba_tinting() {
		assert_eq!([0x11, 0x22, 0x33, 0xff], RGBA::from(0xffffffff).tint(RGBA::from(0x112233ff)).to_array());
		assert_eq!([0x88, 0x90, 0x99, 0xff], RGBA::from(0xffffffff).tint(RGBA::from(0x1122337f)).to_array());
		assert_eq!([0xff, 0xff, 0xff, 0xff], RGBA::from(0xffffffff).tint(RGBA::from(0x11223300)).to_array());
	}

	#[test]
	fn rgba_bytes_to_colors_casting() {
		let mut bytes =
			[0xff, 0x00, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0x00, 0xff, 0xff];

		let colors = unsafe { bytes.as_colors() };
		assert_eq!(
			colors,
			[
				RGBA::from_rgba([0xff, 0x00, 0x00, 0xff]),
				RGBA::from_rgba([0x00, 0xff, 0x00, 0xff]),
				RGBA::from_rgba([0x00, 0x00, 0xff, 0xff]),
				RGBA::from_rgba([0xff, 0x00, 0xff, 0xff]),
			]
		);

		let colors = unsafe { bytes.as_colors_mut() };
		assert_eq!(
			colors,
			[
				RGBA::from_rgba([0xff, 0x00, 0x00, 0xff]),
				RGBA::from_rgba([0x00, 0xff, 0x00, 0xff]),
				RGBA::from_rgba([0x00, 0x00, 0xff, 0xff]),
				RGBA::from_rgba([0xff, 0x00, 0xff, 0xff]),
			]
		);

		// bytes slice which is NOT an exact multiple of 4
		let mut bytes = [
			0xff, 0x00, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0x00, 0xff, 0xff, 0x7f, 0x7f,
		];

		let colors = unsafe { bytes.as_colors() };
		assert_eq!(
			colors,
			[
				RGBA::from_rgba([0xff, 0x00, 0x00, 0xff]),
				RGBA::from_rgba([0x00, 0xff, 0x00, 0xff]),
				RGBA::from_rgba([0x00, 0x00, 0xff, 0xff]),
				RGBA::from_rgba([0xff, 0x00, 0xff, 0xff]),
			]
		);

		let colors = unsafe { bytes.as_colors_mut() };
		assert_eq!(
			colors,
			[
				RGBA::from_rgba([0xff, 0x00, 0x00, 0xff]),
				RGBA::from_rgba([0x00, 0xff, 0x00, 0xff]),
				RGBA::from_rgba([0x00, 0x00, 0xff, 0xff]),
				RGBA::from_rgba([0xff, 0x00, 0xff, 0xff]),
			]
		);
	}

	#[test]
	fn rgba_colors_to_bytes_casting() {
		let mut colors = [
			RGBA::from_rgba([0xff, 0x00, 0x00, 0xff]),
			RGBA::from_rgba([0x00, 0xff, 0x00, 0xff]),
			RGBA::from_rgba([0x00, 0x00, 0xff, 0xff]),
			RGBA::from_rgba([0xff, 0x00, 0xff, 0xff]),
		];

		let bytes = colors.as_bytes();
		assert_eq!(
			bytes,
			[0xff, 0x00, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0x00, 0xff, 0xff]
		);

		let bytes = colors.as_bytes_mut();
		assert_eq!(
			bytes,
			[0xff, 0x00, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0x00, 0xff, 0xff]
		);
	}

	#[test]
	fn rgbaf() {
		let mut color = RGBAf(simd::f32x4::from_array([0.1, 0.2, 0.3, 0.5]));
		assert_eq!(color.r(), 0.1);
		assert_eq!(color.g(), 0.2);
		assert_eq!(color.b(), 0.3);
		assert_eq!(color.a(), 0.5);
		assert_eq!(color.to_array(), [0.1, 0.2, 0.3, 0.5]);

		color.set_a(1.0);
		assert_eq!(color.to_array(), [0.1, 0.2, 0.3, 1.0]);
		color.set_r(0.4);
		assert_eq!(color.to_array(), [0.4, 0.2, 0.3, 1.0]);
		color.set_g(0.5);
		assert_eq!(color.to_array(), [0.4, 0.5, 0.3, 1.0]);
		color.set_b(0.6);
		assert_eq!(color.to_array(), [0.4, 0.5, 0.6, 1.0]);

		let color = RGBAf::from_rgba([0.1, 0.2, 0.3, 0.5]);
		assert_eq!(color.to_array(), [0.1, 0.2, 0.3, 0.5]);

		let color = RGBAf::from_rgb([0.1, 0.2, 0.3]);
		assert_eq!(color.to_array(), [0.1, 0.2, 0.3, 1.0]);

		let color: RGBAf = 0x19334c7f.into();
		assert!(color.a().nearly_equal(0.5, 0.01));
		assert!(color.r().nearly_equal(0.1, 0.01));
		assert!(color.g().nearly_equal(0.2, 0.01));
		assert!(color.b().nearly_equal(0.3, 0.01));

		let other = RGBA::from_rgba([0x19, 0x33, 0x4c, 0x7f]);
		let color: RGBAf = other.into();
		assert!(color.a().nearly_equal(0.5, 0.01));
		assert!(color.r().nearly_equal(0.1, 0.01));
		assert!(color.g().nearly_equal(0.2, 0.01));
		assert!(color.b().nearly_equal(0.3, 0.01));
	}
}
