use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ggdt::prelude::*;

pub fn criterion_benchmark(c: &mut Criterion) {
	let mut framebuffer = IndexedBitmap::new(320, 240).unwrap();
	let (bmp, _) = IndexedBitmap::load_iff_file(Path::new("./test-assets/test-tiles.lbm")).unwrap();

	let mut solid_bmp = IndexedBitmap::new(16, 16).unwrap();
	solid_bmp.blit_region(IndexedBlitMethod::Solid, &bmp, &Rect::new(16, 16, 16, 16), 0, 0);
	let mut trans_bmp = IndexedBitmap::new(16, 16).unwrap();
	trans_bmp.blit_region(IndexedBlitMethod::Solid, &bmp, &Rect::new(160, 0, 16, 16), 0, 0);

	//////

	c.bench_function("blit_single_checked_solid", |b| {
		b.iter(|| {
			framebuffer.blit(black_box(IndexedBlitMethod::Solid), black_box(&solid_bmp), black_box(100), black_box(100))
		})
	});

	c.bench_function("blit_single_unchecked_solid", |b| {
		b.iter(|| unsafe {
			framebuffer.blit_unchecked(
				black_box(IndexedBlitMethod::Solid),
				black_box(&solid_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_single_checked_transparent", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::Transparent(0)),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_single_unchecked_transparent", |b| {
		b.iter(|| unsafe {
			framebuffer.blit_unchecked(
				black_box(IndexedBlitMethod::Transparent(0)),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_solid_flipped_not_flipped", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::SolidFlipped { horizontal_flip: false, vertical_flip: false }),
				black_box(&solid_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_solid_flipped_horizontally", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::SolidFlipped { horizontal_flip: true, vertical_flip: false }),
				black_box(&solid_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_solid_flipped_vertically", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::SolidFlipped { horizontal_flip: false, vertical_flip: true }),
				black_box(&solid_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_solid_flipped_horizontally_and_vertically", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::SolidFlipped { horizontal_flip: true, vertical_flip: true }),
				black_box(&solid_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_transparent_flipped_not_flipped", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentFlipped {
					transparent_color: 0,
					horizontal_flip: false,
					vertical_flip: false,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_transparent_flipped_horizontally", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentFlipped {
					transparent_color: 0,
					horizontal_flip: true,
					vertical_flip: false,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_transparent_flipped_vertically", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentFlipped {
					transparent_color: 0,
					horizontal_flip: false,
					vertical_flip: true,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_transparent_flipped_horizontally_and_vertically", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentFlipped {
					transparent_color: 0,
					horizontal_flip: true,
					vertical_flip: true,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_transparent_single_flipped_not_flipped", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentFlippedSingle {
					transparent_color: 0,
					horizontal_flip: false,
					vertical_flip: false,
					draw_color: 17,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_transparent_single_flipped_horizontally", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentFlippedSingle {
					transparent_color: 0,
					horizontal_flip: true,
					vertical_flip: false,
					draw_color: 17,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_transparent_single_flipped_vertically", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentFlippedSingle {
					transparent_color: 0,
					horizontal_flip: false,
					vertical_flip: true,
					draw_color: 17,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_transparent_single_flipped_horizontally_and_vertically", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentFlippedSingle {
					transparent_color: 0,
					horizontal_flip: true,
					vertical_flip: true,
					draw_color: 17,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_transparent_single", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentSingle { transparent_color: 0, draw_color: 17 }),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_transparent_offset", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentOffset { transparent_color: 0, offset: 42 }),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_transparent_offset_flipped_not_flipped", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentFlippedOffset {
					transparent_color: 0,
					horizontal_flip: false,
					vertical_flip: false,
					offset: 42,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_transparent_offset_flipped_horizontally", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentFlippedOffset {
					transparent_color: 0,
					horizontal_flip: true,
					vertical_flip: false,
					offset: 42,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_transparent_offset_flipped_vertically", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentFlippedOffset {
					transparent_color: 0,
					horizontal_flip: false,
					vertical_flip: true,
					offset: 42,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_transparent_offset_flipped_horizontally_and_vertically", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::TransparentFlippedOffset {
					transparent_color: 0,
					horizontal_flip: true,
					vertical_flip: true,
					offset: 42,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_solid_offset", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::SolidOffset(42)),
				black_box(&solid_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_solid_offset_flipped_not_flipped", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::SolidFlippedOffset {
					horizontal_flip: false,
					vertical_flip: false,
					offset: 42,
				}),
				black_box(&solid_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_solid_offset_flipped_horizontally", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::SolidFlippedOffset {
					horizontal_flip: true,
					vertical_flip: false,
					offset: 42,
				}),
				black_box(&solid_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_solid_offset_flipped_vertically", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::SolidFlippedOffset {
					horizontal_flip: false,
					vertical_flip: true,
					offset: 42,
				}),
				black_box(&solid_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	c.bench_function("blit_solid_offset_flipped_horizontally_and_vertically", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::SolidFlippedOffset {
					horizontal_flip: true,
					vertical_flip: true,
					offset: 42,
				}),
				black_box(&solid_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_rotozoom", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::RotoZoom { angle: 73.0f32.to_radians(), scale_x: 1.2, scale_y: 0.8 }),
				black_box(&solid_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_rotozoom_offset", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::RotoZoomOffset {
					angle: 73.0f32.to_radians(),
					scale_x: 1.2,
					scale_y: 0.8,
					offset: 42,
				}),
				black_box(&solid_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_rotozoom_transparent", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::RotoZoomTransparent {
					angle: 73.0f32.to_radians(),
					scale_x: 1.2,
					scale_y: 0.8,
					transparent_color: 0,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});

	//////

	c.bench_function("blit_rotozoom_offset_transparent", |b| {
		b.iter(|| {
			framebuffer.blit(
				black_box(IndexedBlitMethod::RotoZoomTransparentOffset {
					angle: 73.0f32.to_radians(),
					scale_x: 1.2,
					scale_y: 0.8,
					transparent_color: 0,
					offset: 42,
				}),
				black_box(&trans_bmp),
				black_box(100),
				black_box(100),
			)
		})
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
