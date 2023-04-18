use crate::BACKGROUND_COLOR;
use ggdt::prelude::*;

fn display_mouse_state(system: &mut System<Standard>) {
	let font = &system.res.font;
	let font_opts = FontRenderOpts::Color(0xffffffff);

	system.res.video.print_string(
		&format!(
			"X: {:4} (delta: {:4})\nY: {:4} (delta: {:4})",
			system.res.mouse.x(),
			system.res.mouse.x_delta(),
			system.res.mouse.y(),
			system.res.mouse.y_delta(),
		),
		10,
		10,
		font_opts,
		font,
	);

	system.res.video.print_string("Buttons - down/up/pressed/released", 10, 40, font_opts, font);
	use MouseButton::*;
	for (idx, button) in [Left, Middle, Right, X1, X2].iter().enumerate() {
		system.res.video.print_string(
			&format!(
				"{:?} - {}  {}  {}  {}",
				button,
				system.res.mouse.is_button_down(*button) as i32,
				system.res.mouse.is_button_up(*button) as i32,
				system.res.mouse.is_button_pressed(*button) as i32,
				system.res.mouse.is_button_released(*button) as i32,
			),
			10,
			50 + (idx as i32 * 8),
			font_opts,
			font,
		)
	}
}

#[test]
#[ignore] // should be manually run
fn mouse_with_custom_cursor() {
	let config = StandardConfig::variable_screen_size(320, 240).scale_factor(4);
	let mut system = SystemBuilder::new()
		.window_title("Mouse Input, using a Custom Bitmap Cursor")
		.vsync(true)
		.show_mouse(false)
		.build(config)
		.unwrap();

	system.res.cursor.enable_cursor(true);

	while !system.do_events().unwrap() {
		if system.res.keyboard.is_key_pressed(Scancode::Escape) {
			break;
		}

		system.res.video.clear(BACKGROUND_COLOR);

		system.update().unwrap();

		display_mouse_state(&mut system);
		system.res.video.set_pixel(system.res.mouse.x(), system.res.mouse.y(), to_rgb32([255, 0, 255]));

		system.display().unwrap();
	}
}

#[test]
#[ignore] // should be manually run
fn mouse_with_os_cursor() {
	let config = StandardConfig::variable_screen_size(320, 240).scale_factor(4);
	let mut system = SystemBuilder::new()
		.window_title("Mouse Input, using the OS's Native Cursor")
		.vsync(true)
		.show_mouse(true)
		.build(config)
		.unwrap();

	system.res.cursor.enable_cursor(false);

	while !system.do_events().unwrap() {
		if system.res.keyboard.is_key_pressed(Scancode::Escape) {
			break;
		}

		system.res.video.clear(BACKGROUND_COLOR);

		system.update().unwrap();

		display_mouse_state(&mut system);
		system.res.video.set_pixel(system.res.mouse.x(), system.res.mouse.y(), to_rgb32([255, 0, 255]));

		system.display().unwrap();
	}
}
