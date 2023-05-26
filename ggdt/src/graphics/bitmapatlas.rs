use std::ops::Index;

use thiserror::Error;

use crate::graphics::{GeneralBitmap, GeneralBlitMethod};
use crate::math::Rect;

#[derive(Error, Debug)]
pub enum BitmapAtlasError {
	#[error("Region is out of bounds for the Bitmap used by the BitmapAtlas")]
	OutOfBounds,

	#[error("Tile index {0} is invalid / out of range")]
	InvalidTileIndex(usize),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BitmapAtlas<BitmapType>
where
	BitmapType: GeneralBitmap,
{
	bitmap: BitmapType,
	bounds: Rect,
	tiles: Vec<Rect>,
}

impl<BitmapType> BitmapAtlas<BitmapType>
where
	BitmapType: GeneralBitmap,
{
	pub fn new(bitmap: BitmapType) -> Self {
		let bounds = bitmap.full_bounds();
		BitmapAtlas {
			bitmap, //
			bounds,
			tiles: Vec::new(),
		}
	}

	pub fn add(&mut self, rect: Rect) -> Result<usize, BitmapAtlasError> {
		if !self.bounds.contains_rect(&rect) {
			return Err(BitmapAtlasError::OutOfBounds);
		}

		self.tiles.push(rect);
		Ok(self.tiles.len() - 1)
	}

	pub fn add_grid(&mut self, tile_width: u32, tile_height: u32) -> Result<usize, BitmapAtlasError> {
		if self.bounds.width < tile_width || self.bounds.height < tile_height {
			return Err(BitmapAtlasError::OutOfBounds);
		}

		for yt in 0..(self.bounds.height / tile_height) {
			for xt in 0..(self.bounds.width) / tile_width {
				let x = xt * tile_width;
				let y = yt * tile_height;
				let rect = Rect::new(x as i32, y as i32, tile_width, tile_height);
				self.tiles.push(rect);
			}
		}

		Ok(self.tiles.len() - 1)
	}

	pub fn add_custom_grid(
		&mut self,
		start_x: u32,
		start_y: u32,
		tile_width: u32,
		tile_height: u32,
		x_tiles: u32,
		y_tiles: u32,
		border: u32,
	) -> Result<usize, BitmapAtlasError> {
		// figure out of the grid properties given would result in us creating any
		// rects that lie out of the bounds of this bitmap
		let grid_region = Rect::new(
			start_x as i32,
			start_y as i32,
			(tile_width + border) * x_tiles + border,
			(tile_height + border) * y_tiles + border,
		);
		if !self.bounds.contains_rect(&grid_region) {
			return Err(BitmapAtlasError::OutOfBounds);
		}

		// all good! now create all the tiles needed for the grid specified
		for yt in 0..y_tiles {
			for xt in 0..x_tiles {
				let x = start_x + (tile_width + border) * xt;
				let y = start_y + (tile_height + border) * yt;
				let rect = Rect::new(x as i32, y as i32, tile_width, tile_height);
				self.tiles.push(rect);
			}
		}

		Ok(self.tiles.len() - 1)
	}

	pub fn clear(&mut self) {
		self.tiles.clear()
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.tiles.len()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.tiles.is_empty()
	}

	#[inline]
	pub fn get(&self, index: usize) -> Option<&Rect> {
		self.tiles.get(index)
	}

	#[inline]
	pub fn bitmap(&self) -> &BitmapType {
		&self.bitmap
	}

	pub fn clone_tile(&self, index: usize) -> Result<BitmapType, BitmapAtlasError> {
		if let Some(tile_rect) = self.get(index) {
			let mut tile_bitmap = BitmapType::new(tile_rect.width, tile_rect.height).unwrap();
			tile_bitmap.blit_region(GeneralBlitMethod::Solid, &self.bitmap, tile_rect, 0, 0);
			Ok(tile_bitmap)
		} else {
			Err(BitmapAtlasError::InvalidTileIndex(index))
		}
	}
}

impl<BitmapType> Index<usize> for BitmapAtlas<BitmapType>
where
	BitmapType: GeneralBitmap,
{
	type Output = Rect;

	#[inline]
	fn index(&self, index: usize) -> &Self::Output {
		self.get(index).unwrap()
	}
}

#[cfg(test)]
mod tests {
	use claim::*;

	use crate::graphics::IndexedBitmap;

	use super::*;

	#[test]
	pub fn adding_rects() {
		let bmp = IndexedBitmap::new(64, 64).unwrap();
		let mut atlas = BitmapAtlas::new(bmp);

		let rect = Rect::new(0, 0, 16, 16);
		assert_eq!(0, atlas.add(rect).unwrap());
		assert_eq!(rect, atlas[0]);
		assert_eq!(1, atlas.len());

		let rect = Rect::new(16, 0, 16, 16);
		assert_eq!(1, atlas.add(rect).unwrap());
		assert_eq!(rect, atlas[1]);
		assert_eq!(2, atlas.len());

		assert_matches!(atlas.add(Rect::new(56, 0, 16, 16)), Err(BitmapAtlasError::OutOfBounds));
		assert_eq!(2, atlas.len());

		assert_matches!(atlas.add(Rect::new(-8, 4, 16, 16)), Err(BitmapAtlasError::OutOfBounds));
		assert_eq!(2, atlas.len());

		assert_matches!(atlas.add(Rect::new(0, 0, 128, 128)), Err(BitmapAtlasError::OutOfBounds));
		assert_eq!(2, atlas.len());
	}

	#[test]
	pub fn adding_grid() {
		let bmp = IndexedBitmap::new(64, 64).unwrap();
		let mut atlas = BitmapAtlas::new(bmp);

		assert_eq!(3, atlas.add_custom_grid(0, 0, 8, 8, 2, 2, 0).unwrap());
		assert_eq!(4, atlas.len());
		assert_eq!(Rect::new(0, 0, 8, 8), atlas[0]);
		assert_eq!(Rect::new(8, 0, 8, 8), atlas[1]);
		assert_eq!(Rect::new(0, 8, 8, 8), atlas[2]);
		assert_eq!(Rect::new(8, 8, 8, 8), atlas[3]);

		atlas.clear();
		assert_eq!(0, atlas.len());

		assert_eq!(3, atlas.add_custom_grid(0, 0, 4, 8, 2, 2, 1).unwrap());
		assert_eq!(4, atlas.len());
		assert_eq!(Rect::new(0, 0, 4, 8), atlas[0]);
		assert_eq!(Rect::new(5, 0, 4, 8), atlas[1]);
		assert_eq!(Rect::new(0, 9, 4, 8), atlas[2]);
		assert_eq!(Rect::new(5, 9, 4, 8), atlas[3]);
	}
}
