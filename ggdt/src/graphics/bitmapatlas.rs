use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::ops::Index;
use std::path::Path;

use thiserror::Error;

use crate::graphics::{GeneralBitmap, GeneralBlitMethod};
use crate::math::Rect;

#[derive(Error, Debug)]
pub enum BitmapAtlasError {
	#[error("Region is out of bounds for the Bitmap used by the BitmapAtlas")]
	OutOfBounds,

	#[error("Tile index {0} is invalid / out of range")]
	InvalidTileIndex(usize),

	#[error("Invalid dimensions for region")]
	InvalidDimensions,

	#[error("No bitmap atlas entries in the descriptor")]
	NoEntries,
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

	pub fn from_descriptor(descriptor: &BitmapAtlasDescriptor, bitmap: BitmapType) -> Result<Self, BitmapAtlasError> {
		if descriptor.tiles.is_empty() {
			return Err(BitmapAtlasError::NoEntries);
		}

		let mut atlas = BitmapAtlas::new(bitmap);
		for entry in descriptor.tiles.iter() {
			use BitmapAtlasDescriptorEntry::*;
			match entry {
				Tile { x, y, width, height } => {
					atlas.add(Rect::new(*x as i32, *y as i32, *width, *height))?;
				}
				Autogrid { x, y, tile_width, tile_height, num_tiles_x, num_tiles_y, border } => {
					atlas.add_custom_grid(*x, *y, *tile_width, *tile_height, *num_tiles_x, *num_tiles_y, *border)?;
				}
			}
		}

		Ok(atlas)
	}

	pub fn add(&mut self, rect: Rect) -> Result<usize, BitmapAtlasError> {
		if rect.width == 0 || rect.height == 0 {
			return Err(BitmapAtlasError::InvalidDimensions);
		}
		if !self.bounds.contains_rect(&rect) {
			return Err(BitmapAtlasError::OutOfBounds);
		}

		self.tiles.push(rect);
		Ok(self.tiles.len() - 1)
	}

	pub fn add_grid(&mut self, tile_width: u32, tile_height: u32) -> Result<usize, BitmapAtlasError> {
		if tile_width == 0 || tile_height == 0 {
			return Err(BitmapAtlasError::InvalidDimensions);
		}
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
		if tile_width == 0 || tile_height == 0 {
			return Err(BitmapAtlasError::InvalidDimensions);
		}

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

	pub fn get_uv(&self, index: usize) -> Option<[f32; 4]> {
		self.tiles.get(index).map(|rect| {
			[
				(rect.x as f32 / self.bitmap.width() as f32),
				(rect.y as f32 / self.bitmap.height() as f32),
				((rect.x + rect.width as i32) as f32 / self.bitmap.width() as f32),
				((rect.y + rect.height as i32) as f32 / self.bitmap.height() as f32),
			]
		})
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

#[derive(Error, Debug)]
pub enum BitmapAtlasDescriptorError {
	#[error("Serde Json serialization/deserialization error: {0}")]
	SerdeJsonError(String),

	#[error("I/O error")]
	IOError(#[from] std::io::Error),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BitmapAtlasDescriptorEntry {
	Tile {
		x: u32, //
		y: u32,
		width: u32,
		height: u32,
	},
	Autogrid {
		x: u32, //
		y: u32,
		tile_width: u32,
		tile_height: u32,
		num_tiles_x: u32,
		num_tiles_y: u32,
		border: u32,
	},
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct BitmapAtlasDescriptor {
	pub bitmap: String,
	pub tiles: Vec<BitmapAtlasDescriptorEntry>,
}

impl BitmapAtlasDescriptor {
	pub fn load_from_file(path: &Path) -> Result<Self, BitmapAtlasDescriptorError> {
		let f = File::open(path)?;
		let mut reader = BufReader::new(f);
		Self::load_from_bytes(&mut reader)
	}

	pub fn load_from_bytes<T: Read>(reader: &mut T) -> Result<Self, BitmapAtlasDescriptorError> {
		match serde_json::from_reader(reader) {
			Ok(desc) => Ok(desc),
			Err(error) => Err(BitmapAtlasDescriptorError::SerdeJsonError(error.to_string())),
		}
	}

	pub fn to_file(&self, path: &Path) -> Result<(), BitmapAtlasDescriptorError> {
		let f = File::create(path)?;
		let mut writer = BufWriter::new(f);
		self.to_bytes(&mut writer)
	}

	pub fn to_bytes<T: Write>(&self, writer: &mut T) -> Result<(), BitmapAtlasDescriptorError> {
		if let Err(error) = serde_json::to_writer_pretty(writer, &self) {
			Err(BitmapAtlasDescriptorError::SerdeJsonError(error.to_string()))
		} else {
			Ok(())
		}
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

		assert_eq!(3, atlas.add_grid(32, 32).unwrap());
		assert_eq!(4, atlas.len());
		assert_eq!(Rect::new(0, 0, 32, 32), atlas[0]);
		assert_eq!(Rect::new(32, 0, 32, 32), atlas[1]);
		assert_eq!(Rect::new(0, 32, 32, 32), atlas[2]);
		assert_eq!(Rect::new(32, 32, 32, 32), atlas[3]);

		atlas.clear();
		assert_eq!(0, atlas.len());

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

	#[test]
	pub fn adding_with_invalid_dimensions_fails() {
		let bmp = IndexedBitmap::new(64, 64).unwrap();
		let mut atlas = BitmapAtlas::new(bmp);

		assert_matches!(atlas.add(Rect::new(0, 0, 0, 0)), Err(BitmapAtlasError::InvalidDimensions));
		assert_matches!(atlas.add(Rect::new(16, 16, 0, 0)), Err(BitmapAtlasError::InvalidDimensions));
		assert_matches!(atlas.add(Rect::new(16, 16, 8, 0)), Err(BitmapAtlasError::InvalidDimensions));
		assert_matches!(atlas.add(Rect::new(16, 16, 0, 8)), Err(BitmapAtlasError::InvalidDimensions));
		assert_eq!(0, atlas.len());

		assert_matches!(atlas.add_grid(0, 0), Err(BitmapAtlasError::InvalidDimensions));
		assert_matches!(atlas.add_grid(8, 0), Err(BitmapAtlasError::InvalidDimensions));
		assert_matches!(atlas.add_grid(0, 8), Err(BitmapAtlasError::InvalidDimensions));
		assert_eq!(0, atlas.len());

		assert_matches!(atlas.add_custom_grid(0, 0, 0, 0, 2, 2, 1), Err(BitmapAtlasError::InvalidDimensions));
		assert_matches!(atlas.add_custom_grid(0, 0, 8, 0, 2, 2, 1), Err(BitmapAtlasError::InvalidDimensions));
		assert_matches!(atlas.add_custom_grid(0, 0, 0, 8, 2, 2, 1), Err(BitmapAtlasError::InvalidDimensions));
		assert_eq!(0, atlas.len());
	}
}
