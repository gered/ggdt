use crate::graphics::bitmap::Bitmap;
use crate::graphics::Pixel;
use crate::math::vector2::Vector2;

#[inline]
fn cross(a: Vector2, b: Vector2, c: Vector2) -> f32 {
	(c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
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

		// TODO: optimize further. have had some trouble with some explanations of the optmizations presented in the
		//       linked article and the math presented. in particular i would prefer to have counter-clockwise vertex
		//       ordering but the math presented seems to only work as-is with clockwise ordering ... *grumble*
		// TODO: implement fill rules, probably using top-left ordering as most 3d APIs do i guess

		let min_x = a.x.min(b.x).min(c.x).floor() as i32;
		let min_y = a.y.min(b.y).min(c.y).floor() as i32;
		let max_x = a.x.max(b.x).max(c.x).ceil() as i32;
		let max_y = a.y.max(b.y).max(c.y).ceil() as i32;

		let min_x = std::cmp::max(self.clip_region().x, min_x);
		let min_y = std::cmp::max(self.clip_region().y, min_y);
		let max_x = std::cmp::min(self.clip_region().right(), max_x);
		let max_y = std::cmp::min(self.clip_region().bottom(), max_y);

		let draw_width = (max_x - min_x + 1) as usize;
		let next_row_inc = self.width() as usize;
		let mut pixels = unsafe { self.pixels_at_mut_ptr_unchecked(min_x, min_y) };

		for y in min_y..=max_y {
			let row_pixels = unsafe { std::slice::from_raw_parts_mut(pixels, draw_width) };
			for (idx, pixel) in row_pixels.iter_mut().enumerate() {
				let x = min_x + idx as i32;

				let p = Vector2::new(x as f32, y as f32);
				let w0 = cross(b, c, p);
				let w1 = cross(c, a, p);
				let w2 = cross(a, b, p);

				if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
					pixel_fn(pixel, w0, w1, w2);
				}
			}
			pixels = unsafe { pixels.add(next_row_inc) };
		}
	}
}
