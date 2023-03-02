use std::collections::VecDeque;
use std::fmt::Formatter;

/// An event listener/handler function that returns true if it handled the event and no other
/// listeners/handlers should be called next with the same event, or false if the event was not
/// handled and any subsequent listeners/handlers should be called.
pub type ListenerFn<EventType, ContextType> = fn(event: &EventType, &mut ContextType) -> bool;

/// An event publisher that code can use to queue up events to be handled by an [`EventListeners`]
/// instance. The `EventType` here should usually be an application-specific "events" enum.
#[derive(Clone)]
pub struct EventPublisher<EventType> {
	queue: VecDeque<EventType>,
}

impl<EventType> std::fmt::Debug for EventPublisher<EventType> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("EventPublisher")
			.field("queue.len()", &self.queue.len())
			.finish_non_exhaustive()
	}
}

impl<EventType> EventPublisher<EventType> {
	pub fn new() -> Self {
		EventPublisher {
			queue: VecDeque::new(),
		}
	}

	/// Returns the number of events that have been queued.
	#[inline]
	pub fn len(&self) -> usize {
		self.queue.len()
	}

	/// Clears the current event queue. The events will not be processed/handled.
	#[inline]
	pub fn clear(&mut self) {
		self.queue.clear();
	}

	/// Pushes the given event to the back of the queue.
	#[inline]
	pub fn queue(&mut self, event: EventType) {
		self.queue.push_back(event);
	}

	fn take_queue(&mut self, destination: &mut VecDeque<EventType>) {
		destination.clear();
		destination.append(&mut self.queue);
		self.clear();
	}
}

/// A manager for application event listeners/handlers that can dispatch events queued up by a
/// [`EventPublisher`] to each of the event listeners/handlers registered with this manager.
///
/// The `EventType` specified here should usually be an application-specific "events" enum and
/// should be the same as the type used in your application's [`EventPublisher`].
///
/// The `ContextType` specified here should be some application-specific context type that you
/// want available in all of your event listener/handler functions.
#[derive(Clone)]
pub struct EventListeners<EventType, ContextType> {
	listeners: Vec<ListenerFn<EventType, ContextType>>,
	dispatch_queue: VecDeque<EventType>,
}

impl<EventType, ContextType> std::fmt::Debug for EventListeners<EventType, ContextType> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("EventListeners")
			.field("listeners.len()", &self.listeners.len())
			.field("dispatch_queue.len()", &self.dispatch_queue.len())
			.finish_non_exhaustive()
	}
}

impl<EventType, ContextType> EventListeners<EventType, ContextType> {
	pub fn new() -> Self {
		EventListeners {
			listeners: Vec::new(),
			dispatch_queue: VecDeque::new(),
		}
	}

	/// Returns the number of event listeners/handlers registered with this manager.
	#[inline]
	pub fn len(&self) -> usize {
		self.listeners.len()
	}

	/// Unregisters all event listeners/managers previously registered with this manager.
	#[inline]
	pub fn clear(&mut self) {
		self.listeners.clear();
	}

	/// Adds/Registers the given event listener/handler function with this manager so that
	/// it will be called during dispatching of events. Returns true if the function was added.
	pub fn add(&mut self, listener: ListenerFn<EventType, ContextType>) -> bool {
		// HACK?: most advice i've seen right now for comparing function pointers suggests doing
		//        this, but i've also come across comments suggesting there are times where this
		//        might not be foolproof? (e.g. where generics or lifetimes come into play ... )
		if self.listeners.iter().any(|&l| l as usize == listener as usize) {
			false // don't add a duplicate listener
		} else {
			self.listeners.push(listener);
			true
		}
	}

	/// Removes/Unregisters the specified event listener/handler function from this manager.
	pub fn remove(&mut self, listener: ListenerFn<EventType, ContextType>) -> bool {
		let before_size = self.listeners.len();
		// HACK?: comparing function pointers -- see above "HACK?" comment. same concern here.
		self.listeners.retain(|&l| l as usize != listener as usize);
		// return true if the listener was removed
		return before_size != self.listeners.len();
	}

	/// Moves the queue from the given [`EventPublisher`] to this manager in preparation for
	/// dispatching the queued events via [`EventListeners::dispatch_queue`]. After calling this,
	/// the [`EventPublisher`]'s queue will be empty.
	pub fn take_queue_from(&mut self, publisher: &mut EventPublisher<EventType>) -> usize {
		publisher.take_queue(&mut self.dispatch_queue);
		self.dispatch_queue.len()
	}

	/// Dispatches the previous obtained event queue (via a call to
	/// [`EventListeners::take_queue_from`]) to all of the registered event listeners/handlers,
	/// passing each of them the given context argument. Not all of the event listeners/handlers
	/// will necessarily be called for each event being dispatched depending on which ones handled
	/// which events.
	pub fn dispatch_queue(&mut self, context: &mut ContextType) {
		while let Some(event) = self.dispatch_queue.pop_front() {
			for listener in &self.listeners {
				if listener(&event, context) {
					break;
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug, Eq, PartialEq, Copy, Clone)]
	enum TestEvent {
		Dummy,
		Foobar(i32),
		Message(&'static str),
	}

	struct DummyContext;

	struct TestContext {
		pub count: i32,
		pub events: Vec<TestEvent>,
	}

	impl TestContext {
		pub fn new() -> Self {
			TestContext { count: 0, events: Vec::new() }
		}
	}

	fn dummy_listener(_event: &TestEvent, _context: &mut DummyContext) -> bool {
		false
	}

	fn other_dummy_listener(_event: &TestEvent, _context: &mut DummyContext) -> bool {
		false
	}

	fn event_logger(event: &TestEvent, context: &mut TestContext) -> bool {
		context.events.push(*event);
		false
	}

	fn event_counter(_event: &TestEvent, context: &mut TestContext) -> bool {
		context.count += 1;
		false
	}

	fn message_filter(event: &TestEvent, _context: &mut TestContext) -> bool {
		match event {
			TestEvent::Message(s) => {
				if *s == "filter" {
					true // means event was handled, and no subsequent listeners should be called
				} else {
					false
				}
			}
			_ => false
		}
	}

	#[test]
	pub fn adding_and_removing_listeners() {
		let mut listeners = EventListeners::<TestEvent, DummyContext>::new();

		// add and remove
		assert_eq!(0, listeners.len());
		assert!(listeners.add(dummy_listener));
		assert_eq!(1, listeners.len());
		assert!(!listeners.add(dummy_listener));
		assert_eq!(1, listeners.len());
		assert!(listeners.remove(dummy_listener));
		assert_eq!(0, listeners.len());
		assert!(!listeners.remove(dummy_listener));
		assert_eq!(0, listeners.len());

		// add and remove multiple
		assert!(listeners.add(dummy_listener));
		assert_eq!(1, listeners.len());
		assert!(listeners.add(other_dummy_listener));
		assert_eq!(2, listeners.len());
		assert!(listeners.remove(dummy_listener));
		assert_eq!(1, listeners.len());
		assert!(!listeners.remove(dummy_listener));
		assert_eq!(1, listeners.len());
		assert!(listeners.remove(other_dummy_listener));
		assert_eq!(0, listeners.len());

		// clear all
		assert!(listeners.add(dummy_listener));
		assert!(listeners.add(other_dummy_listener));
		assert_eq!(2, listeners.len());
		listeners.clear();
		assert_eq!(0, listeners.len());
	}

	#[test]
	pub fn queueing_events() {
		use TestEvent::*;

		let mut publisher = EventPublisher::<TestEvent>::new();
		assert_eq!(0, publisher.len());
		publisher.queue(Dummy);
		assert_eq!(1, publisher.len());
		publisher.queue(Foobar(1));
		assert_eq!(2, publisher.len());
		publisher.queue(Foobar(2));
		assert_eq!(3, publisher.len());

		let mut queue = VecDeque::<TestEvent>::new();
		publisher.take_queue(&mut queue);
		assert_eq!(0, publisher.len());
		assert_eq!(Dummy, queue.pop_front().unwrap());
		assert_eq!(Foobar(1), queue.pop_front().unwrap());
		assert_eq!(Foobar(2), queue.pop_front().unwrap());
		assert!(queue.pop_front().is_none());

		publisher.queue(Dummy);
		assert_eq!(1, publisher.len());
		publisher.clear();
		assert_eq!(0, publisher.len());
		let mut queue = VecDeque::<TestEvent>::new();
		publisher.take_queue(&mut queue);
		assert_eq!(0, publisher.len());
		assert_eq!(0, queue.len());
	}

	#[test]
	pub fn listeners_receive_events() {
		use TestEvent::*;

		let mut listeners = EventListeners::<TestEvent, TestContext>::new();
		assert!(listeners.add(event_logger));

		let mut publisher = EventPublisher::<TestEvent>::new();
		publisher.queue(Dummy);
		publisher.queue(Foobar(1));
		publisher.queue(Dummy);
		publisher.queue(Foobar(42));
		assert_eq!(4, listeners.take_queue_from(&mut publisher));

		let mut context = TestContext::new();
		assert!(context.events.is_empty());
		assert_eq!(0, context.count);
		listeners.dispatch_queue(&mut context);
		assert!(!context.events.is_empty());
		assert_eq!(0, context.count);
		assert_eq!(
			vec![Dummy, Foobar(1), Dummy, Foobar(42)],
			context.events
		);

		let mut context = TestContext::new();
		assert!(context.events.is_empty());
		assert_eq!(0, context.count);
		listeners.dispatch_queue(&mut context);
		assert!(context.events.is_empty());

		assert!(listeners.add(event_counter));
		publisher.queue(Foobar(10));
		publisher.queue(Foobar(20));
		publisher.queue(Dummy);
		listeners.take_queue_from(&mut publisher);
		let mut context = TestContext::new();
		listeners.dispatch_queue(&mut context);
		assert!(!context.events.is_empty());
		assert_eq!(3, context.count);
		assert_eq!(
			vec![Foobar(10), Foobar(20), Dummy],
			context.events
		);
	}

	#[test]
	pub fn listener_filtering() {
		use TestEvent::*;

		let mut listeners = EventListeners::<TestEvent, TestContext>::new();
		assert!(listeners.add(message_filter));
		assert!(listeners.add(event_logger));
		assert!(listeners.add(event_counter));

		let mut publisher = EventPublisher::<TestEvent>::new();
		publisher.queue(Message("hello"));
		publisher.queue(Dummy);
		publisher.queue(Message("filter"));
		publisher.queue(Foobar(3));
		assert_eq!(4, listeners.take_queue_from(&mut publisher));

		let mut context = TestContext::new();
		assert!(context.events.is_empty());
		assert_eq!(0, context.count);
		listeners.dispatch_queue(&mut context);
		assert!(!context.events.is_empty());
		assert_eq!(3, context.count);
		assert_eq!(
			vec![Message("hello"), Dummy, Foobar(3)],
			context.events
		);
	}
}
