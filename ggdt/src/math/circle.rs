use crate::math::*;

/// Represents a 2D circle, using integer center coordinates and radius.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Circle {
	pub x: i32,
	pub y: i32,
	pub radius: u32,
}

impl Circle {
	#[inline]
	pub fn new(x: i32, y: i32, radius: u32) -> Self {
		Circle { x, y, radius }
	}

	pub fn new_encapsulating(points: &[Vector2]) -> Option<Circle> {
		if points.is_empty() {
			return None;
		}

		let mut min_x = points[0].x;
		let mut min_y = points[0].y;
		let mut max_x = min_x;
		let mut max_y = min_y;

		for i in 0..points.len() {
			let point = &points[i];
			min_x = point.x.min(min_x);
			min_y = point.y.min(min_y);
			max_x = point.x.max(max_x);
			max_y = point.y.max(max_y);
		}

		let radius = distance_between(min_x, min_y, max_x, max_y) * 0.5;
		let center_x = (max_x - min_x) / 2.0;
		let center_y = (max_y - min_y) / 2.0;

		Some(Circle {
			x: center_x as i32,
			y: center_y as i32,
			radius: radius as u32,
		})
	}

	/// Calculates the diameter of the circle.
	#[inline]
	pub fn diameter(&self) -> u32 {
		self.radius * 2
	}

	/// Returns true if the given point is contained within the bounds of this circle.
	pub fn contains_point(&self, x: i32, y: i32) -> bool {
		let distance_squared =
			distance_squared_between(self.x as f32, self.y as f32, x as f32, y as f32);
		let radius_squared = (self.radius * self.radius) as f32;
		distance_squared <= radius_squared
	}

	/// Returns true if the given circle at least partially overlaps the bounds of this circle.
	pub fn overlaps(&self, other: &Circle) -> bool {
		let distance_squared =
			distance_squared_between(self.x as f32, self.y as f32, other.x as f32, other.y as f32);
		let minimum_distance_squared =
			((self.radius + other.radius) * (self.radius + other.radius)) as f32;
		distance_squared <= minimum_distance_squared
	}
}

#[cfg(test)]
pub mod tests {
	use super::*;

	#[test]
	pub fn test_new() {
		let c = Circle::new(1, 2, 4);
		assert_eq!(1, c.x);
		assert_eq!(2, c.y);
		assert_eq!(4, c.radius);
	}

	#[test]
	pub fn test_diameter() {
		let c = Circle::new(4, 4, 3);
		assert_eq!(6, c.diameter());
	}

	#[test]
	pub fn test_contains_point() {
		let c = Circle::new(1, 1, 6);
		assert!(c.contains_point(4, 4));
		assert!(!c.contains_point(8, 4));

		let c = Circle::new(0, 1, 2);
		assert!(!c.contains_point(3, 3));
		assert!(c.contains_point(0, 0));
	}

	#[test]
	pub fn test_overlaps() {
		let a = Circle::new(3, 4, 5);
		let b = Circle::new(14, 18, 8);
		assert!(!a.overlaps(&b));

		let a = Circle::new(2, 3, 12);
		let b = Circle::new(15, 28, 10);
		assert!(!a.overlaps(&b));

		let a = Circle::new(-10, 8, 30);
		let b = Circle::new(14, -24, 10);
		assert!(a.overlaps(&b));
	}
}
