use std::path::Path;

use anyhow::{Context, Result};

use ggdt::prelude::*;

use crate::{Game, TILE_HEIGHT, TILE_WIDTH};

pub fn load_palette(path: &Path) -> Result<Palette> {
	Palette::load_from_file(path, PaletteFormat::Vga).context(format!("Loading palette: {:?}", path))
}

pub fn load_font(path: &Path) -> Result<BitmaskFont> {
	BitmaskFont::load_from_file(path).context(format!("Loading font: {:?}", path))
}

pub fn load_bitmap_atlas_autogrid(path: &Path) -> Result<BitmapAtlas<IndexedBitmap>> {
	let (bmp, _) = IndexedBitmap::load_file(path).context(format!("Loading bitmap atlas: {:?}", path))?;
	let mut atlas = BitmapAtlas::new(bmp);
	atlas.add_grid(TILE_WIDTH, TILE_HEIGHT)?;
	Ok(atlas)
}

pub fn load_bitmap_atlas(path: &Path) -> Result<BitmapAtlas<IndexedBitmap>> {
	let (bmp, _) = IndexedBitmap::load_file(path).context(format!("Loading bitmap atlas: {:?}", path))?;
	let atlas = BitmapAtlas::new(bmp);
	Ok(atlas)
}

pub fn draw_window(dest: &mut IndexedBitmap, ui: &BitmapAtlas<IndexedBitmap>, left: i32, top: i32, right: i32, bottom: i32) {
	dest.filled_rect(left + 8, top + 8, right - 8, bottom - 8, 1);

	// corners
	dest.blit_region(IndexedBlitMethod::Transparent(0), &ui.bitmap(), &ui[2], left, top);
	dest.blit_region(IndexedBlitMethod::Transparent(0), &ui.bitmap(), &ui[3], right - 8, top);
	dest.blit_region(IndexedBlitMethod::Transparent(0), &ui.bitmap(), &ui[4], left, bottom - 8);
	dest.blit_region(IndexedBlitMethod::Transparent(0), &ui.bitmap(), &ui[5], right - 8, bottom - 8);

	// top and bottom edges
	for i in 0..((right - left) / 8) - 2 {
		let x = left + 8 + (i * 8);
		dest.blit_region(IndexedBlitMethod::Transparent(0), &ui.bitmap(), &ui[9], x, top);
		dest.blit_region(IndexedBlitMethod::Transparent(0), &ui.bitmap(), &ui[8], x, bottom - 8);
	}

	// left and right edges
	for i in 0..((bottom - top) / 8) - 2 {
		let y = top + 8 + (i * 8);
		dest.blit_region(IndexedBlitMethod::Transparent(0), &ui.bitmap(), &ui[6], left, y);
		dest.blit_region(IndexedBlitMethod::Transparent(0), &ui.bitmap(), &ui[7], right - 8, y);
	}
}

pub fn update_fade_transition(state: State, fade: &mut f32, delta: f32, context: &mut Game) -> bool {
	match state {
		State::TransitionIn => {
			*fade += delta;
			if *fade >= 1.0 {
				*fade = 1.0;
				context.core.system.res.palette = context.core.palette.clone();
				true
			} else {
				context.core.system.res.palette.lerp(0..=255, &context.core.fade_out_palette, &context.core.palette, *fade);
				false
			}
		}
		State::TransitionOut(_) => {
			*fade -= delta;
			if *fade <= 0.0 {
				*fade = 0.0;
				context.core.system.res.palette = context.core.fade_out_palette.clone();
				true
			} else {
				context.core.system.res.palette.lerp(0..=255, &context.core.fade_out_palette, &context.core.palette, *fade);
				false
			}
		}
		_ => {
			true
		}
	}
}
