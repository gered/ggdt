use crate::graphics::bitmap::Bitmap;
use crate::graphics::Pixel;
use crate::math::vector2::Vector2;

#[inline]
fn cross(a: Vector2, b: Vector2, c: Vector2) -> f32 {
	(b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

impl<PixelType: Pixel> Bitmap<PixelType> {
	pub fn triangle_2d_solid_color(&mut self, a: Vector2, b: Vector2, c: Vector2, color: PixelType) {
		self.triangle_2d_custom(
			a, //
			b,
			c,
			|dest_pixels, _w0, _w1, _w2| {
				*dest_pixels = color;
			},
		)
	}

	pub fn triangle_2d_textured(
		&mut self,
		a: Vector2,
		a_tex: Vector2,
		b: Vector2,
		b_tex: Vector2,
		c: Vector2,
		c_tex: Vector2,
		texture: &Bitmap<PixelType>,
	) {
		let inverse_area = 1.0 / cross(a, b, c); // inverting to avoid division
		self.triangle_2d_custom(
			a, //
			b,
			c,
			|dest_pixels, w0, w1, w2| {
				let u = (w0 * a_tex.x + w1 * b_tex.x + w2 * c_tex.x) * inverse_area;
				let v = (w0 * a_tex.y + w1 * b_tex.y + w2 * c_tex.y) * inverse_area;
				*dest_pixels = texture.sample_at(u, v);
			},
		)
	}

	#[inline]
	pub fn triangle_2d_custom(
		&mut self,
		a: Vector2,
		b: Vector2,
		c: Vector2,
		pixel_fn: impl Fn(&mut PixelType, f32, f32, f32),
	) {
		// based off the triangle rasterization algorithm presented in these article series' (as well as others)
		// https://fgiesen.wordpress.com/2013/02/17/optimizing-sw-occlusion-culling-index/
		// https://www.scratchapixel.com/lessons/3d-basic-rendering/rasterization-practical-implementation/rasterization-stage.html
		// https://kitsunegames.com/post/development/2016/07/28/software-3d-rendering-in-javascript-pt2/

		// TODO: implement fill rules, probably using top-left ordering as most 3d APIs do i guess

		let min_x = a.x.min(b.x).min(c.x).floor() as i32;
		let min_y = a.y.min(b.y).min(c.y).floor() as i32;
		let max_x = a.x.max(b.x).max(c.x).ceil() as i32;
		let max_y = a.y.max(b.y).max(c.y).ceil() as i32;

		let min_x = std::cmp::max(self.clip_region().x, min_x);
		let min_y = std::cmp::max(self.clip_region().y, min_y);
		let max_x = std::cmp::min(self.clip_region().right(), max_x);
		let max_y = std::cmp::min(self.clip_region().bottom(), max_y);

		let a01 = a.y - b.y;
		let b01 = b.x - a.x;
		let a12 = b.y - c.y;
		let b12 = c.x - b.x;
		let a20 = c.y - a.y;
		let b20 = a.x - c.x;

		let p = Vector2::new(min_x as f32, min_y as f32);
		let mut w0_row = cross(b, c, p);
		let mut w1_row = cross(c, a, p);
		let mut w2_row = cross(a, b, p);

		let draw_width = (max_x - min_x + 1) as usize;
		let next_row_inc = self.width() as usize;
		let mut pixels = unsafe { self.pixels_at_mut_ptr_unchecked(min_x, min_y) };

		for _ in min_y..=max_y {
			let mut w0 = w0_row;
			let mut w1 = w1_row;
			let mut w2 = w2_row;

			let row_pixels = unsafe { std::slice::from_raw_parts_mut(pixels, draw_width) };
			for pixel in row_pixels.iter_mut() {
				// note that for a counter-clockwise vertex winding order with the direction of Y+ going down instead
				// of up, we need to test for *negative* area when checking if we're inside the triangle
				if w0 < 0.0 && w1 < 0.0 && w2 < 0.0 {
					pixel_fn(pixel, w0, w1, w2);
				}

				w0 += a12;
				w1 += a20;
				w2 += a01;
			}

			w0_row += b12;
			w1_row += b20;
			w2_row += b01;
			pixels = unsafe { pixels.add(next_row_inc) };
		}
	}
}
