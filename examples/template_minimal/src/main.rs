use anyhow::Result;
use sdl2::keyboard::Scancode;

use libretrogd::{SCREEN_BOTTOM, SCREEN_RIGHT};
use libretrogd::graphics::*;
use libretrogd::system::*;
use libretrogd::utils::rnd_value;

fn main() -> Result<()> {
    let mut system = SystemBuilder::new().window_title("Minimal Template").vsync(true).build()?;

    let mut is_running = true;
    let font = BitmaskFont::new_vga_font()?;

    system.video.clear(0);
    system.video.print_string("Hello, world!", 20, 20, 15, &font);

    while is_running {
        system.do_events_with(|event| {
            if let sdl2::event::Event::Quit { .. } = event {
                is_running = false;
            }
        });

        if system.keyboard.is_key_pressed(Scancode::Escape) {
            is_running = false;
        }

        let x = rnd_value(0, SCREEN_RIGHT) as i32;
        let y = rnd_value(0, SCREEN_BOTTOM) as i32;
        let color = rnd_value(0, 255);
        system.video.set_pixel(x, y, color);

        system.display()?;
    }

    Ok(())
}
