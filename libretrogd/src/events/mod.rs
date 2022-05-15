use std::collections::VecDeque;

pub type ListenerFn<EventType, ContextType> = fn(event: &EventType, &mut ContextType) -> bool;

pub struct EventPublisher<EventType> {
    queue: VecDeque<EventType>,
}

impl<EventType> EventPublisher<EventType> {
    pub fn new() -> Self {
        EventPublisher {
            queue: VecDeque::new(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    #[inline]
    pub fn queue(&mut self, event: EventType) {
        self.queue.push_back(event);
    }

    pub fn take_queue(&mut self, destination: &mut VecDeque<EventType>) {
        destination.clear();
        destination.append(&mut self.queue);
        self.clear();
    }
}

pub struct EventListeners<EventType, ContextType> {
    listeners: Vec<ListenerFn<EventType, ContextType>>,
    dispatch_queue: VecDeque<EventType>,
}

impl<EventType, ContextType> EventListeners<EventType, ContextType> {
    pub fn new() -> Self {
        EventListeners {
            listeners: Vec::new(),
            dispatch_queue: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.listeners.len()
    }

    pub fn clear(&mut self) {
        self.listeners.clear();
    }

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

    pub fn remove(&mut self, listener: ListenerFn<EventType, ContextType>) -> bool {
        let before_size = self.listeners.len();
        // HACK?: comparing function pointers -- see above "HACK?" comment. same concern here.
        self.listeners.retain(|&l| l as usize != listener as usize);
        // return true if the listener was removed
        return before_size != self.listeners.len()
    }

    pub fn take_queue_from(&mut self, publisher: &mut EventPublisher<EventType>) -> usize {
        publisher.take_queue(&mut self.dispatch_queue);
        self.dispatch_queue.len()
    }

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
            },
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
