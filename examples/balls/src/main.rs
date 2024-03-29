use std::path::Path;

use anyhow::Result;

use ggdt::prelude::*;

const NUM_BALLS: usize = 128;
const NUM_BALL_SPRITES: usize = 16;
const BALL_WIDTH: u32 = 8;
const BALL_HEIGHT: u32 = 8;

struct Ball {
	x: i32,
	y: i32,
	dir_x: i32,
	dir_y: i32,
	sprite: usize,
}

fn main() -> Result<()> {
	let config = DosLikeConfig::default();
	let mut system = SystemBuilder::new() //
		.window_title("Flying Balls!")
		.vsync(true)
		.build(config)?;

	let font = BitmaskFont::new_vga_font()?;

	let (balls_bmp, balls_palette) = IndexedBitmap::load_pcx_file(Path::new("./assets/balls.pcx"))?;
	system.res.palette = balls_palette.clone();

	let mut sprites = Vec::<IndexedBitmap>::new();
	let mut balls = Vec::<Ball>::new();

	for i in 0..NUM_BALL_SPRITES {
		let mut sprite = IndexedBitmap::new(BALL_WIDTH, BALL_HEIGHT)?;
		sprite.blit_region(
			IndexedBlitMethod::Solid,
			&balls_bmp,
			&Rect::new(i as i32 * BALL_WIDTH as i32, 0, BALL_WIDTH, BALL_HEIGHT),
			0,
			0,
		);
		sprites.push(sprite);
	}

	for _ in 0..NUM_BALLS {
		let speed = rnd_value(1, 3);
		let ball = Ball {
			x: rnd_value(0, system.res.video.width() as i32 - 1),
			y: rnd_value(0, system.res.video.height() as i32 - 1),
			dir_x: if rnd_value(0, 1) == 0 { -speed } else { speed },
			dir_y: if rnd_value(0, 1) == 0 { -speed } else { speed },
			sprite: rnd_value(0, NUM_BALL_SPRITES - 1),
		};
		balls.push(ball);
	}

	while !system.do_events()? {
		if system.res.keyboard.is_key_pressed(Scancode::Escape) {
			break;
		}

		if system.res.keyboard.is_key_up(Scancode::S) {
			for i in 0..NUM_BALLS {
				let ball = &mut balls[i];
				ball.x += ball.dir_x;
				ball.y += ball.dir_y;

				if ball.dir_x < 0 {
					if ball.x <= 0 {
						ball.dir_x = -ball.dir_x;
						ball.x = 0;
					}
				} else {
					if ball.x >= (system.res.video.width() - BALL_WIDTH) as i32 {
						ball.dir_x = -ball.dir_x;
						ball.x = (system.res.video.width() - BALL_WIDTH) as i32;
					}
				}

				if ball.dir_y < 0 {
					if ball.y <= 0 {
						ball.dir_y = -ball.dir_y;
						ball.y = 0;
					}
				} else {
					if ball.y >= (system.res.video.height() - BALL_HEIGHT) as i32 {
						ball.dir_y = -ball.dir_y;
						ball.y = (system.res.video.height() - BALL_HEIGHT) as i32;
					}
				}
			}
		}

		system.update()?;
		system.res.video.clear(2);

		system.res.video.print_string("hello, world!", 10, 10, FontRenderOpts::Color(15), &font);

		for i in 0..NUM_BALLS {
			system.res.video.blit(IndexedBlitMethod::Transparent(0), &sprites[balls[i].sprite], balls[i].x, balls[i].y);
		}

		system.display()?;
	}

	Ok(())
}
