use anyhow::Result;

use ggdt::{SCREEN_BOTTOM, SCREEN_RIGHT};
use ggdt::graphics::indexed::*;
use ggdt::system::*;
use ggdt::utils::rnd_value;

fn main() -> Result<()> {
	let config = DosLikeConfig::new();
	let mut system = SystemBuilder::new()
		.window_title("Minimal Template")
		.vsync(true)
		.build(config)?;

	system.res.video.clear(0);
	system.res.video.print_string("Hello, world!", 20, 20, FontRenderOpts::Color(15), &system.res.font);

	while !system.do_events()? {
		if system.res.keyboard.is_key_pressed(Scancode::Escape) {
			break;
		}

		system.update()?;

		let x = rnd_value(0, SCREEN_RIGHT) as i32;
		let y = rnd_value(0, SCREEN_BOTTOM) as i32;
		let color = rnd_value(0, 255);
		system.res.video.set_pixel(x, y, color);

		system.display()?;
	}

	Ok(())
}
