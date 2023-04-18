use std::path::{Path, PathBuf};

use ggdt::prelude::*;
use helpers::test_assets_file;

pub mod helpers;

const LIGHTER_BACKGROUND: u32 = 0xff2c3041;

pub const COLOR_BLACK_HALF_ALPHA: u32 = 0x7f000000;
pub const COLOR_BLUE_HALF_ALPHA: u32 = 0x7f0000aa;
pub const COLOR_GREEN_HALF_ALPHA: u32 = 0x7f00aa00;
pub const COLOR_CYAN_HALF_ALPHA: u32 = 0x7f00aaaa;
pub const COLOR_RED_HALF_ALPHA: u32 = 0x7faa0000;
pub const COLOR_MAGENTA_HALF_ALPHA: u32 = 0x7faa00aa;
pub const COLOR_BROWN_HALF_ALPHA: u32 = 0x7faa5500;
pub const COLOR_LIGHT_GRAY_HALF_ALPHA: u32 = 0x7faaaaaa;
pub const COLOR_DARK_GRAY_HALF_ALPHA: u32 = 0x7f555555;
pub const COLOR_BRIGHT_BLUE_HALF_ALPHA: u32 = 0x7f5555ff;
pub const COLOR_BRIGHT_GREEN_HALF_ALPHA: u32 = 0x7f55ff55;
pub const COLOR_BRIGHT_CYAN_HALF_ALPHA: u32 = 0x7f55ffff;
pub const COLOR_BRIGHT_RED_HALF_ALPHA: u32 = 0x7fff5555;
pub const COLOR_BRIGHT_MAGENTA_HALF_ALPHA: u32 = 0x7fff55ff;
pub const COLOR_BRIGHT_YELLOW_HALF_ALPHA: u32 = 0x7fffff55;
pub const COLOR_BRIGHT_WHITE_HALF_ALPHA: u32 = 0x7fffffff;

const SCREEN_WIDTH: u32 = 320;
const SCREEN_HEIGHT: u32 = 240;

const BASE_PATH: &str = "./tests/ref/rgba/";

fn reference_file(file: &Path) -> PathBuf {
	PathBuf::from(BASE_PATH).join(file)
}

fn setup() -> RgbaBitmap {
	RgbaBitmap::new(SCREEN_WIDTH, SCREEN_HEIGHT).unwrap()
}

fn setup_for_blending() -> RgbaBitmap {
	let (texture, _) = RgbaBitmap::load_file(test_assets_file(Path::new("texture.lbm")).as_path()).unwrap();
	let mut screen = RgbaBitmap::new(SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
	for y in 0..(SCREEN_HEIGHT as f32 / texture.height() as f32).ceil() as i32 {
		for x in 0..(SCREEN_WIDTH as f32 / texture.width() as f32).ceil() as i32 {
			screen.blit(RgbaBlitMethod::Solid, &texture, x * texture.width() as i32, y * texture.height() as i32);
		}
	}
	screen
}

fn setup_for_blending_half_solid_half_semi_transparent() -> RgbaBitmap {
	let (texture, _) = RgbaBitmap::load_file(test_assets_file(Path::new("texture.lbm")).as_path()).unwrap();
	let mut screen = RgbaBitmap::new(SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
	for y in 0..(screen.height() as f32 / texture.height() as f32).ceil() as i32 {
		for x in 0..(screen.width() as f32 / texture.width() as f32).ceil() as i32 {
			screen.blit(RgbaBlitMethod::Solid, &texture, x * texture.width() as i32, y * texture.height() as i32);
		}
	}
	// change the alpha value for all the pixels in the lower half of the screen to be half alpha
	for y in (screen.height() / 2)..screen.height() {
		for x in 0..screen.width() {
			unsafe {
				let pixel = screen.get_pixel_unchecked(x as i32, y as i32);
				let [r, g, b] = from_rgb32(pixel);
				screen.set_pixel_unchecked(x as i32, y as i32, to_argb32([127, r, g, b]));
			}
		}
	}
	screen
}

fn verify_visual(screen: &RgbaBitmap, source: &Path) -> bool {
	let (source_bmp, _) = RgbaBitmap::load_file(source).unwrap();
	*screen == source_bmp
}

#[test]
fn pixel_addressing() {
	let mut screen = setup();

	unsafe {
		let mut pixels = screen.pixels_at_mut_ptr(10, 10).unwrap();
		let mut i = 0;
		for _y in 0..16 {
			for _x in 0..16 {
				*pixels = to_rgb32([i, i, i]);
				i = i.wrapping_add(1);
				pixels = pixels.offset(1);
			}
			pixels = pixels.offset((SCREEN_WIDTH - 16) as isize);
		}
	}

	unsafe {
		let mut pixels = screen.pixels_at_mut_ptr(0, 0).unwrap();
		for _ in 0..10 {
			*pixels = COLOR_BRIGHT_WHITE;
			pixels = pixels.offset((SCREEN_WIDTH + 1) as isize);
		}
	}

	unsafe {
		let mut pixels = screen.pixels_at_mut_ptr(10, 0).unwrap();
		for _ in 0..10 {
			*pixels = COLOR_BRIGHT_WHITE;
			pixels = pixels.offset(SCREEN_WIDTH as isize);
		}
	}

	unsafe {
		let mut pixels = screen.pixels_at_mut_ptr(0, 10).unwrap();
		for _ in 0..10 {
			*pixels = COLOR_BRIGHT_WHITE;
			pixels = pixels.offset(1);
		}
	}

	let path = reference_file(Path::new("pixel_addressing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn pixel_drawing() {
	let mut screen = setup();

	screen.set_pixel(0, 0, COLOR_BLUE);
	screen.set_pixel(319, 0, COLOR_GREEN);
	screen.set_pixel(0, 239, COLOR_CYAN);
	screen.set_pixel(319, 239, COLOR_RED);

	unsafe {
		screen.set_pixel_unchecked(10, 0, COLOR_BLUE);
		screen.set_pixel_unchecked(309, 0, COLOR_GREEN);
		screen.set_pixel_unchecked(10, 239, COLOR_CYAN);
		screen.set_pixel_unchecked(309, 239, COLOR_RED);
	}

	let c1 = screen.get_pixel(0, 0).unwrap();
	let c2 = screen.get_pixel(319, 0).unwrap();
	let c3 = screen.get_pixel(0, 239).unwrap();
	let c4 = screen.get_pixel(319, 239).unwrap();

	screen.set_pixel(1, 1, c1);
	screen.set_pixel(318, 1, c2);
	screen.set_pixel(1, 238, c3);
	screen.set_pixel(318, 238, c4);

	unsafe {
		let c1 = screen.get_pixel_unchecked(10, 0);
		let c2 = screen.get_pixel_unchecked(309, 0);
		let c3 = screen.get_pixel_unchecked(10, 239);
		let c4 = screen.get_pixel_unchecked(309, 239);

		screen.set_pixel_unchecked(11, 1, c1);
		screen.set_pixel_unchecked(308, 1, c2);
		screen.set_pixel_unchecked(11, 238, c3);
		screen.set_pixel_unchecked(308, 238, c4);
	}

	//////

	for i in 0..10 {
		screen.set_pixel(5 - i, 100, COLOR_BRIGHT_WHITE);
		screen.set_pixel(i + 314, 100, COLOR_BRIGHT_WHITE);
		screen.set_pixel(160, 5 - i, COLOR_BRIGHT_WHITE);
		screen.set_pixel(160, i + 234, COLOR_BRIGHT_WHITE);
	}

	let path = reference_file(Path::new("pixel_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_pixel_drawing() {
	let mut screen = setup_for_blending();

	let blend = BlendFunction::Blend;

	for i in 0..10 {
		screen.set_blended_pixel(i, i, COLOR_BLUE_HALF_ALPHA, blend);
		screen.set_blended_pixel(319 - i, i, COLOR_GREEN_HALF_ALPHA, blend);
		screen.set_blended_pixel(i, 239 - i, COLOR_CYAN_HALF_ALPHA, blend);
		screen.set_blended_pixel(319 - i, 239 - i, COLOR_RED_HALF_ALPHA, blend);
	}

	unsafe {
		for i in 0..10 {
			screen.set_blended_pixel_unchecked(5 + i, i, COLOR_BLUE_HALF_ALPHA, blend);
			screen.set_blended_pixel_unchecked(314 - i, i, COLOR_GREEN_HALF_ALPHA, blend);
			screen.set_blended_pixel_unchecked(5 + i, 239 - i, COLOR_CYAN_HALF_ALPHA, blend);
			screen.set_blended_pixel_unchecked(314 - i, 239 - i, COLOR_RED_HALF_ALPHA, blend);
		}
	}

	//////

	for i in 0..10 {
		screen.set_blended_pixel(5 - i, 100, COLOR_BRIGHT_WHITE_HALF_ALPHA, blend);
		screen.set_blended_pixel(i + 314, 100, COLOR_BRIGHT_WHITE_HALF_ALPHA, blend);
		screen.set_blended_pixel(160, 5 - i, COLOR_BRIGHT_WHITE_HALF_ALPHA, blend);
		screen.set_blended_pixel(160, i + 234, COLOR_BRIGHT_WHITE_HALF_ALPHA, blend);
	}

	let path = reference_file(Path::new("blended_pixel_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn horiz_line_drawing() {
	let mut screen = setup();

	screen.horiz_line(10, 100, 20, COLOR_BLUE);
	screen.horiz_line(10, 100, 30, COLOR_GREEN);

	//////

	screen.horiz_line(-50, 50, 6, COLOR_CYAN);
	screen.horiz_line(300, 340, 130, COLOR_MAGENTA);

	screen.horiz_line(100, 200, -10, COLOR_BROWN);
	screen.horiz_line(20, 80, 250, COLOR_DARK_GRAY);

	let path = reference_file(Path::new("horiz_line_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_horiz_line_drawing() {
	let mut screen = setup_for_blending();

	let blend = BlendFunction::Blend;

	screen.blended_horiz_line(10, 100, 20, COLOR_BLUE_HALF_ALPHA, blend);
	screen.blended_horiz_line(10, 100, 30, COLOR_GREEN_HALF_ALPHA, blend);

	//////

	screen.blended_horiz_line(-50, 50, 6, COLOR_CYAN_HALF_ALPHA, blend);
	screen.blended_horiz_line(300, 340, 130, COLOR_MAGENTA_HALF_ALPHA, blend);

	screen.blended_horiz_line(100, 200, -10, COLOR_BROWN_HALF_ALPHA, blend);
	screen.blended_horiz_line(20, 80, 250, COLOR_LIGHT_GRAY_HALF_ALPHA, blend);

	let path = reference_file(Path::new("blended_horiz_line_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn vert_line_drawing() {
	let mut screen = setup();

	screen.vert_line(50, 10, 200, COLOR_BLUE);
	screen.vert_line(60, 10, 200, COLOR_GREEN);

	//////

	screen.vert_line(20, -32, 32, COLOR_CYAN);
	screen.vert_line(270, 245, 165, COLOR_MAGENTA);

	screen.vert_line(-17, 10, 20, COLOR_BROWN);
	screen.vert_line(400, 100, 300, COLOR_LIGHT_GRAY);

	let path = reference_file(Path::new("vert_line_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_vert_line_drawing() {
	let mut screen = setup_for_blending();

	let blend = BlendFunction::Blend;

	screen.blended_vert_line(50, 10, 200, COLOR_BLUE_HALF_ALPHA, blend);
	screen.blended_vert_line(60, 10, 200, COLOR_GREEN_HALF_ALPHA, blend);

	//////

	screen.blended_vert_line(20, -32, 32, COLOR_CYAN_HALF_ALPHA, blend);
	screen.blended_vert_line(270, 245, 165, COLOR_MAGENTA_HALF_ALPHA, blend);

	screen.blended_vert_line(-17, 10, 20, COLOR_BROWN_HALF_ALPHA, blend);
	screen.blended_vert_line(400, 100, 300, COLOR_LIGHT_GRAY_HALF_ALPHA, blend);

	let path = reference_file(Path::new("blended_vert_line_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn line_drawing() {
	let mut screen = setup();

	screen.line(10, 10, 20, 20, COLOR_BLUE);
	screen.line(10, 100, 20, 150, COLOR_GREEN);
	screen.line(60, 150, 50, 100, COLOR_CYAN);

	//////

	screen.line(50, 10, 100, 10, COLOR_MAGENTA);
	screen.line(100, 50, 20, 50, COLOR_BROWN);
	screen.line(290, 10, 290, 100, COLOR_LIGHT_GRAY);
	screen.line(310, 100, 310, 10, COLOR_DARK_GRAY);

	//////

	screen.line(50, 200, -50, 200, COLOR_MAGENTA);
	screen.line(300, 210, 340, 210, COLOR_BROWN);
	screen.line(120, -30, 120, 30, COLOR_LIGHT_GRAY);
	screen.line(130, 200, 130, 270, COLOR_DARK_GRAY);

	screen.line(250, 260, 190, 200, COLOR_BRIGHT_BLUE);
	screen.line(180, 30, 240, -30, COLOR_BRIGHT_GREEN);
	screen.line(-20, 140, 20, 180, COLOR_BRIGHT_CYAN);
	screen.line(300, 130, 340, 170, COLOR_BRIGHT_RED);

	screen.line(10, -30, 100, -30, COLOR_BLUE);
	screen.line(70, 250, 170, 250, COLOR_GREEN);
	screen.line(-100, 120, -100, 239, COLOR_CYAN);
	screen.line(320, 99, 320, 199, COLOR_MAGENTA);

	let path = reference_file(Path::new("line_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_line_drawing() {
	let mut screen = setup_for_blending();

	let blend = BlendFunction::Blend;

	screen.blended_line(10, 10, 20, 20, COLOR_BLUE_HALF_ALPHA, blend);
	screen.blended_line(10, 100, 20, 150, COLOR_GREEN_HALF_ALPHA, blend);
	screen.blended_line(60, 150, 50, 100, COLOR_CYAN_HALF_ALPHA, blend);

	//////

	screen.blended_line(50, 10, 100, 10, COLOR_MAGENTA_HALF_ALPHA, blend);
	screen.blended_line(100, 50, 20, 50, COLOR_BROWN_HALF_ALPHA, blend);
	screen.blended_line(290, 10, 290, 100, COLOR_LIGHT_GRAY_HALF_ALPHA, blend);
	screen.blended_line(310, 100, 310, 10, COLOR_DARK_GRAY_HALF_ALPHA, blend);

	//////

	screen.blended_line(50, 200, -50, 200, COLOR_MAGENTA_HALF_ALPHA, blend);
	screen.blended_line(300, 210, 340, 210, COLOR_BROWN_HALF_ALPHA, blend);
	screen.blended_line(120, -30, 120, 30, COLOR_LIGHT_GRAY_HALF_ALPHA, blend);
	screen.blended_line(130, 200, 130, 270, COLOR_DARK_GRAY_HALF_ALPHA, blend);

	screen.blended_line(250, 260, 190, 200, COLOR_BRIGHT_BLUE_HALF_ALPHA, blend);
	screen.blended_line(180, 30, 240, -30, COLOR_BRIGHT_GREEN_HALF_ALPHA, blend);
	screen.blended_line(-20, 140, 20, 180, COLOR_BRIGHT_CYAN_HALF_ALPHA, blend);
	screen.blended_line(300, 130, 340, 170, COLOR_BRIGHT_RED_HALF_ALPHA, blend);

	screen.blended_line(10, -30, 100, -30, COLOR_BLUE_HALF_ALPHA, blend);
	screen.blended_line(70, 250, 170, 250, COLOR_GREEN_HALF_ALPHA, blend);
	screen.blended_line(-100, 120, -100, 239, COLOR_CYAN_HALF_ALPHA, blend);
	screen.blended_line(320, 99, 320, 199, COLOR_MAGENTA_HALF_ALPHA, blend);

	let path = reference_file(Path::new("blended_line_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn rect_drawing() {
	let mut screen = setup();

	screen.rect(10, 10, 90, 90, COLOR_BLUE);
	screen.rect(10, 110, 90, 190, COLOR_GREEN);
	screen.rect(190, 90, 110, 10, COLOR_CYAN);

	//////

	screen.rect(-8, 10, 7, 25, COLOR_MAGENTA);
	screen.rect(20, -8, 35, 7, COLOR_BROWN);
	screen.rect(313, 170, 328, 185, COLOR_LIGHT_GRAY);
	screen.rect(285, 233, 300, 248, COLOR_DARK_GRAY);

	screen.rect(-16, 30, -1, 46, COLOR_BRIGHT_BLUE);
	screen.rect(40, -16, 55, -1, COLOR_BRIGHT_GREEN);
	screen.rect(320, 150, 335, 165, COLOR_BRIGHT_CYAN);
	screen.rect(265, 240, 280, 255, COLOR_BRIGHT_RED);

	screen.rect(300, 20, 340, -20, COLOR_BRIGHT_MAGENTA);
	screen.rect(20, 220, -20, 260, COLOR_BRIGHT_YELLOW);

	let path = reference_file(Path::new("rect_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_rect_drawing() {
	let mut screen = setup_for_blending();

	let blend = BlendFunction::Blend;

	screen.blended_rect(10, 10, 90, 90, COLOR_BLUE_HALF_ALPHA, blend);
	screen.blended_rect(10, 110, 90, 190, COLOR_GREEN_HALF_ALPHA, blend);
	screen.blended_rect(190, 90, 110, 10, COLOR_CYAN_HALF_ALPHA, blend);

	//////

	screen.blended_rect(-8, 10, 7, 25, COLOR_MAGENTA_HALF_ALPHA, blend);
	screen.blended_rect(20, -8, 35, 7, COLOR_BROWN_HALF_ALPHA, blend);
	screen.blended_rect(313, 170, 328, 185, COLOR_LIGHT_GRAY_HALF_ALPHA, blend);
	screen.blended_rect(285, 233, 300, 248, COLOR_DARK_GRAY_HALF_ALPHA, blend);

	screen.blended_rect(-16, 30, -1, 46, COLOR_BRIGHT_BLUE_HALF_ALPHA, blend);
	screen.blended_rect(40, -16, 55, -1, COLOR_BRIGHT_GREEN_HALF_ALPHA, blend);
	screen.blended_rect(320, 150, 335, 165, COLOR_BRIGHT_CYAN_HALF_ALPHA, blend);
	screen.blended_rect(265, 240, 280, 255, COLOR_BRIGHT_RED_HALF_ALPHA, blend);

	screen.blended_rect(300, 20, 340, -20, COLOR_BRIGHT_MAGENTA_HALF_ALPHA, blend);
	screen.blended_rect(20, 220, -20, 260, COLOR_BRIGHT_YELLOW_HALF_ALPHA, blend);

	let path = reference_file(Path::new("blended_rect_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn filled_rect_drawing() {
	let mut screen = setup();

	screen.filled_rect(10, 10, 90, 90, COLOR_BLUE);
	screen.filled_rect(10, 110, 90, 190, COLOR_GREEN);
	screen.filled_rect(190, 90, 110, 10, COLOR_CYAN);

	//////

	screen.filled_rect(-8, 10, 7, 25, COLOR_MAGENTA);
	screen.filled_rect(20, -8, 35, 7, COLOR_BROWN);
	screen.filled_rect(313, 170, 328, 185, COLOR_LIGHT_GRAY);
	screen.filled_rect(285, 233, 300, 248, COLOR_DARK_GRAY);

	screen.filled_rect(-16, 30, -1, 46, COLOR_BRIGHT_BLUE);
	screen.filled_rect(40, -16, 55, -1, COLOR_BRIGHT_GREEN);
	screen.filled_rect(320, 150, 335, 165, COLOR_BRIGHT_CYAN);
	screen.filled_rect(265, 240, 280, 255, COLOR_BRIGHT_RED);

	screen.filled_rect(300, 20, 340, -20, COLOR_BRIGHT_MAGENTA);
	screen.filled_rect(20, 220, -20, 260, COLOR_BRIGHT_YELLOW);

	let path = reference_file(Path::new("filled_rect_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_filled_rect_drawing() {
	let mut screen = setup_for_blending();

	let blend = BlendFunction::Blend;

	screen.blended_filled_rect(10, 10, 90, 90, COLOR_BLUE_HALF_ALPHA, blend);
	screen.blended_filled_rect(10, 110, 90, 190, COLOR_GREEN_HALF_ALPHA, blend);
	screen.blended_filled_rect(190, 90, 110, 10, COLOR_CYAN_HALF_ALPHA, blend);

	//////

	screen.blended_filled_rect(-8, 10, 7, 25, COLOR_MAGENTA_HALF_ALPHA, blend);
	screen.blended_filled_rect(20, -8, 35, 7, COLOR_BROWN_HALF_ALPHA, blend);
	screen.blended_filled_rect(313, 170, 328, 185, COLOR_LIGHT_GRAY_HALF_ALPHA, blend);
	screen.blended_filled_rect(285, 233, 300, 248, COLOR_DARK_GRAY_HALF_ALPHA, blend);

	screen.blended_filled_rect(-16, 30, -1, 46, COLOR_BRIGHT_BLUE_HALF_ALPHA, blend);
	screen.blended_filled_rect(40, -16, 55, -1, COLOR_BRIGHT_GREEN_HALF_ALPHA, blend);
	screen.blended_filled_rect(320, 150, 335, 165, COLOR_BRIGHT_CYAN_HALF_ALPHA, blend);
	screen.blended_filled_rect(265, 240, 280, 255, COLOR_BRIGHT_RED_HALF_ALPHA, blend);

	screen.blended_filled_rect(300, 20, 340, -20, COLOR_BRIGHT_MAGENTA_HALF_ALPHA, blend);
	screen.blended_filled_rect(20, 220, -20, 260, COLOR_BRIGHT_YELLOW_HALF_ALPHA, blend);

	let path = reference_file(Path::new("blended_filled_rect_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn circle_drawing() {
	let mut screen = setup();

	screen.circle(48, 48, 32, COLOR_BLUE);
	screen.circle(128, 48, 24, COLOR_GREEN);
	screen.circle(48, 128, 40, COLOR_CYAN);

	//////

	screen.circle(0, 30, 16, COLOR_MAGENTA);
	screen.circle(40, 2, 11, COLOR_BROWN);
	screen.circle(319, 211, 17, COLOR_LIGHT_GRAY);
	screen.circle(290, 241, 21, COLOR_DARK_GRAY);

	screen.circle(319, 1, 22, COLOR_BRIGHT_BLUE);
	screen.circle(2, 242, 19, COLOR_BRIGHT_GREEN);

	let path = reference_file(Path::new("circle_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn filled_circle_drawing() {
	let mut screen = setup();

	screen.filled_circle(48, 48, 32, COLOR_BLUE);
	screen.filled_circle(128, 48, 24, COLOR_GREEN);
	screen.filled_circle(48, 128, 40, COLOR_CYAN);

	//////

	screen.filled_circle(0, 30, 16, COLOR_MAGENTA);
	screen.filled_circle(40, 2, 11, COLOR_BROWN);
	screen.filled_circle(319, 211, 17, COLOR_LIGHT_GRAY);
	screen.filled_circle(290, 241, 21, COLOR_DARK_GRAY);

	screen.filled_circle(319, 1, 22, COLOR_BRIGHT_BLUE);
	screen.filled_circle(2, 242, 19, COLOR_BRIGHT_GREEN);

	let path = reference_file(Path::new("filled_circle_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn text_drawing() {
	let mut screen = setup();

	let font = BitmaskFont::new_vga_font().unwrap();
	let small_font = BitmaskFont::load_from_file(test_assets_file(Path::new("small.fnt")).as_path()).unwrap();
	let chunky_font = BitmaskFont::load_from_file(test_assets_file(Path::new("chunky.fnt")).as_path()).unwrap();

	let message = "Hello, world! HELLO, WORLD!\nTesting 123";

	screen.print_string(message, 20, 20, FontRenderOpts::Color(COLOR_BLUE), &font);
	screen.print_string(message, 20, 40, FontRenderOpts::Color(COLOR_GREEN), &small_font);
	screen.print_string(message, 20, 60, FontRenderOpts::Color(COLOR_CYAN), &chunky_font);

	screen.filled_rect(58, 218, 162, 230, COLOR_LIGHT_GRAY);
	screen.print_string("transparency!", 60, 220, FontRenderOpts::Color(COLOR_BRIGHT_BLUE), &font);

	let mut s = String::with_capacity(256);
	for i in 1..=127 {
		if i % 8 == 0 {
			s += "\n";
		}
		if i == 10 {
			s += " ";
		} else {
			s += &char::from(i).to_string();
		}
	}

	screen.print_string(&s, 20, 80, FontRenderOpts::Color(COLOR_BRIGHT_WHITE), &font);
	screen.print_string(&s, 110, 80, FontRenderOpts::Color(COLOR_BRIGHT_WHITE), &small_font);
	screen.print_string(&s, 190, 80, FontRenderOpts::Color(COLOR_BRIGHT_WHITE), &chunky_font);

	//////

	let message = "Hello, world!";

	screen.print_string(message, -35, 10, FontRenderOpts::Color(COLOR_BRIGHT_BLUE), &font);
	screen.print_string(message, 80, -4, FontRenderOpts::Color(COLOR_BRIGHT_GREEN), &font);
	screen.print_string(message, 285, 120, FontRenderOpts::Color(COLOR_BRIGHT_CYAN), &font);
	screen.print_string(message, 200, 236, FontRenderOpts::Color(COLOR_BRIGHT_RED), &font);
	screen.print_string(message, -232, 10, FontRenderOpts::Color(COLOR_MAGENTA), &font);
	screen.print_string(message, 80, -24, FontRenderOpts::Color(COLOR_BROWN), &font);
	screen.print_string(message, 360, 120, FontRenderOpts::Color(COLOR_LIGHT_GRAY), &font);
	screen.print_string(message, 200, 250, FontRenderOpts::Color(COLOR_DARK_GRAY), &font);

	let path = reference_file(Path::new("text_drawing.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

fn generate_bitmap(width: i32, height: i32) -> RgbaBitmap {
	let x_third = width / 3;
	let y_third = height / 3;

	let mut bitmap = RgbaBitmap::new(width as u32, height as u32).unwrap();

	bitmap.filled_rect(0, 0, x_third, y_third, COLOR_BLUE);
	bitmap.filled_rect(x_third * 2 + 1, y_third * 2 + 1, width - 1, height - 1, COLOR_GREEN);
	bitmap.filled_rect(0, y_third * 2 + 1, x_third, height - 1, COLOR_CYAN);
	bitmap.filled_rect(x_third * 2 + 1, 0, width - 1, y_third, COLOR_RED);
	bitmap.filled_rect(x_third, y_third, x_third * 2 + 1, y_third * 2 + 1, COLOR_MAGENTA);
	bitmap.rect(0, 0, width - 1, height - 1, COLOR_BROWN);

	bitmap
}

fn generate_bitmap_with_varied_alpha(width: i32, height: i32) -> RgbaBitmap {
	let x_third = width / 3;
	let y_third = height / 3;

	let mut bitmap = RgbaBitmap::new(width as u32, height as u32).unwrap();
	bitmap.clear(0); // alpha=0

	bitmap.filled_rect(0, 0, x_third, y_third, 0x330000aa);
	bitmap.filled_rect(x_third * 2 + 1, y_third * 2 + 1, width - 1, height - 1, 0x6600aa00);
	bitmap.filled_rect(0, y_third * 2 + 1, x_third, height - 1, 0x9900aaaa);
	bitmap.filled_rect(x_third * 2 + 1, 0, width - 1, y_third, 0xccaa0000);
	bitmap.filled_rect(x_third, y_third, x_third * 2 + 1, y_third * 2 + 1, COLOR_MAGENTA);
	bitmap.rect(0, 0, width - 1, height - 1, COLOR_BROWN);

	bitmap
}

fn generate_solid_bitmap_with_varied_alpha(width: i32, height: i32) -> RgbaBitmap {
	let x_third = width / 3;
	let y_third = height / 3;

	let mut bitmap = RgbaBitmap::new(width as u32, height as u32).unwrap();
	bitmap.clear(to_argb32([255, 0, 0, 0]));

	bitmap.filled_rect(0, 0, x_third, y_third, 0x330000aa);
	bitmap.filled_rect(x_third * 2 + 1, y_third * 2 + 1, width - 1, height - 1, 0x6600aa00);
	bitmap.filled_rect(0, y_third * 2 + 1, x_third, height - 1, 0x9900aaaa);
	bitmap.filled_rect(x_third * 2 + 1, 0, width - 1, y_third, 0xccaa0000);
	bitmap.filled_rect(x_third, y_third, x_third * 2 + 1, y_third * 2 + 1, COLOR_MAGENTA);
	bitmap.rect(0, 0, width - 1, height - 1, COLOR_BROWN);

	bitmap
}

#[test]
fn solid_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let bmp16 = generate_bitmap(16, 16);
	let bmp12 = generate_bitmap(12, 12);
	let bmp21 = generate_bitmap(21, 21);
	let bmp3 = generate_bitmap(3, 3);

	let method = Solid;

	let x = 40;
	let y = 20;
	screen.blit(method.clone(), &bmp16, x + 16, y + 48);
	screen.blit(method.clone(), &bmp12, x + 80, y + 48);
	screen.blit(method.clone(), &bmp21, x + 144, y + 48);
	screen.blit(method.clone(), &bmp3, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(method.clone(), &bmp16, x + 16, y + 48);
		screen.blit_unchecked(method.clone(), &bmp12, x + 80, y + 48);
		screen.blit_unchecked(method.clone(), &bmp21, x + 144, y + 48);
		screen.blit_unchecked(method.clone(), &bmp3, x + 208, y + 48);
	}

	//////

	screen.blit(method.clone(), &bmp16, -3, 46);
	screen.blit(method.clone(), &bmp16, -4, 76);
	screen.blit(method.clone(), &bmp16, -8, 106);
	screen.blit(method.clone(), &bmp16, -12, 136);
	screen.blit(method.clone(), &bmp16, -13, 166);
	screen.blit(method.clone(), &bmp16, -14, 196);
	screen.blit(method.clone(), &bmp16, -16, 226);

	screen.blit(method.clone(), &bmp16, 46, -3);
	screen.blit(method.clone(), &bmp16, 76, -4);
	screen.blit(method.clone(), &bmp16, 106, -8);
	screen.blit(method.clone(), &bmp16, 136, -12);
	screen.blit(method.clone(), &bmp16, 166, -13);
	screen.blit(method.clone(), &bmp16, 196, -14);
	screen.blit(method.clone(), &bmp16, 226, -16);

	screen.blit(method.clone(), &bmp16, 307, 46);
	screen.blit(method.clone(), &bmp16, 308, 76);
	screen.blit(method.clone(), &bmp16, 312, 106);
	screen.blit(method.clone(), &bmp16, 316, 136);
	screen.blit(method.clone(), &bmp16, 317, 166);
	screen.blit(method.clone(), &bmp16, 318, 196);
	screen.blit(method.clone(), &bmp16, 320, 226);

	screen.blit(method.clone(), &bmp16, 46, 227);
	screen.blit(method.clone(), &bmp16, 76, 228);
	screen.blit(method.clone(), &bmp16, 106, 232);
	screen.blit(method.clone(), &bmp16, 136, 236);
	screen.blit(method.clone(), &bmp16, 166, 237);
	screen.blit(method.clone(), &bmp16, 196, 238);
	screen.blit(method.clone(), &bmp16, 226, 240);

	let path = reference_file(Path::new("solid_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn solid_tinted_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let bmp16 = generate_bitmap(16, 16);
	let bmp12 = generate_bitmap(12, 12);
	let bmp21 = generate_bitmap(21, 21);
	let bmp3 = generate_bitmap(3, 3);

	let method = SolidTinted(to_argb32([127, 155, 242, 21]));

	let x = 40;
	let y = 20;
	screen.blit(method.clone(), &bmp16, x + 16, y + 48);
	screen.blit(method.clone(), &bmp12, x + 80, y + 48);
	screen.blit(method.clone(), &bmp21, x + 144, y + 48);
	screen.blit(method.clone(), &bmp3, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(method.clone(), &bmp16, x + 16, y + 48);
		screen.blit_unchecked(method.clone(), &bmp12, x + 80, y + 48);
		screen.blit_unchecked(method.clone(), &bmp21, x + 144, y + 48);
		screen.blit_unchecked(method.clone(), &bmp3, x + 208, y + 48);
	}

	//////

	screen.blit(method.clone(), &bmp16, -3, 46);
	screen.blit(method.clone(), &bmp16, -4, 76);
	screen.blit(method.clone(), &bmp16, -8, 106);
	screen.blit(method.clone(), &bmp16, -12, 136);
	screen.blit(method.clone(), &bmp16, -13, 166);
	screen.blit(method.clone(), &bmp16, -14, 196);
	screen.blit(method.clone(), &bmp16, -16, 226);

	screen.blit(method.clone(), &bmp16, 46, -3);
	screen.blit(method.clone(), &bmp16, 76, -4);
	screen.blit(method.clone(), &bmp16, 106, -8);
	screen.blit(method.clone(), &bmp16, 136, -12);
	screen.blit(method.clone(), &bmp16, 166, -13);
	screen.blit(method.clone(), &bmp16, 196, -14);
	screen.blit(method.clone(), &bmp16, 226, -16);

	screen.blit(method.clone(), &bmp16, 307, 46);
	screen.blit(method.clone(), &bmp16, 308, 76);
	screen.blit(method.clone(), &bmp16, 312, 106);
	screen.blit(method.clone(), &bmp16, 316, 136);
	screen.blit(method.clone(), &bmp16, 317, 166);
	screen.blit(method.clone(), &bmp16, 318, 196);
	screen.blit(method.clone(), &bmp16, 320, 226);

	screen.blit(method.clone(), &bmp16, 46, 227);
	screen.blit(method.clone(), &bmp16, 76, 228);
	screen.blit(method.clone(), &bmp16, 106, 232);
	screen.blit(method.clone(), &bmp16, 136, 236);
	screen.blit(method.clone(), &bmp16, 166, 237);
	screen.blit(method.clone(), &bmp16, 196, 238);
	screen.blit(method.clone(), &bmp16, 226, 240);

	let path = reference_file(Path::new("solid_tinted_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_solid_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup_for_blending();

	let bmp16 = generate_solid_bitmap_with_varied_alpha(16, 16);
	let bmp12 = generate_solid_bitmap_with_varied_alpha(12, 12);
	let bmp21 = generate_solid_bitmap_with_varied_alpha(21, 21);
	let bmp3 = generate_solid_bitmap_with_varied_alpha(3, 3);

	let method = SolidBlended(BlendFunction::Blend);

	let x = 40;
	let y = 20;
	screen.blit(method.clone(), &bmp16, x + 16, y + 48);
	screen.blit(method.clone(), &bmp12, x + 80, y + 48);
	screen.blit(method.clone(), &bmp21, x + 144, y + 48);
	screen.blit(method.clone(), &bmp3, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(method.clone(), &bmp16, x + 16, y + 48);
		screen.blit_unchecked(method.clone(), &bmp12, x + 80, y + 48);
		screen.blit_unchecked(method.clone(), &bmp21, x + 144, y + 48);
		screen.blit_unchecked(method.clone(), &bmp3, x + 208, y + 48);
	}

	//////

	screen.blit(method.clone(), &bmp16, -3, 46);
	screen.blit(method.clone(), &bmp16, -4, 76);
	screen.blit(method.clone(), &bmp16, -8, 106);
	screen.blit(method.clone(), &bmp16, -12, 136);
	screen.blit(method.clone(), &bmp16, -13, 166);
	screen.blit(method.clone(), &bmp16, -14, 196);
	screen.blit(method.clone(), &bmp16, -16, 226);

	screen.blit(method.clone(), &bmp16, 46, -3);
	screen.blit(method.clone(), &bmp16, 76, -4);
	screen.blit(method.clone(), &bmp16, 106, -8);
	screen.blit(method.clone(), &bmp16, 136, -12);
	screen.blit(method.clone(), &bmp16, 166, -13);
	screen.blit(method.clone(), &bmp16, 196, -14);
	screen.blit(method.clone(), &bmp16, 226, -16);

	screen.blit(method.clone(), &bmp16, 307, 46);
	screen.blit(method.clone(), &bmp16, 308, 76);
	screen.blit(method.clone(), &bmp16, 312, 106);
	screen.blit(method.clone(), &bmp16, 316, 136);
	screen.blit(method.clone(), &bmp16, 317, 166);
	screen.blit(method.clone(), &bmp16, 318, 196);
	screen.blit(method.clone(), &bmp16, 320, 226);

	screen.blit(method.clone(), &bmp16, 46, 227);
	screen.blit(method.clone(), &bmp16, 76, 228);
	screen.blit(method.clone(), &bmp16, 106, 232);
	screen.blit(method.clone(), &bmp16, 136, 236);
	screen.blit(method.clone(), &bmp16, 166, 237);
	screen.blit(method.clone(), &bmp16, 196, 238);
	screen.blit(method.clone(), &bmp16, 226, 240);

	let path = reference_file(Path::new("blended_solid_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn solid_flipped_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(SolidFlipped { horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(SolidFlipped { horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(SolidFlipped { horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(SolidFlipped { horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: false }, &bmp, -3, 46);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: false }, &bmp, -4, 76);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: true }, &bmp, -8, 106);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: true }, &bmp, -12, 136);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: true }, &bmp, -13, 166);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: false }, &bmp, -14, 196);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: true }, &bmp, -16, 226);

	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: false }, &bmp, 46, -3);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: false }, &bmp, 76, -4);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: true }, &bmp, 106, -8);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: true }, &bmp, 136, -12);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: false }, &bmp, 166, -13);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: false }, &bmp, 196, -14);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: true }, &bmp, 226, -16);

	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: false }, &bmp, 307, 46);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: false }, &bmp, 308, 76);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: true }, &bmp, 312, 106);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: true }, &bmp, 316, 136);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: false }, &bmp, 317, 166);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: false }, &bmp, 318, 196);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: true }, &bmp, 320, 226);

	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: false }, &bmp, 46, 227);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: false }, &bmp, 76, 228);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: true }, &bmp, 106, 232);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: true }, &bmp, 136, 236);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: false }, &bmp, 166, 237);
	screen.blit(SolidFlipped { horizontal_flip: true, vertical_flip: false }, &bmp, 196, 238);
	screen.blit(SolidFlipped { horizontal_flip: false, vertical_flip: true }, &bmp, 226, 240);

	let path = reference_file(Path::new("solid_flipped_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn solid_flipped_tinted_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let bmp = generate_bitmap(16, 16);

	let tint_color = to_argb32([127, 155, 242, 21]);

	let x = 40;
	let y = 20;
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, -3, 46);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, -4, 76);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, -8, 106);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: true }, &bmp, -12, 136);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, -13, 166);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, -14, 196);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, -16, 226);

	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, 46, -3);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, 76, -4);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, 106, -8);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: true }, &bmp, 136, -12);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, 166, -13);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, 196, -14);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, 226, -16);

	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, 307, 46);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, 308, 76);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, 312, 106);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: true }, &bmp, 316, 136);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, 317, 166);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, 318, 196);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, 320, 226);

	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, 46, 227);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, 76, 228);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, 106, 232);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: true }, &bmp, 136, 236);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, 166, 237);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, 196, 238);
	screen.blit(SolidFlippedTinted { tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, 226, 240);

	let path = reference_file(Path::new("solid_flipped_tinted_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn blended_solid_flipped_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup_for_blending();

	let bmp = generate_solid_bitmap_with_varied_alpha(16, 16);

	let blend = BlendFunction::Blend;

	let x = 40;
	let y = 20;
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend }, &bmp, x + 16, y + 48);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend }, &bmp, x + 80, y + 48);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend }, &bmp, x + 144, y + 48);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: true, blend }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(SolidFlippedBlended { horizontal_flip: true, vertical_flip: true, blend }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend }, &bmp, -3, 46);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend }, &bmp, -4, 76);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend }, &bmp, -8, 106);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: true, blend }, &bmp, -12, 136);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend }, &bmp, -13, 166);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend }, &bmp, -14, 196);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend }, &bmp, -16, 226);

	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend }, &bmp, 46, -3);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend }, &bmp, 76, -4);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend }, &bmp, 106, -8);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: true, blend }, &bmp, 136, -12);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend }, &bmp, 166, -13);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend }, &bmp, 196, -14);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend }, &bmp, 226, -16);

	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend }, &bmp, 307, 46);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend }, &bmp, 308, 76);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend }, &bmp, 312, 106);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: true, blend }, &bmp, 316, 136);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend }, &bmp, 317, 166);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend }, &bmp, 318, 196);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend }, &bmp, 320, 226);

	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend }, &bmp, 46, 227);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend }, &bmp, 76, 228);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend }, &bmp, 106, 232);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: true, blend }, &bmp, 136, 236);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend }, &bmp, 166, 237);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend }, &bmp, 196, 238);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend }, &bmp, 226, 240);

	let path = reference_file(Path::new("blended_solid_flipped_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn transparent_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let bmp16 = generate_bitmap(16, 16);
	let bmp12 = generate_bitmap(12, 12);
	let bmp21 = generate_bitmap(21, 21);
	let bmp3 = generate_bitmap(3, 3);

	let method = Transparent(to_rgb32([0, 0, 0]));

	let x = 40;
	let y = 20;
	screen.blit(method.clone(), &bmp16, x + 16, y + 48);
	screen.blit(method.clone(), &bmp12, x + 80, y + 48);
	screen.blit(method.clone(), &bmp21, x + 144, y + 48);
	screen.blit(method.clone(), &bmp3, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(method.clone(), &bmp16, x + 16, y + 48);
		screen.blit_unchecked(method.clone(), &bmp12, x + 80, y + 48);
		screen.blit_unchecked(method.clone(), &bmp21, x + 144, y + 48);
		screen.blit_unchecked(method.clone(), &bmp3, x + 208, y + 48);
	}

	//////

	screen.blit(method.clone(), &bmp16, -3, 46);
	screen.blit(method.clone(), &bmp16, -4, 76);
	screen.blit(method.clone(), &bmp16, -8, 106);
	screen.blit(method.clone(), &bmp16, -12, 136);
	screen.blit(method.clone(), &bmp16, -13, 166);
	screen.blit(method.clone(), &bmp16, -14, 196);
	screen.blit(method.clone(), &bmp16, -16, 226);

	screen.blit(method.clone(), &bmp16, 46, -3);
	screen.blit(method.clone(), &bmp16, 76, -4);
	screen.blit(method.clone(), &bmp16, 106, -8);
	screen.blit(method.clone(), &bmp16, 136, -12);
	screen.blit(method.clone(), &bmp16, 166, -13);
	screen.blit(method.clone(), &bmp16, 196, -14);
	screen.blit(method.clone(), &bmp16, 226, -16);

	screen.blit(method.clone(), &bmp16, 307, 46);
	screen.blit(method.clone(), &bmp16, 308, 76);
	screen.blit(method.clone(), &bmp16, 312, 106);
	screen.blit(method.clone(), &bmp16, 316, 136);
	screen.blit(method.clone(), &bmp16, 317, 166);
	screen.blit(method.clone(), &bmp16, 318, 196);
	screen.blit(method.clone(), &bmp16, 320, 226);

	screen.blit(method.clone(), &bmp16, 46, 227);
	screen.blit(method.clone(), &bmp16, 76, 228);
	screen.blit(method.clone(), &bmp16, 106, 232);
	screen.blit(method.clone(), &bmp16, 136, 236);
	screen.blit(method.clone(), &bmp16, 166, 237);
	screen.blit(method.clone(), &bmp16, 196, 238);
	screen.blit(method.clone(), &bmp16, 226, 240);

	let path = reference_file(Path::new("transparent_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn transparent_tinted_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let bmp16 = generate_bitmap(16, 16);
	let bmp12 = generate_bitmap(12, 12);
	let bmp21 = generate_bitmap(21, 21);
	let bmp3 = generate_bitmap(3, 3);

	let method =
		TransparentTinted { transparent_color: to_rgb32([0, 0, 0]), tint_color: to_argb32([127, 155, 242, 21]) };

	let x = 40;
	let y = 20;
	screen.blit(method.clone(), &bmp16, x + 16, y + 48);
	screen.blit(method.clone(), &bmp12, x + 80, y + 48);
	screen.blit(method.clone(), &bmp21, x + 144, y + 48);
	screen.blit(method.clone(), &bmp3, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(method.clone(), &bmp16, x + 16, y + 48);
		screen.blit_unchecked(method.clone(), &bmp12, x + 80, y + 48);
		screen.blit_unchecked(method.clone(), &bmp21, x + 144, y + 48);
		screen.blit_unchecked(method.clone(), &bmp3, x + 208, y + 48);
	}

	//////

	screen.blit(method.clone(), &bmp16, -3, 46);
	screen.blit(method.clone(), &bmp16, -4, 76);
	screen.blit(method.clone(), &bmp16, -8, 106);
	screen.blit(method.clone(), &bmp16, -12, 136);
	screen.blit(method.clone(), &bmp16, -13, 166);
	screen.blit(method.clone(), &bmp16, -14, 196);
	screen.blit(method.clone(), &bmp16, -16, 226);

	screen.blit(method.clone(), &bmp16, 46, -3);
	screen.blit(method.clone(), &bmp16, 76, -4);
	screen.blit(method.clone(), &bmp16, 106, -8);
	screen.blit(method.clone(), &bmp16, 136, -12);
	screen.blit(method.clone(), &bmp16, 166, -13);
	screen.blit(method.clone(), &bmp16, 196, -14);
	screen.blit(method.clone(), &bmp16, 226, -16);

	screen.blit(method.clone(), &bmp16, 307, 46);
	screen.blit(method.clone(), &bmp16, 308, 76);
	screen.blit(method.clone(), &bmp16, 312, 106);
	screen.blit(method.clone(), &bmp16, 316, 136);
	screen.blit(method.clone(), &bmp16, 317, 166);
	screen.blit(method.clone(), &bmp16, 318, 196);
	screen.blit(method.clone(), &bmp16, 320, 226);

	screen.blit(method.clone(), &bmp16, 46, 227);
	screen.blit(method.clone(), &bmp16, 76, 228);
	screen.blit(method.clone(), &bmp16, 106, 232);
	screen.blit(method.clone(), &bmp16, 136, 236);
	screen.blit(method.clone(), &bmp16, 166, 237);
	screen.blit(method.clone(), &bmp16, 196, 238);
	screen.blit(method.clone(), &bmp16, 226, 240);

	let path = reference_file(Path::new("transparent_tinted_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_transparent_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup_for_blending();

	let bmp16 = generate_solid_bitmap_with_varied_alpha(16, 16);
	let bmp12 = generate_solid_bitmap_with_varied_alpha(12, 12);
	let bmp21 = generate_solid_bitmap_with_varied_alpha(21, 21);
	let bmp3 = generate_solid_bitmap_with_varied_alpha(3, 3);

	let method = TransparentBlended { transparent_color: to_argb32([255, 0, 0, 0]), blend: BlendFunction::Blend };

	let x = 40;
	let y = 20;
	screen.blit(method.clone(), &bmp16, x + 16, y + 48);
	screen.blit(method.clone(), &bmp12, x + 80, y + 48);
	screen.blit(method.clone(), &bmp21, x + 144, y + 48);
	screen.blit(method.clone(), &bmp3, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(method.clone(), &bmp16, x + 16, y + 48);
		screen.blit_unchecked(method.clone(), &bmp12, x + 80, y + 48);
		screen.blit_unchecked(method.clone(), &bmp21, x + 144, y + 48);
		screen.blit_unchecked(method.clone(), &bmp3, x + 208, y + 48);
	}

	//////

	screen.blit(method.clone(), &bmp16, -3, 46);
	screen.blit(method.clone(), &bmp16, -4, 76);
	screen.blit(method.clone(), &bmp16, -8, 106);
	screen.blit(method.clone(), &bmp16, -12, 136);
	screen.blit(method.clone(), &bmp16, -13, 166);
	screen.blit(method.clone(), &bmp16, -14, 196);
	screen.blit(method.clone(), &bmp16, -16, 226);

	screen.blit(method.clone(), &bmp16, 46, -3);
	screen.blit(method.clone(), &bmp16, 76, -4);
	screen.blit(method.clone(), &bmp16, 106, -8);
	screen.blit(method.clone(), &bmp16, 136, -12);
	screen.blit(method.clone(), &bmp16, 166, -13);
	screen.blit(method.clone(), &bmp16, 196, -14);
	screen.blit(method.clone(), &bmp16, 226, -16);

	screen.blit(method.clone(), &bmp16, 307, 46);
	screen.blit(method.clone(), &bmp16, 308, 76);
	screen.blit(method.clone(), &bmp16, 312, 106);
	screen.blit(method.clone(), &bmp16, 316, 136);
	screen.blit(method.clone(), &bmp16, 317, 166);
	screen.blit(method.clone(), &bmp16, 318, 196);
	screen.blit(method.clone(), &bmp16, 320, 226);

	screen.blit(method.clone(), &bmp16, 46, 227);
	screen.blit(method.clone(), &bmp16, 76, 228);
	screen.blit(method.clone(), &bmp16, 106, 232);
	screen.blit(method.clone(), &bmp16, 136, 236);
	screen.blit(method.clone(), &bmp16, 166, 237);
	screen.blit(method.clone(), &bmp16, 196, 238);
	screen.blit(method.clone(), &bmp16, 226, 240);

	let path = reference_file(Path::new("blended_transparent_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn transparent_flipped_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let transparent_color = to_rgb32([0, 0, 0]);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: false }, &bmp, -3, 46);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: false }, &bmp, -4, 76);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: true }, &bmp, -8, 106);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: true }, &bmp, -12, 136);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: true }, &bmp, -13, 166);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: false }, &bmp, -14, 196);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: true }, &bmp, -16, 226);

	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: false }, &bmp, 46, -3);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: false }, &bmp, 76, -4);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: true }, &bmp, 106, -8);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: true }, &bmp, 136, -12);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: false }, &bmp, 166, -13);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: false }, &bmp, 196, -14);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: true }, &bmp, 226, -16);

	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: false }, &bmp, 307, 46);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: false }, &bmp, 308, 76);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: true }, &bmp, 312, 106);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: true }, &bmp, 316, 136);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: false }, &bmp, 317, 166);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: false }, &bmp, 318, 196);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: true }, &bmp, 320, 226);

	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: false }, &bmp, 46, 227);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: false }, &bmp, 76, 228);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: true }, &bmp, 106, 232);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: true }, &bmp, 136, 236);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: false }, &bmp, 166, 237);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: true, vertical_flip: false }, &bmp, 196, 238);
	screen.blit(TransparentFlipped { transparent_color, horizontal_flip: false, vertical_flip: true }, &bmp, 226, 240);

	let path = reference_file(Path::new("transparent_flipped_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn transparent_flipped_tinted_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let transparent_color = to_rgb32([0, 0, 0]);
	let tint_color = to_argb32([127, 155, 242, 21]);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, -3, 46);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, -4, 76);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, -8, 106);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: true }, &bmp, -12, 136);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, -13, 166);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, -14, 196);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, -16, 226);

	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, 46, -3);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, 76, -4);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, 106, -8);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: true }, &bmp, 136, -12);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, 166, -13);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, 196, -14);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, 226, -16);

	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, 307, 46);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, 308, 76);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, 312, 106);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: true }, &bmp, 316, 136);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, 317, 166);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, 318, 196);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, 320, 226);

	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, 46, 227);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, 76, 228);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, 106, 232);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: true }, &bmp, 136, 236);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: false }, &bmp, 166, 237);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: true, vertical_flip: false }, &bmp, 196, 238);
	screen.blit(TransparentFlippedTinted { transparent_color, tint_color, horizontal_flip: false, vertical_flip: true }, &bmp, 226, 240);

	let path = reference_file(Path::new("transparent_flipped_tinted_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn blended_transparent_flipped_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup_for_blending();

	let bmp = generate_solid_bitmap_with_varied_alpha(16, 16);

	let transparent_color = to_argb32([255, 0, 0, 0]);
	let blend = BlendFunction::Blend;

	let x = 40;
	let y = 20;
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: false, blend }, &bmp, x + 16, y + 48);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: false, blend }, &bmp, x + 80, y + 48);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: true, blend }, &bmp, x + 144, y + 48);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: true, blend }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: false, blend }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: false, blend }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: true, blend }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: true, blend }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: false, blend }, &bmp, -3, 46);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: false, blend }, &bmp, -4, 76);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: true, blend }, &bmp, -8, 106);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: true, blend }, &bmp, -12, 136);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: true, blend }, &bmp, -13, 166);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: false, blend }, &bmp, -14, 196);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: true, blend }, &bmp, -16, 226);

	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: false, blend }, &bmp, 46, -3);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: false, blend }, &bmp, 76, -4);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: true, blend }, &bmp, 106, -8);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: true, blend }, &bmp, 136, -12);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: false, blend }, &bmp, 166, -13);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: false, blend }, &bmp, 196, -14);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: true, blend }, &bmp, 226, -16);

	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: false, blend }, &bmp, 307, 46);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: false, blend }, &bmp, 308, 76);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: true, blend }, &bmp, 312, 106);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: true, blend }, &bmp, 316, 136);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: false, blend }, &bmp, 317, 166);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: false, blend }, &bmp, 318, 196);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: true, blend }, &bmp, 320, 226);

	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: false, blend }, &bmp, 46, 227);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: false, blend }, &bmp, 76, 228);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: true, blend }, &bmp, 106, 232);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: true, blend }, &bmp, 136, 236);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: false, blend }, &bmp, 166, 237);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: true, vertical_flip: false, blend }, &bmp, 196, 238);
	screen.blit(TransparentFlippedBlended { transparent_color, horizontal_flip: false, vertical_flip: true, blend }, &bmp, 226, 240);

	let path = reference_file(Path::new("blended_transparent_flipped_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn transparent_single_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let transparent_color = to_rgb32([0, 0, 0]);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(TransparentSingle { transparent_color, draw_color: COLOR_BLUE }, &bmp, x + 16, y + 48);
	screen.blit(TransparentSingle { transparent_color, draw_color: COLOR_RED }, &bmp, x + 80, y + 48);
	screen.blit(TransparentSingle { transparent_color, draw_color: COLOR_LIGHT_GRAY }, &bmp, x + 144, y + 48);
	screen.blit(TransparentSingle { transparent_color, draw_color: COLOR_BRIGHT_MAGENTA }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(TransparentSingle { transparent_color, draw_color: COLOR_BLUE }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(TransparentSingle { transparent_color, draw_color: COLOR_RED }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(TransparentSingle { transparent_color, draw_color: COLOR_LIGHT_GRAY }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(TransparentSingle { transparent_color, draw_color: COLOR_BRIGHT_MAGENTA }, &bmp, x + 208, y + 48);
	}

	//////

	let method = TransparentSingle { transparent_color, draw_color: COLOR_CYAN };
	screen.blit(method.clone(), &bmp, -3, 46);
	screen.blit(method.clone(), &bmp, -4, 76);
	screen.blit(method.clone(), &bmp, -8, 106);
	screen.blit(method.clone(), &bmp, -12, 136);
	screen.blit(method.clone(), &bmp, -13, 166);
	screen.blit(method.clone(), &bmp, -14, 196);
	screen.blit(method.clone(), &bmp, -16, 226);

	let method = TransparentSingle { transparent_color, draw_color: COLOR_DARK_GRAY };
	screen.blit(method.clone(), &bmp, 46, -3);
	screen.blit(method.clone(), &bmp, 76, -4);
	screen.blit(method.clone(), &bmp, 106, -8);
	screen.blit(method.clone(), &bmp, 136, -12);
	screen.blit(method.clone(), &bmp, 166, -13);
	screen.blit(method.clone(), &bmp, 196, -14);
	screen.blit(method.clone(), &bmp, 226, -16);

	let method = TransparentSingle { transparent_color, draw_color: COLOR_BRIGHT_WHITE };
	screen.blit(method.clone(), &bmp, 307, 46);
	screen.blit(method.clone(), &bmp, 308, 76);
	screen.blit(method.clone(), &bmp, 312, 106);
	screen.blit(method.clone(), &bmp, 316, 136);
	screen.blit(method.clone(), &bmp, 317, 166);
	screen.blit(method.clone(), &bmp, 318, 196);
	screen.blit(method.clone(), &bmp, 320, 226);

	let method = TransparentSingle { transparent_color, draw_color: COLOR_BRIGHT_GREEN };
	screen.blit(method.clone(), &bmp, 46, 227);
	screen.blit(method.clone(), &bmp, 76, 228);
	screen.blit(method.clone(), &bmp, 106, 232);
	screen.blit(method.clone(), &bmp, 136, 236);
	screen.blit(method.clone(), &bmp, 166, 237);
	screen.blit(method.clone(), &bmp, 196, 238);
	screen.blit(method.clone(), &bmp, 226, 240);

	let path = reference_file(Path::new("transparent_single_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn transparent_flipped_single_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let transparent_color = to_rgb32([0, 0, 0]);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BLUE, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_RED, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_LIGHT_GRAY, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_MAGENTA, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BLUE, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(TransparentFlippedSingle { transparent_color, draw_color: COLOR_RED, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(TransparentFlippedSingle { transparent_color, draw_color: COLOR_LIGHT_GRAY, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_MAGENTA, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_CYAN, horizontal_flip: false, vertical_flip: false }, &bmp, -3, 46);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_CYAN, horizontal_flip: true, vertical_flip: false }, &bmp, -4, 76);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_CYAN, horizontal_flip: false, vertical_flip: true }, &bmp, -8, 106);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_CYAN, horizontal_flip: true, vertical_flip: true }, &bmp, -12, 136);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_CYAN, horizontal_flip: false, vertical_flip: true }, &bmp, -13, 166);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_CYAN, horizontal_flip: true, vertical_flip: false }, &bmp, -14, 196);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_CYAN, horizontal_flip: false, vertical_flip: true }, &bmp, -16, 226);

	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_DARK_GRAY, horizontal_flip: false, vertical_flip: false }, &bmp, 46, -3);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_DARK_GRAY, horizontal_flip: true, vertical_flip: false }, &bmp, 76, -4);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_DARK_GRAY, horizontal_flip: false, vertical_flip: true }, &bmp, 106, -8);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_DARK_GRAY, horizontal_flip: true, vertical_flip: true }, &bmp, 136, -12);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_DARK_GRAY, horizontal_flip: false, vertical_flip: false }, &bmp, 166, -13);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_DARK_GRAY, horizontal_flip: true, vertical_flip: false }, &bmp, 196, -14);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_DARK_GRAY, horizontal_flip: false, vertical_flip: true }, &bmp, 226, -16);

	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_WHITE, horizontal_flip: false, vertical_flip: false }, &bmp, 307, 46);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_WHITE, horizontal_flip: true, vertical_flip: false }, &bmp, 308, 76);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_WHITE, horizontal_flip: false, vertical_flip: true }, &bmp, 312, 106);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_WHITE, horizontal_flip: true, vertical_flip: true }, &bmp, 316, 136);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_WHITE, horizontal_flip: false, vertical_flip: false }, &bmp, 317, 166);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_WHITE, horizontal_flip: true, vertical_flip: false }, &bmp, 318, 196);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_WHITE, horizontal_flip: false, vertical_flip: true }, &bmp, 320, 226);

	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_GREEN, horizontal_flip: false, vertical_flip: false }, &bmp, 46, 227);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_GREEN, horizontal_flip: true, vertical_flip: false }, &bmp, 76, 228);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_GREEN, horizontal_flip: false, vertical_flip: true }, &bmp, 106, 232);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_GREEN, horizontal_flip: true, vertical_flip: true }, &bmp, 136, 236);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_GREEN, horizontal_flip: false, vertical_flip: false }, &bmp, 166, 237);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_GREEN, horizontal_flip: true, vertical_flip: false }, &bmp, 196, 238);
	screen.blit(TransparentFlippedSingle { transparent_color, draw_color: COLOR_BRIGHT_GREEN, horizontal_flip: false, vertical_flip: true }, &bmp, 226, 240);

	let path = reference_file(Path::new("transparent_flipped_single_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn rotozoom_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, x + 16, y + 48);
	screen.blit(RotoZoom { angle: 0.3, scale_x: 1.5, scale_y: 1.0 }, &bmp, x + 80, y + 48);
	screen.blit(RotoZoom { angle: 0.6, scale_x: 1.0, scale_y: 1.5 }, &bmp, x + 144, y + 48);
	screen.blit(RotoZoom { angle: 2.0, scale_x: 0.7, scale_y: 0.7 }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(RotoZoom { angle: 0.3, scale_x: 1.5, scale_y: 1.0 }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(RotoZoom { angle: 0.6, scale_x: 1.0, scale_y: 1.5 }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(RotoZoom { angle: 2.0, scale_x: 0.7, scale_y: 0.7 }, &bmp, x + 208, y + 48);
	}

	//////

	let method = RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 };

	screen.blit(method.clone(), &bmp, -3, 46);
	screen.blit(method.clone(), &bmp, -4, 76);
	screen.blit(method.clone(), &bmp, -8, 106);
	screen.blit(method.clone(), &bmp, -12, 136);
	screen.blit(method.clone(), &bmp, -13, 166);
	screen.blit(method.clone(), &bmp, -14, 196);
	screen.blit(method.clone(), &bmp, -16, 226);

	screen.blit(method.clone(), &bmp, 46, -3);
	screen.blit(method.clone(), &bmp, 76, -4);
	screen.blit(method.clone(), &bmp, 106, -8);
	screen.blit(method.clone(), &bmp, 136, -12);
	screen.blit(method.clone(), &bmp, 166, -13);
	screen.blit(method.clone(), &bmp, 196, -14);
	screen.blit(method.clone(), &bmp, 226, -16);

	screen.blit(method.clone(), &bmp, 307, 46);
	screen.blit(method.clone(), &bmp, 308, 76);
	screen.blit(method.clone(), &bmp, 312, 106);
	screen.blit(method.clone(), &bmp, 316, 136);
	screen.blit(method.clone(), &bmp, 317, 166);
	screen.blit(method.clone(), &bmp, 318, 196);
	screen.blit(method.clone(), &bmp, 320, 226);

	screen.blit(method.clone(), &bmp, 46, 227);
	screen.blit(method.clone(), &bmp, 76, 228);
	screen.blit(method.clone(), &bmp, 106, 232);
	screen.blit(method.clone(), &bmp, 136, 236);
	screen.blit(method.clone(), &bmp, 166, 237);
	screen.blit(method.clone(), &bmp, 196, 238);
	screen.blit(method.clone(), &bmp, 226, 240);

	let path = reference_file(Path::new("rotozoom_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn rotozoom_tinted_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let bmp = generate_bitmap(16, 16);

	let tint_color = to_argb32([127, 155, 242, 21]);

	let x = 40;
	let y = 20;
	screen.blit(RotoZoomTinted { tint_color, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, x + 16, y + 48);
	screen.blit(RotoZoomTinted { tint_color, angle: 0.3, scale_x: 1.5, scale_y: 1.0 }, &bmp, x + 80, y + 48);
	screen.blit(RotoZoomTinted { tint_color, angle: 0.6, scale_x: 1.0, scale_y: 1.5 }, &bmp, x + 144, y + 48);
	screen.blit(RotoZoomTinted { tint_color, angle: 2.0, scale_x: 0.7, scale_y: 0.7 }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(RotoZoomTinted { tint_color, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(RotoZoomTinted { tint_color, angle: 0.3, scale_x: 1.5, scale_y: 1.0 }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(RotoZoomTinted { tint_color, angle: 0.6, scale_x: 1.0, scale_y: 1.5 }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(RotoZoomTinted { tint_color, angle: 2.0, scale_x: 0.7, scale_y: 0.7 }, &bmp, x + 208, y + 48);
	}

	//////

	let method = RotoZoomTinted { tint_color, angle: 1.3, scale_x: 1.0, scale_y: 1.0 };

	screen.blit(method.clone(), &bmp, -3, 46);
	screen.blit(method.clone(), &bmp, -4, 76);
	screen.blit(method.clone(), &bmp, -8, 106);
	screen.blit(method.clone(), &bmp, -12, 136);
	screen.blit(method.clone(), &bmp, -13, 166);
	screen.blit(method.clone(), &bmp, -14, 196);
	screen.blit(method.clone(), &bmp, -16, 226);

	screen.blit(method.clone(), &bmp, 46, -3);
	screen.blit(method.clone(), &bmp, 76, -4);
	screen.blit(method.clone(), &bmp, 106, -8);
	screen.blit(method.clone(), &bmp, 136, -12);
	screen.blit(method.clone(), &bmp, 166, -13);
	screen.blit(method.clone(), &bmp, 196, -14);
	screen.blit(method.clone(), &bmp, 226, -16);

	screen.blit(method.clone(), &bmp, 307, 46);
	screen.blit(method.clone(), &bmp, 308, 76);
	screen.blit(method.clone(), &bmp, 312, 106);
	screen.blit(method.clone(), &bmp, 316, 136);
	screen.blit(method.clone(), &bmp, 317, 166);
	screen.blit(method.clone(), &bmp, 318, 196);
	screen.blit(method.clone(), &bmp, 320, 226);

	screen.blit(method.clone(), &bmp, 46, 227);
	screen.blit(method.clone(), &bmp, 76, 228);
	screen.blit(method.clone(), &bmp, 106, 232);
	screen.blit(method.clone(), &bmp, 136, 236);
	screen.blit(method.clone(), &bmp, 166, 237);
	screen.blit(method.clone(), &bmp, 196, 238);
	screen.blit(method.clone(), &bmp, 226, 240);

	let path = reference_file(Path::new("rotozoom_tinted_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_rotozoom_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup_for_blending();

	let bmp = generate_bitmap_with_varied_alpha(16, 16);

	let blend = BlendFunction::Blend;

	let x = 40;
	let y = 20;
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend }, &bmp, x + 16, y + 48);
	screen.blit(RotoZoomBlended { angle: 0.3, scale_x: 1.5, scale_y: 1.0, blend }, &bmp, x + 80, y + 48);
	screen.blit(RotoZoomBlended { angle: 0.6, scale_x: 1.0, scale_y: 1.5, blend }, &bmp, x + 144, y + 48);
	screen.blit(RotoZoomBlended { angle: 2.0, scale_x: 0.7, scale_y: 0.7, blend }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(RotoZoomBlended { angle: 0.3, scale_x: 1.5, scale_y: 1.0, blend }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(RotoZoomBlended { angle: 0.6, scale_x: 1.0, scale_y: 1.5, blend }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(RotoZoomBlended { angle: 2.0, scale_x: 0.7, scale_y: 0.7, blend }, &bmp, x + 208, y + 48);
	}

	//////

	let method = RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend };

	screen.blit(method.clone(), &bmp, -3, 46);
	screen.blit(method.clone(), &bmp, -4, 76);
	screen.blit(method.clone(), &bmp, -8, 106);
	screen.blit(method.clone(), &bmp, -12, 136);
	screen.blit(method.clone(), &bmp, -13, 166);
	screen.blit(method.clone(), &bmp, -14, 196);
	screen.blit(method.clone(), &bmp, -16, 226);

	screen.blit(method.clone(), &bmp, 46, -3);
	screen.blit(method.clone(), &bmp, 76, -4);
	screen.blit(method.clone(), &bmp, 106, -8);
	screen.blit(method.clone(), &bmp, 136, -12);
	screen.blit(method.clone(), &bmp, 166, -13);
	screen.blit(method.clone(), &bmp, 196, -14);
	screen.blit(method.clone(), &bmp, 226, -16);

	screen.blit(method.clone(), &bmp, 307, 46);
	screen.blit(method.clone(), &bmp, 308, 76);
	screen.blit(method.clone(), &bmp, 312, 106);
	screen.blit(method.clone(), &bmp, 316, 136);
	screen.blit(method.clone(), &bmp, 317, 166);
	screen.blit(method.clone(), &bmp, 318, 196);
	screen.blit(method.clone(), &bmp, 320, 226);

	screen.blit(method.clone(), &bmp, 46, 227);
	screen.blit(method.clone(), &bmp, 76, 228);
	screen.blit(method.clone(), &bmp, 106, 232);
	screen.blit(method.clone(), &bmp, 136, 236);
	screen.blit(method.clone(), &bmp, 166, 237);
	screen.blit(method.clone(), &bmp, 196, 238);
	screen.blit(method.clone(), &bmp, 226, 240);

	let path = reference_file(Path::new("blended_rotozoom_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn rotozoom_transparent_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let transparent_color = to_rgb32([0, 0, 0]);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(RotoZoomTransparent { transparent_color, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, x + 16, y + 48);
	screen.blit(RotoZoomTransparent { transparent_color, angle: 0.3, scale_x: 1.5, scale_y: 1.0 }, &bmp, x + 80, y + 48);
	screen.blit(RotoZoomTransparent { transparent_color, angle: 0.6, scale_x: 1.0, scale_y: 1.5 }, &bmp, x + 144, y + 48);
	screen.blit(RotoZoomTransparent { transparent_color, angle: 2.0, scale_x: 0.7, scale_y: 0.7 }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(RotoZoomTransparent { transparent_color, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(RotoZoomTransparent { transparent_color, angle: 0.3, scale_x: 1.5, scale_y: 1.0 }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(RotoZoomTransparent { transparent_color, angle: 0.6, scale_x: 1.0, scale_y: 1.5 }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(RotoZoomTransparent { transparent_color, angle: 2.0, scale_x: 0.7, scale_y: 0.7 }, &bmp, x + 208, y + 48);
	}

	//////

	let method = RotoZoomTransparent { transparent_color, angle: 1.3, scale_x: 1.0, scale_y: 1.0 };

	screen.blit(method.clone(), &bmp, -3, 46);
	screen.blit(method.clone(), &bmp, -4, 76);
	screen.blit(method.clone(), &bmp, -8, 106);
	screen.blit(method.clone(), &bmp, -12, 136);
	screen.blit(method.clone(), &bmp, -13, 166);
	screen.blit(method.clone(), &bmp, -14, 196);
	screen.blit(method.clone(), &bmp, -16, 226);

	screen.blit(method.clone(), &bmp, 46, -3);
	screen.blit(method.clone(), &bmp, 76, -4);
	screen.blit(method.clone(), &bmp, 106, -8);
	screen.blit(method.clone(), &bmp, 136, -12);
	screen.blit(method.clone(), &bmp, 166, -13);
	screen.blit(method.clone(), &bmp, 196, -14);
	screen.blit(method.clone(), &bmp, 226, -16);

	screen.blit(method.clone(), &bmp, 307, 46);
	screen.blit(method.clone(), &bmp, 308, 76);
	screen.blit(method.clone(), &bmp, 312, 106);
	screen.blit(method.clone(), &bmp, 316, 136);
	screen.blit(method.clone(), &bmp, 317, 166);
	screen.blit(method.clone(), &bmp, 318, 196);
	screen.blit(method.clone(), &bmp, 320, 226);

	screen.blit(method.clone(), &bmp, 46, 227);
	screen.blit(method.clone(), &bmp, 76, 228);
	screen.blit(method.clone(), &bmp, 106, 232);
	screen.blit(method.clone(), &bmp, 136, 236);
	screen.blit(method.clone(), &bmp, 166, 237);
	screen.blit(method.clone(), &bmp, 196, 238);
	screen.blit(method.clone(), &bmp, 226, 240);

	let path = reference_file(Path::new("rotozoom_transparent_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn rotozoom_transparent_tinted_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let transparent_color = to_rgb32([0, 0, 0]);
	let tint_color = to_argb32([127, 155, 242, 21]);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(RotoZoomTransparentTinted { transparent_color, tint_color, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, x + 16, y + 48);
	screen.blit(RotoZoomTransparentTinted { transparent_color, tint_color, angle: 0.3, scale_x: 1.5, scale_y: 1.0 }, &bmp, x + 80, y + 48);
	screen.blit(RotoZoomTransparentTinted { transparent_color, tint_color, angle: 0.6, scale_x: 1.0, scale_y: 1.5 }, &bmp, x + 144, y + 48);
	screen.blit(RotoZoomTransparentTinted { transparent_color, tint_color, angle: 2.0, scale_x: 0.7, scale_y: 0.7 }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(RotoZoomTransparentTinted { transparent_color, tint_color, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(RotoZoomTransparentTinted { transparent_color, tint_color, angle: 0.3, scale_x: 1.5, scale_y: 1.0 }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(RotoZoomTransparentTinted { transparent_color, tint_color, angle: 0.6, scale_x: 1.0, scale_y: 1.5 }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(RotoZoomTransparentTinted { transparent_color, tint_color, angle: 2.0, scale_x: 0.7, scale_y: 0.7 }, &bmp, x + 208, y + 48);
	}

	//////

	let method = RotoZoomTransparentTinted { transparent_color, tint_color, angle: 1.3, scale_x: 1.0, scale_y: 1.0 };

	screen.blit(method.clone(), &bmp, -3, 46);
	screen.blit(method.clone(), &bmp, -4, 76);
	screen.blit(method.clone(), &bmp, -8, 106);
	screen.blit(method.clone(), &bmp, -12, 136);
	screen.blit(method.clone(), &bmp, -13, 166);
	screen.blit(method.clone(), &bmp, -14, 196);
	screen.blit(method.clone(), &bmp, -16, 226);

	screen.blit(method.clone(), &bmp, 46, -3);
	screen.blit(method.clone(), &bmp, 76, -4);
	screen.blit(method.clone(), &bmp, 106, -8);
	screen.blit(method.clone(), &bmp, 136, -12);
	screen.blit(method.clone(), &bmp, 166, -13);
	screen.blit(method.clone(), &bmp, 196, -14);
	screen.blit(method.clone(), &bmp, 226, -16);

	screen.blit(method.clone(), &bmp, 307, 46);
	screen.blit(method.clone(), &bmp, 308, 76);
	screen.blit(method.clone(), &bmp, 312, 106);
	screen.blit(method.clone(), &bmp, 316, 136);
	screen.blit(method.clone(), &bmp, 317, 166);
	screen.blit(method.clone(), &bmp, 318, 196);
	screen.blit(method.clone(), &bmp, 320, 226);

	screen.blit(method.clone(), &bmp, 46, 227);
	screen.blit(method.clone(), &bmp, 76, 228);
	screen.blit(method.clone(), &bmp, 106, 232);
	screen.blit(method.clone(), &bmp, 136, 236);
	screen.blit(method.clone(), &bmp, 166, 237);
	screen.blit(method.clone(), &bmp, 196, 238);
	screen.blit(method.clone(), &bmp, 226, 240);

	let path = reference_file(Path::new("rotozoom_transparent_tinted_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn blended_rotozoom_transparent_blits() {
	use RgbaBlitMethod::*;

	let mut screen = setup_for_blending();

	let bmp = generate_solid_bitmap_with_varied_alpha(16, 16);

	let transparent_color = to_argb32([255, 0, 0, 0]);
	let blend = BlendFunction::Blend;

	let x = 40;
	let y = 20;
	screen.blit(RotoZoomTransparentBlended { transparent_color, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend }, &bmp, x + 16, y + 48);
	screen.blit(RotoZoomTransparentBlended { transparent_color, angle: 0.3, scale_x: 1.5, scale_y: 1.0, blend }, &bmp, x + 80, y + 48);
	screen.blit(RotoZoomTransparentBlended { transparent_color, angle: 0.6, scale_x: 1.0, scale_y: 1.5, blend }, &bmp, x + 144, y + 48);
	screen.blit(RotoZoomTransparentBlended { transparent_color, angle: 2.0, scale_x: 0.7, scale_y: 0.7, blend }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(RotoZoomTransparentBlended { transparent_color, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(RotoZoomTransparentBlended { transparent_color, angle: 0.3, scale_x: 1.5, scale_y: 1.0, blend }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(RotoZoomTransparentBlended { transparent_color, angle: 0.6, scale_x: 1.0, scale_y: 1.5, blend }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(RotoZoomTransparentBlended { transparent_color, angle: 2.0, scale_x: 0.7, scale_y: 0.7, blend }, &bmp, x + 208, y + 48);
	}

	//////

	let method = RotoZoomTransparentBlended { transparent_color, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend };

	screen.blit(method.clone(), &bmp, -3, 46);
	screen.blit(method.clone(), &bmp, -4, 76);
	screen.blit(method.clone(), &bmp, -8, 106);
	screen.blit(method.clone(), &bmp, -12, 136);
	screen.blit(method.clone(), &bmp, -13, 166);
	screen.blit(method.clone(), &bmp, -14, 196);
	screen.blit(method.clone(), &bmp, -16, 226);

	screen.blit(method.clone(), &bmp, 46, -3);
	screen.blit(method.clone(), &bmp, 76, -4);
	screen.blit(method.clone(), &bmp, 106, -8);
	screen.blit(method.clone(), &bmp, 136, -12);
	screen.blit(method.clone(), &bmp, 166, -13);
	screen.blit(method.clone(), &bmp, 196, -14);
	screen.blit(method.clone(), &bmp, 226, -16);

	screen.blit(method.clone(), &bmp, 307, 46);
	screen.blit(method.clone(), &bmp, 308, 76);
	screen.blit(method.clone(), &bmp, 312, 106);
	screen.blit(method.clone(), &bmp, 316, 136);
	screen.blit(method.clone(), &bmp, 317, 166);
	screen.blit(method.clone(), &bmp, 318, 196);
	screen.blit(method.clone(), &bmp, 320, 226);

	screen.blit(method.clone(), &bmp, 46, 227);
	screen.blit(method.clone(), &bmp, 76, 228);
	screen.blit(method.clone(), &bmp, 106, 232);
	screen.blit(method.clone(), &bmp, 136, 236);
	screen.blit(method.clone(), &bmp, 166, 237);
	screen.blit(method.clone(), &bmp, 196, 238);
	screen.blit(method.clone(), &bmp, 226, 240);

	let path = reference_file(Path::new("blended_rotozoom_transparent_blits.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blend_function_blend() {
	let mut screen = setup_for_blending_half_solid_half_semi_transparent();

	let bmp_solid = generate_bitmap(32, 32);
	let bmp_solid_with_varied_alpha = generate_solid_bitmap_with_varied_alpha(32, 32);
	let bmp_with_varied_alpha = generate_bitmap_with_varied_alpha(32, 32);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::Blend);

	screen.blit(method.clone(), &bmp_solid, 10, 10);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 10);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 10);

	//////

	screen.blit(method.clone(), &bmp_solid, 10, 130);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 130);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 130);

	let path = reference_file(Path::new("blend_function_blend.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blend_function_tinted_blend() {
	let mut screen = setup_for_blending_half_solid_half_semi_transparent();

	let bmp_solid = generate_bitmap(32, 32);
	let bmp_solid_with_varied_alpha = generate_solid_bitmap_with_varied_alpha(32, 32);
	let bmp_with_varied_alpha = generate_bitmap_with_varied_alpha(32, 32);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::TintedBlend(to_argb32([255, 155, 242, 21])));
	screen.blit(method.clone(), &bmp_solid, 10, 5);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 5);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 5);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::TintedBlend(to_argb32([127, 155, 242, 21])));
	screen.blit(method.clone(), &bmp_solid, 10, 40);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 40);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 40);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::TintedBlend(to_argb32([0, 155, 242, 21])));
	screen.blit(method.clone(), &bmp_solid, 10, 75);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 75);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 75);

	//////

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::TintedBlend(to_argb32([255, 155, 242, 21])));
	screen.blit(method.clone(), &bmp_solid, 10, 125);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 125);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 125);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::TintedBlend(to_argb32([127, 155, 242, 21])));
	screen.blit(method.clone(), &bmp_solid, 10, 160);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 160);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 160);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::TintedBlend(to_argb32([0, 155, 242, 21])));
	screen.blit(method.clone(), &bmp_solid, 10, 195);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 195);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 195);

	let path = reference_file(Path::new("blend_function_tinted_blend.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blend_function_blend_source_with_alpha() {
	let mut screen = setup_for_blending_half_solid_half_semi_transparent();

	let bmp_solid = generate_bitmap(32, 32);
	let bmp_solid_with_varied_alpha = generate_solid_bitmap_with_varied_alpha(32, 32);
	let bmp_with_varied_alpha = generate_bitmap_with_varied_alpha(32, 32);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::BlendSourceWithAlpha(255));
	screen.blit(method.clone(), &bmp_solid, 10, 5);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 5);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 5);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::BlendSourceWithAlpha(127));
	screen.blit(method.clone(), &bmp_solid, 10, 40);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 40);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 40);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::BlendSourceWithAlpha(0));
	screen.blit(method.clone(), &bmp_solid, 10, 75);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 75);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 75);

	//////

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::BlendSourceWithAlpha(255));
	screen.blit(method.clone(), &bmp_solid, 10, 125);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 125);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 125);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::BlendSourceWithAlpha(127));
	screen.blit(method.clone(), &bmp_solid, 10, 160);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 160);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 160);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::BlendSourceWithAlpha(0));
	screen.blit(method.clone(), &bmp_solid, 10, 195);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 195);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 195);

	let path = reference_file(Path::new("blend_function_blend_source_with_alpha.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blend_function_multiplied_blend() {
	let mut screen = setup_for_blending_half_solid_half_semi_transparent();

	let bmp_solid = generate_bitmap(32, 32);
	let bmp_solid_with_varied_alpha = generate_solid_bitmap_with_varied_alpha(32, 32);
	let bmp_with_varied_alpha = generate_bitmap_with_varied_alpha(32, 32);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::MultipliedBlend(to_argb32([255, 242, 29, 81])));
	screen.blit(method.clone(), &bmp_solid, 10, 5);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 5);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 5);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::MultipliedBlend(to_argb32([127, 242, 29, 81])));
	screen.blit(method.clone(), &bmp_solid, 10, 40);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 40);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 40);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::MultipliedBlend(to_argb32([0, 242, 29, 81])));
	screen.blit(method.clone(), &bmp_solid, 10, 75);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 75);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 75);

	//////

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::MultipliedBlend(to_argb32([255, 242, 29, 81])));
	screen.blit(method.clone(), &bmp_solid, 10, 125);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 125);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 125);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::MultipliedBlend(to_argb32([127, 242, 29, 81])));
	screen.blit(method.clone(), &bmp_solid, 10, 160);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 160);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 160);

	let method = RgbaBlitMethod::SolidBlended(BlendFunction::MultipliedBlend(to_argb32([0, 242, 29, 81])));
	screen.blit(method.clone(), &bmp_solid, 10, 195);
	screen.blit(method.clone(), &bmp_solid_with_varied_alpha, 100, 195);
	screen.blit(method.clone(), &bmp_with_varied_alpha, 200, 195);

	let path = reference_file(Path::new("blend_function_multiplied_blend.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn triangle_2d() {
	use RgbaTriangle2d::*;

	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let color = COLOR_BLUE;
	let v1 = Vector2::new(32.0, 36.0);
	let v2 = Vector2::new(32.0, 63.0);
	let v3 = Vector2::new(73.0, 36.0);
	screen.triangle_2d(&Solid { position: [v1, v2, v3], color });
	screen.triangle_2d(&Solid {
		position: [
			v1 - Vector2::new(50.0, 0.0), //
			v2 - Vector2::new(50.0, 0.0),
			v3 - Vector2::new(50.0, 0.0),
		],
		color,
	});
	screen.triangle_2d(&Solid {
		position: [
			v1 - Vector2::new(0.0, 50.0), //
			v2 - Vector2::new(0.0, 50.0),
			v3 - Vector2::new(0.0, 50.0),
		],
		color,
	});

	let color = COLOR_GREEN;
	let v1 = Vector2::new(123.0, 60.0);
	let v2 = Vector2::new(162.0, 60.0);
	let v3 = Vector2::new(144.0, 32.0);
	screen.triangle_2d(&Solid { position: [v1, v2, v3], color });
	screen.triangle_2d(&Solid {
		position: [
			v1 - Vector2::new(0.0, 45.0), //
			v2 - Vector2::new(0.0, 45.0),
			v3 - Vector2::new(0.0, 45.0),
		],
		color,
	});

	let color = COLOR_CYAN;
	let v1 = Vector2::new(265.0, 74.0);
	let v2 = Vector2::new(265.0, 37.0);
	let v3 = Vector2::new(231.0, 37.0);
	screen.triangle_2d(&Solid { position: [v1, v2, v3], color });
	screen.triangle_2d(&Solid {
		position: [
			v1 - Vector2::new(-70.0, 0.0), //
			v2 - Vector2::new(-70.0, 0.0),
			v3 - Vector2::new(-70.0, 0.0),
		],
		color,
	});
	screen.triangle_2d(&Solid {
		position: [
			v1 - Vector2::new(0.0, 55.0), //
			v2 - Vector2::new(0.0, 55.0),
			v3 - Vector2::new(0.0, 55.0),
		],
		color,
	});

	let color = COLOR_RED;
	let v1 = Vector2::new(33.0, 108.0);
	let v2 = Vector2::new(33.0, 137.0);
	let v3 = Vector2::new(59.0, 122.0);
	screen.triangle_2d(&Solid { position: [v1, v2, v3], color });
	screen.triangle_2d(&Solid {
		position: [
			v1 - Vector2::new(45.0, 0.0), //
			v2 - Vector2::new(45.0, 0.0),
			v3 - Vector2::new(45.0, 0.0),
		],
		color,
	});

	let color = COLOR_MAGENTA;
	let v1 = Vector2::new(161.0, 132.0);
	let v2 = Vector2::new(145.0, 92.0);
	let v3 = Vector2::new(120.0, 115.0);
	screen.triangle_2d(&Solid { position: [v1, v2, v3], color });

	let color = COLOR_BROWN;
	let v1 = Vector2::new(237.0, 120.0);
	let v2 = Vector2::new(267.0, 136.0);
	let v3 = Vector2::new(267.0, 105.0);
	screen.triangle_2d(&Solid { position: [v1, v2, v3], color });
	screen.triangle_2d(&Solid {
		position: [
			v1 - Vector2::new(-70.0, 0.0), //
			v2 - Vector2::new(-70.0, 0.0),
			v3 - Vector2::new(-70.0, 0.0),
		],
		color,
	});

	let color = COLOR_LIGHT_GRAY;
	let v1 = Vector2::new(29.0, 194.0);
	let v2 = Vector2::new(62.0, 194.0);
	let v3 = Vector2::new(29.0, 163.0);
	screen.triangle_2d(&Solid { position: [v1, v2, v3], color });
	screen.triangle_2d(&Solid {
		position: [
			v1 - Vector2::new(45.0, 0.0), //
			v2 - Vector2::new(45.0, 0.0),
			v3 - Vector2::new(45.0, 0.0),
		],
		color,
	});
	screen.triangle_2d(&Solid {
		position: [
			v1 - Vector2::new(0.0, -55.0), //
			v2 - Vector2::new(0.0, -55.0),
			v3 - Vector2::new(0.0, -55.0),
		],
		color,
	});

	let color = COLOR_DARK_GRAY;
	let v1 = Vector2::new(130.0, 164.0);
	let v2 = Vector2::new(155.0, 190.0);
	let v3 = Vector2::new(177.0, 164.0);
	screen.triangle_2d(&Solid { position: [v1, v2, v3], color });
	screen.triangle_2d(&Solid {
		position: [
			v1 - Vector2::new(0.0, -60.0), //
			v2 - Vector2::new(0.0, -60.0),
			v3 - Vector2::new(0.0, -60.0),
		],
		color,
	});

	let color = COLOR_BRIGHT_BLUE;
	let v1 = Vector2::new(235.0, 193.0);
	let v2 = Vector2::new(269.0, 193.0);
	let v3 = Vector2::new(269.0, 163.0);
	screen.triangle_2d(&Solid { position: [v1, v2, v3], color });
	screen.triangle_2d(&Solid {
		position: [
			v1 - Vector2::new(-70.0, 0.0), //
			v2 - Vector2::new(-70.0, 0.0),
			v3 - Vector2::new(-70.0, 0.0),
		],
		color,
	});
	screen.triangle_2d(&Solid {
		position: [
			v1 - Vector2::new(0.0, -60.0), //
			v2 - Vector2::new(0.0, -60.0),
			v3 - Vector2::new(0.0, -60.0),
		],
		color,
	});

	// totally off screen

	let color = COLOR_BRIGHT_RED;

	screen.triangle_2d(&Solid {
		position: [
			Vector2::new(-32.0, 36.0), //
			Vector2::new(-32.0, 63.0),
			Vector2::new(-73.0, 36.0),
		],
		color,
	});
	screen.triangle_2d(&Solid {
		position: [
			Vector2::new(265.0, -26.0), //
			Vector2::new(265.0, -63.0),
			Vector2::new(231.0, -63.0),
		],
		color,
	});
	screen.triangle_2d(&Solid {
		position: [
			Vector2::new(29.0, 294.0), //
			Vector2::new(62.0, 294.0),
			Vector2::new(29.0, 263.0),
		],
		color,
	});
	screen.triangle_2d(&Solid {
		position: [
			Vector2::new(335.0, 193.0), //
			Vector2::new(369.0, 193.0),
			Vector2::new(369.0, 163.0),
		],
		color,
	});

	// wrong vertex winding (clockwise instead of counter-clockwise)

	let color = COLOR_BRIGHT_RED;

	screen.triangle_2d(&Solid {
		position: [
			Vector2::new(120.0, 115.0), //
			Vector2::new(145.0, 92.0),
			Vector2::new(161.0, 132.0),
		],
		color,
	});

	let path = reference_file(Path::new("triangle_2d.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TriangleType {
	Solid = 0,
	SolidBlended = 1,
	SolidMultiColorBlended = 2,
	SolidTextured = 3,
	SolidTexturedColored = 4,
	SolidTexturedColoredBlended = 5,
	SolidTexturedMultiColored = 6,
	SolidTexturedMultiColoredBlended = 7,
	SolidTexturedTinted = 8,
	SolidTexturedBlended = 9,
}

fn get_quad(
	mode: TriangleType,
	texture: Option<&RgbaBitmap>,
	transform: Matrix3x3,
	top_left: Vector2,
	top_right: Vector2,
	bottom_left: Vector2,
	bottom_right: Vector2,
) -> [RgbaTriangle2d; 2] {
	let top_left = transform * top_left;
	let top_right = transform * top_right;
	let bottom_left = transform * bottom_left;
	let bottom_right = transform * bottom_right;

	let positions_1 = [top_left, bottom_left, bottom_right];
	let positions_2 = [top_left, bottom_right, top_right];
	let texcoords_1 = [Vector2::new(0.0, 0.0), Vector2::new(0.0, 1.0), Vector2::new(1.0, 1.0)];
	let texcoords_2 = [Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0), Vector2::new(1.0, 0.0)];
	let single_color = to_argb32([128, 255, 0, 255]);
	let colors_1 = [to_rgb32([255, 0, 0]), to_rgb32([0, 255, 0]), to_rgb32([0, 0, 255])];
	let colors_2 = [to_rgb32([255, 0, 0]), to_rgb32([0, 0, 255]), to_rgb32([255, 255, 255])];
	let tint_color = to_argb32([128, 192, 47, 160]);

	match mode {
		TriangleType::Solid => [
			RgbaTriangle2d::Solid { position: positions_1, color: to_rgb32([255, 0, 255]) },
			RgbaTriangle2d::Solid { position: positions_2, color: to_rgb32([255, 0, 255]) },
		],
		TriangleType::SolidBlended => [
			RgbaTriangle2d::SolidBlended { position: positions_1, color: single_color, blend: BlendFunction::Blend },
			RgbaTriangle2d::SolidBlended { position: positions_2, color: single_color, blend: BlendFunction::Blend },
		],
		TriangleType::SolidMultiColorBlended => [
			RgbaTriangle2d::SolidMultiColorBlended {
				position: positions_1,
				color: colors_1,
				blend: BlendFunction::BlendSourceWithAlpha(128),
			},
			RgbaTriangle2d::SolidMultiColorBlended {
				position: positions_2,
				color: colors_2,
				blend: BlendFunction::BlendSourceWithAlpha(128),
			},
		],
		TriangleType::SolidTextured => [
			RgbaTriangle2d::SolidTextured { position: positions_1, texcoord: texcoords_1, bitmap: &texture.unwrap() },
			RgbaTriangle2d::SolidTextured { position: positions_2, texcoord: texcoords_2, bitmap: &texture.unwrap() },
		],
		TriangleType::SolidTexturedColored => [
			RgbaTriangle2d::SolidTexturedColored {
				position: positions_1,
				texcoord: texcoords_1,
				color: single_color,
				bitmap: &texture.unwrap(),
			},
			RgbaTriangle2d::SolidTexturedColored {
				position: positions_2,
				texcoord: texcoords_2,
				color: single_color,
				bitmap: &texture.unwrap(),
			},
		],
		TriangleType::SolidTexturedColoredBlended => [
			RgbaTriangle2d::SolidTexturedColoredBlended {
				position: positions_1,
				texcoord: texcoords_1,
				color: single_color,
				bitmap: &texture.unwrap(),
				blend: BlendFunction::BlendSourceWithAlpha(128),
			},
			RgbaTriangle2d::SolidTexturedColoredBlended {
				position: positions_2,
				texcoord: texcoords_2,
				color: single_color,
				bitmap: &texture.unwrap(),
				blend: BlendFunction::BlendSourceWithAlpha(128),
			},
		],
		TriangleType::SolidTexturedMultiColored => [
			RgbaTriangle2d::SolidTexturedMultiColored {
				position: positions_1,
				texcoord: texcoords_1,
				color: colors_1,
				bitmap: &texture.unwrap(),
			},
			RgbaTriangle2d::SolidTexturedMultiColored {
				position: positions_2,
				texcoord: texcoords_2,
				color: colors_2,
				bitmap: &texture.unwrap(),
			},
		],
		TriangleType::SolidTexturedMultiColoredBlended => [
			RgbaTriangle2d::SolidTexturedMultiColoredBlended {
				position: positions_1,
				texcoord: texcoords_1,
				color: colors_1,
				bitmap: &texture.unwrap(),
				blend: BlendFunction::BlendSourceWithAlpha(192),
			},
			RgbaTriangle2d::SolidTexturedMultiColoredBlended {
				position: positions_2,
				texcoord: texcoords_2,
				color: colors_2,
				bitmap: &texture.unwrap(),
				blend: BlendFunction::BlendSourceWithAlpha(192),
			},
		],
		TriangleType::SolidTexturedTinted => [
			RgbaTriangle2d::SolidTexturedTinted {
				position: positions_1,
				texcoord: texcoords_1,
				bitmap: &texture.unwrap(),
				tint: tint_color,
			},
			RgbaTriangle2d::SolidTexturedTinted {
				position: positions_2,
				texcoord: texcoords_2,
				bitmap: &texture.unwrap(),
				tint: tint_color,
			},
		],
		TriangleType::SolidTexturedBlended => [
			RgbaTriangle2d::SolidTexturedBlended {
				position: positions_1,
				texcoord: texcoords_1,
				bitmap: &texture.unwrap(),
				blend: BlendFunction::BlendSourceWithAlpha(128),
			},
			RgbaTriangle2d::SolidTexturedBlended {
				position: positions_2,
				texcoord: texcoords_2,
				bitmap: &texture.unwrap(),
				blend: BlendFunction::BlendSourceWithAlpha(128),
			},
		],
	}
}

#[rustfmt::skip]
#[test]
fn triangle_2d_solid_blended() {
	let mut screen = setup_for_blending();

	let top_left = Vector2::new(0.0, 0.0);
	let top_right = Vector2::new(32.0, 0.0);
	let bottom_left = Vector2::new(0.0, 32.0);
	let bottom_right = Vector2::new(32.0, 32.0);
	
	let rotate = Matrix3x3::new_2d_rotation(RADIANS_45);
	let scale = Matrix3x3::new_2d_scaling(2.0, 2.0);
	
	let mode = TriangleType::SolidBlended;

	let triangles = get_quad(mode, None, Matrix3x3::new_2d_translation(40.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, None, scale * Matrix3x3::new_2d_translation(200.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, None, scale * rotate * Matrix3x3::new_2d_translation(120.0, 120.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let path = reference_file(Path::new("triangle_2d_solid_blended.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn triangle_2d_solid_multicolor_blended() {
	let mut screen = setup_for_blending();

	let top_left = Vector2::new(0.0, 0.0);
	let top_right = Vector2::new(32.0, 0.0);
	let bottom_left = Vector2::new(0.0, 32.0);
	let bottom_right = Vector2::new(32.0, 32.0);

	let rotate = Matrix3x3::new_2d_rotation(RADIANS_45);
	let scale = Matrix3x3::new_2d_scaling(2.0, 2.0);

	let mode = TriangleType::SolidMultiColorBlended;

	let triangles = get_quad(mode, None, Matrix3x3::new_2d_translation(40.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, None, scale * Matrix3x3::new_2d_translation(200.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, None, scale * rotate * Matrix3x3::new_2d_translation(120.0, 120.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let path = reference_file(Path::new("triangle_2d_solid_multicolor_blended.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn triangle_2d_solid_textured() {
	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let texture = generate_bitmap(32, 32);

	let top_left = Vector2::new(0.0, 0.0);
	let top_right = Vector2::new(32.0, 0.0);
	let bottom_left = Vector2::new(0.0, 32.0);
	let bottom_right = Vector2::new(32.0, 32.0);

	let rotate = Matrix3x3::new_2d_rotation(RADIANS_45);
	let scale = Matrix3x3::new_2d_scaling(2.0, 2.0);

	let mode = TriangleType::SolidTextured;

	let triangles = get_quad(mode, Some(&texture), Matrix3x3::new_2d_translation(40.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * Matrix3x3::new_2d_translation(200.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * rotate * Matrix3x3::new_2d_translation(120.0, 120.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let path = reference_file(Path::new("triangle_2d_solid_textured.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn triangle_2d_solid_textured_colored() {
	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let texture = generate_bitmap(32, 32);

	let top_left = Vector2::new(0.0, 0.0);
	let top_right = Vector2::new(32.0, 0.0);
	let bottom_left = Vector2::new(0.0, 32.0);
	let bottom_right = Vector2::new(32.0, 32.0);

	let rotate = Matrix3x3::new_2d_rotation(RADIANS_45);
	let scale = Matrix3x3::new_2d_scaling(2.0, 2.0);

	let mode = TriangleType::SolidTexturedColored;

	let triangles = get_quad(mode, Some(&texture), Matrix3x3::new_2d_translation(40.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * Matrix3x3::new_2d_translation(200.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * rotate * Matrix3x3::new_2d_translation(120.0, 120.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let path = reference_file(Path::new("triangle_2d_solid_textured_colored.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn triangle_2d_solid_textured_colored_blended() {
	let mut screen = setup_for_blending();

	let texture = generate_bitmap(32, 32);

	let top_left = Vector2::new(0.0, 0.0);
	let top_right = Vector2::new(32.0, 0.0);
	let bottom_left = Vector2::new(0.0, 32.0);
	let bottom_right = Vector2::new(32.0, 32.0);

	let rotate = Matrix3x3::new_2d_rotation(RADIANS_45);
	let scale = Matrix3x3::new_2d_scaling(2.0, 2.0);

	let mode = TriangleType::SolidTexturedColoredBlended;

	let triangles = get_quad(mode, Some(&texture), Matrix3x3::new_2d_translation(40.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * Matrix3x3::new_2d_translation(200.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * rotate * Matrix3x3::new_2d_translation(120.0, 120.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let path = reference_file(Path::new("triangle_2d_solid_textured_colored_blended.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn triangle_2d_solid_textured_multicolored() {
	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let texture = generate_bitmap(32, 32);

	let top_left = Vector2::new(0.0, 0.0);
	let top_right = Vector2::new(32.0, 0.0);
	let bottom_left = Vector2::new(0.0, 32.0);
	let bottom_right = Vector2::new(32.0, 32.0);

	let rotate = Matrix3x3::new_2d_rotation(RADIANS_45);
	let scale = Matrix3x3::new_2d_scaling(2.0, 2.0);

	let mode = TriangleType::SolidTexturedMultiColored;

	let triangles = get_quad(mode, Some(&texture), Matrix3x3::new_2d_translation(40.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * Matrix3x3::new_2d_translation(200.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * rotate * Matrix3x3::new_2d_translation(120.0, 120.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let path = reference_file(Path::new("triangle_2d_solid_textured_multicolored.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn triangle_2d_solid_textured_multicolored_blended() {
	let mut screen = setup_for_blending();

	let texture = generate_bitmap(32, 32);

	let top_left = Vector2::new(0.0, 0.0);
	let top_right = Vector2::new(32.0, 0.0);
	let bottom_left = Vector2::new(0.0, 32.0);
	let bottom_right = Vector2::new(32.0, 32.0);

	let rotate = Matrix3x3::new_2d_rotation(RADIANS_45);
	let scale = Matrix3x3::new_2d_scaling(2.0, 2.0);

	let mode = TriangleType::SolidTexturedMultiColoredBlended;

	let triangles = get_quad(mode, Some(&texture), Matrix3x3::new_2d_translation(40.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * Matrix3x3::new_2d_translation(200.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * rotate * Matrix3x3::new_2d_translation(120.0, 120.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let path = reference_file(Path::new("triangle_2d_solid_textured_multicolored_blended.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn triangle_2d_solid_textured_tinted() {
	let mut screen = setup();
	screen.clear(LIGHTER_BACKGROUND);

	let texture = generate_bitmap(32, 32);

	let top_left = Vector2::new(0.0, 0.0);
	let top_right = Vector2::new(32.0, 0.0);
	let bottom_left = Vector2::new(0.0, 32.0);
	let bottom_right = Vector2::new(32.0, 32.0);

	let rotate = Matrix3x3::new_2d_rotation(RADIANS_45);
	let scale = Matrix3x3::new_2d_scaling(2.0, 2.0);

	let mode = TriangleType::SolidTexturedTinted;

	let triangles = get_quad(mode, Some(&texture), Matrix3x3::new_2d_translation(40.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * Matrix3x3::new_2d_translation(200.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * rotate * Matrix3x3::new_2d_translation(120.0, 120.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let path = reference_file(Path::new("triangle_2d_solid_textured_tinted.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}

#[rustfmt::skip]
#[test]
fn triangle_2d_solid_textured_blended() {
	let mut screen = setup_for_blending();

	let texture = generate_bitmap(32, 32);

	let top_left = Vector2::new(0.0, 0.0);
	let top_right = Vector2::new(32.0, 0.0);
	let bottom_left = Vector2::new(0.0, 32.0);
	let bottom_right = Vector2::new(32.0, 32.0);

	let rotate = Matrix3x3::new_2d_rotation(RADIANS_45);
	let scale = Matrix3x3::new_2d_scaling(2.0, 2.0);

	let mode = TriangleType::SolidTexturedBlended;

	let triangles = get_quad(mode, Some(&texture), Matrix3x3::new_2d_translation(40.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * Matrix3x3::new_2d_translation(200.0, 40.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let triangles = get_quad(mode, Some(&texture), scale * rotate * Matrix3x3::new_2d_translation(120.0, 120.0), top_left, top_right, bottom_left, bottom_right);
	screen.triangle_list_2d(&triangles);

	let path = reference_file(Path::new("triangle_2d_solid_textured_blended.png"));
	//screen.to_png_file(path.as_path(), PngFormat::RGBA).unwrap();
	assert!(verify_visual(&screen, &path), "bitmap differs from source image: {:?}", path);
}
