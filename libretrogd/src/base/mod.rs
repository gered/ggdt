//! Optional, extra types and helpers that can be used to get game's main loop boilerplate up and
//! running quicker.

use thiserror::Error;

use crate::events::*;
use crate::states::*;
use crate::system::*;

pub trait CoreState {
	fn system(&self) -> &System;
	fn system_mut(&mut self) -> &mut System;

	fn delta(&self) -> f32;
	fn set_delta(&mut self, delta: f32);

	fn update_frame_delta(&mut self, last_ticks: u64) -> u64 {
		let ticks = self.system().ticks();
		let tick_frequency = self.system().tick_frequency();
		let elapsed = ticks - last_ticks;
		self.set_delta((elapsed as f64 / tick_frequency as f64) as f32);
		ticks
	}
}

pub trait CoreStateWithEvents<EventType>: CoreState {
	fn event_publisher(&mut self) -> &mut EventPublisher<EventType>;
}

pub trait SupportSystems {}

pub trait SupportSystemsWithEvents<EventType>: SupportSystems
{
	type ContextType: CoreStateWithEvents<EventType>;
	fn event_listeners(&mut self) -> &mut EventListeners<EventType, Self::ContextType>;

	fn do_events(&mut self, context: &mut Self::ContextType) {
		self.event_listeners().take_queue_from(context.event_publisher());
		self.event_listeners().dispatch_queue(context);
	}
}

pub trait AppContext {
	type CoreType: CoreState;
	type SupportType: SupportSystems;

	fn core(&mut self) -> &mut Self::CoreType;
	fn support(&mut self) -> &mut Self::SupportType;
}

#[derive(Error, Debug)]
pub enum MainLoopError {
	#[error("States error: {0}")]
	StateError(#[from] StateError),

	#[error("System error: {0}")]
	SystemError(#[from] SystemError),
}

pub fn main_loop<ContextType, State>(
	mut app: ContextType,
	initial_state: State,
) -> Result<(), MainLoopError>
where
	ContextType: AppContext,
	State: AppState<ContextType> + 'static,
{
	let mut states = States::new();
	states.push(initial_state)?;

	let mut is_running = true;
	let mut last_ticks = app.core().system().ticks();

	while is_running && !states.is_empty() {
		app.core().system_mut().do_events_with(|event| match event {
			SystemEvent::Quit => {
				is_running = false;
			}
			_ => {}
		});

		last_ticks = app.core().update_frame_delta(last_ticks);
		states.update(&mut app)?;
		states.render(&mut app);
		app.core().system_mut().display()?;
	}

	Ok(())
}
