use std::io::Cursor;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ggdt::prelude::*;

pub static SMALL_GIF_FILE_BYTES: &[u8] = include_bytes!("../test-assets/gif/small.gif");
pub static LARGE_GIF_FILE_BYTES: &[u8] = include_bytes!("../test-assets/gif/large_1.gif");

pub fn criterion_benchmark(c: &mut Criterion) {
	c.bench_function("loading_small_gif", |b| {
		b.iter(|| {
			let mut reader = Cursor::new(SMALL_GIF_FILE_BYTES);
			IndexedBitmap::load_gif_bytes(black_box(&mut reader)).unwrap();
		})
	});

	c.bench_function("loading_large_gif", |b| {
		b.iter(|| {
			let mut reader = Cursor::new(LARGE_GIF_FILE_BYTES);
			IndexedBitmap::load_gif_bytes(black_box(&mut reader)).unwrap();
		})
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
