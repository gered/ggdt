use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ggdt::prelude::*;

pub fn criterion_benchmark(c: &mut Criterion) {
	let width = 320;
	let height = 240;

	let mut dest = RgbaBitmap::new(width, height).unwrap();
	let (texture, _) = RgbaBitmap::load_gif_file(std::path::Path::new("./test-assets/gif/small.gif")).unwrap();

	let big_v1 = Vector2::new(47.0, 23.0);
	let big_v2 = Vector2::new(60.0, 192.0);
	let big_v3 = Vector2::new(280.0, 153.0);

	let small_v1 = Vector2::new(16.0, 16.0);
	let small_v2 = Vector2::new(16.0, 31.0);
	let small_v3 = Vector2::new(31.0, 31.0);

	let texcoord_0_0 = Vector2::new(0.0, 0.0);
	let texcoord_1_0 = Vector2::new(1.0, 0.0);
	let texcoord_0_1 = Vector2::new(0.0, 1.0);
	let texcoord_1_1 = Vector2::new(1.0, 1.0);

	let color = ARGBu8x4::from_rgb([255, 255, 255]);
	let color_1 = ARGBu8x4::from_argb([255, 255, 0, 0]);
	let color_2 = ARGBu8x4::from_argb([255, 0, 255, 0]);
	let color_3 = ARGBu8x4::from_argb([255, 0, 0, 255]);

	c.bench_function("rgbabitmap_triangle_2d_solid_color", |b| {
		let triangle = RgbaTriangle2d::Solid { position: [big_v1, big_v2, big_v3], color };
		b.iter(|| {
			dest.triangle_2d(black_box(&triangle));
		})
	});

	c.bench_function("rgbabitmap_triangle_2d_solid_color_small", |b| {
		let triangle = RgbaTriangle2d::Solid { position: [small_v1, small_v2, small_v3], color };
		b.iter(|| {
			dest.triangle_2d(black_box(&triangle));
		})
	});

	c.bench_function("rgbabitmap_triangle_2d_solid_multicolor", |b| {
		let triangle =
			RgbaTriangle2d::SolidMultiColor { position: [big_v1, big_v2, big_v3], color: [color_1, color_2, color_3] };
		b.iter(|| {
			dest.triangle_2d(black_box(&triangle));
		})
	});

	c.bench_function("rgbabitmap_triangle_2d_solid_multicolor_small", |b| {
		let triangle = RgbaTriangle2d::SolidMultiColor {
			position: [small_v1, small_v2, small_v3],
			color: [color_1, color_2, color_3],
		};
		b.iter(|| {
			dest.triangle_2d(black_box(&triangle));
		})
	});

	c.bench_function("rgbabitmap_triangle_2d_solid_multicolor_blended", |b| {
		let triangle = RgbaTriangle2d::SolidMultiColorBlended {
			position: [big_v1, big_v2, big_v3],
			color: [color_1, color_2, color_3],
			blend: BlendFunction::Blend,
		};
		b.iter(|| {
			dest.triangle_2d(black_box(&triangle));
		})
	});

	c.bench_function("rgbabitmap_triangle_2d_solid_multicolor_blended_small", |b| {
		let triangle = RgbaTriangle2d::SolidMultiColorBlended {
			position: [small_v1, small_v2, small_v3],
			color: [color_1, color_2, color_3],
			blend: BlendFunction::Blend,
		};
		b.iter(|| {
			dest.triangle_2d(black_box(&triangle));
		})
	});

	c.bench_function("rgbabitmap_triangle_2d_textured", |b| {
		let triangle = RgbaTriangle2d::SolidTextured {
			position: [big_v1, big_v2, big_v3],
			texcoord: [texcoord_0_0, texcoord_1_0, texcoord_1_1],
			bitmap: &texture,
		};
		b.iter(|| {
			dest.triangle_2d(black_box(&triangle));
		})
	});

	c.bench_function("rgbabitmap_triangle_2d_textured_small", |b| {
		let triangle = RgbaTriangle2d::SolidTextured {
			position: [small_v1, small_v2, small_v3],
			texcoord: [texcoord_0_0, texcoord_1_0, texcoord_1_1],
			bitmap: &texture,
		};
		b.iter(|| {
			dest.triangle_2d(black_box(&triangle));
		})
	});

	c.bench_function("rgbabitmap_triangle_2d_textured_multicolored_blended", |b| {
		let triangle = RgbaTriangle2d::SolidTexturedMultiColoredBlended {
			position: [big_v1, big_v2, big_v3],
			texcoord: [texcoord_0_0, texcoord_1_0, texcoord_1_1],
			color: [color_1, color_2, color_3],
			bitmap: &texture,
			blend: BlendFunction::Blend,
		};
		b.iter(|| {
			dest.triangle_2d(black_box(&triangle));
		})
	});

	c.bench_function("rgbabitmap_triangle_2d_textured_multicolored_blended_small", |b| {
		let triangle = RgbaTriangle2d::SolidTexturedMultiColoredBlended {
			position: [small_v1, small_v2, small_v3],
			texcoord: [texcoord_0_0, texcoord_1_0, texcoord_1_1],
			color: [color_1, color_2, color_3],
			bitmap: &texture,
			blend: BlendFunction::Blend,
		};
		b.iter(|| {
			dest.triangle_2d(black_box(&triangle));
		})
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
