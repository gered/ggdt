use anyhow::Result;

use ggdt::prelude::*;
use ggdt_imgui::UiSupport;

use crate::context::GameContext;
use crate::entities::{Forces, Position, Slime};
use crate::tilemap::{TILE_HEIGHT, TILE_WIDTH};

mod context;
mod entities;
mod support;
mod tilemap;

#[derive(Default)]
pub struct DemoState {
	new_x: i32,
	new_y: i32,
	selected_entity: EntityId,
}

impl AppState<GameContext> for DemoState {
	fn update(&mut self, _state: State, context: &mut GameContext) -> Option<StateChange<GameContext>> {
		if context.core.system.res.keyboard.is_key_pressed(Scancode::Escape) {
			return Some(StateChange::Pop(1));
		}

		let ui = context.support.imgui.new_frame(&context.core.system.res.video);
		ui.window("Entities")
			.position([10.0, 10.0], imgui::Condition::FirstUseEver)
			.size([240.0, 200.0], imgui::Condition::FirstUseEver)
			.build(|| {
				ui.text(format!("Camera: {}, {}", context.core.camera_x, context.core.camera_y));

				ui.separator();
				ui.text_colored([1.0, 1.0, 0.0, 1.0], "Slimes");
				let mut positions = context.core.entities.components_mut::<Position>().unwrap();
				let mut slime_types = context.core.entities.components::<Slime>();
				for (slime, _) in context.core.entities.components::<Slime>().unwrap().iter() {
					let position = positions.get(slime).unwrap();
					let slime_type = slime_types.get(slime).unwrap();
					ui.text(format!("{:2} @ {:3.0},{:3.0}", *slime, position.0.x, position.0.y));

					if let Some(slime_type_texture_id) = context.core.slime_texture_id_map.get(&slime_type.0) {
						ui.same_line();
						ui.image("Slime Type", *slime_type_texture_id, [16.0, 16.0]);
					}

					ui.same_line();
					let clicked = {
						let _id = ui.push_id_ptr(slime);
						ui.button("Move")
					};
					if clicked {
						self.new_x = 0;
						self.new_y = 0;
						self.selected_entity = *slime;
						ui.open_popup("Move Entity");
					}

					ui.same_line();
					let clicked = {
						let _id = ui.push_id_ptr(slime);
						ui.button("Push")
					};
					if clicked {
						let mut forces = context.core.entities.components_mut::<Forces>().unwrap();
						let slime_forces = forces.get_mut(slime).unwrap();
						slime_forces.add(Vector2::from_angle(rnd_value(RADIANS_0, RADIANS_360)) * 5.0, 0.8);
					}
				}

				if let Some(_token) = ui.modal_popup_config("Move Entity").always_auto_resize(true).begin_popup() {
					ui.text(format!("Move entity {} to:", self.selected_entity));
					ui.input_int("X", &mut self.new_x).step(16).build();
					ui.input_int("Y", &mut self.new_y).step(16).build();
					if ui.button("Move") {
						let position = positions.get_mut(&self.selected_entity).unwrap();
						position.0.x = self.new_x as f32;
						position.0.y = self.new_y as f32;
						ui.close_current_popup();
					}
					ui.same_line();
					if ui.button("Cancel") {
						ui.close_current_popup();
					}
				}
			});

		if !ui.is_any_hovered() && !ui.is_any_focused() {
			if context.core.system.res.mouse.is_button_down(MouseButton::Right) {
				context.core.camera_x -= context.core.system.res.mouse.x_delta() * 2;
				context.core.camera_y -= context.core.system.res.mouse.y_delta() * 2;
			}
		}

		context.support.do_events(&mut context.core);
		context.support.component_systems.update(&mut context.core);

		None
	}

	fn render(&mut self, _state: State, context: &mut GameContext) {
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

	fn transition(&mut self, _state: State, _context: &mut GameContext) -> bool {
		true
	}

	fn state_change(&mut self, new_state: State, _old_state: State, context: &mut GameContext) {
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
	states.push(DemoState::default())?;

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
		states.update(&mut game)?;
		game.core.system.update()?;
		states.render(&mut game);
		game.core.system.display()?;
	}

	Ok(())
}
