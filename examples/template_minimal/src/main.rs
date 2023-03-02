use anyhow::Result;

use libretrogd::{SCREEN_BOTTOM, SCREEN_RIGHT};
use libretrogd::graphics::*;
use libretrogd::system::*;
use libretrogd::utils::rnd_value;

fn main() -> Result<()> {
	let mut system = SystemBuilder::new().window_title("Minimal Template").vsync(true).build()?;

	let font = BitmaskFont::new_vga_font()?;

	system.video.clear(0);
	system.video.print_string("Hello, world!", 20, 20, FontRenderOpts::Color(15), &font);

	while !system.do_events() {
		if system.input_devices.keyboard.is_key_pressed(Scancode::Escape) {
			break;
		}

		let x = rnd_value(0, SCREEN_RIGHT) as i32;
		let y = rnd_value(0, SCREEN_BOTTOM) as i32;
		let color = rnd_value(0, 255);
		system.video.set_pixel(x, y, color);

		system.display()?;
	}

	Ok(())
}
