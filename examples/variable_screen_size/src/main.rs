use anyhow::{Context, Result};
use serde::Deserialize;

use ggdt::prelude::*;

const TILE_WIDTH: u32 = 16;
const TILE_HEIGHT: u32 = 16;

fn load_palette(path: &std::path::Path) -> Result<Palette> {
	Palette::load_from_file(path, PaletteFormat::Vga).context(format!("Loading palette: {:?}", path))
}

fn load_font(path: &std::path::Path) -> Result<BitmaskFont> {
	BitmaskFont::load_from_file(path).context(format!("Loading font: {:?}", path))
}

fn load_bitmap_atlas_autogrid(path: &std::path::Path) -> Result<BitmapAtlas<IndexedBitmap>> {
	let (bmp, _) = IndexedBitmap::load_file(path).context(format!("Loading bitmap atlas: {:?}", path))?;
	let mut atlas = BitmapAtlas::new(bmp);
	atlas.add_grid(TILE_WIDTH, TILE_HEIGHT)?;
	Ok(atlas)
}

#[derive(Debug, Deserialize)]
struct TileMap {
	width: u32,
	height: u32,
	layers: Vec<Box<[i32]>>,
}

impl TileMap {
	pub fn load_from(path: &std::path::Path) -> Result<Self> {
		let f = std::fs::File::open(path)?;
		let reader = std::io::BufReader::new(f);
		serde_json::from_reader(reader).context(format!("Loading json tilemap: {:?}", path))
	}

	#[inline]
	pub fn index_to(&self, x: i32, y: i32) -> Option<usize> {
		if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
			Some(((y * self.width as i32) + x) as usize)
		} else {
			None
		}
	}

	pub fn draw(&self, dest: &mut IndexedBitmap, tiles: &BitmapAtlas<IndexedBitmap>, camera_x: i32, camera_y: i32) {
		let xt = camera_x / TILE_WIDTH as i32;
		let yt = camera_y / TILE_HEIGHT as i32;
		let xp = camera_x % TILE_WIDTH as i32;
		let yp = camera_y % TILE_HEIGHT as i32;

		let tiles_y = (dest.height() as f32 / TILE_HEIGHT as f32).ceil() as i32 + 1;
		let tiles_x = (dest.width() as f32 / TILE_WIDTH as f32).ceil() as i32 + 1;

		for y in 0..tiles_y {
			for x in 0..tiles_x {
				if let Some(index) = self.index_to(x + xt, y + yt) {
					let xd = (x * TILE_WIDTH as i32) - xp;
					let yd = (y * TILE_HEIGHT as i32) - yp;

					let lower = self.layers[0][index];
					if lower >= 0 {
						dest.blit_region(IndexedBlitMethod::Solid, tiles.bitmap(), &tiles[lower as usize], xd, yd);
					}
					let upper = self.layers[1][index];
					if upper >= 0 {
						dest.blit_region(
							IndexedBlitMethod::Transparent(0),
							tiles.bitmap(),
							&tiles[upper as usize],
							xd,
							yd,
						);
					}
				}
			}
		}
	}
}

fn main() -> Result<()> {
	let config = DosLikeConfig::variable_screen_size(320, 240).scale_factor(3);
	let mut system = SystemBuilder::new() //
		.window_title("Variable Screen Size")
		.vsync(true)
		.build(config)?;

	let palette = load_palette(std::path::Path::new("./assets/db16.pal"))?;
	let font = load_font(std::path::Path::new("./assets/dp.fnt"))?;
	let tiles = load_bitmap_atlas_autogrid(std::path::Path::new("./assets/tiles.pcx"))?;
	let tilemap = TileMap::load_from(std::path::Path::new("./assets/arena.map.json"))?;

	system.res.palette = palette;
	system.res.cursor.enable_cursor(true);

	let mut camera_x = 0;
	let mut camera_y = 0;

	while !system.do_events()? {
		if system.res.keyboard.is_key_pressed(Scancode::Escape) {
			break;
		}
		if system.res.mouse.is_button_down(1) {
			camera_x -= system.res.mouse.x_delta() * 2;
			camera_y -= system.res.mouse.y_delta() * 2;
		}

		system.update()?;

		system.res.video.clear(0);
		tilemap.draw(&mut system.res.video, &tiles, camera_x, camera_y);

		system.res.video.print_string(
			&format!(
				"Camera: {}, {}\nDisplay Size: {}, {}",
				camera_x,
				camera_y,
				system.res.video.width(),
				system.res.video.height()
			),
			10,
			10,
			FontRenderOpts::Color(15),
			&font,
		);
		system.res.video.print_string(
			"Click-and-drag to scroll",
			10,
			system.res.video.height() as i32 - 20,
			FontRenderOpts::Color(15),
			&font,
		);

		system.display()?;
	}

	Ok(())
}
