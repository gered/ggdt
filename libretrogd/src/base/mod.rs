//! Optional, extra types and helpers that can be used to get game's main loop boilerplate up and
//! running quicker.

use thiserror::Error;

use crate::events::*;
use crate::states::*;
use crate::system::*;

pub trait PrimaryState {
	fn system(&self) -> &System;
	fn system_mut(&mut self) -> &mut System;
}

pub trait PrimaryStateWithFrameTiming: PrimaryState {
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

pub trait PrimaryStateWithEvents<EventType>: PrimaryState {
	fn event_publisher(&mut self) -> &mut EventPublisher<EventType>;
}

pub trait SupportSystems {}

pub trait SupportSystemsWithEvents<EventType, ContextType>: SupportSystems
where
	ContextType: PrimaryStateWithEvents<EventType>,
{
	fn event_listeners(&mut self) -> &mut EventListeners<EventType, ContextType>;

	fn do_events(&mut self, context: &mut ContextType) {
		self.event_listeners().take_queue_from(context.event_publisher());
		self.event_listeners().dispatch_queue(context);
	}
}

pub struct App<PrimaryType, SupportType> {
	pub primary_state: PrimaryType,
	pub support_systems: SupportType,
}

impl<PrimaryType, SupportType> App<PrimaryType, SupportType> {
	pub fn new(primary_state: PrimaryType, support_systems: SupportType) -> Self {
		App { primary_state, support_systems }
	}
}

#[derive(Error, Debug)]
pub enum MainLoopError {
	#[error("States error: {0}")]
	StateError(#[from] StateError),

	#[error("System error: {0}")]
	SystemError(#[from] SystemError),
}

pub fn main_loop<PrimaryType, SupportType, State>(
	primary_state: PrimaryType,
	support_systems: SupportType,
	initial_state: State,
) -> Result<(), MainLoopError>
where
	PrimaryType: PrimaryStateWithFrameTiming,
	SupportType: SupportSystems,
	State: AppState<App<PrimaryType, SupportType>> + 'static,
{
	let mut app = App::new(primary_state, support_systems);
	let mut states = States::new();
	states.push(initial_state)?;

	let mut is_running = true;
	let mut last_ticks = app.primary_state.system().ticks();

	while is_running && !states.is_empty() {
		app.primary_state.system_mut().do_events_with(|event| match event {
			SystemEvent::Quit => {
				is_running = false;
			}
			_ => {}
		});

		last_ticks = app.primary_state.update_frame_delta(last_ticks);
		states.update(&mut app)?;
		states.render(&mut app);
		app.primary_state.system_mut().display()?;
	}

	Ok(())
}
