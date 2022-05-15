use std::cmp::min;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor};
use std::ops::{Bound, Index, IndexMut, RangeBounds};
use std::path::Path;

use byteorder::{ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::utils::abs_diff;
use crate::NUM_COLORS;

// silly "hack" (???) which allows us to alias the generic constraint `RangeBounds<u8> + Iterator<Item = u8>` to `ColorRange`
pub trait ColorRange: RangeBounds<u8> + Iterator<Item = u8> {}
impl<T> ColorRange for T where T: RangeBounds<u8> + Iterator<Item = u8> {}

pub static VGA_PALETTE_BYTES: &[u8] = include_bytes!("../../assets/vga.pal");

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

// vga bios (0-63) format
fn read_256color_6bit_palette<T: ReadBytesExt>(
    reader: &mut T,
) -> Result<[u32; NUM_COLORS], PaletteError> {
    let mut colors = [0u32; NUM_COLORS];
    for color in colors.iter_mut() {
        let r = reader.read_u8()?;
        let g = reader.read_u8()?;
        let b = reader.read_u8()?;
        *color = to_rgb32(r * 4, g * 4, b * 4);
    }
    Ok(colors)
}

fn write_256color_6bit_palette<T: WriteBytesExt>(
    writer: &mut T,
    colors: &[u32; NUM_COLORS],
) -> Result<(), PaletteError> {
    for color in colors.iter() {
        let (r, g, b) = from_rgb32(*color);
        writer.write_u8(r / 4)?;
        writer.write_u8(g / 4)?;
        writer.write_u8(b / 4)?;
    }
    Ok(())
}

// normal (0-255) format
fn read_256color_8bit_palette<T: ReadBytesExt>(
    reader: &mut T,
) -> Result<[u32; NUM_COLORS], PaletteError> {
    let mut colors = [0u32; NUM_COLORS];
    for color in colors.iter_mut() {
        let r = reader.read_u8()?;
        let g = reader.read_u8()?;
        let b = reader.read_u8()?;
        *color = to_rgb32(r, g, b);
    }
    Ok(colors)
}

fn write_256color_8bit_palette<T: WriteBytesExt>(
    writer: &mut T,
    colors: &[u32; NUM_COLORS],
) -> Result<(), PaletteError> {
    for color in colors.iter() {
        let (r, g, b) = from_rgb32(*color);
        writer.write_u8(r)?;
        writer.write_u8(g)?;
        writer.write_u8(b)?;
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum PaletteError {
    #[error("Palette I/O error")]
    IOError(#[from] std::io::Error),
}

pub enum PaletteFormat {
    /// Individual RGB components in 6-bits (0-63) for VGA BIOS compatibility
    Vga,
    /// Individual RGB components in 8-bits (0-255)
    Normal,
}

/// Contains a 256 color palette, and provides methods useful for working with palettes. The
/// colors are all stored individually as 32-bit packed values in the format 0xAARRGGBB.
#[derive(Debug, Clone)]
pub struct Palette {
    colors: [u32; NUM_COLORS],
}

impl Palette {
    /// Creates a new Palette with all black colors.
    pub fn new() -> Palette {
        Palette {
            colors: [0; NUM_COLORS],
        }
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
    pub fn load_from_bytes<T: ReadBytesExt>(
        reader: &mut T,
        format: PaletteFormat,
    ) -> Result<Palette, PaletteError> {
        let colors = match format {
            PaletteFormat::Vga => read_256color_6bit_palette(reader)?,
            PaletteFormat::Normal => read_256color_8bit_palette(reader)?,
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
    pub fn to_bytes<T: WriteBytesExt>(
        &self,
        writer: &mut T,
        format: PaletteFormat,
    ) -> Result<(), PaletteError> {
        match format {
            PaletteFormat::Vga => write_256color_6bit_palette(writer, &self.colors),
            PaletteFormat::Normal => write_256color_8bit_palette(writer, &self.colors),
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
    pub fn fade_color_toward_rgb(
        &mut self,
        color: u8,
        target_r: u8,
        target_g: u8,
        target_b: u8,
        step: u8,
    ) -> bool {
        let mut modified = false;

        let (mut r, mut g, mut b) = from_rgb32(self.colors[color as usize]);

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
            self.colors[color as usize] = to_rgb32(r, g, b);
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
    pub fn fade_colors_toward_palette<T: ColorRange>(
        &mut self,
        colors: T,
        palette: &Palette,
        step: u8,
    ) -> bool {
        let mut all_faded = true;
        for color in colors {
            let (r, g, b) = from_rgb32(palette[color]);
            if !self.fade_color_toward_rgb(color, r, g, b, step) {
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
            self[color] = lerp_rgb32(a[color], b[color], t);
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
            -1 => subset.rotate_left(step.abs() as usize),
            1 => subset.rotate_right(step.abs() as usize),
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
            let (this_r, this_g, this_b) = from_rgb32(*color);

            // this comparison method is using the sRGB Euclidean formula described here:
            // https://en.wikipedia.org/wiki/Color_difference

            let distance = abs_diff(this_r, r) as u32
                + abs_diff(this_g, g) as u32
                + abs_diff(this_b, b) as u32;

            if distance < closest_distance {
                closest = index as u8;
                closest_distance = distance;
            }
        }

        closest
    }
}

impl Index<u8> for Palette {
    type Output = u32;

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

impl PartialEq for Palette {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.colors == other.colors
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

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
    fn get_and_set_colors() {
        let mut palette = Palette::new();
        assert_eq!(0, palette[0]);
        assert_eq!(0, palette[1]);
        palette[0] = 0x11223344;
        assert_eq!(0x11223344, palette[0]);
        assert_eq!(0, palette[1]);
    }

    fn assert_vga_palette(palette: &Palette) {
        assert_eq!(0xff000000, palette[0]);
        assert_eq!(0xff0000a8, palette[1]);
        assert_eq!(0xff00a800, palette[2]);
        assert_eq!(0xff00a8a8, palette[3]);
        assert_eq!(0xffa80000, palette[4]);
        assert_eq!(0xffa800a8, palette[5]);
        assert_eq!(0xffa85400, palette[6]);
        assert_eq!(0xffa8a8a8, palette[7]);
        assert_eq!(0xff545454, palette[8]);
        assert_eq!(0xff5454fc, palette[9]);
        assert_eq!(0xff54fc54, palette[10]);
        assert_eq!(0xff54fcfc, palette[11]);
        assert_eq!(0xfffc5454, palette[12]);
        assert_eq!(0xfffc54fc, palette[13]);
        assert_eq!(0xfffcfc54, palette[14]);
        assert_eq!(0xfffcfcfc, palette[15]);
    }

    #[test]
    fn load_and_save() -> Result<(), PaletteError> {
        let tmp_dir = TempDir::new()?;

        // vga format

        let palette = Palette::load_from_file(Path::new("./assets/vga.pal"), PaletteFormat::Vga)?;
        assert_vga_palette(&palette);

        let save_path = tmp_dir.path().join("test_save_vga_format.pal");
        palette.to_file(&save_path, PaletteFormat::Vga)?;
        let reloaded_palette = Palette::load_from_file(&save_path, PaletteFormat::Vga)?;
        assert_eq!(palette, reloaded_palette);

        // normal format

        let palette =
            Palette::load_from_file(Path::new("./test-assets/dp2.pal"), PaletteFormat::Normal)?;

        let save_path = tmp_dir.path().join("test_save_normal_format.pal");
        palette.to_file(&save_path, PaletteFormat::Normal)?;
        let reloaded_palette = Palette::load_from_file(&save_path, PaletteFormat::Normal)?;
        assert_eq!(palette, reloaded_palette);

        Ok(())
    }
}
