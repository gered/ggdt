use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use ggdt::prelude::dos_like::*;

use crate::{TILE_HEIGHT, TILE_WIDTH};

pub const TILE_FLAG_NONE: i32 = 1;
pub const TILE_FLAG_COLLISION: i32 = 0;
pub const TILE_FLAG_SPAWNABLE: i32 = 1;

#[derive(Debug, Deserialize)]
pub struct TileMap {
	width: u32,
	height: u32,
	layers: Vec<Box<[i32]>>,
}

impl TileMap {
	pub fn load_from(path: &Path) -> Result<Self> {
		let f = File::open(path)?;
		let reader = BufReader::new(f);
		serde_json::from_reader(reader).context(format!("Loading json tilemap: {:?}", path))
	}

	#[inline]
	pub fn width(&self) -> u32 {
		self.width
	}

	#[inline]
	pub fn height(&self) -> u32 {
		self.height
	}

	#[inline]
	pub fn index_to(&self, x: i32, y: i32) -> Option<usize> {
		if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
			Some(((y * self.width as i32) + x) as usize)
		} else {
			None
		}
	}

	#[inline]
	pub fn lower(&self) -> &Box<[i32]> {
		&self.layers[0]
	}

	#[inline]
	pub fn upper(&self) -> &Box<[i32]> {
		&self.layers[1]
	}

	#[inline]
	pub fn collision(&self) -> &Box<[i32]> {
		&self.layers[2]
	}

	pub fn draw(&self, dest: &mut Bitmap, tiles: &BitmapAtlas<Bitmap>, camera_x: i32, camera_y: i32) {
		let xt = camera_x / TILE_WIDTH as i32;
		let yt = camera_y / TILE_HEIGHT as i32;
		let xp = camera_x % TILE_WIDTH as i32;
		let yp = camera_y % TILE_HEIGHT as i32;

		for y in 0..=15 {
			for x in 0..=20 {
				if let Some(index) = self.index_to(x + xt, y + yt) {
					let xd = (x * TILE_WIDTH as i32) - xp;
					let yd = (y * TILE_HEIGHT as i32) - yp;

					let lower = self.layers[0][index];
					if lower >= 0 {
						dest.blit_region(BlitMethod::Solid, tiles.bitmap(), &tiles[lower as usize], xd, yd);
					}
					let upper = self.layers[1][index];
					if upper >= 0 {
						dest.blit_region(BlitMethod::Transparent(0), tiles.bitmap(), &tiles[upper as usize], xd, yd);
					}
				}
			}
		}
	}

	pub fn is_colliding(&self, rect: &Rect) -> bool {
		let x1 = rect.x / TILE_WIDTH as i32;
		let y1 = rect.y / TILE_HEIGHT as i32;
		let x2 = rect.right() / TILE_WIDTH as i32;
		let y2 = rect.bottom() / TILE_HEIGHT as i32;

		for y in y1..=y2 {
			for x in x1..=x2 {
				match self.index_to(x, y) {
					Some(index) => {
						if self.collision()[index] == TILE_FLAG_COLLISION {
							return true;
						}
					}
					None => return true
				}
			}
		}
		false
	}

	pub fn get_random_spawnable_coordinates(&self) -> (i32, i32) {
		// TODO: do this better
		let mut x;
		let mut y;

		loop {
			x = rnd_value(0, self.width as i32 - 1);
			y = rnd_value(0, self.height as i32 - 1);
			if self.collision()[self.index_to(x, y).unwrap()] == TILE_FLAG_SPAWNABLE {
				break;
			}
		}

		(x, y)
	}
}
