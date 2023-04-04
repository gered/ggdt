// these colours are taken from the default VGA palette

pub const COLOR_BLACK: u32 = 0xff000000;
pub const COLOR_BLUE: u32 = 0xff0000aa;
pub const COLOR_GREEN: u32 = 0xff00aa00;
pub const COLOR_CYAN: u32 = 0xff00aaaa;
pub const COLOR_RED: u32 = 0xffaa0000;
pub const COLOR_MAGENTA: u32 = 0xffaa00aa;
pub const COLOR_BROWN: u32 = 0xffaa5500;
pub const COLOR_LIGHT_GRAY: u32 = 0xffaaaaaa;
pub const COLOR_DARK_GRAY: u32 = 0xff555555;
pub const COLOR_BRIGHT_BLUE: u32 = 0xff5555ff;
pub const COLOR_BRIGHT_GREEN: u32 = 0xff55ff55;
pub const COLOR_BRIGHT_CYAN: u32 = 0xff55ffff;
pub const COLOR_BRIGHT_RED: u32 = 0xffff5555;
pub const COLOR_BRIGHT_MAGENTA: u32 = 0xffff55ff;
pub const COLOR_BRIGHT_YELLOW: u32 = 0xffffff55;
pub const COLOR_BRIGHT_WHITE: u32 = 0xffffffff;

// TODO: probably should name these better, after i do much more reading on the subject :-)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlendFunction {
	Blend,
	BlendSourceWithAlpha(u8),
	TintedBlend(u32),
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
	pub fn blend(&self, src: u32, dest: u32) -> u32 {
		use BlendFunction::*;
		match self {
			Blend => blend_argb32(src, dest),
			BlendSourceWithAlpha(opacity) => blend_argb32_source_by(src, dest, *opacity),
			TintedBlend(tint) => {
				let tinted = tint_argb32(src, *tint);
				blend_argb32(tinted, dest)
			}
		}
	}
}

/// Converts a set of individual ARGB components to a combined 32-bit color value, packed into
/// the format 0xAARRGGBB
///
/// # Arguments
///
/// * `a`: the alpha component (0-255)
/// * `r`: the red component (0-255)
/// * `g`: the green component (0-255)
/// * `b`: the blue component (0-255)
///
/// returns: the u32 packed color
#[inline]
pub fn to_argb32(a: u8, r: u8, g: u8, b: u8) -> u32 {
	(b as u32) + ((g as u32) << 8) + ((r as u32) << 16) + ((a as u32) << 24)
}

/// Converts a set of individual ARGB normalized components to a combined 32-bit color value,
/// packed into the format 0xAARRGGBB.
///
/// # Arguments
///
/// * `a`: the normalized alpha component (0.0 to 1.0)
/// * `r`: the normalized red component (0.0 to 1.0)
/// * `g`: the normalized green component (0.0 to 1.0)
/// * `b`: the normalized blue component (0.0 to 1.0)
///
/// returns: the u32 packed color
#[inline]
pub fn to_argb32_normalized(a: f32, r: f32, g: f32, b: f32) -> u32 {
	(((b * 255.0) as u32) & 0xff)
		+ ((((g * 255.0) as u32) & 0xff) << 8)
		+ ((((r * 255.0) as u32) & 0xff) << 16)
		+ ((((a * 255.0) as u32) & 0xff) << 24)
}

/// Extracts the individual ARGB components out of a combined 32-bit color value which is in the
/// format 0xAARRGGBB
///
/// # Arguments
///
/// * `argb`: the 32-bit packed color
///
/// returns: the individual ARGB color components (0-255 each) in order: alpha, red, green, blue
#[inline]
pub fn from_argb32(argb: u32) -> (u8, u8, u8, u8) {
	let a = ((argb & 0xff000000) >> 24) as u8;
	let r = ((argb & 0x00ff0000) >> 16) as u8;
	let g = ((argb & 0x0000ff00) >> 8) as u8;
	let b = (argb & 0x000000ff) as u8;
	(a, r, g, b)
}

/// Extracts the individual ARGB normalized components out of a combined 32-bit color value which
/// is in the format 0xAARRGGBB
///
/// # Arguments
///
/// * `argb`: the 32-bit packed color
///
/// returns: the individual ARGB normalized color components (0.0 to 1.0 each) in order: alpha, red, green, blue
#[inline]
pub fn from_argb32_normalized(argb: u32) -> (f32, f32, f32, f32) {
	let a = ((argb & 0xff000000) >> 24) as f32 / 255.0;
	let r = ((argb & 0x00ff0000) >> 16) as f32 / 255.0;
	let g = ((argb & 0x0000ff00) >> 8) as f32 / 255.0;
	let b = (argb & 0x000000ff) as f32 / 255.0;
	(a, r, g, b)
}

/// Converts a set of individual RGB components to a combined 32-bit color value, packed into
/// the format 0xAARRGGBB. Substitutes a value of 255 for the missing alpha component.
///
/// # Arguments
///
/// * `r`: the red component (0-255)
/// * `g`: the green component (0-255)
/// * `b`: the blue component (0-255)
///
/// returns: the u32 packed color
#[inline]
pub fn to_rgb32(r: u8, g: u8, b: u8) -> u32 {
	to_argb32(255, r, g, b)
}

/// Converts a set of individual RGB normalized components to a combined 32-bit color value, packed
/// into the format 0xAARRGGBB. Substitutes a value of 1.0 for the missing alpha component.
///
/// # Arguments
///
/// * `r`: the normalized red component (0.0 to 1.0)
/// * `g`: the normalized green component (0.0 to 1.0)
/// * `b`: the normalized blue component (0.0 to 1.0)
///
/// returns: the u32 packed color
#[inline]
pub fn to_rgb32_normalized(r: f32, g: f32, b: f32) -> u32 {
	to_argb32_normalized(1.0, r, g, b)
}

/// Extracts the individual RGB components out of a combined 32-bit color value which is in the
/// format 0xAARRGGBB. Ignores the alpha component.
///
/// # Arguments
///
/// * `argb`: the 32-bit packed color
///
/// returns: the individual ARGB color components (0-255 each) in order: red, green, blue
#[inline]
pub fn from_rgb32(rgb: u32) -> (u8, u8, u8) {
	// ignore alpha component at 0xff000000 ...
	let r = ((rgb & 0x00ff0000) >> 16) as u8;
	let g = ((rgb & 0x0000ff00) >> 8) as u8;
	let b = (rgb & 0x000000ff) as u8;
	(r, g, b)
}

/// Extracts the individual RGB normalized components out of a combined 32-bit color value which
/// is in the format 0xAARRGGBB. Ignores the alpha component.
///
/// # Arguments
///
/// * `argb`: the 32-bit packed color
///
/// returns: the individual ARGB normalized color components (0.0 to 1.0 each) in order: red, green, blue
#[inline]
pub fn from_rgb32_normalized(rgb: u32) -> (f32, f32, f32) {
	// ignore alpha component at 0xff000000 ...
	let r = ((rgb & 0x00ff0000) >> 16) as f32 / 255.0;
	let g = ((rgb & 0x0000ff00) >> 8) as f32 / 255.0;
	let b = (rgb & 0x000000ff) as f32 / 255.0;
	(r, g, b)
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

/// Alpha blends the components of a source and destination color. Both colors should be 32-bit
/// packed colors in the format 0xAARRGGBB.
///
/// # Arguments
///
/// * `src`: the source color that is to be blended onto the destination
/// * `dest`: the destination color that the source is being blended into
///
/// returns: the blended result
#[inline]
pub fn blend_argb32(src: u32, dest: u32) -> u32 {
	let (src_a, src_r, src_g, src_b) = from_argb32(src);
	let (dest_a, dest_r, dest_g, dest_b) = from_argb32(dest);
	to_argb32(
		blend_components(src_a, src_a, dest_a),
		blend_components(src_a, src_r, dest_r),
		blend_components(src_a, src_g, dest_g),
		blend_components(src_a, src_b, dest_b),
	)
}

/// Blends the source and destination colors together, where the alpha value used to blend the two
/// colors is derived from the given alpha value multiplied with the source color's alpha component.
/// This allows for more flexibility in directly controling how transparent the source
/// color is overtop of the destination. Both colors should be 32-bit packed colors in the format
/// 0xAARRGGBB.
///
/// # Arguments
///
/// * `src`: the source color that is to be blended onto the destination. the alpha component of this
///          color is used during the blend.
/// * `dest`: the destination color that the source is being blended into. the alpha component of this
///           color is ignored.
/// * `alpha`: the transparency or opacity of the source color over the destination color. this is
///            multipled together with the source color's alpha component to arrive at the final
///            alpha value used for blending the source and destination color's RGB components.
///
/// returns: the blended result
#[inline]
pub fn blend_argb32_source_by(src: u32, dest: u32, alpha: u8) -> u32 {
	let (src_a, src_r, src_g, src_b) = from_argb32(src);
	let (dest_r, dest_g, dest_b) = from_rgb32(dest);
	let alpha = ((alpha as u16 * src_a as u16) / 255) as u8;
	to_argb32(
		alpha,
		blend_components(alpha, src_r, dest_r),
		blend_components(alpha, src_g, dest_g),
		blend_components(alpha, src_b, dest_b),
	)
}

/// Applies a tint to a color, using the tint color's alpha component as the strength of the tint,
/// where 0 means no tint and 255 means full tint. The original color's alpha component is preserved in
/// the result. Both the source color and tint color should be 32-bit packed colors in the format
/// 0xAARRGGBB.
///
/// # Arguments
///
/// * `color`: the color to be tinted
/// * `tint`: the tint to be applied to the color, where the alpha component represents the tint strength
///
/// returns: the resulting tinted color
#[inline]
pub fn tint_argb32(color: u32, tint: u32) -> u32 {
	let (color_a, color_r, color_g, color_b) = from_argb32(color);
	let (tint_a, tint_r, tint_g, tint_b) = from_argb32(tint);
	to_argb32(
		color_a,
		blend_components(tint_a, tint_r, color_r),
		blend_components(tint_a, tint_g, color_g),
		blend_components(tint_a, tint_b, color_b),
	)
}

/// Multiplies two colors together, returing the result. The multiplication is performed by
/// individually multiplying each color component using the formula `(component * component) / 255`.
/// Both colors should be 32-bit packed colors in the format 0xAARRGGBB.
///
/// # Arguments
///
/// * `a`: the first 32-bit packed color
/// * `b`: the second 32-bit packed color
///
/// returns: the resulting color from the multiplication
#[inline]
pub fn multiply_argb32(a: u32, b: u32) -> u32 {
	let (a_a, a_r, a_g, a_b) = from_argb32(a);
	let (b_a, b_r, b_g, b_b) = from_argb32(b);
	to_argb32(
		((a_a as u32 * b_a as u32) / 255) as u8,
		((a_r as u32 * b_r as u32) / 255) as u8,
		((a_g as u32 * b_g as u32) / 255) as u8,
		((a_b as u32 * b_b as u32) / 255) as u8,
	)
}

/// Linearly interpolates between two 32-bit packed colors in the format 0xAARRGGBB.
///
/// # Arguments
///
/// * `a`: the first 32-bit packed color
/// * `b`: the second 32-bit packed color
/// * `t`: the amount to interpolate between the two values, specified as a fraction.
#[inline]
pub fn lerp_argb32(a: u32, b: u32, t: f32) -> u32 {
	let (a1, r1, g1, b1) = from_argb32(a);
	let (a2, r2, g2, b2) = from_argb32(b);
	to_argb32(
		((a1 as f32) + ((a2 as f32) - (a1 as f32)) * t) as u8,
		((r1 as f32) + ((r2 as f32) - (r1 as f32)) * t) as u8,
		((g1 as f32) + ((g2 as f32) - (g1 as f32)) * t) as u8,
		((b1 as f32) + ((b2 as f32) - (b1 as f32)) * t) as u8,
	)
}

/// Linearly interpolates between two 32-bit packed colors in the format 0xAARRGGBB. Ignores the
/// alpha component, which will always be set to 255 in the return value.
///
/// # Arguments
///
/// * `a`: the first 32-bit packed color
/// * `b`: the second 32-bit packed color
/// * `t`: the amount to interpolate between the two values, specified as a fraction.
#[inline]
pub fn lerp_rgb32(a: u32, b: u32, t: f32) -> u32 {
	let (r1, g1, b1) = from_rgb32(a);
	let (r2, g2, b2) = from_rgb32(b);
	to_rgb32(
		((r1 as f32) + ((r2 as f32) - (r1 as f32)) * t) as u8,
		((g1 as f32) + ((g2 as f32) - (g1 as f32)) * t) as u8,
		((b1 as f32) + ((b2 as f32) - (b1 as f32)) * t) as u8,
	)
}

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
pub fn luminance(r: u8, g: u8, b: u8) -> f32 {
	(LUMINANCE_RED * srgb_to_linearized(r))
		+ (LUMINANCE_GREEN * srgb_to_linearized(g))
		+ (LUMINANCE_BLUE * srgb_to_linearized(b))
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
pub fn greyscale(r: u8, b: u8, g: u8) -> u8 {
	(brightness(luminance(r, g, b)) * 255.0) as u8
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::math::NearlyEqual;

	#[test]
	fn argb_conversions() {
		let argb = to_argb32(0x11, 0x22, 0x33, 0x44);
		assert_eq!(argb, 0x11223344);

		let argb = to_rgb32(0x22, 0x33, 0x44);
		assert_eq!(argb, 0xff223344);

		let (a, r, g, b) = from_argb32(0x11223344);
		assert_eq!(0x11, a);
		assert_eq!(0x22, r);
		assert_eq!(0x33, g);
		assert_eq!(0x44, b);

		let (r, g, b) = from_rgb32(0x11223344);
		assert_eq!(0x22, r);
		assert_eq!(0x33, g);
		assert_eq!(0x44, b);
	}

	#[test]
	fn normalized_argb_conversions() {
		let argb = to_argb32_normalized(0.5, 0.1, 0.2, 0.3);
		assert_eq!(argb, 0x7f19334c);

		let argb = to_rgb32_normalized(0.1, 0.2, 0.3);
		assert_eq!(argb, 0xff19334c);

		// floating-point accuracy is a real bitch here ... lol.
		// the low-accuracy epsilon values in these asserts is not an accident or oversight

		let (a, r, g, b) = from_argb32_normalized(0x7f19334c);
		assert!(a.nearly_equal(0.5, 0.01));
		assert!(r.nearly_equal(0.1, 0.01));
		assert!(g.nearly_equal(0.2, 0.01));
		assert!(b.nearly_equal(0.3, 0.01));

		let (r, g, b) = from_rgb32_normalized(0x7f19334c);
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
}
