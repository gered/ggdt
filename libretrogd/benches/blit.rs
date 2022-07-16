use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use libretrogd::graphics::*;
use libretrogd::math::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut framebuffer = Bitmap::new(320, 240).unwrap();
    let (bmp, _) = Bitmap::load_iff_file(Path::new("./test-assets/test-tiles.lbm")).unwrap();

    let mut solid_bmp = Bitmap::new(16, 16).unwrap();
    solid_bmp.blit_region(BlitMethod::Solid, &bmp, &Rect::new(16, 16, 16, 16), 0, 0);
    let mut trans_bmp = Bitmap::new(16, 16).unwrap();
    trans_bmp.blit_region(BlitMethod::Solid, &bmp, &Rect::new(160, 0, 16, 16), 0, 0);

    c.bench_function("blit_single_checked_solid", |b| {
        b.iter(|| {
            framebuffer.blit(
                black_box(BlitMethod::Solid),
                black_box(&solid_bmp),
                black_box(100),
                black_box(100),
            )
        })
    });

    c.bench_function("blit_single_unchecked_solid", |b| {
        b.iter(|| unsafe {
            framebuffer.blit_unchecked(
                black_box(BlitMethod::Solid),
                black_box(&solid_bmp),
                black_box(100),
                black_box(100),
            )
        })
    });

    c.bench_function("blit_single_checked_transparent", |b| {
        b.iter(|| {
            framebuffer.blit(
                black_box(BlitMethod::Transparent(0)),
                black_box(&trans_bmp),
                black_box(100),
                black_box(100),
            )
        })
    });

    c.bench_function("blit_single_unchecked_transparent", |b| {
        b.iter(|| unsafe {
            framebuffer.blit_unchecked(
                black_box(BlitMethod::Transparent(0)),
                black_box(&trans_bmp),
                black_box(100),
                black_box(100),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
