use crate::graphics::bitmap::rgb::RgbaBitmap;
use crate::graphics::bitmap::triangles::{edge_function, per_pixel_triangle_2d};
use crate::math::vector2::Vector2;
use crate::prelude::{from_rgb32, from_rgb32_normalized, to_rgb32_normalized};

#[derive(Debug, Clone, PartialEq)]
pub enum RgbaTriangle2d<'a> {
	SolidSingleColor {
		position: [Vector2; 3], //
		color: u32,
	},
	SolidMultiColor {
		position: [Vector2; 3], //
		color: [u32; 3],
	},
	SolidTextured {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		bitmap: &'a RgbaBitmap,
	},
}

impl RgbaBitmap {
	pub fn triangle_2d(&mut self, triangle: &RgbaTriangle2d) {
		use RgbaTriangle2d::*;
		match triangle {
			SolidSingleColor { position, color } => {
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, _w0, _w1, _w2| *dest_pixels = *color,
				)
			}
			SolidMultiColor { position, color } => {
				let inverse_area = 1.0 / edge_function(position[0], position[1], position[2]);
				let (r1, g1, b1) = from_rgb32_normalized(color[0]);
				let (r2, g2, b2) = from_rgb32_normalized(color[1]);
				let (r3, g3, b3) = from_rgb32_normalized(color[2]);
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, w0, w1, w2| {
						let r = (w0 * r1 + w1 * r2 + w2 * r3) * inverse_area;
						let g = (w0 * g1 + w1 * g2 + w2 * g3) * inverse_area;
						let b = (w0 * b1 + w1 * b2 + w2 * b3) * inverse_area;
						*dest_pixels = to_rgb32_normalized(r, g, b)
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
		}
	}
}
