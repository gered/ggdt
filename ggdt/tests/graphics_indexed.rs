use std::ops::Add;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ggdt::prelude::*;

fn setup() -> (IndexedBitmap, Palette) {
	let palette = Palette::new_vga_palette().unwrap();
	let screen = IndexedBitmap::new(SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
	(screen, palette)
}

fn setup_for_blending() -> (IndexedBitmap, Palette, BlendMap) {
	let (texture, palette) = IndexedBitmap::load_file(Path::new("test-assets/texture.lbm")).unwrap();
	let blend_map = BlendMap::load_from_file(Path::new("test-assets/test.blendmap")).unwrap();
	let mut screen = IndexedBitmap::new(SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
	for y in 0..(SCREEN_HEIGHT as f32 / texture.height() as f32).ceil() as i32 {
		for x in 0..(SCREEN_WIDTH as f32 / texture.width() as f32).ceil() as i32 {
			screen.blit(IndexedBlitMethod::Solid, &texture, x * texture.width() as i32, y * texture.height() as i32);
		}
	}
	(screen, palette, blend_map)
}

fn verify_visual(screen: &IndexedBitmap, palette: &Palette, source: &Path) -> bool {
	let (source_bmp, source_pal) = IndexedBitmap::load_file(source).unwrap();
	*screen == source_bmp && *palette == source_pal
}

fn make_ref_path(ref_filename: &str) -> PathBuf {
	let path = String::from("tests/ref/indexed/").add(ref_filename);
	PathBuf::from(&path)
}

#[test]
fn pixel_addressing() {
	let (mut screen, palette) = setup();

	unsafe {
		let mut pixels = screen.pixels_at_mut_ptr(10, 10).unwrap();
		let mut i = 0;
		for _y in 0..16 {
			for _x in 0..16 {
				*pixels = i;
				i = i.wrapping_add(1);
				pixels = pixels.offset(1);
			}
			pixels = pixels.offset((SCREEN_WIDTH - 16) as isize);
		}
	}

	unsafe {
		let mut pixels = screen.pixels_at_mut_ptr(0, 0).unwrap();
		for _ in 0..10 {
			*pixels = 15;
			pixels = pixels.offset((SCREEN_WIDTH + 1) as isize);
		}
	}

	unsafe {
		let mut pixels = screen.pixels_at_mut_ptr(10, 0).unwrap();
		for _ in 0..10 {
			*pixels = 15;
			pixels = pixels.offset(SCREEN_WIDTH as isize);
		}
	}

	unsafe {
		let mut pixels = screen.pixels_at_mut_ptr(0, 10).unwrap();
		for _ in 0..10 {
			*pixels = 15;
			pixels = pixels.offset(1);
		}
	}

	let path = make_ref_path("pixel_addressing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn pixel_drawing() {
	let (mut screen, palette) = setup();

	screen.set_pixel(0, 0, 1);
	screen.set_pixel(319, 0, 2);
	screen.set_pixel(0, 239, 3);
	screen.set_pixel(319, 239, 4);

	unsafe {
		screen.set_pixel_unchecked(10, 0, 1);
		screen.set_pixel_unchecked(309, 0, 2);
		screen.set_pixel_unchecked(10, 239, 3);
		screen.set_pixel_unchecked(309, 239, 4);
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
		screen.set_pixel(5 - i, 100, 15);
		screen.set_pixel(i + 314, 100, 15);
		screen.set_pixel(160, 5 - i, 15);
		screen.set_pixel(160, i + 234, 15);
	}

	let path = make_ref_path("pixel_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_pixel_drawing() {
	let (mut screen, palette, blend_map) = setup_for_blending();

	for i in 0..10 {
		screen.set_blended_pixel(0 + i, 0 + i, 1, &blend_map);
		screen.set_blended_pixel(319 - i, 0 + i, 2, &blend_map);
		screen.set_blended_pixel(0 + i, 239 - i, 3, &blend_map);
		screen.set_blended_pixel(319 - i, 239 - i, 4, &blend_map);
	}

	//////

	for i in 0..10 {
		screen.set_blended_pixel(5 - i, 100, 15, &blend_map);
		screen.set_blended_pixel(i + 314, 100, 15, &blend_map);
		screen.set_blended_pixel(160, 5 - i, 15, &blend_map);
		screen.set_blended_pixel(160, i + 234, 15, &blend_map);
	}

	let path = make_ref_path("blended_pixel_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}


#[test]
fn horiz_line_drawing() {
	let (mut screen, palette) = setup();

	screen.horiz_line(10, 100, 20, 1);
	screen.horiz_line(10, 100, 30, 2);

	//////

	screen.horiz_line(-50, 50, 6, 3);
	screen.horiz_line(300, 340, 130, 5);

	screen.horiz_line(100, 200, -10, 6);
	screen.horiz_line(20, 80, 250, 7);

	let path = make_ref_path("horiz_line_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_horiz_line_drawing() {
	let (mut screen, palette, blend_map) = setup_for_blending();

	screen.blended_horiz_line(10, 100, 20, 1, &blend_map);
	screen.blended_horiz_line(10, 100, 30, 2, &blend_map);

	//////

	screen.blended_horiz_line(-50, 50, 6, 3, &blend_map);
	screen.blended_horiz_line(300, 340, 130, 5, &blend_map);

	screen.blended_horiz_line(100, 200, -10, 6, &blend_map);
	screen.blended_horiz_line(20, 80, 250, 7, &blend_map);

	let path = make_ref_path("blended_horiz_line_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn vert_line_drawing() {
	let (mut screen, palette) = setup();

	screen.vert_line(50, 10, 200, 1);
	screen.vert_line(60, 10, 200, 2);

	//////

	screen.vert_line(20, -32, 32, 3);
	screen.vert_line(270, 245, 165, 5);

	screen.vert_line(-17, 10, 20, 6);
	screen.vert_line(400, 100, 300, 7);

	let path = make_ref_path("vert_line_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_vert_line_drawing() {
	let (mut screen, palette, blend_map) = setup_for_blending();

	screen.blended_vert_line(50, 10, 200, 1, &blend_map);
	screen.blended_vert_line(60, 10, 200, 2, &blend_map);

	//////

	screen.blended_vert_line(20, -32, 32, 3, &blend_map);
	screen.blended_vert_line(270, 245, 165, 5, &blend_map);

	screen.blended_vert_line(-17, 10, 20, 6, &blend_map);
	screen.blended_vert_line(400, 100, 300, 7, &blend_map);

	let path = make_ref_path("blended_vert_line_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn line_drawing() {
	let (mut screen, palette) = setup();

	screen.line(10, 10, 20, 20, 1);
	screen.line(10, 100, 20, 150, 2);
	screen.line(60, 150, 50, 100, 3);

	//////

	screen.line(50, 10, 100, 10, 5);
	screen.line(100, 50, 20, 50, 6);
	screen.line(290, 10, 290, 100, 7);
	screen.line(310, 100, 310, 10, 8);

	//////

	screen.line(50, 200, -50, 200, 5);
	screen.line(300, 210, 340, 210, 6);
	screen.line(120, -30, 120, 30, 7);
	screen.line(130, 200, 130, 270, 8);

	screen.line(250, 260, 190, 200, 9);
	screen.line(180, 30, 240, -30, 10);
	screen.line(-20, 140, 20, 180, 11);
	screen.line(300, 130, 340, 170, 12);

	screen.line(10, -30, 100, -30, 1);
	screen.line(70, 250, 170, 250, 2);
	screen.line(-100, 120, -100, 239, 3);
	screen.line(320, 99, 320, 199, 5);

	let path = make_ref_path("line_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_line_drawing() {
	let (mut screen, palette, blend_map) = setup_for_blending();

	screen.blended_line(10, 10, 20, 20, 1, &blend_map);
	screen.blended_line(10, 100, 20, 150, 2, &blend_map);
	screen.blended_line(60, 150, 50, 100, 3, &blend_map);

	//////

	screen.blended_line(50, 10, 100, 10, 5, &blend_map);
	screen.blended_line(100, 50, 20, 50, 6, &blend_map);
	screen.blended_line(290, 10, 290, 100, 7, &blend_map);
	screen.blended_line(310, 100, 310, 10, 8, &blend_map);

	//////

	screen.blended_line(50, 200, -50, 200, 5, &blend_map);
	screen.blended_line(300, 210, 340, 210, 6, &blend_map);
	screen.blended_line(120, -30, 120, 30, 7, &blend_map);
	screen.blended_line(130, 200, 130, 270, 8, &blend_map);

	screen.blended_line(250, 260, 190, 200, 9, &blend_map);
	screen.blended_line(180, 30, 240, -30, 10, &blend_map);
	screen.blended_line(-20, 140, 20, 180, 11, &blend_map);
	screen.blended_line(300, 130, 340, 170, 12, &blend_map);

	screen.blended_line(10, -30, 100, -30, 1, &blend_map);
	screen.blended_line(70, 250, 170, 250, 2, &blend_map);
	screen.blended_line(-100, 120, -100, 239, 3, &blend_map);
	screen.blended_line(320, 99, 320, 199, 5, &blend_map);

	let path = make_ref_path("blended_line_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn rect_drawing() {
	let (mut screen, palette) = setup();

	screen.rect(10, 10, 90, 90, 1);
	screen.rect(10, 110, 90, 190, 2);
	screen.rect(190, 90, 110, 10, 3);

	//////

	screen.rect(-8, 10, 7, 25, 5);
	screen.rect(20, -8, 35, 7, 6);
	screen.rect(313, 170, 328, 185, 7);
	screen.rect(285, 233, 300, 248, 8);

	screen.rect(-16, 30, -1, 46, 9);
	screen.rect(40, -16, 55, -1, 10);
	screen.rect(320, 150, 335, 165, 11);
	screen.rect(265, 240, 280, 255, 12);

	screen.rect(300, 20, 340, -20, 13);
	screen.rect(20, 220, -20, 260, 14);

	let path = make_ref_path("rect_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_rect_drawing() {
	let (mut screen, palette, blend_map) = setup_for_blending();

	screen.blended_rect(10, 10, 90, 90, 1, &blend_map);
	screen.blended_rect(10, 110, 90, 190, 2, &blend_map);
	screen.blended_rect(190, 90, 110, 10, 3, &blend_map);

	//////

	screen.blended_rect(-8, 10, 7, 25, 5, &blend_map);
	screen.blended_rect(20, -8, 35, 7, 6, &blend_map);
	screen.blended_rect(313, 170, 328, 185, 7, &blend_map);
	screen.blended_rect(285, 233, 300, 248, 8, &blend_map);

	screen.blended_rect(-16, 30, -1, 46, 9, &blend_map);
	screen.blended_rect(40, -16, 55, -1, 10, &blend_map);
	screen.blended_rect(320, 150, 335, 165, 11, &blend_map);
	screen.blended_rect(265, 240, 280, 255, 12, &blend_map);

	screen.blended_rect(300, 20, 340, -20, 13, &blend_map);
	screen.blended_rect(20, 220, -20, 260, 14, &blend_map);

	let path = make_ref_path("blended_rect_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn filled_rect_drawing() {
	let (mut screen, palette) = setup();

	screen.filled_rect(10, 10, 90, 90, 1);
	screen.filled_rect(10, 110, 90, 190, 2);
	screen.filled_rect(190, 90, 110, 10, 3);

	//////

	screen.filled_rect(-8, 10, 7, 25, 5);
	screen.filled_rect(20, -8, 35, 7, 6);
	screen.filled_rect(313, 170, 328, 185, 7);
	screen.filled_rect(285, 233, 300, 248, 8);

	screen.filled_rect(-16, 30, -1, 46, 9);
	screen.filled_rect(40, -16, 55, -1, 10);
	screen.filled_rect(320, 150, 335, 165, 11);
	screen.filled_rect(265, 240, 280, 255, 12);

	screen.filled_rect(300, 20, 340, -20, 13);
	screen.filled_rect(20, 220, -20, 260, 14);

	let path = make_ref_path("filled_rect_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_filled_rect_drawing() {
	let (mut screen, palette, blend_map) = setup_for_blending();

	screen.blended_filled_rect(10, 10, 90, 90, 1, &blend_map);
	screen.blended_filled_rect(10, 110, 90, 190, 2, &blend_map);
	screen.blended_filled_rect(190, 90, 110, 10, 3, &blend_map);

	//////

	screen.blended_filled_rect(-8, 10, 7, 25, 5, &blend_map);
	screen.blended_filled_rect(20, -8, 35, 7, 6, &blend_map);
	screen.blended_filled_rect(313, 170, 328, 185, 7, &blend_map);
	screen.blended_filled_rect(285, 233, 300, 248, 8, &blend_map);

	screen.blended_filled_rect(-16, 30, -1, 46, 9, &blend_map);
	screen.blended_filled_rect(40, -16, 55, -1, 10, &blend_map);
	screen.blended_filled_rect(320, 150, 335, 165, 11, &blend_map);
	screen.blended_filled_rect(265, 240, 280, 255, 12, &blend_map);

	screen.blended_filled_rect(300, 20, 340, -20, 13, &blend_map);
	screen.blended_filled_rect(20, 220, -20, 260, 14, &blend_map);

	let path = make_ref_path("blended_filled_rect_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn circle_drawing() {
	let (mut screen, palette) = setup();

	screen.circle(48, 48, 32, 1);
	screen.circle(128, 48, 24, 2);
	screen.circle(48, 128, 40, 3);

	//////

	screen.circle(0, 30, 16, 5);
	screen.circle(40, 2, 11, 6);
	screen.circle(319, 211, 17, 7);
	screen.circle(290, 241, 21, 8);

	screen.circle(319, 1, 22, 9);
	screen.circle(2, 242, 19, 10);

	let path = make_ref_path("circle_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn filled_circle_drawing() {
	let (mut screen, palette) = setup();

	screen.filled_circle(48, 48, 32, 1);
	screen.filled_circle(128, 48, 24, 2);
	screen.filled_circle(48, 128, 40, 3);

	//////

	screen.filled_circle(0, 30, 16, 5);
	screen.filled_circle(40, 2, 11, 6);
	screen.filled_circle(319, 211, 17, 7);
	screen.filled_circle(290, 241, 21, 8);

	screen.filled_circle(319, 1, 22, 9);
	screen.filled_circle(2, 242, 19, 10);

	let path = make_ref_path("filled_circle_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn text_drawing() {
	let (mut screen, palette) = setup();

	let font = BitmaskFont::new_vga_font().unwrap();
	let small_font = BitmaskFont::load_from_file(Path::new("./test-assets/small.fnt")).unwrap();
	let chunky_font = BitmaskFont::load_from_file(Path::new("./test-assets/chunky.fnt")).unwrap();

	let message = "Hello, world! HELLO, WORLD!\nTesting 123";

	screen.print_string(message, 20, 20, FontRenderOpts::Color(1), &font);
	screen.print_string(message, 20, 40, FontRenderOpts::Color(2), &small_font);
	screen.print_string(message, 20, 60, FontRenderOpts::Color(3), &chunky_font);

	screen.filled_rect(58, 218, 162, 230, 7);
	screen.print_string("transparency!", 60, 220, FontRenderOpts::Color(9), &font);

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

	screen.print_string(&s, 20, 80, FontRenderOpts::Color(15), &font);
	screen.print_string(&s, 110, 80, FontRenderOpts::Color(15), &small_font);
	screen.print_string(&s, 190, 80, FontRenderOpts::Color(15), &chunky_font);

	//////

	let message = "Hello, world!";

	screen.print_string(message, -35, 10, FontRenderOpts::Color(9), &font);
	screen.print_string(message, 80, -4, FontRenderOpts::Color(10), &font);
	screen.print_string(message, 285, 120, FontRenderOpts::Color(11), &font);
	screen.print_string(message, 200, 236, FontRenderOpts::Color(12), &font);
	screen.print_string(message, -232, 10, FontRenderOpts::Color(5), &font);
	screen.print_string(message, 80, -24, FontRenderOpts::Color(6), &font);
	screen.print_string(message, 360, 120, FontRenderOpts::Color(7), &font);
	screen.print_string(message, 200, 250, FontRenderOpts::Color(8), &font);

	let path = make_ref_path("text_drawing.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

fn generate_bitmap(width: i32, height: i32) -> IndexedBitmap {
	let x_third = width / 3;
	let y_third = height / 3;

	let mut bitmap = IndexedBitmap::new(width as u32, height as u32).unwrap();

	bitmap.filled_rect(0, 0, x_third, y_third, 1);
	bitmap.filled_rect(x_third * 2 + 1, y_third * 2 + 1, width - 1, height - 1, 2);
	bitmap.filled_rect(0, y_third * 2 + 1, x_third, height - 1, 3);
	bitmap.filled_rect(x_third * 2 + 1, 0, width - 1, y_third, 4);
	bitmap.filled_rect(x_third, y_third, x_third * 2 + 1, y_third * 2 + 1, 5);
	bitmap.rect(0, 0, width - 1, height - 1, 6);

	bitmap
}

#[test]
fn solid_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

	let bmp16 = generate_bitmap(16, 16);
	let bmp12 = generate_bitmap(12, 12);
	let bmp21 = generate_bitmap(21, 21);
	let bmp3 = generate_bitmap(3, 3);

	let x = 40;
	let y = 20;
	screen.blit(Solid, &bmp16, x + 16, y + 48);
	screen.blit(Solid, &bmp12, x + 80, y + 48);
	screen.blit(Solid, &bmp21, x + 144, y + 48);
	screen.blit(Solid, &bmp3, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(Solid, &bmp16, x + 16, y + 48);
		screen.blit_unchecked(Solid, &bmp12, x + 80, y + 48);
		screen.blit_unchecked(Solid, &bmp21, x + 144, y + 48);
		screen.blit_unchecked(Solid, &bmp3, x + 208, y + 48);
	}

	//////

	screen.blit(Solid, &bmp16, -3, 46);
	screen.blit(Solid, &bmp16, -4, 76);
	screen.blit(Solid, &bmp16, -8, 106);
	screen.blit(Solid, &bmp16, -12, 136);
	screen.blit(Solid, &bmp16, -13, 166);
	screen.blit(Solid, &bmp16, -14, 196);
	screen.blit(Solid, &bmp16, -16, 226);

	screen.blit(Solid, &bmp16, 46, -3);
	screen.blit(Solid, &bmp16, 76, -4);
	screen.blit(Solid, &bmp16, 106, -8);
	screen.blit(Solid, &bmp16, 136, -12);
	screen.blit(Solid, &bmp16, 166, -13);
	screen.blit(Solid, &bmp16, 196, -14);
	screen.blit(Solid, &bmp16, 226, -16);

	screen.blit(Solid, &bmp16, 307, 46);
	screen.blit(Solid, &bmp16, 308, 76);
	screen.blit(Solid, &bmp16, 312, 106);
	screen.blit(Solid, &bmp16, 316, 136);
	screen.blit(Solid, &bmp16, 317, 166);
	screen.blit(Solid, &bmp16, 318, 196);
	screen.blit(Solid, &bmp16, 320, 226);

	screen.blit(Solid, &bmp16, 46, 227);
	screen.blit(Solid, &bmp16, 76, 228);
	screen.blit(Solid, &bmp16, 106, 232);
	screen.blit(Solid, &bmp16, 136, 236);
	screen.blit(Solid, &bmp16, 166, 237);
	screen.blit(Solid, &bmp16, 196, 238);
	screen.blit(Solid, &bmp16, 226, 240);

	let path = make_ref_path("solid_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_solid_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette, blend_map) = setup_for_blending();
	let blend_map = Rc::new(blend_map);

	let bmp16 = generate_bitmap(16, 16);
	let bmp12 = generate_bitmap(12, 12);
	let bmp21 = generate_bitmap(21, 21);
	let bmp3 = generate_bitmap(3, 3);

	let x = 40;
	let y = 20;
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, x + 16, y + 48);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp12, x + 80, y + 48);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp21, x + 144, y + 48);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp3, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(SolidBlended { blend_map: blend_map.clone() }, &bmp16, x + 16, y + 48);
		screen.blit_unchecked(SolidBlended { blend_map: blend_map.clone() }, &bmp12, x + 80, y + 48);
		screen.blit_unchecked(SolidBlended { blend_map: blend_map.clone() }, &bmp21, x + 144, y + 48);
		screen.blit_unchecked(SolidBlended { blend_map: blend_map.clone() }, &bmp3, x + 208, y + 48);
	}

	//////

	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, -3, 46);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, -4, 76);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, -8, 106);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, -12, 136);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, -13, 166);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, -14, 196);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, -16, 226);

	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 46, -3);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 76, -4);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 106, -8);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 136, -12);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 166, -13);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 196, -14);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 226, -16);

	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 307, 46);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 308, 76);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 312, 106);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 316, 136);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 317, 166);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 318, 196);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 320, 226);

	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 46, 227);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 76, 228);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 106, 232);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 136, 236);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 166, 237);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 196, 238);
	screen.blit(SolidBlended { blend_map: blend_map.clone() }, &bmp16, 226, 240);

	let path = make_ref_path("blended_solid_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn solid_flipped_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

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

	let path = make_ref_path("solid_flipped_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_solid_flipped_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette, blend_map) = setup_for_blending();
	let blend_map = Rc::new(blend_map);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, x + 16, y + 48);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, x + 80, y + 48);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, x + 144, y + 48);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(SolidFlippedBlended { horizontal_flip: true, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, -3, 46);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, -4, 76);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, -8, 106);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, -12, 136);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, -13, 166);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, -14, 196);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, -16, 226);

	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 46, -3);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 76, -4);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 106, -8);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 136, -12);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 166, -13);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 196, -14);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 226, -16);

	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 307, 46);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 308, 76);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 312, 106);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 316, 136);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 317, 166);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 318, 196);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 320, 226);

	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 46, 227);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 76, 228);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 106, 232);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 136, 236);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 166, 237);
	screen.blit(SolidFlippedBlended { horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 196, 238);
	screen.blit(SolidFlippedBlended { horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 226, 240);

	let path = make_ref_path("blended_solid_flipped_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn solid_offset_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(SolidOffset(0), &bmp, x + 16, y + 48);
	screen.blit(SolidOffset(4), &bmp, x + 80, y + 48);
	screen.blit(SolidOffset(7), &bmp, x + 144, y + 48);
	screen.blit(SolidOffset(13), &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(SolidOffset(0), &bmp, x + 16, y + 48);
		screen.blit_unchecked(SolidOffset(4), &bmp, x + 80, y + 48);
		screen.blit_unchecked(SolidOffset(7), &bmp, x + 144, y + 48);
		screen.blit_unchecked(SolidOffset(13), &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(SolidOffset(3), &bmp, -3, 46);
	screen.blit(SolidOffset(3), &bmp, -4, 76);
	screen.blit(SolidOffset(3), &bmp, -8, 106);
	screen.blit(SolidOffset(3), &bmp, -12, 136);
	screen.blit(SolidOffset(3), &bmp, -13, 166);
	screen.blit(SolidOffset(3), &bmp, -14, 196);
	screen.blit(SolidOffset(3), &bmp, -16, 226);

	screen.blit(SolidOffset(8), &bmp, 46, -3);
	screen.blit(SolidOffset(8), &bmp, 76, -4);
	screen.blit(SolidOffset(8), &bmp, 106, -8);
	screen.blit(SolidOffset(8), &bmp, 136, -12);
	screen.blit(SolidOffset(8), &bmp, 166, -13);
	screen.blit(SolidOffset(8), &bmp, 196, -14);
	screen.blit(SolidOffset(8), &bmp, 226, -16);

	screen.blit(SolidOffset(15), &bmp, 307, 46);
	screen.blit(SolidOffset(15), &bmp, 308, 76);
	screen.blit(SolidOffset(15), &bmp, 312, 106);
	screen.blit(SolidOffset(15), &bmp, 316, 136);
	screen.blit(SolidOffset(15), &bmp, 317, 166);
	screen.blit(SolidOffset(15), &bmp, 318, 196);
	screen.blit(SolidOffset(15), &bmp, 320, 226);

	screen.blit(SolidOffset(22), &bmp, 46, 227);
	screen.blit(SolidOffset(22), &bmp, 76, 228);
	screen.blit(SolidOffset(22), &bmp, 106, 232);
	screen.blit(SolidOffset(22), &bmp, 136, 236);
	screen.blit(SolidOffset(22), &bmp, 166, 237);
	screen.blit(SolidOffset(22), &bmp, 196, 238);
	screen.blit(SolidOffset(22), &bmp, 226, 240);

	let path = make_ref_path("solid_offset_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn solid_flipped_offset_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(SolidFlippedOffset { offset: 0, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
	screen.blit(SolidFlippedOffset { offset: 4, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
	screen.blit(SolidFlippedOffset { offset: 7, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
	screen.blit(SolidFlippedOffset { offset: 13, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(SolidFlippedOffset { offset: 0, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(SolidFlippedOffset { offset: 4, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(SolidFlippedOffset { offset: 7, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(SolidFlippedOffset { offset: 13, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(SolidFlippedOffset { offset: 3, horizontal_flip: false, vertical_flip: false }, &bmp, -3, 46);
	screen.blit(SolidFlippedOffset { offset: 3, horizontal_flip: true, vertical_flip: false }, &bmp, -4, 76);
	screen.blit(SolidFlippedOffset { offset: 3, horizontal_flip: false, vertical_flip: true }, &bmp, -8, 106);
	screen.blit(SolidFlippedOffset { offset: 3, horizontal_flip: true, vertical_flip: true }, &bmp, -12, 136);
	screen.blit(SolidFlippedOffset { offset: 3, horizontal_flip: false, vertical_flip: true }, &bmp, -13, 166);
	screen.blit(SolidFlippedOffset { offset: 3, horizontal_flip: true, vertical_flip: false }, &bmp, -14, 196);
	screen.blit(SolidFlippedOffset { offset: 3, horizontal_flip: false, vertical_flip: true }, &bmp, -16, 226);

	screen.blit(SolidFlippedOffset { offset: 8, horizontal_flip: false, vertical_flip: false }, &bmp, 46, -3);
	screen.blit(SolidFlippedOffset { offset: 8, horizontal_flip: true, vertical_flip: false }, &bmp, 76, -4);
	screen.blit(SolidFlippedOffset { offset: 8, horizontal_flip: false, vertical_flip: true }, &bmp, 106, -8);
	screen.blit(SolidFlippedOffset { offset: 8, horizontal_flip: true, vertical_flip: true }, &bmp, 136, -12);
	screen.blit(SolidFlippedOffset { offset: 8, horizontal_flip: false, vertical_flip: false }, &bmp, 166, -13);
	screen.blit(SolidFlippedOffset { offset: 8, horizontal_flip: true, vertical_flip: false }, &bmp, 196, -14);
	screen.blit(SolidFlippedOffset { offset: 8, horizontal_flip: false, vertical_flip: true }, &bmp, 226, -16);

	screen.blit(SolidFlippedOffset { offset: 15, horizontal_flip: false, vertical_flip: false }, &bmp, 307, 46);
	screen.blit(SolidFlippedOffset { offset: 15, horizontal_flip: true, vertical_flip: false }, &bmp, 308, 76);
	screen.blit(SolidFlippedOffset { offset: 15, horizontal_flip: false, vertical_flip: true }, &bmp, 312, 106);
	screen.blit(SolidFlippedOffset { offset: 15, horizontal_flip: true, vertical_flip: true }, &bmp, 316, 136);
	screen.blit(SolidFlippedOffset { offset: 15, horizontal_flip: false, vertical_flip: false }, &bmp, 317, 166);
	screen.blit(SolidFlippedOffset { offset: 15, horizontal_flip: true, vertical_flip: false }, &bmp, 318, 196);
	screen.blit(SolidFlippedOffset { offset: 15, horizontal_flip: false, vertical_flip: true }, &bmp, 320, 226);

	screen.blit(SolidFlippedOffset { offset: 22, horizontal_flip: false, vertical_flip: false }, &bmp, 46, 227);
	screen.blit(SolidFlippedOffset { offset: 22, horizontal_flip: true, vertical_flip: false }, &bmp, 76, 228);
	screen.blit(SolidFlippedOffset { offset: 22, horizontal_flip: false, vertical_flip: true }, &bmp, 106, 232);
	screen.blit(SolidFlippedOffset { offset: 22, horizontal_flip: true, vertical_flip: true }, &bmp, 136, 236);
	screen.blit(SolidFlippedOffset { offset: 22, horizontal_flip: false, vertical_flip: false }, &bmp, 166, 237);
	screen.blit(SolidFlippedOffset { offset: 22, horizontal_flip: true, vertical_flip: false }, &bmp, 196, 238);
	screen.blit(SolidFlippedOffset { offset: 22, horizontal_flip: false, vertical_flip: true }, &bmp, 226, 240);

	let path = make_ref_path("solid_flipped_offset_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn transparent_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

	let bmp16 = generate_bitmap(16, 16);
	let bmp12 = generate_bitmap(12, 12);
	let bmp21 = generate_bitmap(21, 21);
	let bmp3 = generate_bitmap(3, 3);

	let x = 40;
	let y = 20;
	screen.blit(Transparent(0), &bmp16, x + 16, y + 48);
	screen.blit(Transparent(0), &bmp12, x + 80, y + 48);
	screen.blit(Transparent(0), &bmp21, x + 144, y + 48);
	screen.blit(Transparent(0), &bmp3, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(Transparent(0), &bmp16, x + 16, y + 48);
		screen.blit_unchecked(Transparent(0), &bmp12, x + 80, y + 48);
		screen.blit_unchecked(Transparent(0), &bmp21, x + 144, y + 48);
		screen.blit_unchecked(Transparent(0), &bmp3, x + 208, y + 48);
	}

	//////

	screen.blit(Transparent(0), &bmp16, -3, 46);
	screen.blit(Transparent(0), &bmp16, -4, 76);
	screen.blit(Transparent(0), &bmp16, -8, 106);
	screen.blit(Transparent(0), &bmp16, -12, 136);
	screen.blit(Transparent(0), &bmp16, -13, 166);
	screen.blit(Transparent(0), &bmp16, -14, 196);
	screen.blit(Transparent(0), &bmp16, -16, 226);

	screen.blit(Transparent(0), &bmp16, 46, -3);
	screen.blit(Transparent(0), &bmp16, 76, -4);
	screen.blit(Transparent(0), &bmp16, 106, -8);
	screen.blit(Transparent(0), &bmp16, 136, -12);
	screen.blit(Transparent(0), &bmp16, 166, -13);
	screen.blit(Transparent(0), &bmp16, 196, -14);
	screen.blit(Transparent(0), &bmp16, 226, -16);

	screen.blit(Transparent(0), &bmp16, 307, 46);
	screen.blit(Transparent(0), &bmp16, 308, 76);
	screen.blit(Transparent(0), &bmp16, 312, 106);
	screen.blit(Transparent(0), &bmp16, 316, 136);
	screen.blit(Transparent(0), &bmp16, 317, 166);
	screen.blit(Transparent(0), &bmp16, 318, 196);
	screen.blit(Transparent(0), &bmp16, 320, 226);

	screen.blit(Transparent(0), &bmp16, 46, 227);
	screen.blit(Transparent(0), &bmp16, 76, 228);
	screen.blit(Transparent(0), &bmp16, 106, 232);
	screen.blit(Transparent(0), &bmp16, 136, 236);
	screen.blit(Transparent(0), &bmp16, 166, 237);
	screen.blit(Transparent(0), &bmp16, 196, 238);
	screen.blit(Transparent(0), &bmp16, 226, 240);

	let path = make_ref_path("transparent_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_transparent_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette, blend_map) = setup_for_blending();
	let blend_map = Rc::new(blend_map);

	let bmp16 = generate_bitmap(16, 16);
	let bmp12 = generate_bitmap(12, 12);
	let bmp21 = generate_bitmap(21, 21);
	let bmp3 = generate_bitmap(3, 3);

	let x = 40;
	let y = 20;
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, x + 16, y + 48);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp12, x + 80, y + 48);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp21, x + 144, y + 48);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp3, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, x + 16, y + 48);
		screen.blit_unchecked(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp12, x + 80, y + 48);
		screen.blit_unchecked(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp21, x + 144, y + 48);
		screen.blit_unchecked(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp3, x + 208, y + 48);
	}

	//////

	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, -3, 46);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, -4, 76);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, -8, 106);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, -12, 136);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, -13, 166);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, -14, 196);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, -16, 226);

	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 46, -3);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 76, -4);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 106, -8);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 136, -12);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 166, -13);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 196, -14);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 226, -16);

	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 307, 46);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 308, 76);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 312, 106);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 316, 136);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 317, 166);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 318, 196);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 320, 226);

	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 46, 227);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 76, 228);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 106, 232);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 136, 236);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 166, 237);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 196, 238);
	screen.blit(TransparentBlended { transparent_color: 0, blend_map: blend_map.clone() }, &bmp16, 226, 240);

	let path = make_ref_path("blended_transparent_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn transparent_flipped_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: false }, &bmp, -3, 46);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: false }, &bmp, -4, 76);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: true }, &bmp, -8, 106);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: true }, &bmp, -12, 136);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: true }, &bmp, -13, 166);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: false }, &bmp, -14, 196);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: true }, &bmp, -16, 226);

	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: false }, &bmp, 46, -3);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: false }, &bmp, 76, -4);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: true }, &bmp, 106, -8);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: true }, &bmp, 136, -12);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: false }, &bmp, 166, -13);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: false }, &bmp, 196, -14);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: true }, &bmp, 226, -16);

	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: false }, &bmp, 307, 46);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: false }, &bmp, 308, 76);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: true }, &bmp, 312, 106);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: true }, &bmp, 316, 136);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: false }, &bmp, 317, 166);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: false }, &bmp, 318, 196);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: true }, &bmp, 320, 226);

	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: false }, &bmp, 46, 227);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: false }, &bmp, 76, 228);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: true }, &bmp, 106, 232);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: true }, &bmp, 136, 236);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: false }, &bmp, 166, 237);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: true, vertical_flip: false }, &bmp, 196, 238);
	screen.blit(TransparentFlipped { transparent_color: 0, horizontal_flip: false, vertical_flip: true }, &bmp, 226, 240);

	let path = make_ref_path("transparent_flipped_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_transparent_flipped_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette, blend_map) = setup_for_blending();
	let blend_map = Rc::new(blend_map);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, x + 16, y + 48);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, x + 80, y + 48);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, x + 144, y + 48);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, -3, 46);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, -4, 76);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, -8, 106);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, -12, 136);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, -13, 166);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, -14, 196);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, -16, 226);

	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 46, -3);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 76, -4);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 106, -8);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 136, -12);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 166, -13);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 196, -14);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 226, -16);

	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 307, 46);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 308, 76);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 312, 106);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 316, 136);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 317, 166);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 318, 196);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 320, 226);

	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 46, 227);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 76, 228);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 106, 232);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 136, 236);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 166, 237);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: true, vertical_flip: false, blend_map: blend_map.clone() }, &bmp, 196, 238);
	screen.blit(TransparentFlippedBlended { transparent_color: 0, horizontal_flip: false, vertical_flip: true, blend_map: blend_map.clone() }, &bmp, 226, 240);

	let path = make_ref_path("blended_transparent_flipped_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn transparent_offset_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(TransparentOffset { transparent_color: 0, offset: 0 }, &bmp, x + 16, y + 48);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 4 }, &bmp, x + 80, y + 48);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 7 }, &bmp, x + 144, y + 48);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 13 }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(TransparentOffset { transparent_color: 0, offset: 0 }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(TransparentOffset { transparent_color: 0, offset: 4 }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(TransparentOffset { transparent_color: 0, offset: 7 }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(TransparentOffset { transparent_color: 0, offset: 13 }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(TransparentOffset { transparent_color: 0, offset: 3 }, &bmp, -3, 46);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 3 }, &bmp, -4, 76);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 3 }, &bmp, -8, 106);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 3 }, &bmp, -12, 136);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 3 }, &bmp, -13, 166);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 3 }, &bmp, -14, 196);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 3 }, &bmp, -16, 226);

	screen.blit(TransparentOffset { transparent_color: 0, offset: 8 }, &bmp, 46, -3);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 8 }, &bmp, 76, -4);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 8 }, &bmp, 106, -8);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 8 }, &bmp, 136, -12);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 8 }, &bmp, 166, -13);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 8 }, &bmp, 196, -14);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 8 }, &bmp, 226, -16);

	screen.blit(TransparentOffset { transparent_color: 0, offset: 15 }, &bmp, 307, 46);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 15 }, &bmp, 308, 76);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 15 }, &bmp, 312, 106);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 15 }, &bmp, 316, 136);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 15 }, &bmp, 317, 166);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 15 }, &bmp, 318, 196);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 15 }, &bmp, 320, 226);

	screen.blit(TransparentOffset { transparent_color: 0, offset: 22 }, &bmp, 46, 227);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 22 }, &bmp, 76, 228);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 22 }, &bmp, 106, 232);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 22 }, &bmp, 136, 236);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 22 }, &bmp, 166, 237);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 22 }, &bmp, 196, 238);
	screen.blit(TransparentOffset { transparent_color: 0, offset: 22 }, &bmp, 226, 240);

	let path = make_ref_path("transparent_offset_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn transparent_flipped_offset_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 0, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 4, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 7, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 13, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(TransparentFlippedOffset { transparent_color: 0, offset: 0, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(TransparentFlippedOffset { transparent_color: 0, offset: 4, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(TransparentFlippedOffset { transparent_color: 0, offset: 7, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(TransparentFlippedOffset { transparent_color: 0, offset: 13, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 3, horizontal_flip: false, vertical_flip: false }, &bmp, -3, 46);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 3, horizontal_flip: true, vertical_flip: false }, &bmp, -4, 76);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 3, horizontal_flip: false, vertical_flip: true }, &bmp, -8, 106);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 3, horizontal_flip: true, vertical_flip: true }, &bmp, -12, 136);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 3, horizontal_flip: false, vertical_flip: true }, &bmp, -13, 166);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 3, horizontal_flip: true, vertical_flip: false }, &bmp, -14, 196);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 3, horizontal_flip: false, vertical_flip: true }, &bmp, -16, 226);

	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 8, horizontal_flip: false, vertical_flip: false }, &bmp, 46, -3);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 8, horizontal_flip: true, vertical_flip: false }, &bmp, 76, -4);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 8, horizontal_flip: false, vertical_flip: true }, &bmp, 106, -8);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 8, horizontal_flip: true, vertical_flip: true }, &bmp, 136, -12);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 8, horizontal_flip: false, vertical_flip: false }, &bmp, 166, -13);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 8, horizontal_flip: true, vertical_flip: false }, &bmp, 196, -14);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 8, horizontal_flip: false, vertical_flip: true }, &bmp, 226, -16);

	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 15, horizontal_flip: false, vertical_flip: false }, &bmp, 307, 46);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 15, horizontal_flip: true, vertical_flip: false }, &bmp, 308, 76);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 15, horizontal_flip: false, vertical_flip: true }, &bmp, 312, 106);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 15, horizontal_flip: true, vertical_flip: true }, &bmp, 316, 136);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 15, horizontal_flip: false, vertical_flip: false }, &bmp, 317, 166);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 15, horizontal_flip: true, vertical_flip: false }, &bmp, 318, 196);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 15, horizontal_flip: false, vertical_flip: true }, &bmp, 320, 226);

	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 22, horizontal_flip: false, vertical_flip: false }, &bmp, 46, 227);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 22, horizontal_flip: true, vertical_flip: false }, &bmp, 76, 228);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 22, horizontal_flip: false, vertical_flip: true }, &bmp, 106, 232);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 22, horizontal_flip: true, vertical_flip: true }, &bmp, 136, 236);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 22, horizontal_flip: false, vertical_flip: false }, &bmp, 166, 237);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 22, horizontal_flip: true, vertical_flip: false }, &bmp, 196, 238);
	screen.blit(TransparentFlippedOffset { transparent_color: 0, offset: 22, horizontal_flip: false, vertical_flip: true }, &bmp, 226, 240);

	let path = make_ref_path("transparent_flipped_offset_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn transparent_single_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 1 }, &bmp, x + 16, y + 48);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 4 }, &bmp, x + 80, y + 48);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 7 }, &bmp, x + 144, y + 48);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 13 }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(TransparentSingle { transparent_color: 0, draw_color: 1 }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(TransparentSingle { transparent_color: 0, draw_color: 4 }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(TransparentSingle { transparent_color: 0, draw_color: 7 }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(TransparentSingle { transparent_color: 0, draw_color: 13 }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 3 }, &bmp, -3, 46);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 3 }, &bmp, -4, 76);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 3 }, &bmp, -8, 106);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 3 }, &bmp, -12, 136);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 3 }, &bmp, -13, 166);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 3 }, &bmp, -14, 196);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 3 }, &bmp, -16, 226);

	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 8 }, &bmp, 46, -3);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 8 }, &bmp, 76, -4);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 8 }, &bmp, 106, -8);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 8 }, &bmp, 136, -12);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 8 }, &bmp, 166, -13);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 8 }, &bmp, 196, -14);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 8 }, &bmp, 226, -16);

	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 15 }, &bmp, 307, 46);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 15 }, &bmp, 308, 76);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 15 }, &bmp, 312, 106);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 15 }, &bmp, 316, 136);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 15 }, &bmp, 317, 166);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 15 }, &bmp, 318, 196);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 15 }, &bmp, 320, 226);

	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 22 }, &bmp, 46, 227);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 22 }, &bmp, 76, 228);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 22 }, &bmp, 106, 232);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 22 }, &bmp, 136, 236);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 22 }, &bmp, 166, 237);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 22 }, &bmp, 196, 238);
	screen.blit(TransparentSingle { transparent_color: 0, draw_color: 22 }, &bmp, 226, 240);

	let path = make_ref_path("transparent_single_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn transparent_flipped_single_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 1, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 4, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 7, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 13, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(TransparentFlippedSingle { transparent_color: 0, draw_color: 1, horizontal_flip: false, vertical_flip: false }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(TransparentFlippedSingle { transparent_color: 0, draw_color: 4, horizontal_flip: true, vertical_flip: false }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(TransparentFlippedSingle { transparent_color: 0, draw_color: 7, horizontal_flip: false, vertical_flip: true }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(TransparentFlippedSingle { transparent_color: 0, draw_color: 13, horizontal_flip: true, vertical_flip: true }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 3, horizontal_flip: false, vertical_flip: false }, &bmp, -3, 46);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 3, horizontal_flip: true, vertical_flip: false }, &bmp, -4, 76);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 3, horizontal_flip: false, vertical_flip: true }, &bmp, -8, 106);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 3, horizontal_flip: true, vertical_flip: true }, &bmp, -12, 136);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 3, horizontal_flip: false, vertical_flip: true }, &bmp, -13, 166);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 3, horizontal_flip: true, vertical_flip: false }, &bmp, -14, 196);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 3, horizontal_flip: false, vertical_flip: true }, &bmp, -16, 226);

	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 8, horizontal_flip: false, vertical_flip: false }, &bmp, 46, -3);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 8, horizontal_flip: true, vertical_flip: false }, &bmp, 76, -4);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 8, horizontal_flip: false, vertical_flip: true }, &bmp, 106, -8);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 8, horizontal_flip: true, vertical_flip: true }, &bmp, 136, -12);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 8, horizontal_flip: false, vertical_flip: false }, &bmp, 166, -13);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 8, horizontal_flip: true, vertical_flip: false }, &bmp, 196, -14);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 8, horizontal_flip: false, vertical_flip: true }, &bmp, 226, -16);

	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 15, horizontal_flip: false, vertical_flip: false }, &bmp, 307, 46);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 15, horizontal_flip: true, vertical_flip: false }, &bmp, 308, 76);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 15, horizontal_flip: false, vertical_flip: true }, &bmp, 312, 106);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 15, horizontal_flip: true, vertical_flip: true }, &bmp, 316, 136);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 15, horizontal_flip: false, vertical_flip: false }, &bmp, 317, 166);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 15, horizontal_flip: true, vertical_flip: false }, &bmp, 318, 196);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 15, horizontal_flip: false, vertical_flip: true }, &bmp, 320, 226);

	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 22, horizontal_flip: false, vertical_flip: false }, &bmp, 46, 227);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 22, horizontal_flip: true, vertical_flip: false }, &bmp, 76, 228);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 22, horizontal_flip: false, vertical_flip: true }, &bmp, 106, 232);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 22, horizontal_flip: true, vertical_flip: true }, &bmp, 136, 236);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 22, horizontal_flip: false, vertical_flip: false }, &bmp, 166, 237);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 22, horizontal_flip: true, vertical_flip: false }, &bmp, 196, 238);
	screen.blit(TransparentFlippedSingle { transparent_color: 0, draw_color: 22, horizontal_flip: false, vertical_flip: true }, &bmp, 226, 240);

	let path = make_ref_path("transparent_flipped_single_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn rotozoom_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

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

	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -3, 46);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -4, 76);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -8, 106);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -12, 136);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -13, 166);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -14, 196);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -16, 226);

	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 46, -3);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 76, -4);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 106, -8);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 136, -12);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 166, -13);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 196, -14);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 226, -16);

	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 307, 46);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 308, 76);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 312, 106);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 316, 136);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 317, 166);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 318, 196);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 320, 226);

	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 46, 227);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 76, 228);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 106, 232);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 136, 236);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 166, 237);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 196, 238);
	screen.blit(RotoZoom { angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 226, 240);

	let path = make_ref_path("rotozoom_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_rotozoom_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette, blend_map) = setup_for_blending();
	let blend_map = Rc::new(blend_map);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, x + 16, y + 48);
	screen.blit(RotoZoomBlended { angle: 0.3, scale_x: 1.5, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, x + 80, y + 48);
	screen.blit(RotoZoomBlended { angle: 0.6, scale_x: 1.0, scale_y: 1.5, blend_map: blend_map.clone() }, &bmp, x + 144, y + 48);
	screen.blit(RotoZoomBlended { angle: 2.0, scale_x: 0.7, scale_y: 0.7, blend_map: blend_map.clone() }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(RotoZoomBlended { angle: 0.3, scale_x: 1.5, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(RotoZoomBlended { angle: 0.6, scale_x: 1.0, scale_y: 1.5, blend_map: blend_map.clone() }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(RotoZoomBlended { angle: 2.0, scale_x: 0.7, scale_y: 0.7, blend_map: blend_map.clone() }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -3, 46);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -4, 76);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -8, 106);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -12, 136);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -13, 166);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -14, 196);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -16, 226);

	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 46, -3);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 76, -4);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 106, -8);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 136, -12);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 166, -13);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 196, -14);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 226, -16);

	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 307, 46);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 308, 76);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 312, 106);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 316, 136);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 317, 166);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 318, 196);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 320, 226);

	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 46, 227);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 76, 228);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 106, 232);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 136, 236);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 166, 237);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 196, 238);
	screen.blit(RotoZoomBlended { angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 226, 240);

	let path = make_ref_path("blended_rotozoom_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn rotozoom_offset_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 0 }, &bmp, x + 16, y + 48);
	screen.blit(RotoZoomOffset { angle: 0.3, scale_x: 1.5, scale_y: 1.0, offset: 4 }, &bmp, x + 80, y + 48);
	screen.blit(RotoZoomOffset { angle: 0.6, scale_x: 1.0, scale_y: 1.5, offset: 7 }, &bmp, x + 144, y + 48);
	screen.blit(RotoZoomOffset { angle: 2.0, scale_x: 0.7, scale_y: 0.7, offset: 13 }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 0 }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(RotoZoomOffset { angle: 0.3, scale_x: 1.5, scale_y: 1.0, offset: 4 }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(RotoZoomOffset { angle: 0.6, scale_x: 1.0, scale_y: 1.5, offset: 7 }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(RotoZoomOffset { angle: 2.0, scale_x: 0.7, scale_y: 0.7, offset: 13 }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 3 }, &bmp, -3, 46);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 3 }, &bmp, -4, 76);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 3 }, &bmp, -8, 106);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 3 }, &bmp, -12, 136);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 3 }, &bmp, -13, 166);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 3 }, &bmp, -14, 196);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 3 }, &bmp, -16, 226);

	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 8 }, &bmp, 46, -3);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 8 }, &bmp, 76, -4);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 8 }, &bmp, 106, -8);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 8 }, &bmp, 136, -12);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 8 }, &bmp, 166, -13);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 8 }, &bmp, 196, -14);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 8 }, &bmp, 226, -16);

	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 15 }, &bmp, 307, 46);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 15 }, &bmp, 308, 76);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 15 }, &bmp, 312, 106);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 15 }, &bmp, 316, 136);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 15 }, &bmp, 317, 166);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 15 }, &bmp, 318, 196);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 15 }, &bmp, 320, 226);

	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 22 }, &bmp, 46, 227);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 22 }, &bmp, 76, 228);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 22 }, &bmp, 106, 232);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 22 }, &bmp, 136, 236);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 22 }, &bmp, 166, 237);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 22 }, &bmp, 196, 238);
	screen.blit(RotoZoomOffset { angle: 1.3, scale_x: 1.0, scale_y: 1.0, offset: 22 }, &bmp, 226, 240);

	let path = make_ref_path("rotozoom_offset_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn rotozoom_transparent_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, x + 16, y + 48);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 0.3, scale_x: 1.5, scale_y: 1.0 }, &bmp, x + 80, y + 48);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 0.6, scale_x: 1.0, scale_y: 1.5 }, &bmp, x + 144, y + 48);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 2.0, scale_x: 0.7, scale_y: 0.7 }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(RotoZoomTransparent { transparent_color: 0, angle: 0.3, scale_x: 1.5, scale_y: 1.0 }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(RotoZoomTransparent { transparent_color: 0, angle: 0.6, scale_x: 1.0, scale_y: 1.5 }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(RotoZoomTransparent { transparent_color: 0, angle: 2.0, scale_x: 0.7, scale_y: 0.7 }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -3, 46);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -4, 76);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -8, 106);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -12, 136);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -13, 166);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -14, 196);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -16, 226);

	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 46, -3);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 76, -4);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 106, -8);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 136, -12);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 166, -13);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 196, -14);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 226, -16);

	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 307, 46);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 308, 76);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 312, 106);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 316, 136);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 317, 166);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 318, 196);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 320, 226);

	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 46, 227);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 76, 228);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 106, 232);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 136, 236);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 166, 237);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 196, 238);
	screen.blit(RotoZoomTransparent { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 226, 240);

	let path = make_ref_path("rotozoom_transparent_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn blended_rotozoom_transparent_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette, blend_map) = setup_for_blending();
	let blend_map = Rc::new(blend_map);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, x + 16, y + 48);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 0.3, scale_x: 1.5, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, x + 80, y + 48);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 0.6, scale_x: 1.0, scale_y: 1.5, blend_map: blend_map.clone() }, &bmp, x + 144, y + 48);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 2.0, scale_x: 0.7, scale_y: 0.7, blend_map: blend_map.clone() }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(RotoZoomTransparentBlended { transparent_color: 0, angle: 0.3, scale_x: 1.5, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(RotoZoomTransparentBlended { transparent_color: 0, angle: 0.6, scale_x: 1.0, scale_y: 1.5, blend_map: blend_map.clone() }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(RotoZoomTransparentBlended { transparent_color: 0, angle: 2.0, scale_x: 0.7, scale_y: 0.7, blend_map: blend_map.clone() }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -3, 46);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -4, 76);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -8, 106);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -12, 136);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -13, 166);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -14, 196);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, -16, 226);

	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 46, -3);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 76, -4);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 106, -8);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 136, -12);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 166, -13);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 196, -14);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 226, -16);

	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 307, 46);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 308, 76);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 312, 106);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 316, 136);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 317, 166);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 318, 196);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 320, 226);

	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 46, 227);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 76, 228);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 106, 232);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 136, 236);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 166, 237);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 196, 238);
	screen.blit(RotoZoomTransparentBlended { transparent_color: 0, angle: 1.3, scale_x: 1.0, scale_y: 1.0, blend_map: blend_map.clone() }, &bmp, 226, 240);

	let path = make_ref_path("blended_rotozoom_transparent_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}

#[test]
fn rotozoom_transparent_offset_blits() {
	use IndexedBlitMethod::*;

	let (mut screen, palette) = setup();
	screen.clear(247);

	let bmp = generate_bitmap(16, 16);

	let x = 40;
	let y = 20;
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 1, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, x + 16, y + 48);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 4, angle: 0.3, scale_x: 1.5, scale_y: 1.0 }, &bmp, x + 80, y + 48);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 7, angle: 0.6, scale_x: 1.0, scale_y: 1.5 }, &bmp, x + 144, y + 48);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 13, angle: 2.0, scale_x: 0.7, scale_y: 0.7 }, &bmp, x + 208, y + 48);

	let x = 40;
	let y = 110;
	unsafe {
		screen.blit_unchecked(RotoZoomTransparentOffset { transparent_color: 0, offset: 1, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, x + 16, y + 48);
		screen.blit_unchecked(RotoZoomTransparentOffset { transparent_color: 0, offset: 4, angle: 0.3, scale_x: 1.5, scale_y: 1.0 }, &bmp, x + 80, y + 48);
		screen.blit_unchecked(RotoZoomTransparentOffset { transparent_color: 0, offset: 7, angle: 0.6, scale_x: 1.0, scale_y: 1.5 }, &bmp, x + 144, y + 48);
		screen.blit_unchecked(RotoZoomTransparentOffset { transparent_color: 0, offset: 13, angle: 2.0, scale_x: 0.7, scale_y: 0.7 }, &bmp, x + 208, y + 48);
	}

	//////

	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 3, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -3, 46);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 3, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -4, 76);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 3, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -8, 106);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 3, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -12, 136);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 3, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -13, 166);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 3, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -14, 196);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 3, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, -16, 226);

	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 8, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 46, -3);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 8, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 76, -4);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 8, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 106, -8);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 8, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 136, -12);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 8, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 166, -13);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 8, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 196, -14);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 8, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 226, -16);

	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 15, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 307, 46);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 15, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 308, 76);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 15, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 312, 106);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 15, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 316, 136);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 15, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 317, 166);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 15, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 318, 196);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 15, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 320, 226);

	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 22, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 46, 227);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 22, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 76, 228);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 22, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 106, 232);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 22, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 136, 236);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 22, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 166, 237);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 22, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 196, 238);
	screen.blit(RotoZoomTransparentOffset { transparent_color: 0, offset: 22, angle: 1.3, scale_x: 1.0, scale_y: 1.0 }, &bmp, 226, 240);

	let path = make_ref_path("rotozoom_transparent_offset_blits.pcx");
	//screen.to_pcx_file(path.as_path(), &palette).unwrap();
	assert!(verify_visual(&screen, &palette, &path), "bitmap differs from source image: {:?}", path);
}


