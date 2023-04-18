use crate::draw_base_screen;
use ggdt::prelude::*;

fn draw_state(system: &mut System<Standard>) {
	system.res.video.print_string(
		&format!(
			"{}x{} (Standard)\n\n{:3}, {:3}",
			system.res.video.width(),
			system.res.video.height(),
			system.res.mouse.x(),
			system.res.mouse.y()
		),
		10,
		10,
		FontRenderOpts::Color(COLOR_BRIGHT_WHITE),
		&system.res.font,
	);
}

fn simple_main_loop(mut system: System<Standard>) {
	while !system.do_events().unwrap() {
		if system.res.keyboard.is_key_pressed(Scancode::Escape) {
			break;
		}

		system.update().unwrap();

		draw_base_screen(
			&mut system.res.video,
			to_rgb32([32, 32, 32]),
			to_rgb32([44, 44, 44]),
			COLOR_BRIGHT_WHITE,
			COLOR_BRIGHT_RED,
		);
		draw_state(&mut system);

		system.display().unwrap();
	}
}

#[test]
#[ignore]
fn fixed_screen_size_integer_scaling() {
	let config = StandardConfig::fixed_screen_size(320, 240, true).scale_factor(3);
	let mut system = SystemBuilder::new()
		.window_title("Fixed Screen Size with Integer Scaling (Standard)")
		.vsync(true)
		.show_mouse(false)
		.build(config)
		.unwrap();
	system.res.cursor.enable_cursor(true);
	simple_main_loop(system);
}

#[test]
#[ignore]
fn fixed_screen_size_variable_scaling() {
	let config = StandardConfig::fixed_screen_size(320, 240, false).scale_factor(3);
	let mut system = SystemBuilder::new()
		.window_title("Fixed Screen Size with Variable Scaling (Standard)")
		.vsync(true)
		.show_mouse(false)
		.build(config)
		.unwrap();
	system.res.cursor.enable_cursor(true);
	simple_main_loop(system);
}

#[test]
#[ignore]
fn variable_screen_size_fixed_scale_factor() {
	let config = StandardConfig::variable_screen_size(320, 240).scale_factor(3);
	let mut system = SystemBuilder::new()
		.window_title("Variable Screen Size with Fixed Scale Factor (Standard)")
		.vsync(true)
		.show_mouse(false)
		.build(config)
		.unwrap();
	system.res.cursor.enable_cursor(true);
	simple_main_loop(system);
}
