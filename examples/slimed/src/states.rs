use std::path::Path;

use ggdt::base::*;
use ggdt::entities::*;
use ggdt::graphics::indexed::*;
use ggdt::states::*;
use ggdt::system::*;

use crate::entities::*;
use crate::Game;
use crate::support::*;

pub struct MainMenuState {
	fade: f32,
	selection: i32,
}

impl MainMenuState {
	pub fn new() -> Self {
		MainMenuState {
			fade: 0.0,
			selection: 0,
		}
	}
}

impl AppState<Game> for MainMenuState {
	fn update(&mut self, state: State, context: &mut Game) -> Option<StateChange<Game>> {
		if state == State::Active {
			if context.core.system.res.keyboard.is_key_pressed(Scancode::Escape) {
				return Some(StateChange::Pop(1));
			}
			if context.core.system.res.keyboard.is_key_pressed(Scancode::Up) {
				self.selection = (self.selection - 1).clamp(0, 1);
			}
			if context.core.system.res.keyboard.is_key_pressed(Scancode::Down) {
				self.selection = (self.selection + 1).clamp(0, 1);
			}

			if context.core.system.res.keyboard.is_key_pressed(Scancode::Return) {
				match self.selection {
					0 => return Some(StateChange::Push(Box::new(GamePlayState::new()))),
					1 => return Some(StateChange::Pop(1)),
					_ => {}
				}
			}
		}

		context.support.do_events(&mut context.core);
		context.support.component_systems.update(&mut context.core);

		None
	}

	fn render(&mut self, state: State, context: &mut Game) {
		context.core.tilemap.draw(&mut context.core.system.res.video, &context.core.tiles, 0, 0);
		context.support.component_systems.render(&mut context.core);

		let x = 32;
		let y = 160;
		let width = 48;
		let height = 40;
		const SPACER: i32 = 8;
		draw_window(&mut context.core.system.res.video, &context.core.ui, x, y, x + width, y + height);

		let selection_y = y + SPACER + (self.selection as i32 * 16);
		context.core.system.res.video.print_string(">", x + SPACER, selection_y, FontRenderOpts::Color(15), &context.core.font);

		context.core.system.res.video.print_string("Play", x + SPACER + SPACER, y + SPACER, FontRenderOpts::Color(15), &context.core.font);
		context.core.system.res.video.print_string("Quit", x + SPACER + SPACER, y + SPACER + 16, FontRenderOpts::Color(15), &context.core.font);
	}

	fn transition(&mut self, state: State, context: &mut Game) -> bool {
		update_fade_transition(state, &mut self.fade, context.core.delta * 3.0, context)
	}

	fn state_change(&mut self, new_state: State, old_state: State, context: &mut Game) {
		match new_state {
			State::Pending | State::Resume => {
				init_everything(context, Path::new("./assets/title_screen.map.json"), 0.2, 1.0, 32);
			}
			State::TransitionIn => {
				self.fade = 0.0;
			}
			State::TransitionOut(_) => {
				self.fade = 1.0;
			}
			State::Paused => {
				context.core.system.res.palette = context.core.palette.clone();
			}
			_ => {}
		}
	}
}

pub struct GamePlayState {
	fade: f32,
	in_menu: bool,
	selection: i32,
}

impl GamePlayState {
	pub fn new() -> Self {
		GamePlayState {
			fade: 0.0,
			in_menu: false,
			selection: 0,
		}
	}
}

impl AppState<Game> for GamePlayState {
	fn update(&mut self, state: State, context: &mut Game) -> Option<StateChange<Game>> {
		if state == State::Active {
			if self.in_menu {
				if context.core.system.res.keyboard.is_key_pressed(Scancode::Escape) {
					self.in_menu = false;
				}
				if context.core.system.res.keyboard.is_key_pressed(Scancode::Up) {
					self.selection = (self.selection - 1).clamp(0, 1);
				}
				if context.core.system.res.keyboard.is_key_pressed(Scancode::Down) {
					self.selection = (self.selection + 1).clamp(0, 1);
				}

				if context.core.system.res.keyboard.is_key_pressed(Scancode::Return) {
					match self.selection {
						0 => self.in_menu = false,
						1 => return Some(StateChange::Pop(1)),
						_ => {}
					}
				}
			} else {
				if context.core.system.res.keyboard.is_key_pressed(Scancode::Escape) {
					self.in_menu = true;
				}

				if let Some((player_entity, _)) = context.core.entities.components::<Player>().single() {
					if context.core.system.res.keyboard.is_key_down(Scancode::Up) {
						context.core.event_publisher.queue(Event::TurnAndMove(*player_entity, Direction::North));
					}
					if context.core.system.res.keyboard.is_key_down(Scancode::Down) {
						context.core.event_publisher.queue(Event::TurnAndMove(*player_entity, Direction::South));
					}
					if context.core.system.res.keyboard.is_key_down(Scancode::Left) {
						context.core.event_publisher.queue(Event::TurnAndMove(*player_entity, Direction::West));
					}
					if context.core.system.res.keyboard.is_key_down(Scancode::Right) {
						context.core.event_publisher.queue(Event::TurnAndMove(*player_entity, Direction::East));
					}
					if context.core.system.res.keyboard.is_key_pressed(Scancode::Space) {
						context.core.event_publisher.queue(Event::Attack(*player_entity));
					}
				}
			}
		}

		context.support.do_events(&mut context.core);
		context.support.component_systems.update(&mut context.core);

		None
	}

	fn render(&mut self, state: State, context: &mut Game) {
		if let Some((_, camera)) = context.core.entities.components::<Camera>().single() {
			context.core.tilemap.draw(&mut context.core.system.res.video, &context.core.tiles, camera.x, camera.y);
		}
		context.support.component_systems.render(&mut context.core);

		if self.in_menu {
			let x = 32;
			let y = 160;
			let width = 80;
			let height = 40;
			const SPACER: i32 = 8;
			draw_window(&mut context.core.system.res.video, &context.core.ui, x, y, x + width, y + height);

			let selection_y = y + SPACER + (self.selection as i32 * 16);
			context.core.system.res.video.print_string(">", x + SPACER, selection_y, FontRenderOpts::Color(15), &context.core.font);

			context.core.system.res.video.print_string("Continue", x + SPACER + SPACER, y + SPACER, FontRenderOpts::Color(15), &context.core.font);
			context.core.system.res.video.print_string("Quit", x + SPACER + SPACER, y + SPACER + 16, FontRenderOpts::Color(15), &context.core.font);
		}
	}

	fn transition(&mut self, state: State, context: &mut Game) -> bool {
		update_fade_transition(state, &mut self.fade, context.core.delta * 3.0, context)
	}

	fn state_change(&mut self, new_state: State, old_state: State, context: &mut Game) {
		match new_state {
			State::Pending => {
				init_everything(context, Path::new("./assets/arena.map.json"), 0.5, 2.0, 100);
				spawn_player_randomly(&mut context.core);
			}
			State::TransitionIn => {
				self.fade = 0.0;
			}
			State::TransitionOut(_) => {
				self.fade = 1.0;
			}
			_ => {}
		}
	}
}
