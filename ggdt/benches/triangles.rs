use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ggdt::prelude::*;

pub fn criterion_benchmark(c: &mut Criterion) {
	let width = 320;
	let height = 240;

	let mut dest = IndexedBitmap::new(width, height).unwrap();

	c.bench_function("indexedbitmap_triangle_2d_solid_color", |b| {
		b.iter(|| {
			dest.triangle_2d_solid_color(
				black_box(Vector2::new(47.0, 23.0)),
				black_box(Vector2::new(60.0, 192.0)),
				black_box(Vector2::new(280.0, 153.0)),
				black_box(5),
			);
		})
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
