use crate::tilemap::{TILE_HEIGHT, TILE_WIDTH};
use anyhow::{Context, Result};
use ggdt::prelude::*;

pub fn load_palette(path: &std::path::Path) -> Result<Palette> {
	Palette::load_from_file(path, PaletteFormat::Vga).context(format!("Loading palette: {:?}", path))
}

pub fn load_font(path: &std::path::Path) -> Result<BitmaskFont> {
	BitmaskFont::load_from_file(path).context(format!("Loading font: {:?}", path))
}

pub fn load_bitmap_atlas_autogrid(path: &std::path::Path) -> Result<BitmapAtlas<RgbaBitmap>> {
	let (bmp, _) = RgbaBitmap::load_file(path).context(format!("Loading bitmap atlas: {:?}", path))?;
	let mut atlas = BitmapAtlas::new(bmp);
	atlas.add_grid(TILE_WIDTH, TILE_HEIGHT)?;
	Ok(atlas)
}
