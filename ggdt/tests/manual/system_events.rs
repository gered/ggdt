use crate::BACKGROUND_COLOR;
use ggdt::prelude::*;
use std::collections::VecDeque;

#[test]
#[ignore] // should be manually run
fn system_events_display() {
	let config = StandardConfig::variable_screen_size(640, 480).scale_factor(2);
	let mut system = SystemBuilder::new()
		.window_title("Displaying all SystemEvents")
		.vsync(true)
		.show_mouse(true)
		.build(config)
		.unwrap();

	let mut recent_events = VecDeque::new();

	'mainloop: loop {
		while recent_events.len() > (system.res.video.height() as usize / system.res.font.line_height() as usize) {
			recent_events.pop_back();
		}
		system.res.update_event_state().unwrap();
		for event in system.event_pump.poll_iter() {
			system.res.handle_event(&event).unwrap();
			recent_events.push_front(event.clone());
			if event == SystemEvent::Quit {
				break 'mainloop;
			}
		}
		system.update().unwrap();

		if system.res.keyboard.is_key_pressed(Scancode::Escape) {
			break 'mainloop;
		}

		system.res.video.clear(BACKGROUND_COLOR);

		for (idx, event) in recent_events.iter().enumerate() {
			system.res.video.print_string(
				&format!("{:?}", event),
				2,
				system.res.video.height() as i32 - 10 - (idx as i32 * 10),
				FontRenderOpts::Color(to_rgb32([255, 255, 255])),
				&system.res.font,
			);
		}

		system.display().unwrap();
	}
}
