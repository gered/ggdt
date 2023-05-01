use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ggdt::prelude::*;

pub fn criterion_benchmark(c: &mut Criterion) {
	let width = 320;
	let height = 240;

	let mut dest = RgbaBitmap::new(width, height).unwrap();

	// note that none of these are all-inclusive benchmarks that cover all kinds of different scenarios and such
	// where the rendering logic might change based on the actual arguments given (e.g. i am not benchmarking
	// anything which does clipping as of yet).
	// maybe i will eventually add that kind of detailed benchmarking, but this for now is to just get a very,
	// very, VERY, basic idea about general performance ... moreso so that i can easily compare before/after
	// as i change other things in the future

	c.bench_function("rgbabitmap_primitives_set_pixel", |b| {
		dest.clear(0xff000000);
		b.iter(|| dest.set_pixel(black_box(100), black_box(100), black_box(0xffff00ff)))
	});

	c.bench_function("rgbabitmap_primitives_set_blended_pixel", |b| {
		dest.clear(0xff000000);
		b.iter(|| {
			dest.set_blended_pixel(
				black_box(100),
				black_box(100),
				black_box(0x7fff00ff),
				black_box(BlendFunction::Blend),
			)
		})
	});

	c.bench_function("rgbabitmap_primitives_set_pixel_unchecked", |b| {
		dest.clear(0xff000000);
		b.iter(|| unsafe { dest.set_pixel_unchecked(black_box(100), black_box(100), black_box(0xffff00ff)) })
	});

	c.bench_function("rgbabitmap_primitives_set_blended_pixel_unchecked", |b| {
		dest.clear(0xff000000);
		b.iter(|| unsafe {
			dest.set_blended_pixel_unchecked(
				black_box(100),
				black_box(100),
				black_box(0x7fff00ff),
				black_box(BlendFunction::Blend),
			)
		})
	});

	c.bench_function("rgbabitmap_primitives_line", |b| {
		dest.clear(0xff000000);
		b.iter(|| dest.line(black_box(10), black_box(50), black_box(310), black_box(120), black_box(0xffff00ff)))
	});

	c.bench_function("rgbabitmap_primitives_blended_line", |b| {
		dest.clear(0xff000000);
		b.iter(|| {
			dest.blended_line(
				black_box(10),
				black_box(50),
				black_box(310),
				black_box(120),
				black_box(0x7fff00ff),
				black_box(BlendFunction::Blend),
			)
		})
	});

	c.bench_function("rgbabitmap_primitives_horiz_line", |b| {
		dest.clear(0xff000000);
		b.iter(|| dest.horiz_line(black_box(10), black_box(310), black_box(70), black_box(0xffff00ff)))
	});

	c.bench_function("rgbabitmap_primitives_blended_horiz_line", |b| {
		dest.clear(0xff000000);
		b.iter(|| {
			dest.blended_horiz_line(
				black_box(10),
				black_box(310),
				black_box(70),
				black_box(0x7fff00ff),
				black_box(BlendFunction::Blend),
			)
		})
	});

	c.bench_function("rgbabitmap_primitives_vert_line", |b| {
		dest.clear(0xff000000);
		b.iter(|| dest.vert_line(black_box(90), black_box(10), black_box(230), black_box(0xffff00ff)))
	});

	c.bench_function("rgbabitmap_primitives_blended_vert_line", |b| {
		dest.clear(0xff000000);
		b.iter(|| {
			dest.blended_vert_line(
				black_box(90),
				black_box(10),
				black_box(230),
				black_box(0x7fff00ff),
				black_box(BlendFunction::Blend),
			)
		})
	});

	c.bench_function("rgbabitmap_primitives_rect", |b| {
		dest.clear(0xff000000);
		b.iter(|| dest.rect(black_box(10), black_box(10), black_box(310), black_box(230), black_box(0xffff00ff)))
	});

	c.bench_function("rgbabitmap_primitives_blended_rect", |b| {
		dest.clear(0xff000000);
		b.iter(|| {
			dest.blended_rect(
				black_box(10),
				black_box(10),
				black_box(310),
				black_box(230),
				black_box(0x7fff00ff),
				black_box(BlendFunction::Blend),
			)
		})
	});

	c.bench_function("rgbabitmap_primitives_filled_rect", |b| {
		dest.clear(0xff000000);
		b.iter(|| dest.filled_rect(black_box(10), black_box(10), black_box(310), black_box(230), black_box(0xffff00ff)))
	});

	c.bench_function("rgbabitmap_primitives_blended_filled_rect", |b| {
		dest.clear(0xff000000);
		b.iter(|| {
			dest.blended_filled_rect(
				black_box(10),
				black_box(10),
				black_box(310),
				black_box(230),
				black_box(0x7fff00ff),
				black_box(BlendFunction::Blend),
			)
		})
	});

	c.bench_function("rgbabitmap_primitives_circle", |b| {
		dest.clear(0xff000000);
		b.iter(|| dest.circle(black_box(160), black_box(120), black_box(80), black_box(0xffff00ff)))
	});

	c.bench_function("rgbabitmap_primitives_filled_circle", |b| {
		dest.clear(0xff000000);
		b.iter(|| dest.filled_circle(black_box(160), black_box(120), black_box(80), black_box(0xffff00ff)))
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
