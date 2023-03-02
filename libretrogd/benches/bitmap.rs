use criterion::{black_box, Criterion, criterion_group, criterion_main};

use libretrogd::{SCREEN_HEIGHT, SCREEN_WIDTH};
use libretrogd::graphics::*;

pub fn criterion_benchmark(c: &mut Criterion) {
	let mut source = Bitmap::new(SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
	let mut dest = vec![0u32; (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize].into_boxed_slice();
	let palette = Palette::new_vga_palette().unwrap();

	c.bench_function("deindex_bitmap_pixels", |b| {
		b.iter(|| source.copy_as_argb_to(&mut dest, &palette))
	});

	c.bench_function("set_pixel", |b| {
		b.iter(|| source.set_pixel(black_box(100), black_box(100), black_box(42)))
	});

	c.bench_function("set_pixel_unchecked", |b| {
		b.iter(|| unsafe {
			source.set_pixel_unchecked(black_box(100), black_box(100), black_box(42))
		})
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
