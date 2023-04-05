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
	pub fn solid_triangle_2d(&mut self, positions: &[Vector2; 3], color: u32) {
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, _w0, _w1, _w2| *dest_pixels = color,
		)
	}

	pub fn solid_blended_triangle_2d(&mut self, positions: &[Vector2; 3], color: u32, blend: BlendFunction) {
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, _w0, _w1, _w2| *dest_pixels = blend.blend(color, *dest_pixels),
		)
	}

	pub fn solid_multicolor_triangle_2d(&mut self, positions: &[Vector2; 3], colors: &[u32; 3]) {
		let area = edge_function(positions[0], positions[1], positions[2]);
		let (r1, g1, b1) = from_rgb32(colors[0]);
		let (r2, g2, b2) = from_rgb32(colors[1]);
		let (r3, g3, b3) = from_rgb32(colors[2]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let r = ((w0 * r1 as f32 + w1 * r2 as f32 + w2 * r3 as f32) / area) as u8;
				let g = ((w0 * g1 as f32 + w1 * g2 as f32 + w2 * g3 as f32) / area) as u8;
				let b = ((w0 * b1 as f32 + w1 * b2 as f32 + w2 * b3 as f32) / area) as u8;
				*dest_pixels = to_rgb32(r, g, b)
			},
		)
	}

	pub fn solid_multicolor_blended_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		colors: &[u32; 3],
		blend: BlendFunction,
	) {
		let area = edge_function(positions[0], positions[1], positions[2]);
		let (a1, r1, g1, b1) = from_argb32(colors[0]);
		let (a2, r2, g2, b2) = from_argb32(colors[1]);
		let (a3, r3, g3, b3) = from_argb32(colors[2]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let a = ((w0 * a1 as f32 + w1 * a2 as f32 + w2 * a3 as f32) / area) as u8;
				let r = ((w0 * r1 as f32 + w1 * r2 as f32 + w2 * r3 as f32) / area) as u8;
				let g = ((w0 * g1 as f32 + w1 * g2 as f32 + w2 * g3 as f32) / area) as u8;
				let b = ((w0 * b1 as f32 + w1 * b2 as f32 + w2 * b3 as f32) / area) as u8;
				*dest_pixels = blend.blend(to_argb32(a, r, g, b), *dest_pixels)
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

	pub fn solid_textured_colored_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		texcoords: &[Vector2; 3],
		color: u32,
		bitmap: &Self,
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
				*dest_pixels = multiply_argb32(bitmap.sample_at(u, v), color)
			},
		)
	}

	pub fn solid_textured_colored_blended_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		texcoords: &[Vector2; 3],
		color: u32,
		bitmap: &Self,
		blend: BlendFunction,
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
				let src = multiply_argb32(bitmap.sample_at(u, v), color);
				*dest_pixels = blend.blend(src, *dest_pixels)
			},
		)
	}

	pub fn solid_textured_multicolor_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		texcoords: &[Vector2; 3],
		colors: &[u32; 3],
		bitmap: &Self,
	) {
		let area = edge_function(positions[0], positions[1], positions[2]);
		let (r1, g1, b1) = from_rgb32(colors[0]);
		let (r2, g2, b2) = from_rgb32(colors[1]);
		let (r3, g3, b3) = from_rgb32(colors[2]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let r = ((w0 * r1 as f32 + w1 * r2 as f32 + w2 * r3 as f32) / area) as u8;
				let g = ((w0 * g1 as f32 + w1 * g2 as f32 + w2 * g3 as f32) / area) as u8;
				let b = ((w0 * b1 as f32 + w1 * b2 as f32 + w2 * b3 as f32) / area) as u8;
				let u = (w0 * texcoords[0].x + w1 * texcoords[1].x + w2 * texcoords[2].x) / area;
				let v = (w0 * texcoords[0].y + w1 * texcoords[1].y + w2 * texcoords[2].y) / area;
				*dest_pixels = multiply_argb32(bitmap.sample_at(u, v), to_rgb32(r, g, b))
			},
		)
	}

	pub fn solid_textured_multicolor_blended_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		texcoords: &[Vector2; 3],
		colors: &[u32; 3],
		bitmap: &Self,
		blend: BlendFunction,
	) {
		let area = edge_function(positions[0], positions[1], positions[2]);
		let (a1, r1, g1, b1) = from_argb32(colors[0]);
		let (a2, r2, g2, b2) = from_argb32(colors[1]);
		let (a3, r3, g3, b3) = from_argb32(colors[2]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let a = ((w0 * a1 as f32 + w1 * a2 as f32 + w2 * a3 as f32) / area) as u8;
				let r = ((w0 * r1 as f32 + w1 * r2 as f32 + w2 * r3 as f32) / area) as u8;
				let g = ((w0 * g1 as f32 + w1 * g2 as f32 + w2 * g3 as f32) / area) as u8;
				let b = ((w0 * b1 as f32 + w1 * b2 as f32 + w2 * b3 as f32) / area) as u8;
				let u = (w0 * texcoords[0].x + w1 * texcoords[1].x + w2 * texcoords[2].x) / area;
				let v = (w0 * texcoords[0].y + w1 * texcoords[1].y + w2 * texcoords[2].y) / area;
				let src = multiply_argb32(bitmap.sample_at(u, v), to_argb32(a, r, g, b));
				*dest_pixels = blend.blend(src, *dest_pixels)
			},
		)
	}

	pub fn solid_textured_tinted_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		texcoords: &[Vector2; 3],
		bitmap: &Self,
		tint: u32,
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
				*dest_pixels = tint_argb32(bitmap.sample_at(u, v), tint);
			},
		)
	}

	pub fn solid_textured_blended_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		texcoords: &[Vector2; 3],
		bitmap: &Self,
		blend: BlendFunction,
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
				*dest_pixels = blend.blend(bitmap.sample_at(u, v), *dest_pixels);
			},
		)
	}

	pub fn triangle_2d(&mut self, triangle: &RgbaTriangle2d) {
		use RgbaTriangle2d::*;
		match triangle {
			Solid { position, color } => self.solid_triangle_2d(position, *color),
			SolidBlended { position, color, blend } => self.solid_blended_triangle_2d(position, *color, *blend),
			SolidMultiColor { position, color } => self.solid_multicolor_triangle_2d(position, color),
			SolidMultiColorBlended { position, color, blend } => {
				self.solid_multicolor_blended_triangle_2d(position, color, *blend)
			}
			SolidTextured { position, texcoord, bitmap } => self.solid_textured_triangle_2d(position, texcoord, bitmap),
			SolidTexturedColored { position, texcoord, color, bitmap } => {
				self.solid_textured_colored_triangle_2d(position, texcoord, *color, bitmap)
			}
			SolidTexturedColoredBlended { position, texcoord, color, bitmap, blend } => {
				self.solid_textured_colored_blended_triangle_2d(position, texcoord, *color, bitmap, *blend)
			}
			SolidTexturedMultiColored { position, texcoord, color, bitmap } => {
				self.solid_textured_multicolor_triangle_2d(position, texcoord, color, bitmap)
			}
			SolidTexturedMultiColoredBlended { position, texcoord, color, bitmap, blend } => {
				self.solid_textured_multicolor_blended_triangle_2d(position, texcoord, color, bitmap, *blend)
			}
			SolidTexturedTinted { position, texcoord, bitmap, tint } => {
				self.solid_textured_tinted_triangle_2d(position, texcoord, bitmap, *tint)
			}
			SolidTexturedBlended { position, texcoord, bitmap, blend } => {
				self.solid_textured_blended_triangle_2d(position, texcoord, bitmap, *blend)
			}
		}
	}

	pub fn triangle_list_2d(&mut self, triangles: &[RgbaTriangle2d]) {
		for triangle in triangles.iter() {
			self.triangle_2d(triangle);
		}
	}
}
