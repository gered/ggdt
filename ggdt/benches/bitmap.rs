use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ggdt::prelude::*;

pub fn criterion_benchmark(c: &mut Criterion) {
	let width = 320;
	let height = 240;

	let mut source = IndexedBitmap::new(width, height).unwrap();
	let mut dest = vec![RGBA::default(); (width * height) as usize].into_boxed_slice();
	let palette = Palette::new_vga_palette().unwrap();

	c.bench_function("deindex_bitmap_pixels", |b| b.iter(|| source.copy_as_rgba_to(&mut dest, &palette)));

	c.bench_function("set_pixel", |b| b.iter(|| source.set_pixel(black_box(100), black_box(100), black_box(42))));

	c.bench_function("set_pixel_unchecked", |b| {
		b.iter(|| unsafe { source.set_pixel_unchecked(black_box(100), black_box(100), black_box(42)) })
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
