use std::io::Cursor;

use criterion::{black_box, Criterion, criterion_group, criterion_main};

use libretrogd::graphics::*;

pub static SMALL_GIF_FILE_BYTES: &[u8] = include_bytes!("../test-assets/test.gif");
pub static LARGE_GIF_FILE_BYTES: &[u8] = include_bytes!("../test-assets/test_image.gif");

pub fn criterion_benchmark(c: &mut Criterion) {
	c.bench_function("loading_small_gif", |b| {
		b.iter(|| {
			let mut reader = Cursor::new(SMALL_GIF_FILE_BYTES);
			Bitmap::load_gif_bytes(black_box(&mut reader)).unwrap();
		})
	});

	c.bench_function("loading_large_gif", |b| {
		b.iter(|| {
			let mut reader = Cursor::new(LARGE_GIF_FILE_BYTES);
			Bitmap::load_gif_bytes(black_box(&mut reader)).unwrap();
		})
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
