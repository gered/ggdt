use crate::graphics::bitmap::rgb::RgbaBitmap;
use crate::graphics::bitmap::triangles::{edge_function, per_pixel_triangle_2d};
use crate::graphics::color::{
	from_argb32, from_rgb32, multiply_argb32, tint_argb32, to_argb32, to_rgb32, BlendFunction,
};
use crate::math::vector2::Vector2;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RgbaTriangle2d<'a> {
	Solid {
		position: [Vector2; 3], //
		color: u32,
	},
	SolidBlended {
		position: [Vector2; 3], //
		color: u32,
		blend: BlendFunction,
	},
	SolidMultiColor {
		position: [Vector2; 3], //
		color: [u32; 3],
	},
	SolidMultiColorBlended {
		position: [Vector2; 3], //
		color: [u32; 3],
		blend: BlendFunction,
	},
	SolidTextured {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		bitmap: &'a RgbaBitmap,
	},
	SolidTexturedColored {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		color: u32,
		bitmap: &'a RgbaBitmap,
	},
	SolidTexturedColoredBlended {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		color: u32,
		bitmap: &'a RgbaBitmap,
		blend: BlendFunction,
	},
	SolidTexturedMultiColored {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		color: [u32; 3],
		bitmap: &'a RgbaBitmap,
	},
	SolidTexturedMultiColoredBlended {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		color: [u32; 3],
		bitmap: &'a RgbaBitmap,
		blend: BlendFunction,
	},
	SolidTexturedTinted {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		bitmap: &'a RgbaBitmap,
		tint: u32,
	},
	SolidTexturedBlended {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		bitmap: &'a RgbaBitmap,
		blend: BlendFunction,
	},
}

impl RgbaBitmap {
	pub fn triangle_2d(&mut self, triangle: &RgbaTriangle2d) {
		use RgbaTriangle2d::*;
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
			SolidBlended { position, color, blend } => {
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, _w0, _w1, _w2| *dest_pixels = blend.blend(*color, *dest_pixels),
				)
			}
			SolidMultiColor { position, color } => {
				let area = edge_function(position[0], position[1], position[2]);
				let (r1, g1, b1) = from_rgb32(color[0]);
				let (r2, g2, b2) = from_rgb32(color[1]);
				let (r3, g3, b3) = from_rgb32(color[2]);
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, w0, w1, w2| {
						let r = ((w0 * r1 as f32 + w1 * r2 as f32 + w2 * r3 as f32) / area) as u8;
						let g = ((w0 * g1 as f32 + w1 * g2 as f32 + w2 * g3 as f32) / area) as u8;
						let b = ((w0 * b1 as f32 + w1 * b2 as f32 + w2 * b3 as f32) / area) as u8;
						*dest_pixels = to_rgb32(r, g, b)
					},
				)
			}
			SolidMultiColorBlended { position, color, blend } => {
				let area = edge_function(position[0], position[1], position[2]);
				let (a1, r1, g1, b1) = from_argb32(color[0]);
				let (a2, r2, g2, b2) = from_argb32(color[1]);
				let (a3, r3, g3, b3) = from_argb32(color[2]);
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, w0, w1, w2| {
						let a = ((w0 * a1 as f32 + w1 * a2 as f32 + w2 * a3 as f32) / area) as u8;
						let r = ((w0 * r1 as f32 + w1 * r2 as f32 + w2 * r3 as f32) / area) as u8;
						let g = ((w0 * g1 as f32 + w1 * g2 as f32 + w2 * g3 as f32) / area) as u8;
						let b = ((w0 * b1 as f32 + w1 * b2 as f32 + w2 * b3 as f32) / area) as u8;
						*dest_pixels = blend.blend(to_argb32(a, r, g, b), *dest_pixels)
					},
				)
			}
			SolidTextured { position, texcoord, bitmap } => {
				let area = edge_function(position[0], position[1], position[2]);
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, w0, w1, w2| {
						let u = (w0 * texcoord[0].x + w1 * texcoord[1].x + w2 * texcoord[2].x) / area;
						let v = (w0 * texcoord[0].y + w1 * texcoord[1].y + w2 * texcoord[2].y) / area;
						*dest_pixels = bitmap.sample_at(u, v);
					},
				)
			}
			SolidTexturedColored { position, texcoord, color, bitmap } => {
				let area = edge_function(position[0], position[1], position[2]);
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, w0, w1, w2| {
						let u = (w0 * texcoord[0].x + w1 * texcoord[1].x + w2 * texcoord[2].x) / area;
						let v = (w0 * texcoord[0].y + w1 * texcoord[1].y + w2 * texcoord[2].y) / area;
						*dest_pixels = multiply_argb32(bitmap.sample_at(u, v), *color)
					},
				)
			}
			SolidTexturedColoredBlended { position, texcoord, color, bitmap, blend } => {
				let area = edge_function(position[0], position[1], position[2]);
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, w0, w1, w2| {
						let u = (w0 * texcoord[0].x + w1 * texcoord[1].x + w2 * texcoord[2].x) / area;
						let v = (w0 * texcoord[0].y + w1 * texcoord[1].y + w2 * texcoord[2].y) / area;
						let src = multiply_argb32(bitmap.sample_at(u, v), *color);
						*dest_pixels = blend.blend(src, *dest_pixels)
					},
				)
			}
			SolidTexturedMultiColored { position, texcoord, color, bitmap } => {
				let area = edge_function(position[0], position[1], position[2]);
				let (r1, g1, b1) = from_rgb32(color[0]);
				let (r2, g2, b2) = from_rgb32(color[1]);
				let (r3, g3, b3) = from_rgb32(color[2]);
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, w0, w1, w2| {
						let r = ((w0 * r1 as f32 + w1 * r2 as f32 + w2 * r3 as f32) / area) as u8;
						let g = ((w0 * g1 as f32 + w1 * g2 as f32 + w2 * g3 as f32) / area) as u8;
						let b = ((w0 * b1 as f32 + w1 * b2 as f32 + w2 * b3 as f32) / area) as u8;
						let u = (w0 * texcoord[0].x + w1 * texcoord[1].x + w2 * texcoord[2].x) / area;
						let v = (w0 * texcoord[0].y + w1 * texcoord[1].y + w2 * texcoord[2].y) / area;
						*dest_pixels = multiply_argb32(bitmap.sample_at(u, v), to_rgb32(r, g, b))
					},
				)
			}
			SolidTexturedMultiColoredBlended { position, texcoord, color, bitmap, blend } => {
				let area = edge_function(position[0], position[1], position[2]);
				let (a1, r1, g1, b1) = from_argb32(color[0]);
				let (a2, r2, g2, b2) = from_argb32(color[1]);
				let (a3, r3, g3, b3) = from_argb32(color[2]);
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, w0, w1, w2| {
						let a = ((w0 * a1 as f32 + w1 * a2 as f32 + w2 * a3 as f32) / area) as u8;
						let r = ((w0 * r1 as f32 + w1 * r2 as f32 + w2 * r3 as f32) / area) as u8;
						let g = ((w0 * g1 as f32 + w1 * g2 as f32 + w2 * g3 as f32) / area) as u8;
						let b = ((w0 * b1 as f32 + w1 * b2 as f32 + w2 * b3 as f32) / area) as u8;
						let u = (w0 * texcoord[0].x + w1 * texcoord[1].x + w2 * texcoord[2].x) / area;
						let v = (w0 * texcoord[0].y + w1 * texcoord[1].y + w2 * texcoord[2].y) / area;
						let src = multiply_argb32(bitmap.sample_at(u, v), to_argb32(a, r, g, b));
						*dest_pixels = blend.blend(src, *dest_pixels)
					},
				)
			}
			SolidTexturedTinted { position, texcoord, bitmap, tint } => {
				let area = edge_function(position[0], position[1], position[2]);
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, w0, w1, w2| {
						let u = (w0 * texcoord[0].x + w1 * texcoord[1].x + w2 * texcoord[2].x) / area;
						let v = (w0 * texcoord[0].y + w1 * texcoord[1].y + w2 * texcoord[2].y) / area;
						*dest_pixels = tint_argb32(bitmap.sample_at(u, v), *tint);
					},
				)
			}
			SolidTexturedBlended { position, texcoord, bitmap, blend } => {
				let area = edge_function(position[0], position[1], position[2]);
				per_pixel_triangle_2d(
					self, //
					position[0],
					position[1],
					position[2],
					|dest_pixels, w0, w1, w2| {
						let u = (w0 * texcoord[0].x + w1 * texcoord[1].x + w2 * texcoord[2].x) / area;
						let v = (w0 * texcoord[0].y + w1 * texcoord[1].y + w2 * texcoord[2].y) / area;
						*dest_pixels = blend.blend(bitmap.sample_at(u, v), *dest_pixels);
					},
				)
			}
		}
	}

	pub fn triangle_list_2d(&mut self, triangles: &[RgbaTriangle2d]) {
		for triangle in triangles.iter() {
			self.triangle_2d(triangle);
		}
	}
}
