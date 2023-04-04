use crate::graphics::bitmap::indexed::IndexedBitmap;
use crate::graphics::bitmap::triangles::{edge_function, per_pixel_triangle_2d};
use crate::graphics::blendmap::BlendMap;
use crate::math::vector2::Vector2;

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
	pub fn triangle_2d(&mut self, triangle: &IndexedTriangle2d) {
		use IndexedTriangle2d::*;
		match triangle {
			Solid { position, color } => {
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, _w0, _w1, _w2| *dest_pixels = *color,
				)
			}
			SolidBlended { position, color, blendmap } => {
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, _w0, _w1, _w2| {
						*dest_pixels =
							if let Some(blended) = blendmap.blend(*color, *dest_pixels) { blended } else { *color };
					},
				)
			}
			SolidTextured { position, texcoord, bitmap } => {
				let inverse_area = 1.0 / edge_function(position[0], position[1], position[2]);
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, w0, w1, w2| {
						let u = (w0 * texcoord[0].x + w1 * texcoord[1].x + w2 * texcoord[2].x) * inverse_area;
						let v = (w0 * texcoord[0].y + w1 * texcoord[1].y + w2 * texcoord[2].y) * inverse_area;
						*dest_pixels = bitmap.sample_at(u, v);
					},
				)
			}
			SolidTexturedBlended { position, texcoord, bitmap, blendmap } => {
				let inverse_area = 1.0 / edge_function(position[0], position[1], position[2]);
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, w0, w1, w2| {
						let u = (w0 * texcoord[0].x + w1 * texcoord[1].x + w2 * texcoord[2].x) * inverse_area;
						let v = (w0 * texcoord[0].y + w1 * texcoord[1].y + w2 * texcoord[2].y) * inverse_area;
						let texel = bitmap.sample_at(u, v);
						*dest_pixels =
							if let Some(blended) = blendmap.blend(texel, *dest_pixels) { blended } else { texel };
					},
				)
			}
		}
	}

	pub fn triangle_list_2d(&mut self, triangles: &[IndexedTriangle2d]) {
		for triangle in triangles.iter() {
			self.triangle_2d(triangle);
		}
	}
}
