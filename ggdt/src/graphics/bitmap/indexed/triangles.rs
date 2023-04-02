use crate::graphics::bitmap::indexed::IndexedBitmap;
use crate::graphics::bitmap::triangles::{edge_function, per_pixel_triangle_2d};
use crate::math::vector2::Vector2;

#[derive(Debug, Clone, PartialEq)]
pub enum IndexedTriangle2d<'a> {
	Solid {
		position: [Vector2; 3], //
		color: u8,
	},
	SolidTextured {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		bitmap: &'a IndexedBitmap,
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
		}
	}
}
