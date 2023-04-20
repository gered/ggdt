use crate::graphics::{Bitmap, Pixel};
use crate::math::{NearlyEqual, Rect, Vector2};

#[inline]
pub fn edge_function(a: Vector2, b: Vector2, c: Vector2) -> f32 {
	(b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

#[inline]
pub fn is_bottom_right_edge(v1: Vector2, v2: Vector2) -> bool {
	// definitions of the different edges for counter-clockwise vertex winding
	// - top: is horizontal (v1.y - v2.y == 0) and X decreases as you go across the screen (the edge points left)
	// - left: Y increases as you go down the screen
	// - bottom: is horizontal (v1.y - v2.y == 0) and X increases as you go across the screen (the edge points right)
	// - right: Y decreases as you go up the screen
	// (basically, picture a box where each edge is a vector pointing in a direction, and then move from edge to edge
	// counter-clockwise and think of the X and Y directions as you move along each edge)

	let edge = v1 - v2;
	edge.y < 0.0 || (edge.y.nearly_equal(0.0, f32::EPSILON) && edge.x > 0.0)
}

#[derive(Debug)]
struct TriangleEdge {
	x_inc: f32,
	y_inc: f32,
	is_bottom_right_edge: bool,
	origin: f32,
}

impl TriangleEdge {
	pub fn from(v1: Vector2, v2: Vector2, initial_sample_point: Vector2) -> Self {
		let x_inc = v1.y - v2.y;
		let y_inc = v2.x - v1.x;
		Self {
			x_inc,
			y_inc,
			is_bottom_right_edge: is_bottom_right_edge(v2, v1),
			origin: edge_function(v1, v2, initial_sample_point),
		}
	}

	#[inline]
	pub fn is_inside(&self, value: f32) -> bool {
		// note that for a counter-clockwise vertex winding order with the direction of Y+ going down instead
		// of up, we need to test for *negative* area when checking if we're inside the triangle
		value <= 0.0
	}

	#[inline]
	pub fn is_on_fill_edge(&self, value: f32) -> bool {
		// skip bottom-right edge pixels so we only draw pixels inside the triangle as well as those that lie
		// on the top-left edges. this fixes seam issues with triangles drawn with blending that share an edge
		!(self.is_bottom_right_edge && value.nearly_equal(0.0, f32::EPSILON))
	}

	#[inline]
	pub fn evaluate(&self, value: f32) -> bool {
		self.is_inside(value) && self.is_on_fill_edge(value)
	}

	#[inline]
	pub fn step_x(&self, value: f32) -> f32 {
		value + self.x_inc
	}

	#[inline]
	pub fn step_y(&self, value: f32) -> f32 {
		value + self.y_inc
	}

	#[inline]
	pub fn origin(&self) -> f32 {
		self.origin
	}
}

#[inline]
pub fn per_pixel_triangle_2d<PixelType: Pixel>(
	dest: &mut Bitmap<PixelType>,
	a: Vector2,
	b: Vector2,
	c: Vector2,
	pixel_fn: impl Fn(&mut PixelType, f32, f32, f32),
) {
	// based off the triangle rasterization algorithm presented in these article series' (as well as others)
	// https://fgiesen.wordpress.com/2013/02/17/optimizing-sw-occlusion-culling-index/
	// https://www.scratchapixel.com/lessons/3d-basic-rendering/rasterization-practical-implementation/rasterization-stage.html
	// https://kitsunegames.com/post/development/2016/07/28/software-3d-rendering-in-javascript-pt2/

	let mut bounds = Rect::from_coords(
		a.x.min(b.x).min(c.x).floor() as i32,
		a.y.min(b.y).min(c.y).floor() as i32,
		a.x.max(b.x).max(c.x).ceil() as i32,
		a.y.max(b.y).max(c.y).ceil() as i32,
	);
	if !bounds.clamp_to(dest.clip_region()) {
		return;
	}

	let p = Vector2::new(bounds.x as f32 + 0.5, bounds.y as f32 + 0.5);
	let edge_bc = TriangleEdge::from(b, c, p);
	let edge_ca = TriangleEdge::from(c, a, p);
	let edge_ab = TriangleEdge::from(a, b, p);

	let mut w0_row = edge_bc.origin();
	let mut w1_row = edge_ca.origin();
	let mut w2_row = edge_ab.origin();

	let draw_width = bounds.width as usize;
	let next_row_inc = dest.width() as usize;
	let mut pixels = unsafe { dest.pixels_at_mut_ptr_unchecked(bounds.x, bounds.y) };

	for _ in bounds.y..=bounds.bottom() {
		let mut w0 = w0_row;
		let mut w1 = w1_row;
		let mut w2 = w2_row;

		let row_pixels = unsafe { std::slice::from_raw_parts_mut(pixels, draw_width) };
		for pixel in row_pixels.iter_mut() {
			if edge_bc.evaluate(w0) && edge_ca.evaluate(w1) && edge_ab.evaluate(w2) {
				pixel_fn(pixel, w0, w1, w2)
			}

			w0 = edge_bc.step_x(w0);
			w1 = edge_ca.step_x(w1);
			w2 = edge_ab.step_x(w2);
		}

		w0_row = edge_bc.step_y(w0_row);
		w1_row = edge_ca.step_y(w1_row);
		w2_row = edge_ab.step_y(w2_row);
		pixels = unsafe { pixels.add(next_row_inc) };
	}
}
