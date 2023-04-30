use std::ops::{Mul, MulAssign};
use std::simd;

/// Packed 32-bit color, in the format: 0xAARRGGBB
pub type Color1u32 = u32;

/// Unpacked 24-bit color as 8-bit color components in the order: red, green, blue
pub type Color3u8 = [u8; 3];

/// Unpacked 32-bit color as 8-bit color components in the order: alpha, red, green, blue
pub type Color4u8 = [u8; 4];

/// Unpacked 24-bit color as normalized f32 color components (0.0 to 1.0) in the order: red, green, blue
pub type Color3f32 = [f32; 3];

/// Unpacked 32-bit color as normalized f32 color components (0.0 to 1.0) in the order: alpha, red, green, blue
pub type Color4f32 = [f32; 4];

/// Unpacked 32-bit color in a SIMD vector that is otherwise equivalent to [`Color4u8`]
pub type SimdColor4u8 = simd::u8x4;

/// Unpacked 32-bit color in a SIMD vector that is otherwise equivalent to [`Color4f32`]
pub type SimdColor4f32 = simd::f32x4;

// these colours are taken from the default VGA palette

pub const COLOR_BLACK: Color1u32 = 0xff000000;
pub const COLOR_BLUE: Color1u32 = 0xff0000aa;
pub const COLOR_GREEN: Color1u32 = 0xff00aa00;
pub const COLOR_CYAN: Color1u32 = 0xff00aaaa;
pub const COLOR_RED: Color1u32 = 0xffaa0000;
pub const COLOR_MAGENTA: Color1u32 = 0xffaa00aa;
pub const COLOR_BROWN: Color1u32 = 0xffaa5500;
pub const COLOR_LIGHT_GRAY: Color1u32 = 0xffaaaaaa;
pub const COLOR_DARK_GRAY: Color1u32 = 0xff555555;
pub const COLOR_BRIGHT_BLUE: Color1u32 = 0xff5555ff;
pub const COLOR_BRIGHT_GREEN: Color1u32 = 0xff55ff55;
pub const COLOR_BRIGHT_CYAN: Color1u32 = 0xff55ffff;
pub const COLOR_BRIGHT_RED: Color1u32 = 0xffff5555;
pub const COLOR_BRIGHT_MAGENTA: Color1u32 = 0xffff55ff;
pub const COLOR_BRIGHT_YELLOW: Color1u32 = 0xffffff55;
pub const COLOR_BRIGHT_WHITE: Color1u32 = 0xffffffff;

// TODO: probably should name these better, after i do much more reading on the subject :-)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlendFunction {
	Blend,
	BlendSourceWithAlpha(u8),
	TintedBlend(Color1u32),
	MultipliedBlend(Color1u32),
}

impl BlendFunction {
	#[inline]
	/// Blends the source and destination color together using the function associated with
	/// this enum value. Both colors should be 32-bit packed colors in the format 0xAARRGGBB.
	///
	/// # Arguments
	///
	/// * `src`: the source color to blend
	/// * `dest`: the destination color to blend the source color over
	///
	/// returns: the blended color
	pub fn blend_1u32(&self, src: Color1u32, dest: Color1u32) -> Color1u32 {
		use BlendFunction::*;
		match self {
			Blend => blend_argb32(src, dest),
			BlendSourceWithAlpha(opacity) => blend_argb32_source_by(src, dest, *opacity),
			TintedBlend(tint) => tinted_blend_argb32(*tint, src, dest),
			MultipliedBlend(color) => multiplied_blend_argb32(*color, src, dest),
		}
	}

	#[inline]
	pub fn blend_4u8(&self, src: Color4u8, dest: Color4u8) -> Color4u8 {
		use BlendFunction::*;
		match self {
			Blend => blend_argb(src, dest),
			BlendSourceWithAlpha(opacity) => blend_argb_source_by(src, dest, *opacity),
			TintedBlend(tint) => tinted_blend_argb(from_argb32(*tint), src, dest),
			MultipliedBlend(color) => multiplied_blend_argb(from_argb32(*color), src, dest),
		}
	}

	#[inline]
	pub fn blend_simd(&self, src: SimdColor4u8, dest: SimdColor4u8) -> SimdColor4u8 {
		use BlendFunction::*;
		match self {
			Blend => blend_argb_simd(src, dest),
			BlendSourceWithAlpha(opacity) => blend_argb_simd_source_by(src, dest, *opacity),
			TintedBlend(tint) => tinted_blend_argb_simd(from_argb32_simd(*tint), src, dest),
			MultipliedBlend(color) => multiplied_blend_argb_simd(from_argb32_simd(*color), src, dest),
		}
	}
}

/// Packs a set of individual ARGB components to a packed 32-bit color value.
///
/// # Arguments
///
/// * `argb` the 4 color components (0-255) in the order: alpha, red, green, blue
///
/// returns: the 32-bit packed color
#[inline]
pub fn to_argb32(argb: Color4u8) -> Color1u32 {
	(argb[3] as u32) // b
		+ ((argb[2] as u32) << 8) // g
		+ ((argb[1] as u32) << 16) // r
		+ ((argb[0] as u32) << 24) // a
}

#[inline]
pub fn to_argb32_simd(argb: SimdColor4u8) -> Color1u32 {
	to_argb32(argb.to_array())
}

/// Packs a set of individual ARGB normalized components to a packed 32-bit color value.
///
/// # Arguments
///
/// * `argb` the 4 normalized color components (0.0 to 1.0) in the order: alpha, red, green, blue
///
/// returns: the 32-bit packed color
#[inline]
pub fn to_argb32_normalized(argb: Color4f32) -> Color1u32 {
	(((argb[3] * 255.0) as u32) & 0xff) // b
		+ ((((argb[2] * 255.0) as u32) & 0xff) << 8) // g
		+ ((((argb[1] * 255.0) as u32) & 0xff) << 16) // r
		+ ((((argb[0] * 255.0) as u32) & 0xff) << 24) // a
}

#[inline]
pub fn to_argb32_normalized_simd(argb: SimdColor4f32) -> Color1u32 {
	to_argb32_normalized(argb.to_array())
}

/// Unpacks the individual ARGB components out of a packed 32-bit color value.
///
/// # Arguments
///
/// * `argb`: the 32-bit packed color to unpack
///
/// returns: the unpacked ARGB color components (0-255 each) in order: alpha, red, green, blue
#[inline]
pub fn from_argb32(argb: Color1u32) -> Color4u8 {
	[
		((argb & 0xff000000) >> 24) as u8, // a
		((argb & 0x00ff0000) >> 16) as u8, // r
		((argb & 0x0000ff00) >> 8) as u8,  // g
		(argb & 0x000000ff) as u8,         // b
	]
}

#[inline]
pub fn from_argb32_simd(argb: Color1u32) -> SimdColor4u8 {
	SimdColor4u8::from_array(from_argb32(argb))
}

/// Unpacks the individual ARGB normalized components out of a packed 32-bit color value.
///
/// # Arguments
///
/// * `argb`: the 32-bit packed color to unpack
///
/// returns: the unpacked ARGB normalized color components (0.0 to 1.0 each) in order: alpha, red, green, blue
#[inline]
pub fn from_argb32_normalized(argb: Color1u32) -> Color4f32 {
	[
		((argb & 0xff000000) >> 24) as f32 / 255.0, // a
		((argb & 0x00ff0000) >> 16) as f32 / 255.0, // r
		((argb & 0x0000ff00) >> 8) as f32 / 255.0,  // g
		(argb & 0x000000ff) as f32 / 255.0,         // b
	]
}

#[inline]
pub fn from_argb32_normalized_simd(argb: Color1u32) -> SimdColor4f32 {
	SimdColor4f32::from_array(from_argb32_normalized(argb))
}

/// Packs a set of individual RGB components to a combined 32-bit color value. Substitutes a value of 255 for
/// the missing alpha component.
///
/// # Arguments
///
/// * `rgb` the 3 color components (0-255) to be packed, in the order: red, green, blue
///
/// returns: the 32-bit packed color
#[inline]
pub fn to_rgb32(rgb: Color3u8) -> Color1u32 {
	to_argb32([255, rgb[0], rgb[1], rgb[2]])
}

#[inline]
pub fn to_rgb32_simd(argb: SimdColor4u8) -> Color1u32 {
	to_argb32([255, argb[1], argb[2], argb[3]])
}

/// PAcks a set of individual RGB normalized components to a combined 32-bit color value. Substitutes a value of
/// 1.0 for the missing alpha component.
///
/// # Arguments
///
/// * `rgb` the 3 normalized color components (0.0 to 1.0) to be packed, in the order: red, green, blue
///
/// returns: the u32 packed color
#[inline]
pub fn to_rgb32_normalized(rgb: Color3f32) -> Color1u32 {
	to_argb32_normalized([1.0, rgb[0], rgb[1], rgb[2]])
}

/// Unpacks the individual RGB components out of a combined 32-bit color value. Ignores the alpha component.
///
/// # Arguments
///
/// * `argb`: the 32-bit packed color
///
/// returns: the unpacked ARGB color components (0-255 each) in order: red, green, blue
#[inline]
pub fn from_rgb32(rgb: Color1u32) -> Color3u8 {
	// ignore alpha component at 0xff000000 ...
	[
		((rgb & 0x00ff0000) >> 16) as u8, // r
		((rgb & 0x0000ff00) >> 8) as u8,  // g
		(rgb & 0x000000ff) as u8,         // b
	]
}

#[inline]
pub fn from_rgb32_simd(rgb: Color1u32) -> SimdColor4u8 {
	let [r, g, b] = from_rgb32(rgb);
	SimdColor4u8::from_array([255, r, g, b])
}

/// Unpacks the individual RGB normalized components out of a combined 32-bit color value. Ignores the alpha component.
///
/// # Arguments
///
/// * `argb`: the 32-bit packed color
///
/// returns: the unpacked ARGB normalized color components (0.0 to 1.0 each) in order: red, green, blue
#[inline]
pub fn from_rgb32_normalized(rgb: Color1u32) -> Color3f32 {
	// ignore alpha component at 0xff000000 ...
	[
		((rgb & 0x00ff0000) >> 16) as f32 / 255.0, // r
		((rgb & 0x0000ff00) >> 8) as f32 / 255.0,  // g
		(rgb & 0x000000ff) as f32 / 255.0,         // b
	]
}

/// Blends two color components together using a "strength" factor to control how much of the source
/// color versus destination color is represented in the result. This is using the formula:
/// `(source * strength) + (dest * (1 - strength))`
///
/// # Arguments
///
/// * `strength`: controls how much of the source versus destination is represented in the final output,
///               where 0 means the source component is 100% present in the output while 255 means the
///               destination component is 100% present in the output and 128 means 50% of each.
/// * `src`: the source component to be blended
/// * `dest`: the destination component to be blended
///
/// returns: the blended component result
#[inline]
pub fn blend_components(strength: u8, src: u8, dest: u8) -> u8 {
	(((src as u16 * strength as u16) + (dest as u16 * (255 - strength as u16))) / 255) as u8
}

#[inline]
pub fn blend_components_simd(strength: u8, src: SimdColor4u8, dest: SimdColor4u8) -> SimdColor4u8 {
	let strength = simd::u16x4::splat(strength as u16);
	let max = simd::u16x4::splat(255);
	(((src.cast() * strength) + (dest.cast() * (max - strength))) / max).cast()
}

/// Alpha blends two colors together.
///
/// # Arguments
///
/// * `src`: the 32-bit packed source color that is to be blended onto the destination
/// * `dest`: the 32-bit packed destination color that the source is being blended into
///
/// returns: the 32-bit packed blended color result
#[inline]
pub fn blend_argb32(src: Color1u32, dest: Color1u32) -> Color1u32 {
	let unpacked_src = from_argb32(src);
	let unpacked_dest = from_argb32(dest);
	to_argb32(blend_argb(unpacked_src, unpacked_dest))
}

/// Alpha blends two colors together.
///
/// # Arguments
///
/// * `src`: the 32-bit unpacked source color that is to be blended onto the destination
/// * `dest`: the 32-bit unpacked destination color that the source is being blended into
///
/// returns: the 32-bit unpacked blended color result
#[inline]
pub fn blend_argb(src: Color4u8, dest: Color4u8) -> Color4u8 {
	[
		blend_components(src[0], src[0], dest[0]),
		blend_components(src[0], src[1], dest[1]),
		blend_components(src[0], src[2], dest[2]),
		blend_components(src[0], src[3], dest[3]),
	]
}

#[inline]
pub fn blend_argb_simd(src: SimdColor4u8, dest: SimdColor4u8) -> SimdColor4u8 {
	blend_components_simd(src[0], src, dest)
}

/// Blends the source and destination colors together, where the alpha value used to blend the two
/// colors is derived from the given alpha value multiplied with the source color's alpha component.
/// This allows for more flexibility in directly controling how transparent the source
/// color is overtop of the destination.
///
/// # Arguments
///
/// * `src`: the 32-bit packed source color that is to be blended onto the destination. the alpha component of this
///          color is used during the blend.
/// * `dest`: the 32-bit packed destination color that the source is being blended into. the alpha component of this
///           color is ignored.
/// * `alpha`: the transparency or opacity of the source color over the destination color. this is
///            multipled together with the source color's alpha component to arrive at the final
///            alpha value used for blending the source and destination color's RGB components.
///
/// returns: the 32-bit packed blended color result
#[inline]
pub fn blend_argb32_source_by(src: Color1u32, dest: Color1u32, alpha: u8) -> Color1u32 {
	let unpacked_src = from_argb32(src);
	let unpacked_dest = from_argb32(dest);
	to_argb32(blend_argb_source_by(unpacked_src, unpacked_dest, alpha))
}

/// Blends the source and destination colors together, where the alpha value used to blend the two
/// colors is derived from the given alpha value multiplied with the source color's alpha component.
/// This allows for more flexibility in directly controling how transparent the source
/// color is overtop of the destination.
///
/// # Arguments
///
/// * `src`: the 32-bit unpacked source color that is to be blended onto the destination. the alpha component of this
///          color is used during the blend.
/// * `dest`: the 32-bit unpacked destination color that the source is being blended into. the alpha component of this
///           color is ignored.
/// * `alpha`: the transparency or opacity of the source color over the destination color. this is
///            multipled together with the source color's alpha component to arrive at the final
///            alpha value used for blending the source and destination color's RGB components.
///
/// returns: the 32-bit unpacked blended color result
#[inline]
pub fn blend_argb_source_by(src: Color4u8, dest: Color4u8, alpha: u8) -> Color4u8 {
	let alpha = ((alpha as u16 * src[0] as u16) / 255) as u8;
	[
		alpha,
		blend_components(alpha, src[1], dest[1]),
		blend_components(alpha, src[2], dest[2]),
		blend_components(alpha, src[3], dest[3]),
	]
}

#[inline]
pub fn blend_argb_simd_source_by(src: SimdColor4u8, dest: SimdColor4u8, alpha: u8) -> SimdColor4u8 {
	let alpha = ((alpha as u16 * src[0] as u16) / 255) as u8;
	let mut blended = blend_components_simd(alpha, src, dest);
	blended[0] = alpha;
	blended
}

/// Applies a tint to a color, using the tint color's alpha component as the strength of the tint,
/// where 0 means no tint and 255 means full tint. The original color's alpha component is preserved in
/// the result.
///
/// # Arguments
///
/// * `color`: the 32-bit packed color to be tinted
/// * `tint`: the 32-bit packed tint color to be applied to the color, where the alpha component represents
///           the tint strength
///
/// returns: the resulting 32-bit packed tinted color
#[inline]
pub fn tint_argb32(color: Color1u32, tint: Color1u32) -> Color1u32 {
	let unpacked_color = from_argb32(color);
	let unpacked_tint = from_argb32(tint);
	to_argb32(tint_argb(unpacked_color, unpacked_tint))
}

/// Applies a tint to a color, using the tint color's alpha component as the strength of the tint,
/// where 0 means no tint and 255 means full tint. The original color's alpha component is preserved in
/// the result.
///
/// # Arguments
///
/// * `color`: the 32-bit unpacked color to be tinted
/// * `tint`: the 32-bit unpacked tint color to be applied to the color, where the alpha component represents
///           the tint strength
///
/// returns: the resulting 32-bit unpacked tinted color
#[inline]
pub fn tint_argb(color: Color4u8, tint: Color4u8) -> Color4u8 {
	[
		color[0],
		blend_components(tint[0], tint[1], color[1]),
		blend_components(tint[0], tint[2], color[2]),
		blend_components(tint[0], tint[3], color[3]),
	]
}

#[inline]
pub fn tint_argb_simd(color: SimdColor4u8, mut tint: SimdColor4u8) -> SimdColor4u8 {
	let strength = tint[0];
	tint[0] = color[0];
	blend_components_simd(strength, tint, color)
}

/// Multiplies two colors together, returing the result. The multiplication is performed by
/// individually multiplying each color component using the formula `(component * component) / 255`.
///
/// # Arguments
///
/// * `a`: the first 32-bit packed color
/// * `b`: the second 32-bit packed color
///
/// returns: the resulting 32-bit packed color from the multiplication
#[inline]
pub fn multiply_argb32(a: Color1u32, b: Color1u32) -> Color1u32 {
	let unpacked_a = from_argb32(a);
	let unpacked_b = from_argb32(b);
	to_argb32(multiply_argb(unpacked_a, unpacked_b))
}

/// Multiplies two colors together, returing the result. The multiplication is performed by
/// individually multiplying each color component using the formula `(component * component) / 255`.
///
/// # Arguments
///
/// * `a`: the first 32-bit unpacked color
/// * `b`: the second 32-bit unpacked color
///
/// returns: the resulting 32-bit unpacked color from the multiplication
#[inline]
pub fn multiply_argb(a: Color4u8, b: Color4u8) -> Color4u8 {
	[
		((a[0] as u32 * b[0] as u32) / 255) as u8,
		((a[1] as u32 * b[1] as u32) / 255) as u8,
		((a[2] as u32 * b[2] as u32) / 255) as u8,
		((a[3] as u32 * b[3] as u32) / 255) as u8,
	]
}

#[inline]
pub fn multiply_argb_simd(a: SimdColor4u8, b: SimdColor4u8) -> SimdColor4u8 {
	((a.cast::<u32>() * b.cast::<u32>()) / simd::u32x4::splat(255)).cast()
}

/// Linearly interpolates between two colors.
///
/// # Arguments
///
/// * `a`: the first 32-bit packed color
/// * `b`: the second 32-bit packed color
/// * `t`: the amount to interpolate between the two values, specified as a fraction.
///
/// returns: the interpolated 32-bit packed color result
#[inline]
pub fn lerp_argb32(a: Color1u32, b: Color1u32, t: f32) -> Color1u32 {
	let unpacked_a = from_argb32(a);
	let unpacked_b = from_argb32(b);
	to_argb32(lerp_argb(unpacked_a, unpacked_b, t))
}

/// Linearly interpolates between two colors.
///
/// # Arguments
///
/// * `a`: the first 32-bit unpacked color
/// * `b`: the second 32-bit unpacked color
/// * `t`: the amount to interpolate between the two values, specified as a fraction.
///
/// returns: the interpolated 32-bit unpacked color result
#[inline]
pub fn lerp_argb(a: Color4u8, b: Color4u8, t: f32) -> Color4u8 {
	[
		((a[0] as f32) + ((b[0] as f32) - (a[0] as f32)) * t) as u8,
		((a[1] as f32) + ((b[1] as f32) - (a[1] as f32)) * t) as u8,
		((a[2] as f32) + ((b[2] as f32) - (a[2] as f32)) * t) as u8,
		((a[3] as f32) + ((b[3] as f32) - (a[3] as f32)) * t) as u8,
	]
}

#[inline]
pub fn lerg_argb_simd(a: SimdColor4u8, b: SimdColor4u8, t: f32) -> SimdColor4u8 {
	(a.cast() + (b - a).cast() * simd::f32x4::splat(t)).cast()
}

/// Linearly interpolates between two colors. Ignores the alpha component, which will always be
/// set to 255 in the return value.
///
/// # Arguments
///
/// * `a`: the first 32-bit unpacked color
/// * `b`: the second 32-bit unpacked color
/// * `t`: the amount to interpolate between the two values, specified as a fraction.
///
/// returns: the interpolated 32-bit unpacked color result, which will always have an alpha component of 255
#[inline]
pub fn lerp_rgb32(a: Color1u32, b: Color1u32, t: f32) -> Color1u32 {
	let [r1, g1, b1] = from_rgb32(a);
	let [r2, g2, b2] = from_rgb32(b);
	to_rgb32([
		((r1 as f32) + ((r2 as f32) - (r1 as f32)) * t) as u8,
		((g1 as f32) + ((g2 as f32) - (g1 as f32)) * t) as u8,
		((b1 as f32) + ((b2 as f32) - (b1 as f32)) * t) as u8,
	])
}

#[inline]
pub fn multiplied_blend_argb32(color: Color1u32, src: Color1u32, dest: Color1u32) -> Color1u32 {
	blend_argb32(multiply_argb32(src, color), dest)
}

#[inline]
pub fn multiplied_blend_argb(color: Color4u8, src: Color4u8, dest: Color4u8) -> Color4u8 {
	blend_argb(multiply_argb(src, color), dest)
}

#[inline]
pub fn multiplied_blend_argb_simd(color: SimdColor4u8, src: SimdColor4u8, dest: SimdColor4u8) -> SimdColor4u8 {
	blend_argb_simd(multiply_argb_simd(src, color), dest)
}

#[inline]
pub fn tinted_blend_argb32(tint: Color1u32, src: Color1u32, dest: Color1u32) -> Color1u32 {
	blend_argb32(tint_argb32(src, tint), dest)
}

#[inline]
pub fn tinted_blend_argb(tint: Color4u8, src: Color4u8, dest: Color4u8) -> Color4u8 {
	blend_argb(tint_argb(src, tint), dest)
}

#[inline]
pub fn tinted_blend_argb_simd(tint: SimdColor4u8, src: SimdColor4u8, dest: SimdColor4u8) -> SimdColor4u8 {
	blend_argb_simd(tint_argb_simd(src, tint), dest)
}

///////////////////////////////////////////////////////////////////////////////

/// Unpacked 32-bit color represented as individual 8-bit color components where the components are in the
/// order alpha, red, green, blue.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ARGBu8x4(pub simd::u8x4);

impl ARGBu8x4 {
	#[inline]
	pub const fn from_argb(argb: [u8; 4]) -> Self {
		ARGBu8x4(simd::u8x4::from_array(argb))
	}

	#[inline]
	pub const fn from_rgb(rgb: [u8; 3]) -> Self {
		ARGBu8x4(simd::u8x4::from_array([255, rgb[0], rgb[1], rgb[2]]))
	}

	#[inline]
	pub const fn a(&self) -> u8 {
		self.0.to_array()[0]
	}

	#[inline]
	pub const fn r(&self) -> u8 {
		self.0.to_array()[1]
	}

	#[inline]
	pub const fn g(&self) -> u8 {
		self.0.to_array()[2]
	}

	#[inline]
	pub const fn b(&self) -> u8 {
		self.0.to_array()[3]
	}

	#[inline]
	pub fn set_a(&mut self, value: u8) {
		self.0[0] = value
	}

	#[inline]
	pub fn set_r(&mut self, value: u8) {
		self.0[1] = value
	}

	#[inline]
	pub fn set_g(&mut self, value: u8) {
		self.0[2] = value
	}

	#[inline]
	pub fn set_b(&mut self, value: u8) {
		self.0[3] = value
	}

	#[inline]
	pub const fn to_array(&self) -> [u8; 4] {
		self.0.to_array()
	}

	#[inline]
	pub fn lerp(&self, other: Self, t: f32) -> Self {
		ARGBu8x4((self.0.cast() + (other.0 - self.0).cast() * simd::f32x4::splat(t)).cast())
	}
}

impl Mul for ARGBu8x4 {
	type Output = ARGBu8x4;

	#[inline]
	fn mul(self, rhs: Self) -> Self::Output {
		ARGBu8x4(((self.0.cast::<u32>() * rhs.0.cast::<u32>()) / simd::u32x4::splat(255)).cast())
	}
}

impl MulAssign for ARGBu8x4 {
	#[inline]
	fn mul_assign(&mut self, rhs: Self) {
		self.0 = ((self.0.cast::<u32>() * rhs.0.cast::<u32>()) / simd::u32x4::splat(255)).cast()
	}
}

impl From<u32> for ARGBu8x4 {
	#[inline]
	fn from(value: u32) -> Self {
		ARGBu8x4::from_argb([
			((value & 0xff000000) >> 24) as u8, // a
			((value & 0x00ff0000) >> 16) as u8, // r
			((value & 0x0000ff00) >> 8) as u8,  // g
			(value & 0x000000ff) as u8,         // b
		])
	}
}

impl From<ARGBf32x4> for ARGBu8x4 {
	#[inline]
	fn from(value: ARGBf32x4) -> Self {
		ARGBu8x4::from_argb([
			(value.a() * 255.0) as u8,
			(value.r() * 255.0) as u8,
			(value.g() * 255.0) as u8,
			(value.b() * 255.0) as u8,
		])
	}
}

/// Unpacked 32-bit color represented as individual normalized f32 color components (0.0 to 1.0) where the
/// components are in the order alpha, red, green, blue.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct ARGBf32x4(pub simd::f32x4);

impl ARGBf32x4 {
	#[inline]
	pub const fn from_argb(argb: [f32; 4]) -> Self {
		ARGBf32x4(simd::f32x4::from_array(argb))
	}

	#[inline]
	pub const fn from_rgb(rgb: [f32; 3]) -> Self {
		ARGBf32x4(simd::f32x4::from_array([1.0, rgb[0], rgb[1], rgb[2]]))
	}

	#[inline]
	pub const fn a(&self) -> f32 {
		self.0.to_array()[0]
	}

	#[inline]
	pub const fn r(&self) -> f32 {
		self.0.to_array()[1]
	}

	#[inline]
	pub const fn g(&self) -> f32 {
		self.0.to_array()[2]
	}

	#[inline]
	pub const fn b(&self) -> f32 {
		self.0.to_array()[3]
	}

	#[inline]
	pub fn set_a(&mut self, value: f32) {
		self.0[0] = value
	}

	#[inline]
	pub fn set_r(&mut self, value: f32) {
		self.0[1] = value
	}

	#[inline]
	pub fn set_g(&mut self, value: f32) {
		self.0[2] = value
	}

	#[inline]
	pub fn set_b(&mut self, value: f32) {
		self.0[3] = value
	}

	#[inline]
	pub const fn to_array(&self) -> [f32; 4] {
		self.0.to_array()
	}
}

impl From<ARGBu8x4> for ARGBf32x4 {
	fn from(value: ARGBu8x4) -> Self {
		ARGBf32x4::from_argb([
			value.a() as f32 / 255.0,
			value.r() as f32 / 255.0,
			value.g() as f32 / 255.0,
			value.b() as f32 / 255.0,
		])
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
	fn argb_conversions() {
		let argb = to_argb32([0x11, 0x22, 0x33, 0x44]);
		assert_eq!(argb, 0x11223344);

		let argb = to_rgb32([0x22, 0x33, 0x44]);
		assert_eq!(argb, 0xff223344);

		let [a, r, g, b] = from_argb32(0x11223344);
		assert_eq!(0x11, a);
		assert_eq!(0x22, r);
		assert_eq!(0x33, g);
		assert_eq!(0x44, b);

		let [r, g, b] = from_rgb32(0x11223344);
		assert_eq!(0x22, r);
		assert_eq!(0x33, g);
		assert_eq!(0x44, b);
	}

	#[test]
	fn normalized_argb_conversions() {
		let argb = to_argb32_normalized([0.5, 0.1, 0.2, 0.3]);
		assert_eq!(argb, 0x7f19334c);

		let argb = to_rgb32_normalized([0.1, 0.2, 0.3]);
		assert_eq!(argb, 0xff19334c);

		// floating-point accuracy is a real bitch here ... lol.
		// the low-accuracy epsilon values in these asserts is not an accident or oversight

		let [a, r, g, b] = from_argb32_normalized(0x7f19334c);
		assert!(a.nearly_equal(0.5, 0.01));
		assert!(r.nearly_equal(0.1, 0.01));
		assert!(g.nearly_equal(0.2, 0.01));
		assert!(b.nearly_equal(0.3, 0.01));

		let [r, g, b] = from_rgb32_normalized(0x7f19334c);
		assert!(r.nearly_equal(0.1, 0.01));
		assert!(g.nearly_equal(0.2, 0.01));
		assert!(b.nearly_equal(0.3, 0.01));
	}

	#[test]
	fn blending() {
		// TODO: for blend_argb32, is this really the behaviour we want? the output value's alpha
		//       is blended, but the source color's alpha is what is ultimately used to control
		//       the blend operation. what is best here? the output RGB color looks correct at
		//       any rate, just not sure what the proper output alpha component *should* be in
		//       all cases.

		assert_eq!(0xff112233, blend_argb32(0xff112233, 0xff555555));
		assert_eq!(0xbf333b44, blend_argb32(0x7f112233, 0xff555555));
		assert_eq!(0xff555555, blend_argb32(0x00112233, 0xff555555));

		assert_eq!(0xff112233, blend_argb32(0xff112233, 0x7f555555));
		assert_eq!(0x7f333b44, blend_argb32(0x7f112233, 0x7f555555));
		assert_eq!(0x7f555555, blend_argb32(0x00112233, 0x7f555555));

		assert_eq!(0xff112233, blend_argb32_source_by(0xff112233, 0xff555555, 255));
		assert_eq!(0x7f333b44, blend_argb32_source_by(0x7f112233, 0xff555555, 255));
		assert_eq!(0x00555555, blend_argb32_source_by(0x00112233, 0xff555555, 255));

		assert_eq!(0xff112233, blend_argb32_source_by(0xff112233, 0x7f555555, 255));
		assert_eq!(0x7f333b44, blend_argb32_source_by(0x7f112233, 0x7f555555, 255));
		assert_eq!(0x00555555, blend_argb32_source_by(0x00112233, 0x7f555555, 255));

		assert_eq!(0x80323b43, blend_argb32_source_by(0xff112233, 0xff555555, 128));
		assert_eq!(0x3f44484c, blend_argb32_source_by(0x7f112233, 0xff555555, 128));
		assert_eq!(0x00555555, blend_argb32_source_by(0x00112233, 0xff555555, 128));

		assert_eq!(0x00555555, blend_argb32_source_by(0xff112233, 0xff555555, 0));
		assert_eq!(0x00555555, blend_argb32_source_by(0x7f112233, 0xff555555, 0));
		assert_eq!(0x00555555, blend_argb32_source_by(0x00112233, 0xff555555, 0));
	}

	#[test]
	fn tinting() {
		assert_eq!(0xff112233, tint_argb32(0xffffffff, 0xff112233));
		assert_eq!(0xff889099, tint_argb32(0xffffffff, 0x7f112233));
		assert_eq!(0xffffffff, tint_argb32(0xffffffff, 0x00112233));
	}

	#[test]
	fn multiplying() {
		assert_eq!(0xff112233, multiply_argb32(0xffffffff, 0xff112233));
		assert_eq!(0xff112233, multiply_argb32(0xff112233, 0xffffffff));

		assert_eq!(0x7f030014, multiply_argb32(0x7f330066, 0xff112233));
		assert_eq!(0x7f030014, multiply_argb32(0xff112233, 0x7f330066));
	}

	#[test]
	fn lerping() {
		assert_eq!(0x7f112233, lerp_argb32(0x7f112233, 0xffaabbcc, 0.0));
		assert_eq!(0xbf5d6e7f, lerp_argb32(0x7f112233, 0xffaabbcc, 0.5));
		assert_eq!(0xffaabbcc, lerp_argb32(0x7f112233, 0xffaabbcc, 1.0));

		assert_eq!(0xff112233, lerp_rgb32(0x7f112233, 0xffaabbcc, 0.0));
		assert_eq!(0xff5d6e7f, lerp_rgb32(0x7f112233, 0xffaabbcc, 0.5));
		assert_eq!(0xffaabbcc, lerp_rgb32(0x7f112233, 0xffaabbcc, 1.0));
	}

	#[test]
	fn argbu8x4() {
		let mut color = ARGBu8x4(simd::u8x4::from_array([0x11, 0x22, 0x33, 0x44]));
		assert_eq!(color.a(), 0x11);
		assert_eq!(color.r(), 0x22);
		assert_eq!(color.g(), 0x33);
		assert_eq!(color.b(), 0x44);
		assert_eq!(color.to_array(), [0x11, 0x22, 0x33, 0x44]);

		color.set_a(0x55);
		assert_eq!(color.to_array(), [0x55, 0x22, 0x33, 0x44]);
		color.set_r(0x66);
		assert_eq!(color.to_array(), [0x55, 0x66, 0x33, 0x44]);
		color.set_g(0x77);
		assert_eq!(color.to_array(), [0x55, 0x66, 0x77, 0x44]);
		color.set_b(0x88);
		assert_eq!(color.to_array(), [0x55, 0x66, 0x77, 0x88]);

		let color = ARGBu8x4::from_argb([0x11, 0x22, 0x33, 0x44]);
		assert_eq!(color.to_array(), [0x11, 0x22, 0x33, 0x44]);

		let color = ARGBu8x4::from_rgb([0x11, 0x22, 0x33]);
		assert_eq!(color.to_array(), [0xff, 0x11, 0x22, 0x33]);

		let color: ARGBu8x4 = 0x11223344.into();
		assert_eq!(color.to_array(), [0x11, 0x22, 0x33, 0x44]);

		let other = ARGBf32x4::from_argb([0.5, 0.1, 0.2, 0.3]);
		let color: ARGBu8x4 = other.into();
		assert_eq!(color.to_array(), [0x7f, 0x19, 0x33, 0x4c]);
	}

	#[test]
	fn argbu8x4_multiplication() {
		assert_eq!([0xff, 0x11, 0x22, 0x33], (ARGBu8x4::from(0xffffffff) * ARGBu8x4::from(0xff112233)).to_array());
		assert_eq!([0xff, 0x11, 0x22, 0x33], (ARGBu8x4::from(0xff112233) * ARGBu8x4::from(0xffffffff)).to_array());
		assert_eq!([0x7f, 0x03, 0x00, 0x14], (ARGBu8x4::from(0x7f330066) * ARGBu8x4::from(0xff112233)).to_array());
		assert_eq!([0x7f, 0x03, 0x00, 0x14], (ARGBu8x4::from(0xff112233) * ARGBu8x4::from(0x7f330066)).to_array());

		let mut color = ARGBu8x4::from(0xffffffff);
		color *= ARGBu8x4::from(0xff112233);
		assert_eq!([0xff, 0x11, 0x22, 0x33], color.to_array());
		let mut color = ARGBu8x4::from(0xff112233);
		color *= ARGBu8x4::from(0xffffffff);
		assert_eq!([0xff, 0x11, 0x22, 0x33], color.to_array());
		let mut color = ARGBu8x4::from(0x7f330066);
		color *= ARGBu8x4::from(0xff112233);
		assert_eq!([0x7f, 0x03, 0x00, 0x14], color.to_array());
		let mut color = ARGBu8x4::from(0xff112233);
		color *= ARGBu8x4::from(0x7f330066);
		assert_eq!([0x7f, 0x03, 0x00, 0x14], color.to_array());
	}

	#[test]
	fn argbu8x4_lerping() {
		assert_eq!(
			[0x7f, 0x11, 0x22, 0x33],
			(ARGBu8x4::from(0x7f112233).lerp(ARGBu8x4::from(0xffaabbcc), 0.0).to_array())
		);
		assert_eq!(
			[0xbf, 0x5d, 0x6e, 0x7f],
			(ARGBu8x4::from(0x7f112233).lerp(ARGBu8x4::from(0xffaabbcc), 0.5).to_array())
		);
		assert_eq!(
			[0xff, 0xaa, 0xbb, 0xcc],
			(ARGBu8x4::from(0x7f112233).lerp(ARGBu8x4::from(0xffaabbcc), 1.0).to_array())
		);
	}

	#[test]
	fn argbf32x4() {
		let mut color = ARGBf32x4(simd::f32x4::from_array([0.5, 0.1, 0.2, 0.3]));
		assert_eq!(color.a(), 0.5);
		assert_eq!(color.r(), 0.1);
		assert_eq!(color.g(), 0.2);
		assert_eq!(color.b(), 0.3);
		assert_eq!(color.to_array(), [0.5, 0.1, 0.2, 0.3]);

		color.set_a(1.0);
		assert_eq!(color.to_array(), [1.0, 0.1, 0.2, 0.3]);
		color.set_r(0.4);
		assert_eq!(color.to_array(), [1.0, 0.4, 0.2, 0.3]);
		color.set_g(0.5);
		assert_eq!(color.to_array(), [1.0, 0.4, 0.5, 0.3]);
		color.set_b(0.6);
		assert_eq!(color.to_array(), [1.0, 0.4, 0.5, 0.6]);

		let color = ARGBf32x4::from_argb([0.5, 0.1, 0.2, 0.3]);
		assert_eq!(color.to_array(), [0.5, 0.1, 0.2, 0.3]);

		let color = ARGBf32x4::from_rgb([0.1, 0.2, 0.3]);
		assert_eq!(color.to_array(), [1.0, 0.1, 0.2, 0.3]);

		let other = ARGBu8x4::from_argb([0x7f, 0x19, 0x33, 0x4c]);
		let color: ARGBf32x4 = other.into();
		assert!(color.a().nearly_equal(0.5, 0.01));
		assert!(color.r().nearly_equal(0.1, 0.01));
		assert!(color.g().nearly_equal(0.2, 0.01));
		assert!(color.b().nearly_equal(0.3, 0.01));
	}
}
