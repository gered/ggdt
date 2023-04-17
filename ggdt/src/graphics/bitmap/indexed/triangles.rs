use crate::graphics::{edge_function, per_pixel_triangle_2d, BlendMap, IndexedBitmap};
use crate::math::Vector2;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum IndexedTriangle2d<'a> {
	Solid {
		position: [Vector2; 3], //
		color: u8,
	},
	SolidBlended {
		position: [Vector2; 3], //
		color: u8,
		blendmap: &'a BlendMap,
	},
	SolidTextured {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		bitmap: &'a IndexedBitmap,
	},
	SolidTexturedBlended {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		bitmap: &'a IndexedBitmap,
		blendmap: &'a BlendMap,
	},
}

impl IndexedBitmap {
	pub fn solid_triangle_2d(&mut self, positions: &[Vector2; 3], color: u8) {
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, _w0, _w1, _w2| *dest_pixels = color,
		)
	}

	pub fn solid_blended_triangle_2d(&mut self, positions: &[Vector2; 3], color: u8, blendmap: &BlendMap) {
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, _w0, _w1, _w2| {
				*dest_pixels = if let Some(blended) = blendmap.blend(color, *dest_pixels) { blended } else { color };
			},
		)
	}

	pub fn solid_textured_triangle_2d(&mut self, positions: &[Vector2; 3], texcoords: &[Vector2; 3], bitmap: &Self) {
		let area = edge_function(positions[0], positions[1], positions[2]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let u = (w0 * texcoords[0].x + w1 * texcoords[1].x + w2 * texcoords[2].x) / area;
				let v = (w0 * texcoords[0].y + w1 * texcoords[1].y + w2 * texcoords[2].y) / area;
				*dest_pixels = bitmap.sample_at(u, v);
			},
		)
	}

	pub fn solid_textured_blended_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		texcoords: &[Vector2; 3],
		bitmap: &Self,
		blendmap: &BlendMap,
	) {
		let area = edge_function(positions[0], positions[1], positions[2]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let u = (w0 * texcoords[0].x + w1 * texcoords[1].x + w2 * texcoords[2].x) / area;
				let v = (w0 * texcoords[0].y + w1 * texcoords[1].y + w2 * texcoords[2].y) / area;
				let texel = bitmap.sample_at(u, v);
				*dest_pixels = if let Some(blended) = blendmap.blend(texel, *dest_pixels) { blended } else { texel };
			},
		)
	}

	pub fn triangle_2d(&mut self, triangle: &IndexedTriangle2d) {
		use IndexedTriangle2d::*;
		match triangle {
			Solid { position, color } => self.solid_triangle_2d(position, *color),
			SolidBlended { position, color, blendmap } => self.solid_blended_triangle_2d(position, *color, *blendmap),
			SolidTextured { position, texcoord, bitmap } => self.solid_textured_triangle_2d(position, texcoord, bitmap),
			SolidTexturedBlended { position, texcoord, bitmap, blendmap } => {
				self.solid_textured_blended_triangle_2d(position, texcoord, bitmap, blendmap)
			}
		}
	}

	pub fn triangle_list_2d(&mut self, triangles: &[IndexedTriangle2d]) {
		for triangle in triangles.iter() {
			self.triangle_2d(triangle);
		}
	}
}
