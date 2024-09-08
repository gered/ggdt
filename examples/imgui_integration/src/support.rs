use std::path::Path;

use crate::tilemap::{TILE_HEIGHT, TILE_WIDTH};
use anyhow::{Context, Result};
use ggdt::prelude::*;

pub fn load_palette(path: impl AsRef<Path>) -> Result<Palette> {
	Palette::load_from_file(&path, PaletteFormat::Vga).context(format!("Loading palette: {:?}", path.as_ref()))
}

pub fn load_font(path: impl AsRef<Path>) -> Result<BitmaskFont> {
	BitmaskFont::load_from_file(&path).context(format!("Loading font: {:?}", path.as_ref()))
}

pub fn load_bitmap_atlas_autogrid(path: impl AsRef<Path>) -> Result<BitmapAtlas<RgbaBitmap>> {
	let (bmp, _) = RgbaBitmap::load_file(&path).context(format!("Loading bitmap atlas: {:?}", path.as_ref()))?;
	let mut atlas = BitmapAtlas::new(bmp);
	atlas.add_grid(TILE_WIDTH, TILE_HEIGHT)?;
	Ok(atlas)
}
