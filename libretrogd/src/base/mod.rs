//! Optional, extra types and helpers that can be used to get game's main loop boilerplate up and
//! running quicker.
//!
//! This is all of somewhat dubious quality and value at the moment. And it may continue to be this
//! way for a long while yet. And, truthfully, I suspect I may rip this out eventually. Maybe.
//!
//! The very-long-winded rationale here is that as I've started building more and more things with
//! libretrogd, I started implementing games/apps using a particular pattern which I was largely
//! pushed towards due to the Rust borrow-checker (as is often the case with Rust). My games/apps
//! needed to keep their state (for clarity, the word 'state' here is being used very broadly to
//! refer to all game/app state, and not just referring to the stuff inside `libretrogd::states`)
//! somewhere and my needs were a bit complicated since my game/app state often included things
//! which needed to get passed other things from inside that same "bag" of state.
//!
//! I originally wanted to do something like this, where this `App` struct is our overall "game/app
//! context" grab bag:
//!
//! ```
//! pub enum Event { /* .. various events here .. */ }
//! struct App {
//! 	pub delta: f32,
//! 	pub system: libretrogd::system::System,
//! 	pub entities: libretrogd::entities::Entities,
//! 	pub component_systems: libretrogd::entities::ComponentSystems<App, App>,  // oh no! :'(
//! 	pub event_publisher: libretrogd::events::EventPublisher<Event>,
//! 	pub event_listeners: libretrogd::events::EventListeners<Event, App>,  // oh no again! :'(
//! }
//! ```
//!
//! Of course, we cannot do this, because then we end up trying to get additional mutable borrows
//! of `App` when we eventually try to call certain methods on either the `component_systems` or
//! `event_listeners` instances. Boooo! :-(
//!
//! That of course lead me to split this structure up. I didn't and still don't like this because
//! I really don't know what to call these two things. They're both "context" and they're literally
//! only split up because of borrow-checker issues. But splitting them up did work for me. I
//! initially went with a parent-child split, which seemed logical to me at the time:
//!
//! ```
//! pub enum Event { /* .. various events here .. */ }
//!
//! // "core" because what the heck else do i call this? "InnerContext"? "InnerApp"? ...
//! struct Core {
//! 	pub delta: f32,
//! 	pub system: libretrogd::system::System,
//! 	pub entities: libretrogd::entities::Entities,
//! 	pub event_publisher: libretrogd::events::EventPublisher<Event>,
//! }
//!
//! // i guess this is a bit more obvious what to call it, but still ... doesn't sit right with me
//! struct App {
//! 	pub core: Core,
//! 	pub component_systems: libretrogd::entities::ComponentSystems<Core, Core>,
//! 	pub event_listeners: libretrogd::events::EventListeners<Event, Core>,
//! }
//! ```
//!
//! This structure seemed to work generally well and I've gotten pretty far with it. Keeping the
//! main `libretrogd::states::States` instance _separate_ was also key, and never really a problem
//! since that can (and should) just live at the top in your main loop. Easy.
//!
//! I ended up with some common bits of code that I'd always add to projects using this structure,
//! such as a very simple copy+pasted main loop, as well as a very simple function that calculates
//! the new frame `delta` each iteration of the main loop. As well as event processing via the
//! `event_publisher` and `event_listener` instances. I also expect this set of common bits of code
//! to grow over time. And I, ideally, want a single place to put it all.
//!
//! So, this module here is my attempt at trying to formalize this a bit more and do a bit of
//! refactoring where I can keep this common copy+pasted bits somewhere. As well, I decided to
//! move away from my "context" struct having a parent-child relation for the split of the data
//! kept in these, and instead just "flatten" it out a bit (sort of) as this seems much more
//! future-proof if/when I encounter more borrow-checker issues down the road with other additions
//! to these structures.
//!
//! But again, better naming still eludes me here!
//!
//! ```
//! pub enum Event { /* .. various events here .. */ }
//!
//! // "Core" because it contains the things that probably 90% of game/app code will need to work
//! // with. you'd probably want to put your game/app resources/assets on this struct too.
//! struct Core {
//! 	pub delta: f32,
//! 	pub system: libretrogd::system::System,
//! 	pub entities: libretrogd::entities::Entities,
//! 	pub event_publisher: libretrogd::events::EventPublisher<Event>,
//! }
//!
//! // "Support" because it contains things that support the main/core game state?
//! // kinda grasping at straws here maybe ...
//! struct Support {
//! 	pub component_systems: libretrogd::entities::ComponentSystems<Core, Core>,
//! 	pub event_listeners: libretrogd::events::EventListeners<Event, Core>,
//! }
//!
//! // better, maybe?
//! struct App {
//! 	pub core: Core,
//! 	pub support: Support,
//! }
//! ```
//!
//! Even though it's another struct being added, I do like this more, despite the naming
//! uncertainty.
//!
//! So, with this being my current preferred way to architect a libretrogd-using project, I created
//! some traits here in this module to formalize this all a bit more. `CoreState` and (optionally)
//! `CoreStateWithEvents` are what you'd make your project's `Core` struct (as shown in the above
//! example code) implement, while `SupportSystems` and (optionally) `SupportSystemsWithEvents`
//! are what you'd make your project's `Support` struct (again, as shown in the above example code)
//! implement. Finally, `AppContext` is for your `App` struct that contains the two.
//!
//! Once you have all this (which ironically ends up being _more_ code than if you'd not used these
//! traits ... heh), you can now optionally use the `main_loop` function to get a ready-to-use
//! main loop which is set up to use a `libretrogd::states::State` state manager.
//!
//! Having said all of this ... again, I will reiterate that I don't believe any of this has reached
//! anything resembling a "good design" ... yet. There may be a good design hidden somewhere in
//! here that I've yet to fully discover, but I definitely don't think I've arrived at quite it yet.
//!
//! So, basically, I expect this to evolve over time (probably a _long_ time). And this is all
//! totally optional anyway.
//!

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

	let mut last_ticks = app.core().system().ticks();

	while !app.core().system_mut().do_events() && !states.is_empty() {
		last_ticks = app.core().update_frame_delta(last_ticks);
		states.update(&mut app)?;
		states.render(&mut app);
		app.core().system_mut().display()?;
	}

	Ok(())
}
