use std::simd;

use crate::graphics::{edge_function, per_pixel_triangle_2d, BlendFunction, RgbaBitmap, ARGB};
use crate::math::Vector2;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RgbaTriangle2d<'a> {
	Solid {
		position: [Vector2; 3], //
		color: ARGB,
	},
	SolidBlended {
		position: [Vector2; 3], //
		color: ARGB,
		blend: BlendFunction,
	},
	SolidMultiColor {
		position: [Vector2; 3], //
		color: [ARGB; 3],
	},
	SolidMultiColorBlended {
		position: [Vector2; 3], //
		color: [ARGB; 3],
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
		color: ARGB,
		bitmap: &'a RgbaBitmap,
	},
	SolidTexturedColoredBlended {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		color: ARGB,
		bitmap: &'a RgbaBitmap,
		blend: BlendFunction,
	},
	SolidTexturedMultiColored {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		color: [ARGB; 3],
		bitmap: &'a RgbaBitmap,
	},
	SolidTexturedMultiColoredBlended {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		color: [ARGB; 3],
		bitmap: &'a RgbaBitmap,
		blend: BlendFunction,
	},
	SolidTexturedTinted {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		bitmap: &'a RgbaBitmap,
		tint: ARGB,
	},
	SolidTexturedBlended {
		position: [Vector2; 3], //
		texcoord: [Vector2; 3],
		bitmap: &'a RgbaBitmap,
		blend: BlendFunction,
	},
}

impl RgbaBitmap {
	pub fn solid_triangle_2d(&mut self, positions: &[Vector2; 3], color: ARGB) {
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, _w0, _w1, _w2| *dest_pixels = color,
		)
	}

	pub fn solid_blended_triangle_2d(&mut self, positions: &[Vector2; 3], color: ARGB, blend: BlendFunction) {
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, _w0, _w1, _w2| *dest_pixels = blend.blend(color, *dest_pixels),
		)
	}

	pub fn solid_multicolor_triangle_2d(&mut self, positions: &[Vector2; 3], colors: &[ARGB; 3]) {
		let area = simd::f32x4::splat(edge_function(positions[0], positions[1], positions[2]));
		let color1 = colors[0].0.cast();
		let color2 = colors[1].0.cast();
		let color3 = colors[2].0.cast();
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let color = ((simd::f32x4::splat(w0) * color1
					+ simd::f32x4::splat(w1) * color2
					+ simd::f32x4::splat(w2) * color3)
					/ area)
					.cast();
				*dest_pixels = ARGB(color)
			},
		)
	}

	pub fn solid_multicolor_blended_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		colors: &[ARGB; 3],
		blend: BlendFunction,
	) {
		let area = simd::f32x4::splat(edge_function(positions[0], positions[1], positions[2]));
		let color1 = colors[0].0.cast();
		let color2 = colors[1].0.cast();
		let color3 = colors[2].0.cast();
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let color = ((simd::f32x4::splat(w0) * color1
					+ simd::f32x4::splat(w1) * color2
					+ simd::f32x4::splat(w2) * color3)
					/ area)
					.cast();
				*dest_pixels = blend.blend(ARGB(color), *dest_pixels)
			},
		)
	}

	pub fn solid_textured_triangle_2d(&mut self, positions: &[Vector2; 3], texcoords: &[Vector2; 3], bitmap: &Self) {
		let area = simd::f32x2::splat(edge_function(positions[0], positions[1], positions[2]));
		let texcoord1 = simd::f32x2::from_array([texcoords[0].x, texcoords[0].y]);
		let texcoord2 = simd::f32x2::from_array([texcoords[1].x, texcoords[1].y]);
		let texcoord3 = simd::f32x2::from_array([texcoords[2].x, texcoords[2].y]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let texcoord = (simd::f32x2::splat(w0) * texcoord1
					+ simd::f32x2::splat(w1) * texcoord2
					+ simd::f32x2::splat(w2) * texcoord3)
					/ area;
				*dest_pixels = bitmap.sample_at(texcoord[0], texcoord[1]);
			},
		)
	}

	pub fn solid_textured_colored_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		texcoords: &[Vector2; 3],
		color: ARGB,
		bitmap: &Self,
	) {
		let area = simd::f32x2::splat(edge_function(positions[0], positions[1], positions[2]));
		let texcoord1 = simd::f32x2::from_array([texcoords[0].x, texcoords[0].y]);
		let texcoord2 = simd::f32x2::from_array([texcoords[1].x, texcoords[1].y]);
		let texcoord3 = simd::f32x2::from_array([texcoords[2].x, texcoords[2].y]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let texcoord = (simd::f32x2::splat(w0) * texcoord1
					+ simd::f32x2::splat(w1) * texcoord2
					+ simd::f32x2::splat(w2) * texcoord3)
					/ area;
				let texel = bitmap.sample_at(texcoord[0], texcoord[1]);
				*dest_pixels = texel * color
			},
		)
	}

	pub fn solid_textured_colored_blended_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		texcoords: &[Vector2; 3],
		color: ARGB,
		bitmap: &Self,
		blend: BlendFunction,
	) {
		let area = simd::f32x2::splat(edge_function(positions[0], positions[1], positions[2]));
		let texcoord1 = simd::f32x2::from_array([texcoords[0].x, texcoords[0].y]);
		let texcoord2 = simd::f32x2::from_array([texcoords[1].x, texcoords[1].y]);
		let texcoord3 = simd::f32x2::from_array([texcoords[2].x, texcoords[2].y]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let texcoord = (simd::f32x2::splat(w0) * texcoord1
					+ simd::f32x2::splat(w1) * texcoord2
					+ simd::f32x2::splat(w2) * texcoord3)
					/ area;
				let texel = bitmap.sample_at(texcoord[0], texcoord[1]);
				let src = texel * color;
				*dest_pixels = blend.blend(src, *dest_pixels)
			},
		)
	}

	pub fn solid_textured_multicolor_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		texcoords: &[Vector2; 3],
		colors: &[ARGB; 3],
		bitmap: &Self,
	) {
		let area = simd::f32x4::splat(edge_function(positions[0], positions[1], positions[2]));
		let color1 = colors[0].0.cast();
		let color2 = colors[1].0.cast();
		let color3 = colors[2].0.cast();
		// we are using a f32x4 here with two zero's at the end as dummy values just so that we can
		// do the texture coordinate interpolation in the inner loop as f32x4 operations.
		// however, for the texture coordinates, we only care about the first two lanes in the results ...
		let texcoord1 = simd::f32x4::from_array([texcoords[0].x, texcoords[0].y, 0.0, 0.0]);
		let texcoord2 = simd::f32x4::from_array([texcoords[1].x, texcoords[1].y, 0.0, 0.0]);
		let texcoord3 = simd::f32x4::from_array([texcoords[2].x, texcoords[2].y, 0.0, 0.0]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let w0 = simd::f32x4::splat(w0);
				let w1 = simd::f32x4::splat(w1);
				let w2 = simd::f32x4::splat(w2);
				let color = ((w0 * color1 + w1 * color2 + w2 * color3) / area).cast::<u8>();
				let texcoord = (w0 * texcoord1 + w1 * texcoord2 + w2 * texcoord3) / area;
				let texel = bitmap.sample_at(texcoord[0], texcoord[1]);
				*dest_pixels = texel * ARGB(color)
			},
		)
	}

	pub fn solid_textured_multicolor_blended_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		texcoords: &[Vector2; 3],
		colors: &[ARGB; 3],
		bitmap: &Self,
		blend: BlendFunction,
	) {
		let area = simd::f32x4::splat(edge_function(positions[0], positions[1], positions[2]));
		let color1 = colors[0].0.cast();
		let color2 = colors[1].0.cast();
		let color3 = colors[2].0.cast();
		// we are using a f32x4 here with two zero's at the end as dummy values just so that we can
		// do the texture coordinate interpolation in the inner loop as f32x4 operations.
		// however, for the texture coordinates, we only care about the first two lanes in the results ...
		let texcoord1 = simd::f32x4::from_array([texcoords[0].x, texcoords[0].y, 0.0, 0.0]);
		let texcoord2 = simd::f32x4::from_array([texcoords[1].x, texcoords[1].y, 0.0, 0.0]);
		let texcoord3 = simd::f32x4::from_array([texcoords[2].x, texcoords[2].y, 0.0, 0.0]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let w0 = simd::f32x4::splat(w0);
				let w1 = simd::f32x4::splat(w1);
				let w2 = simd::f32x4::splat(w2);
				let color = ((w0 * color1 + w1 * color2 + w2 * color3) / area).cast::<u8>();
				let texcoord = (w0 * texcoord1 + w1 * texcoord2 + w2 * texcoord3) / area;
				let texel = bitmap.sample_at(texcoord[0], texcoord[1]);
				let src = texel * ARGB(color);
				*dest_pixels = blend.blend(src, *dest_pixels)
			},
		)
	}

	pub fn solid_textured_tinted_triangle_2d(
		&mut self,
		positions: &[Vector2; 3],
		texcoords: &[Vector2; 3],
		bitmap: &Self,
		tint: ARGB,
	) {
		let area = simd::f32x2::splat(edge_function(positions[0], positions[1], positions[2]));
		let texcoord1 = simd::f32x2::from_array([texcoords[0].x, texcoords[0].y]);
		let texcoord2 = simd::f32x2::from_array([texcoords[1].x, texcoords[1].y]);
		let texcoord3 = simd::f32x2::from_array([texcoords[2].x, texcoords[2].y]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let texcoord = (simd::f32x2::splat(w0) * texcoord1
					+ simd::f32x2::splat(w1) * texcoord2
					+ simd::f32x2::splat(w2) * texcoord3)
					/ area;
				let texel = bitmap.sample_at(texcoord[0], texcoord[1]);
				*dest_pixels = texel.tint(tint);
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
		let area = simd::f32x2::splat(edge_function(positions[0], positions[1], positions[2]));
		let texcoord1 = simd::f32x2::from_array([texcoords[0].x, texcoords[0].y]);
		let texcoord2 = simd::f32x2::from_array([texcoords[1].x, texcoords[1].y]);
		let texcoord3 = simd::f32x2::from_array([texcoords[2].x, texcoords[2].y]);
		per_pixel_triangle_2d(
			self, //
			positions[0],
			positions[1],
			positions[2],
			|dest_pixels, w0, w1, w2| {
				let texcoord = (simd::f32x2::splat(w0) * texcoord1
					+ simd::f32x2::splat(w1) * texcoord2
					+ simd::f32x2::splat(w2) * texcoord3)
					/ area;
				let texel = bitmap.sample_at(texcoord[0], texcoord[1]);
				*dest_pixels = blend.blend(texel, *dest_pixels);
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
