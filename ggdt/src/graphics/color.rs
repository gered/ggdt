// these colours are taken from the default VGA palette with a simple conversion from their original
// 0-63 range to 0-255.

pub const COLOR_BLACK: u32 = 0xff000000;
pub const COLOR_BLUE: u32 = 0xff0000a8;
pub const COLOR_GREEN: u32 = 0xff00a800;
pub const COLOR_CYAN: u32 = 0xff00a8a8;
pub const COLOR_RED: u32 = 0xffa80000;
pub const COLOR_MAGENTA: u32 = 0xffa800a8;
pub const COLOR_BROWN: u32 = 0xffa85400;
pub const COLOR_LIGHT_GRAY: u32 = 0xffa8a8a8;
pub const COLOR_DARK_GRAY: u32 = 0xff545454;
pub const COLOR_BRIGHT_BLUE: u32 = 0xff5454fc;
pub const COLOR_BRIGHT_GREEN: u32 = 0xff54fc54;
pub const COLOR_BRIGHT_CYAN: u32 = 0xff54fcfc;
pub const COLOR_BRIGHT_RED: u32 = 0xfffc5454;
pub const COLOR_BRIGHT_MAGENTA: u32 = 0xfffc54fc;
pub const COLOR_BRIGHT_YELLOW: u32 = 0xfffcfc54;
pub const COLOR_BRIGHT_WHITE: u32 = 0xfffcfcfc;

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
	(LUMINANCE_RED * srgb_to_linearized(r)) +
		(LUMINANCE_GREEN * srgb_to_linearized(g)) +
		(LUMINANCE_BLUE * srgb_to_linearized(b))
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
