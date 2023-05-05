use std::cmp::min;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor};
use std::ops::{Bound, Index, IndexMut, RangeBounds};
use std::path::Path;

use byteorder::{ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::graphics::{IndexedBitmap, RGBA};
use crate::utils::abs_diff;

const NUM_COLORS: usize = 256;

/// Common trait to represent a range of indexed colour values.
pub trait ColorRange: RangeBounds<u8> + Iterator<Item = u8> {}
impl<T> ColorRange for T where T: RangeBounds<u8> + Iterator<Item = u8> {}

pub static VGA_PALETTE_BYTES: &[u8] = include_bytes!("../../assets/vga.pal");

#[inline]
fn from_6bit(value: u8) -> u8 {
	value.wrapping_shl(2) | value.wrapping_shr(4)
}

#[inline]
fn to_6bit(value: u8) -> u8 {
	value.wrapping_shr(2)
}

// vga bios (0-63) format
fn read_palette_6bit<T: ReadBytesExt>(reader: &mut T, num_colors: usize) -> Result<[RGBA; NUM_COLORS], PaletteError> {
	if num_colors > NUM_COLORS {
		return Err(PaletteError::OutOfRange(num_colors));
	}
	let mut colors = [RGBA::from_rgba([0, 0, 0, 255]); NUM_COLORS];
	for i in 0..num_colors {
		let r = reader.read_u8()?;
		let g = reader.read_u8()?;
		let b = reader.read_u8()?;
		let color = RGBA::from_rgb([from_6bit(r), from_6bit(g), from_6bit(b)]);
		colors[i] = color;
	}
	Ok(colors)
}

fn write_palette_6bit<T: WriteBytesExt>(
	writer: &mut T,
	colors: &[RGBA; NUM_COLORS],
	num_colors: usize,
) -> Result<(), PaletteError> {
	if num_colors > NUM_COLORS {
		return Err(PaletteError::OutOfRange(num_colors));
	}
	for i in 0..num_colors {
		writer.write_u8(to_6bit(colors[i].r()))?;
		writer.write_u8(to_6bit(colors[i].g()))?;
		writer.write_u8(to_6bit(colors[i].b()))?;
	}
	Ok(())
}

// normal (0-255) format
fn read_palette_8bit<T: ReadBytesExt>(reader: &mut T, num_colors: usize) -> Result<[RGBA; NUM_COLORS], PaletteError> {
	if num_colors > NUM_COLORS {
		return Err(PaletteError::OutOfRange(num_colors));
	}
	let mut colors = [RGBA::from_rgba([0, 0, 0, 255]); NUM_COLORS];
	for i in 0..num_colors {
		let r = reader.read_u8()?;
		let g = reader.read_u8()?;
		let b = reader.read_u8()?;
		let color = RGBA::from_rgb([r, g, b]);
		colors[i] = color;
	}
	Ok(colors)
}

fn write_palette_8bit<T: WriteBytesExt>(
	writer: &mut T,
	colors: &[RGBA; NUM_COLORS],
	num_colors: usize,
) -> Result<(), PaletteError> {
	if num_colors > NUM_COLORS {
		return Err(PaletteError::OutOfRange(num_colors));
	}
	for i in 0..num_colors {
		writer.write_u8(colors[i].r())?;
		writer.write_u8(colors[i].g())?;
		writer.write_u8(colors[i].b())?;
	}
	Ok(())
}

#[derive(Error, Debug)]
pub enum PaletteError {
	#[error("Palette I/O error")]
	IOError(#[from] std::io::Error),

	#[error("Size or index is out of the supported range for palettes: {0}")]
	OutOfRange(usize),
}

pub enum PaletteFormat {
	/// Individual RGB components in 6-bits (0-63) for VGA BIOS compatibility
	Vga,
	/// Individual RGB components in 8-bits (0-255)
	Normal,
}

/// Contains a 256 color palette, and provides methods useful for working with palettes. The
/// colors are all stored individually as 32-bit packed values in the format 0xAARRGGBB.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Palette {
	colors: [RGBA; NUM_COLORS],
}

impl Palette {
	/// Creates a new Palette with all black colors.
	pub fn new() -> Palette {
		Palette { colors: [RGBA::from_rgb([0, 0, 0]); NUM_COLORS] }
	}

	/// Creates a new Palette with all initial colors having the RGB values specified.
	pub fn new_with_default(r: u8, g: u8, b: u8) -> Palette {
		Palette { colors: [RGBA::from_rgb([r, g, b]); NUM_COLORS] }
	}

	/// Creates a new Palette, pre-loaded with the default VGA BIOS colors.
	pub fn new_vga_palette() -> Result<Palette, PaletteError> {
		Palette::load_from_bytes(&mut Cursor::new(VGA_PALETTE_BYTES), PaletteFormat::Vga)
	}

	/// Loads and returns a Palette from a palette file on disk.
	///
	/// # Arguments
	///
	/// * `path`: the path of the palette file to be loaded
	/// * `format`: the format that the palette data is expected to be in
	pub fn load_from_file(path: &Path, format: PaletteFormat) -> Result<Palette, PaletteError> {
		let f = File::open(path)?;
		let mut reader = BufReader::new(f);
		Self::load_from_bytes(&mut reader, format)
	}

	/// Loads and returns a Palette from a reader. The data being loaded is expected to be the same
	/// as if the palette was being loaded from a file on disk.
	///
	/// # Arguments
	///
	/// * `reader`: the reader to load the palette from
	/// * `format`: the format that the palette data is expected to be in
	pub fn load_from_bytes<T: ReadBytesExt>(reader: &mut T, format: PaletteFormat) -> Result<Palette, PaletteError> {
		let colors = match format {
			PaletteFormat::Vga => read_palette_6bit(reader, NUM_COLORS)?,
			PaletteFormat::Normal => read_palette_8bit(reader, NUM_COLORS)?,
		};
		Ok(Palette { colors })
	}

	/// Loads and returns a Palette from a palette file on disk, where the palette only contains
	/// the number of colors specified, less than or equal to 256 otherwise an error is returned.
	/// The remaining color entries will all be 0,0,0 (black) in the returned palette.
	///
	/// # Arguments
	///
	/// * `path`: the path of the palette file to be loaded
	/// * `format`: the format that the palette data is expected to be in
	/// * `num_colors`: the expected number of colors in the palette to be loaded (<= 256)
	pub fn load_num_colors_from_file(
		path: &Path,
		format: PaletteFormat,
		num_colors: usize,
	) -> Result<Palette, PaletteError> {
		let f = File::open(path)?;
		let mut reader = BufReader::new(f);
		Self::load_num_colors_from_bytes(&mut reader, format, num_colors)
	}

	/// Loads and returns a Palette from a reader. The data being loaded is expected to be the same
	/// as if the palette was being loaded from a file on disk. The palette being read should only
	/// contain the number of colors specified, less than or equal to 256 otherwise an error is
	/// returned. The remaining color entries will all be 0,0,0 (black) in the returned palette.
	///
	/// # Arguments
	///
	/// * `reader`: the reader to load the palette from
	/// * `format`: the format that the palette data is expected to be in
	/// * `num_colors`: the expected number of colors in the palette to be loaded (<= 256)
	pub fn load_num_colors_from_bytes<T: ReadBytesExt>(
		reader: &mut T,
		format: PaletteFormat,
		num_colors: usize,
	) -> Result<Palette, PaletteError> {
		if num_colors > NUM_COLORS {
			return Err(PaletteError::OutOfRange(num_colors));
		}
		let colors = match format {
			PaletteFormat::Vga => read_palette_6bit(reader, num_colors)?,
			PaletteFormat::Normal => read_palette_8bit(reader, num_colors)?,
		};
		Ok(Palette { colors })
	}

	/// Writes the palette to a file on disk. If the file already exists, it will be overwritten.
	///
	/// # Arguments
	///
	/// * `path`: the path of the file to save the palette to
	/// * `format`: the format to write the palette data in
	pub fn to_file(&self, path: &Path, format: PaletteFormat) -> Result<(), PaletteError> {
		let f = File::create(path)?;
		let mut writer = BufWriter::new(f);
		self.to_bytes(&mut writer, format)
	}

	/// Writes the palette to a writer, in the same format as if it was writing to a file on disk.
	///
	/// # Arguments
	///
	/// * `writer`: the writer to write palette data to
	/// * `format`: the format to write the palette data in
	pub fn to_bytes<T: WriteBytesExt>(&self, writer: &mut T, format: PaletteFormat) -> Result<(), PaletteError> {
		match format {
			PaletteFormat::Vga => write_palette_6bit(writer, &self.colors, NUM_COLORS),
			PaletteFormat::Normal => write_palette_8bit(writer, &self.colors, NUM_COLORS),
		}
	}

	/// Writes the palette to a file on disk. If the file already exists, it will be overwritten.
	/// Will only write out the specified number of colors to the palette file being written,
	/// starting from the first color in the palette always. If the color count specified is
	/// greater than 256 an error is returned.
	///
	/// # Arguments
	///
	/// * `path`: the path of the file to save the palette to
	/// * `format`: the format to write the palette data in
	/// * `num_colors`: the number of colors from this palette to write out to the file (<= 256)
	pub fn num_colors_to_file(
		&self,
		path: &Path,
		format: PaletteFormat,
		num_colors: usize,
	) -> Result<(), PaletteError> {
		if num_colors > NUM_COLORS {
			return Err(PaletteError::OutOfRange(num_colors));
		}
		let f = File::create(path)?;
		let mut writer = BufWriter::new(f);
		self.num_colors_to_bytes(&mut writer, format, num_colors)
	}

	/// Writes the palette to a writer, in the same format as if it was writing to a file on disk.
	/// Will only write out the specified number of colors to the writer, starting from the first
	/// color in the palette always. If the color count specified is greater than 256 an error is
	/// returned.
	///
	/// # Arguments
	///
	/// * `writer`: the writer to write palette data to
	/// * `format`: the format to write the palette data in
	/// * `num_colors`: the number of colors from this palette to write out (<= 256)
	pub fn num_colors_to_bytes<T: WriteBytesExt>(
		&self,
		writer: &mut T,
		format: PaletteFormat,
		num_colors: usize,
	) -> Result<(), PaletteError> {
		if num_colors > NUM_COLORS {
			return Err(PaletteError::OutOfRange(num_colors));
		}
		match format {
			PaletteFormat::Vga => write_palette_6bit(writer, &self.colors, num_colors),
			PaletteFormat::Normal => write_palette_8bit(writer, &self.colors, num_colors),
		}
	}

	/// Fades a single color in the palette from its current RGB values towards the given RGB
	/// values by up to the step amount given. This function is intended to be run many times
	/// over a number of frames where each run completes a small step towards the complete fade.
	///
	/// # Arguments
	///
	/// * `color`: the color index to fade
	/// * `target_r`: the target red component (0-255) to fade towards
	/// * `target_g`: the target green component (0-255) to fade towards
	/// * `target_b`: the target blue component (0-255) to fade towards
	/// * `step`: the amount to "step" by towards the target RGB values
	///
	/// returns: true if the color has reached the target RGB values, false otherwise
	pub fn fade_color_toward_rgb(&mut self, color: u8, target_r: u8, target_g: u8, target_b: u8, step: u8) -> bool {
		let mut modified = false;

		let mut r = self.colors[color as usize].r();
		let mut g = self.colors[color as usize].g();
		let mut b = self.colors[color as usize].b();

		if r != target_r {
			modified = true;
			let diff_r = r.overflowing_sub(target_r).0;
			if r > target_r {
				r -= min(step, diff_r);
			} else {
				r += min(step, diff_r);
			}
		}

		if g != target_g {
			modified = true;
			let diff_g = g.overflowing_sub(target_g).0;
			if g > target_g {
				g -= min(step, diff_g);
			} else {
				g += min(step, diff_g);
			}
		}

		if b != target_b {
			modified = true;
			let diff_b = b.overflowing_sub(target_b).0;
			if b > target_b {
				b -= min(step, diff_b);
			} else {
				b += min(step, diff_b);
			}
		}

		if modified {
			self.colors[color as usize] = RGBA::from_rgb([r, g, b]);
		}

		(target_r == r) && (target_g == g) && (target_b == b)
	}

	/// Fades a range of colors in the palette from their current RGB values all towards the given
	/// RGB values by up to the step amount given. This function is intended to be run many times
	/// over a number of frames where each run completes a small step towards the complete fade.
	///
	/// # Arguments
	///
	/// * `colors`: the range of colors to be faded
	/// * `target_r`: the target red component (0-255) to fade towards
	/// * `target_g`: the target green component (0-255) to fade towards
	/// * `target_b`: the target blue component (0-255) to fade towards
	/// * `step`: the amount to "step" by towards the target RGB values
	///
	/// returns: true if all of the colors in the range have reached the target RGB values, false
	/// otherwise
	pub fn fade_colors_toward_rgb<T: ColorRange>(
		&mut self,
		colors: T,
		target_r: u8,
		target_g: u8,
		target_b: u8,
		step: u8,
	) -> bool {
		let mut all_faded = true;
		for color in colors {
			if !self.fade_color_toward_rgb(color, target_r, target_g, target_b, step) {
				all_faded = false;
			}
		}
		all_faded
	}

	/// Fades a range of colors in the palette from their current RGB values all towards the RGB
	/// values in the other palette specified, by up to the step amount given. This function is
	/// intended to be run many times over a number of frames where each run completes a small step
	/// towards the complete fade.
	///
	/// # Arguments
	///
	/// * `colors`: the range of colors to be faded
	/// * `palette`: the other palette to use as the target to fade towards
	/// * `step`: the amount to "step" by towards the target RGB values
	///
	/// returns: true if all of the colors in the range have reached the RGB values from the other
	/// target palette, false otherwise
	pub fn fade_colors_toward_palette<T: ColorRange>(&mut self, colors: T, palette: &Palette, step: u8) -> bool {
		let mut all_faded = true;
		for color in colors {
			if !self.fade_color_toward_rgb(color, palette[color].r(), palette[color].g(), palette[color].b(), step) {
				all_faded = false;
			}
		}
		all_faded
	}

	/// Linearly interpolates between the specified colors in two palettes, storing the
	/// interpolation results in this palette.
	///
	/// # Arguments
	///
	/// * `colors`: the range of colors to be interpolated
	/// * `a`: the first palette
	/// * `b`: the second palette
	/// * `t`: the amount to interpolate between the two palettes, specified as a fraction
	pub fn lerp<T: ColorRange>(&mut self, colors: T, a: &Palette, b: &Palette, t: f32) {
		for color in colors {
			self[color] = a[color].lerp(b[color], t);
		}
	}

	/// Rotates a range of colors in the palette by a given amount.
	///
	/// # Arguments
	///
	/// * `colors`: the range of colors to be rotated
	/// * `step`: the number of positions (and direction) to rotate all colors by
	pub fn rotate_colors<T: ColorRange>(&mut self, colors: T, step: i8) {
		use Bound::*;
		let start = match colors.start_bound() {
			Excluded(&start) => start + 1,
			Included(&start) => start,
			Unbounded => 0,
		} as usize;
		let end = match colors.end_bound() {
			Excluded(&end) => end - 1,
			Included(&end) => end,
			Unbounded => 255,
		} as usize;
		let subset = &mut self.colors[start..=end];
		match step.signum() {
			-1 => subset.rotate_left(step.unsigned_abs() as usize),
			1 => subset.rotate_right(step.unsigned_abs() as usize),
			_ => {}
		}
	}

	/// Finds and returns the index of the closest color in this palette to the RGB values provided.
	/// This will not always return great results. It depends largely on the palette and the RGB
	/// values being searched (for example, searching for bright green 0,255,0 in a palette which
	/// contains no green hues at all is not likely to return a useful result).
	pub fn find_color(&self, r: u8, g: u8, b: u8) -> u8 {
		let mut closest_distance = 255 * 3;
		let mut closest = 0;

		for (index, color) in self.colors.iter().enumerate() {
			if r == color.r() && g == color.g() && b == color.b() {
				return index as u8;
			} else {
				// this comparison method is using the sRGB Euclidean formula described here:
				// https://en.wikipedia.org/wiki/Color_difference

				let distance =
					abs_diff(color.r(), r) as u32 + abs_diff(color.g(), g) as u32 + abs_diff(color.b(), b) as u32;

				if distance < closest_distance {
					closest = index as u8;
					closest_distance = distance;
				}
			}
		}

		closest
	}

	/// Debug helper that draws this palette to the given bitmap as a 16x16 pixel grid, where each
	/// pixel is one of the colors from this palette, in ascending order, left-to-right,
	/// top-to-bottom. The coordinates given specify the top-left coordinate on the destination
	/// bitmap to begin drawing the palette at.
	pub fn draw(&self, dest: &mut IndexedBitmap, x: i32, y: i32) {
		let mut color = 0;
		for yd in 0..16 {
			for xd in 0..16 {
				dest.set_pixel(x + xd, y + yd, color);
				color = color.wrapping_add(1);
			}
		}
	}
}

impl Index<u8> for Palette {
	type Output = RGBA;

	#[inline]
	fn index(&self, index: u8) -> &Self::Output {
		&self.colors[index as usize]
	}
}

impl IndexMut<u8> for Palette {
	#[inline]
	fn index_mut(&mut self, index: u8) -> &mut Self::Output {
		&mut self.colors[index as usize]
	}
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use tempfile::TempDir;

	use crate::graphics::color::*;

	use super::*;

	const BASE_PATH: &str = "./test-assets/palette/";

	fn test_file(file: &Path) -> PathBuf {
		PathBuf::from(BASE_PATH).join(file)
	}

	#[test]
	fn get_and_set_colors() {
		let mut palette = Palette::new();
		assert_eq!(RGBA::from_rgb([0, 0, 0]), palette[0]);
		assert_eq!(RGBA::from_rgb([0, 0, 0]), palette[1]);
		palette[0] = 0x11223344.into();
		assert_eq!(RGBA::from(0x11223344), palette[0]);
		assert_eq!(RGBA::from_rgb([0, 0, 0]), palette[1]);
	}

	fn assert_ega_colors(palette: &Palette) {
		assert_eq!(COLOR_BLACK, palette[0]);
		assert_eq!(COLOR_BLUE, palette[1]);
		assert_eq!(COLOR_GREEN, palette[2]);
		assert_eq!(COLOR_CYAN, palette[3]);
		assert_eq!(COLOR_RED, palette[4]);
		assert_eq!(COLOR_MAGENTA, palette[5]);
		assert_eq!(COLOR_BROWN, palette[6]);
		assert_eq!(COLOR_LIGHT_GRAY, palette[7]);
		assert_eq!(COLOR_DARK_GRAY, palette[8]);
		assert_eq!(COLOR_BRIGHT_BLUE, palette[9]);
		assert_eq!(COLOR_BRIGHT_GREEN, palette[10]);
		assert_eq!(COLOR_BRIGHT_CYAN, palette[11]);
		assert_eq!(COLOR_BRIGHT_RED, palette[12]);
		assert_eq!(COLOR_BRIGHT_MAGENTA, palette[13]);
		assert_eq!(COLOR_BRIGHT_YELLOW, palette[14]);
		assert_eq!(COLOR_BRIGHT_WHITE, palette[15]);
	}

	#[test]
	fn load_and_save() -> Result<(), PaletteError> {
		let tmp_dir = TempDir::new()?;

		// vga rgb format (6-bit)

		let palette = Palette::load_from_file(test_file(Path::new("vga.pal")).as_path(), PaletteFormat::Vga)?;
		assert_ega_colors(&palette);

		let save_path = tmp_dir.path().join("test_save_vga_format.pal");
		palette.to_file(&save_path, PaletteFormat::Vga)?;
		let reloaded_palette = Palette::load_from_file(&save_path, PaletteFormat::Vga)?;
		assert_eq!(palette, reloaded_palette);

		// normal rgb format (8-bit)

		let palette = Palette::load_from_file(test_file(Path::new("dp2.pal")).as_path(), PaletteFormat::Normal)?;

		let save_path = tmp_dir.path().join("test_save_normal_format.pal");
		palette.to_file(&save_path, PaletteFormat::Normal)?;
		let reloaded_palette = Palette::load_from_file(&save_path, PaletteFormat::Normal)?;
		assert_eq!(palette, reloaded_palette);

		Ok(())
	}

	#[test]
	fn load_and_save_arbitrary_color_count() -> Result<(), PaletteError> {
		let tmp_dir = TempDir::new()?;

		// vga rgb format (6-bit)

		let palette =
			Palette::load_num_colors_from_file(test_file(Path::new("ega_6bit.pal")).as_path(), PaletteFormat::Vga, 16)?;
		assert_ega_colors(&palette);

		let save_path = tmp_dir.path().join("test_save_vga_format_16_colors.pal");
		palette.num_colors_to_file(&save_path, PaletteFormat::Vga, 16)?;
		let reloaded_palette = Palette::load_num_colors_from_file(&save_path, PaletteFormat::Vga, 16)?;
		assert_eq!(palette, reloaded_palette);

		// normal rgb format (8-bit)

		let palette = Palette::load_num_colors_from_file(
			test_file(Path::new("ega_8bit.pal")).as_path(),
			PaletteFormat::Normal,
			16,
		)?;

		let save_path = tmp_dir.path().join("test_save_normal_format_16_colors.pal");
		palette.to_file(&save_path, PaletteFormat::Normal)?;
		let reloaded_palette = Palette::load_num_colors_from_file(&save_path, PaletteFormat::Normal, 16)?;
		assert_eq!(palette, reloaded_palette);

		Ok(())
	}
}
