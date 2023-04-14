use crate::BACKGROUND_COLOR;
use ggdt::prelude::*;

fn display_raw_keyboard_state(system: &mut System<Standard>, x: i32, y: i32) {
	let font_opts = FontRenderOpts::Color(0xffffffff);
	let font = &system.res.font;

	system.res.video.print_string("Raw Keyboard State", x, x, font_opts, font);

	let mut idx = 0;
	for yc in 0..16 {
		for xc in 0..32 {
			let state = if let Some(scancode) = num::FromPrimitive::from_u32(idx) {
				system.res.keyboard.is_key_down(scancode)
			} else {
				false
			};
			system.res.video.print_char(
				if state { '1' } else { '0' },
				x + (xc * 8),
				y + 10 + (yc * 8),
				font_opts,
				font,
			);

			idx += 1;
		}
	}
}

fn display_key_state(key: Scancode, system: &mut System<Standard>, x: i32, y: i32) {
	system.res.video.print_string(
		&format!(
			"{:?} - {}  {}  {}  {}",
			key,
			system.res.keyboard.is_key_down(key) as i32,
			system.res.keyboard.is_key_up(key) as i32,
			system.res.keyboard.is_key_pressed(key) as i32,
			system.res.keyboard.is_key_released(key) as i32,
		),
		x,
		y,
		FontRenderOpts::Color(0xffffffff),
		&system.res.font,
	);
}

#[test]
#[ignore]
fn keyboard_state() {
	let config = StandardConfig::variable_screen_size(640, 480).scale_factor(2);
	let mut system = SystemBuilder::new()
		.window_title("Keyboard State") //
		.vsync(true)
		.build(config)
		.unwrap();

	while !system.do_events().unwrap() {
		if system.res.keyboard.is_key_pressed(Scancode::Escape) {
			break;
		}

		system.res.video.clear(BACKGROUND_COLOR);

		system.update().unwrap();

		display_raw_keyboard_state(&mut system, 2, 2);

		for (idx, key) in [
			Scancode::A,
			Scancode::LCtrl,
			Scancode::RCtrl,
			Scancode::LAlt,
			Scancode::RAlt,
			Scancode::LShift,
			Scancode::RShift,
			Scancode::Return,
			Scancode::KpEnter,
			Scancode::Up,
			Scancode::Down,
			Scancode::Left,
			Scancode::Right,
		]
		.iter()
		.enumerate()
		{
			display_key_state(*key, &mut system, 2, 160 + (idx as i32 * 10));
		}

		system.res.video.set_pixel(system.res.mouse.x(), system.res.mouse.y(), to_rgb32(255, 0, 255));

		system.display().unwrap();
	}
}
