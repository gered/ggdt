mod context;
mod entities;
mod support;
mod tilemap;

use anyhow::{Context, Result};

use crate::context::GameContext;
use crate::tilemap::{TILE_HEIGHT, TILE_WIDTH};
use ggdt::prelude::*;

pub struct DemoState;

impl AppState<GameContext> for DemoState {
	fn update(&mut self, state: State, context: &mut GameContext) -> Option<StateChange<GameContext>> {
		if context.core.system.res.keyboard.is_key_pressed(Scancode::Escape) {
			return Some(StateChange::Pop(1));
		}

		let ui = context.support.imgui.new_frame(&context.core.system.res.video);
		ui.window("Entities").build(|| {
			ui.text("TODO: display entity list or something");
		});

		let ui_focused = ui.is_window_hovered_with_flags(imgui::WindowHoveredFlags::ANY_WINDOW)
			|| ui.is_window_focused_with_flags(imgui::WindowFocusedFlags::ANY_WINDOW);

		if !ui_focused {
			if context.core.system.res.mouse.is_button_down(1) {
				context.core.camera_x -= context.core.system.res.mouse.x_delta() * 2;
				context.core.camera_y -= context.core.system.res.mouse.y_delta() * 2;
			}
		}

		context.support.do_events(&mut context.core);
		context.support.component_systems.update(&mut context.core);

		None
	}

	fn render(&mut self, state: State, context: &mut GameContext) {
		context.core.system.res.video.clear(context.core.palette[0]);
		context.core.tilemap.draw(
			&mut context.core.system.res.video,
			&context.core.tiles,
			context.core.camera_x,
			context.core.camera_y,
			context.core.transparent_color,
		);
		context.support.component_systems.render(&mut context.core);

		context.support.imgui.render(&mut context.core.system.res.video);
	}

	fn transition(&mut self, state: State, context: &mut GameContext) -> bool {
		true
	}

	fn state_change(&mut self, new_state: State, old_state: State, context: &mut GameContext) {
		if new_state == State::Pending {
			entities::init(context);
			for _ in 0..10 {
				let (x, y) = context.core.tilemap.get_random_spawnable_coordinates();
				entities::new_slime_entity(
					&mut context.core,
					x * TILE_WIDTH as i32,
					y * TILE_HEIGHT as i32,
					entities::Direction::new_random(),
					entities::SlimeColor::new_random(),
				);
			}
		}
	}
}

fn main() -> Result<()> {
	let config = StandardConfig::variable_screen_size(640, 480).scale_factor(2);
	let mut system = SystemBuilder::new() //
		.window_title("ImGui Example Integration")
		.vsync(true)
		.build(config)?;
	system.res.cursor.enable_cursor(true);
	let mut game = GameContext::new(system)?;

	let mut states = States::new();
	states.push(DemoState);

	let mut last_ticks = game.core.system.ticks();

	'mainloop: while !states.is_empty() {
		game.core.system.res.update_event_state()?;
		for event in game.core.system.event_pump.poll_iter() {
			game.core.system.res.handle_event(&event)?;
			game.support.imgui.handle_event(&event);
			if event == SystemEvent::Quit {
				break 'mainloop;
			}
		}

		last_ticks = game.core.update_frame_delta(last_ticks);
		states.update(&mut game);
		game.core.system.update()?;
		states.render(&mut game);
		game.core.system.display()?;
	}

	Ok(())
}
