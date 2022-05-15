use std::collections::VecDeque;
use std::ops::DerefMut;

use thiserror::Error;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TransitionTo {
    Paused,
    Dead,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TransitionDirection {
    In,
    Out
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum State {
    Pending,
    Active,
    Paused,
    TransitionIn,
    TransitionOut(TransitionTo),
    Dead,
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub enum StateChange<ContextType> {
    Push(Box<dyn GameState<ContextType>>),
    Pop,
}

pub trait GameState<ContextType> {
    fn update(&mut self, state: State, context: &mut ContextType) -> Option<StateChange<ContextType>>;
    fn render(&mut self, state: State, context: &mut ContextType);
    fn transition(&mut self, state: State, context: &mut ContextType) -> bool;
    fn state_change(&mut self, new_state: State, old_state: State, context: &mut ContextType);
}

///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Operation cannot currently be performed because there is already a pending state change.")]
    HasPendingStateChange,

    #[error("Operation cannot currently be performed because the GameState's current state ({0:?}) does not allow it.")]
    GameStateInvalidState(State),
}

struct StateContainer<ContextType> {
    current_state: State,
    pending_state_change: Option<State>,
    state: Box<dyn GameState<ContextType>>,
}

impl<ContextType> StateContainer<ContextType> {
    pub fn new(state: Box<dyn GameState<ContextType>>) -> Self {
        StateContainer {
            current_state: State::Dead,
            pending_state_change: None,
            state,
        }
    }

    #[inline]
    pub fn current_state(&self) -> State {
        self.current_state
    }

    #[inline]
    pub fn pending_state_change(&mut self) -> Option<State> {
        self.pending_state_change.take()
    }

    #[inline]
    pub fn change_state(&mut self, new_state: State, context: &mut ContextType) {
        let old_state = self.current_state;
        self.current_state = new_state;
        self.state.state_change(self.current_state, old_state, context);
    }

    #[inline]
    pub fn state(&mut self) -> &mut dyn GameState<ContextType> {
        self.state.deref_mut()
    }

    pub fn transition_out(&mut self, to: TransitionTo, context: &mut ContextType) -> Result<(), StateError> {
        if self.current_state == State::Active {
            self.change_state(State::TransitionOut(to), context);
            Ok(())
        } else {
            Err(StateError::GameStateInvalidState(self.current_state))
        }
    }

    #[inline]
    pub fn pending_transition_out(&mut self, to: TransitionTo) {
        self.pending_state_change = Some(State::TransitionOut(to));
    }

    pub fn transition_in(&mut self, context: &mut ContextType) -> Result<(), StateError> {
        match self.current_state {
            State::Pending | State::Paused => {
                self.change_state(State::TransitionIn, context);
                Ok(())
            },
            _ => {
                Err(StateError::GameStateInvalidState(self.current_state))
            }
        }
    }

    #[inline]
    pub fn pending_transition_in(&mut self) {
        self.pending_state_change = Some(State::TransitionIn);
    }

    pub fn activate(&mut self, context: &mut ContextType) -> Result<(), StateError> {
        self.change_state(State::Active, context);
        Ok(())
    }

    #[inline]
    pub fn pending_activate(&mut self) {
        self.pending_state_change = Some(State::Active);
    }

    pub fn pause(&mut self, context: &mut ContextType) -> Result<(), StateError> {
        self.change_state(State::Paused, context);
        Ok(())
    }

    #[inline]
    pub fn pending_pause(&mut self) {
        self.pending_state_change = Some(State::Paused);
    }

    pub fn kill(&mut self, context: &mut ContextType) -> Result<(), StateError> {
        self.change_state(State::Dead, context);
        Ok(())
    }

    #[inline]
    pub fn pending_kill(&mut self) {
        self.pending_state_change = Some(State::Dead);
    }
}

pub struct States<ContextType> {
    states: VecDeque<StateContainer<ContextType>>,
    command: Option<StateChange<ContextType>>,
    pending_state: Option<Box<dyn GameState<ContextType>>>,
}

impl<ContextType> States<ContextType> {
    pub fn new() -> Self {
        States {
            states: VecDeque::new(),
            command: None,
            pending_state: None,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty() && self.pending_state.is_none() && self.command.is_none()
    }

    fn can_push_or_pop(&self) -> bool {
        if let Some(state) = self.states.front() {
            if state.current_state != State::Active {
                return false;
            }
        }
        if self.pending_state.is_some() {
            return false;
        }

        true
    }

    fn push_boxed_state(&mut self, boxed_state: Box<dyn GameState<ContextType>>) -> Result<(), StateError> {
        if !self.can_push_or_pop() {
            Err(StateError::HasPendingStateChange)
        } else {
            self.command = Some(StateChange::Push(boxed_state));
            Ok(())
        }
    }

    pub fn push(&mut self, state: impl GameState<ContextType> + 'static) -> Result<(), StateError> {
        self.push_boxed_state(Box::new(state))
    }

    pub fn pop(&mut self) -> Result<(), StateError> {
        if !self.can_push_or_pop() {
            Err(StateError::HasPendingStateChange)
        } else {
            if !self.states.is_empty() {
                self.command = Some(StateChange::Pop);
            }
            Ok(())
        }
    }

    fn state_of_front_state(&self) -> Option<State> {
        if let Some(state) = self.states.front() {
            Some(state.current_state())
        } else {
            None
        }
    }

    fn process_state_changes(&mut self, context: &mut ContextType) -> Result<(), StateError> {
        if let Some(command) = self.command.take() {
            match command {
                StateChange::Push(new_state) => {
                    self.pending_state = Some(new_state);
                }
                StateChange::Pop => {
                    if let Some(state) = self.states.front_mut() {
                        state.pending_transition_out(TransitionTo::Dead);
                    }
                }
            }
        }

        if self.pending_state.is_some() {
            if self.states.is_empty() {
                // special case to bootstrap the stack of states when e.g. the system is first set
                // up with the very first state pushed to it.
                let mut new_state = StateContainer::new(self.pending_state.take().unwrap());
                new_state.change_state(State::Pending, context);
                self.states.push_front(new_state);
            } else if self.state_of_front_state() == Some(State::Active) {
                // if the current state is active and there is a pending state waiting to be added,
                // we need to start transitioning out the active state towards a 'paused' state
                let state = self.states.front_mut().unwrap();
                state.pending_transition_out(TransitionTo::Paused);
            }
        }

        // handle any pending state change queued from the previous frame, so that we can
        // process the state as necessary below ...
        if let Some(state) = self.states.front_mut() {
            if let Some(pending_state_change) = state.pending_state_change() {
                match pending_state_change {
                    State::Dead => state.kill(context)?,
                    State::Paused => state.pause(context)?,
                    State::Active => state.activate(context)?,
                    State::TransitionOut(to) => state.transition_out(to, context)?,
                    State::TransitionIn => state.transition_in(context)?,
                    _ => {},
                }
            }
        }


        // now figure out what state change processing is needed based on the current state ...
        match self.state_of_front_state() {
            Some(State::Pending) => {
                // top state is just sitting there pending, lets start it up ...
                let state = self.states.front_mut().unwrap();
                state.pending_transition_in();
            },
            Some(State::Paused) => {
                if self.pending_state.is_some() {
                    // top state is paused and we have a new state waiting to be added.
                    // add the new state
                    let mut new_state = StateContainer::new(self.pending_state.take().unwrap());
                    new_state.change_state(State::Pending, context);
                    self.states.push_front(new_state);
                }
            },
            Some(State::Dead) => {
                // remove the dead state
                self.states.pop_front();

                if self.pending_state.is_some() {
                    // if there is a new pending state waiting, we can add it here right now
                    let mut new_state = StateContainer::new(self.pending_state.take().unwrap());
                    new_state.change_state(State::Pending, context);
                    self.states.push_front(new_state);
                } else if self.state_of_front_state() == Some(State::Paused) {
                    // otherwise, we're probably waking up a state that was paused and needs to
                    // be resumed since it's once again on top
                    let state = self.states.front_mut().unwrap();
                    state.pending_transition_in();
                }
            },
            Some(State::TransitionIn) => {
                let state = self.states.front_mut().unwrap();
                if state.state().transition(State::TransitionIn, context) {
                    // state has indicated it is done transitioning, so we can switch it to active
                    state.pending_activate();
                }
            },
            Some(State::TransitionOut(to)) => {
                let state = self.states.front_mut().unwrap();
                if state.state().transition(State::TransitionOut(to), context) {
                    // state has indicated it is done transitioning, so we can switch it to whatever
                    // it was transitioning to
                    match to {
                        TransitionTo::Paused => { state.pending_pause(); },
                        TransitionTo::Dead => { state.pending_kill(); }
                    }
                }
            },
            _ => {}
        }

        Ok(())
    }

    pub fn update(&mut self, context: &mut ContextType) -> Result<(), StateError> {
        self.process_state_changes(context)?;
        if let Some(state) = self.states.front_mut() {
            let current_state = state.current_state();
            match current_state {
                State::Active | State::TransitionIn | State::TransitionOut(_) => {
                    if let Some(state_change) = state.state().update(current_state, context) {
                        match state_change {
                            StateChange::Push(state) => self.push_boxed_state(state)?,
                            StateChange::Pop => self.pop()?,
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn render(&mut self, context: &mut ContextType) {
        if let Some(state) = self.states.front_mut() {
            let current_state = state.current_state();
            match current_state {
                State::Active | State::TransitionIn | State::TransitionOut(_) => {
                    state.state().render(current_state, context);
                },
                _ => {}
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use claim::*;

    use super::*;

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    enum LogEntry {
        Update(u32, State),
        Render(u32, State),
        Transition(u32, State),
        StateChange(u32, State, State),
    }

    struct TestContext {
        pub log: Vec<LogEntry>,
    }

    impl TestContext {
        pub fn new() -> Self {
            TestContext {
                log: Vec::new()
            }
        }

        pub fn log(&mut self, entry: LogEntry) {
            self.log.push(entry);
        }

        pub fn take_log(&mut self) -> Vec<LogEntry> {
            let taken = self.log.to_vec();
            self.log.clear();
            taken
        }
    }

    struct TestState {
        id: u32,
        counter: u32,
        transition_length: u32,
    }

    impl TestState {
        pub fn new(id: u32) -> Self {
            TestState {
                id,
                counter: 0,
                transition_length: 0,
            }
        }

        pub fn new_with_transition_length(id: u32, transition_length: u32) -> Self {
            TestState {
                id,
                counter: 0,
                transition_length,
            }
        }
    }

    impl GameState<TestContext> for TestState {
        fn update(&mut self, state: State, context: &mut TestContext) -> Option<StateChange<TestContext>> {
            context.log(LogEntry::Update(self.id, state));
            None
        }

        fn render(&mut self, state: State, context: &mut TestContext) {
            context.log(LogEntry::Render(self.id, state));
        }

        fn transition(&mut self, state: State, context: &mut TestContext) -> bool {
            context.log(LogEntry::Transition(self.id, state));
            if self.counter > 0 {
                self.counter -= 1;
            }
            if self.counter == 0 {
                true
            } else {
                false
            }
        }

        fn state_change(&mut self, new_state: State, old_state: State, context: &mut TestContext) {
            context.log(LogEntry::StateChange(self.id, new_state, old_state));
            match new_state {
                State::TransitionIn | State::TransitionOut(_) => {
                    self.counter = self.transition_length;
                },
                _ => {}
            }
        }
    }

    fn tick<ContextType>(states: &mut States<ContextType>, context: &mut ContextType) -> Result<(), StateError> {
        states.update(context)?;
        states.render(context);
        Ok(())
    }

    #[test]
    fn push_and_pop_state() -> Result<(), StateError> {
        use LogEntry::*;
        use State::*;

        const FOO: u32 = 1;

        let mut states = States::<TestContext>::new();
        let mut context = TestContext::new();

        states.push(TestState::new(FOO))?;
        assert_eq!(context.take_log(), vec![]);
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(FOO, Pending, Dead)]);
        // state will transition in
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FOO, TransitionIn, Pending),
                Transition(FOO, TransitionIn),
                Update(FOO, TransitionIn),
                Render(FOO, TransitionIn)
            ]
        );
        // state finished transitioning in, now moves to active
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FOO, Active, TransitionIn),
                Update(FOO, Active),
                Render(FOO, Active)
            ]
        );

        states.pop()?;
        assert_eq!(context.take_log(), vec![]);
        // state begins to transition out to 'dead'
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FOO, TransitionOut(TransitionTo::Dead), Active),
                Transition(FOO, TransitionOut(TransitionTo::Dead)),
                Update(FOO, TransitionOut(TransitionTo::Dead)),
                Render(FOO, TransitionOut(TransitionTo::Dead))
            ]
        );
        // state finished transitioning out, now dies
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(FOO, Dead, TransitionOut(TransitionTo::Dead))]);

        // nothing! no states anymore!
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![]);

        Ok(())
    }

    #[test]
    fn push_and_pop_state_with_longer_transition() -> Result<(), StateError> {
        use LogEntry::*;
        use State::*;

        const FOO: u32 = 1;

        let mut states = States::<TestContext>::new();
        let mut context = TestContext::new();

        states.push(TestState::new_with_transition_length(FOO, 5))?;
        assert_eq!(context.take_log(), vec![]);
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(FOO, Pending, Dead)]);
        // state will transition in
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FOO, TransitionIn, Pending),
                Transition(FOO, TransitionIn),
                Update(FOO, TransitionIn),
                Render(FOO, TransitionIn)
            ]
        );
        // wait for transition to finish
        for _ in 0..4 {
            tick(&mut states, &mut context)?;
            assert_eq!(
                context.take_log(),
                vec![
                    Transition(FOO, TransitionIn),
                    Update(FOO, TransitionIn),
                    Render(FOO, TransitionIn)
                ]
            );
        }
        // state finished transitioning in, now moves to active
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FOO, Active, TransitionIn),
                Update(FOO, Active),
                Render(FOO, Active)
            ]
        );

        states.pop()?;
        assert_eq!(context.take_log(), vec![]);
        // state begins to transition out to 'dead'
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FOO, TransitionOut(TransitionTo::Dead), Active),
                Transition(FOO, TransitionOut(TransitionTo::Dead)),
                Update(FOO, TransitionOut(TransitionTo::Dead)),
                Render(FOO, TransitionOut(TransitionTo::Dead))
            ]
        );
        // wait for transition to finish
        for _ in 0..4 {
            tick(&mut states, &mut context)?;
            assert_eq!(
                context.take_log(),
                vec![
                    Transition(FOO, TransitionOut(TransitionTo::Dead)),
                    Update(FOO, TransitionOut(TransitionTo::Dead)),
                    Render(FOO, TransitionOut(TransitionTo::Dead))
                ]
            );
        }
        // state finished transitioning out, now dies
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(FOO, Dead, TransitionOut(TransitionTo::Dead))]);

        // nothing! no states anymore!
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![]);

        Ok(())
    }

    #[test]
    fn push_and_pop_multiple_states() -> Result<(), StateError> {
        use LogEntry::*;
        use State::*;

        const FIRST: u32 = 1;
        const SECOND: u32 = 2;

        let mut states = States::<TestContext>::new();
        let mut context = TestContext::new();

        // push first state

        states.push(TestState::new(FIRST))?;
        assert_eq!(context.take_log(), vec![]);
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(FIRST, Pending, Dead)]);
        // first state will transition in
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, TransitionIn, Pending),
                Transition(FIRST, TransitionIn),
                Update(FIRST, TransitionIn),
                Render(FIRST, TransitionIn)
            ]
        );
        // first state finished transitioning in, now moves to active
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, Active, TransitionIn),
                Update(FIRST, Active),
                Render(FIRST, Active)
            ]
        );

        // push second state

        states.push(TestState::new(SECOND))?;
        assert_eq!(context.take_log(), vec![]);
        // first state begins to transition out to 'paused' state
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, TransitionOut(TransitionTo::Paused), Active),
                Transition(FIRST, TransitionOut(TransitionTo::Paused)),
                Update(FIRST, TransitionOut(TransitionTo::Paused)),
                Render(FIRST, TransitionOut(TransitionTo::Paused))
            ]
        );
        // state finished transitioning out, now is paused
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, Paused, TransitionOut(TransitionTo::Paused)),
                StateChange(SECOND, Pending, Dead),
            ]
        );
        // second state starts up
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![]);
        // second state will transition in
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(SECOND, TransitionIn, Pending),
                Transition(SECOND, TransitionIn),
                Update(SECOND, TransitionIn),
                Render(SECOND, TransitionIn)
            ]
        );
        // second state finished transitioning in, now moves to active
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(SECOND, Active, TransitionIn),
                Update(SECOND, Active),
                Render(SECOND, Active)
            ]
        );

        // pop second state

        states.pop()?;
        assert_eq!(context.take_log(), vec![]);
        // second state begins to transition out to 'dead'
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(SECOND, TransitionOut(TransitionTo::Dead), Active),
                Transition(SECOND, TransitionOut(TransitionTo::Dead)),
                Update(SECOND, TransitionOut(TransitionTo::Dead)),
                Render(SECOND, TransitionOut(TransitionTo::Dead))
            ]
        );
        // second state finished transitioning out, now dies. first state wakes up again.
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(SECOND, Dead, TransitionOut(TransitionTo::Dead))]);
        // first state will transition in
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, TransitionIn, Paused),
                Transition(FIRST, TransitionIn),
                Update(FIRST, TransitionIn),
                Render(FIRST, TransitionIn)
            ]
        );
        // first state finished transitioning in, now moves to active
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, Active, TransitionIn),
                Update(FIRST, Active),
                Render(FIRST, Active)
            ]
        );

        // pop first state

        states.pop()?;
        assert_eq!(context.take_log(), vec![]);
        // first state begins to transition out to 'dead'
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, TransitionOut(TransitionTo::Dead), Active),
                Transition(FIRST, TransitionOut(TransitionTo::Dead)),
                Update(FIRST, TransitionOut(TransitionTo::Dead)),
                Render(FIRST, TransitionOut(TransitionTo::Dead))
            ]
        );
        // first state finished transitioning out, now dies
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(FIRST, Dead, TransitionOut(TransitionTo::Dead))]);

        // nothing! no states anymore!
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![]);

        Ok(())
    }

    #[test]
    fn push_and_pop_multiple_states_with_longer_transitions() -> Result<(), StateError> {
        use LogEntry::*;
        use State::*;

        const FIRST: u32 = 1;
        const SECOND: u32 = 2;

        let mut states = States::<TestContext>::new();
        let mut context = TestContext::new();

        // push first state

        states.push(TestState::new_with_transition_length(FIRST, 3))?;
        assert_eq!(context.take_log(), vec![]);
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(FIRST, Pending, Dead)]);
        // first state will transition in
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, TransitionIn, Pending),
                Transition(FIRST, TransitionIn),
                Update(FIRST, TransitionIn),
                Render(FIRST, TransitionIn)
            ]
        );
        // wait for transition to finish
        for _ in 0..2 {
            tick(&mut states, &mut context)?;
            assert_eq!(
                context.take_log(),
                vec![
                    Transition(FIRST, TransitionIn),
                    Update(FIRST, TransitionIn),
                    Render(FIRST, TransitionIn)
                ]
            );
        }
        // first state finished transitioning in, now moves to active
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, Active, TransitionIn),
                Update(FIRST, Active),
                Render(FIRST, Active)
            ]
        );

        // push second state

        states.push(TestState::new_with_transition_length(SECOND, 5))?;
        assert_eq!(context.take_log(), vec![]);
        // first state begins to transition out to 'paused' state
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, TransitionOut(TransitionTo::Paused), Active),
                Transition(FIRST, TransitionOut(TransitionTo::Paused)),
                Update(FIRST, TransitionOut(TransitionTo::Paused)),
                Render(FIRST, TransitionOut(TransitionTo::Paused))
            ]
        );
        // wait for transition to finish
        for _ in 0..2 {
            tick(&mut states, &mut context)?;
            assert_eq!(
                context.take_log(),
                vec![
                    Transition(FIRST, TransitionOut(TransitionTo::Paused)),
                    Update(FIRST, TransitionOut(TransitionTo::Paused)),
                    Render(FIRST, TransitionOut(TransitionTo::Paused))
                ]
            );
        }
        // first state finished transitioning out, now is paused
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, Paused, TransitionOut(TransitionTo::Paused)),
                StateChange(SECOND, Pending, Dead)
            ]
        );
        // second state starts up
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![]);
        // second state will transition in
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(SECOND, TransitionIn, Pending),
                Transition(SECOND, TransitionIn),
                Update(SECOND, TransitionIn),
                Render(SECOND, TransitionIn)
            ]
        );
        // wait for transition to finish
        for _ in 0..4 {
            tick(&mut states, &mut context)?;
            assert_eq!(
                context.take_log(),
                vec![
                    Transition(SECOND, TransitionIn),
                    Update(SECOND, TransitionIn),
                    Render(SECOND, TransitionIn)
                ]
            );
        }
        // second state finished transitioning in, now moves to active
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(SECOND, Active, TransitionIn),
                Update(SECOND, Active),
                Render(SECOND, Active)
            ]
        );

        // pop second state

        states.pop()?;
        assert_eq!(context.take_log(), vec![]);
        // second state begins to transition out to 'dead'
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(SECOND, TransitionOut(TransitionTo::Dead), Active),
                Transition(SECOND, TransitionOut(TransitionTo::Dead)),
                Update(SECOND, TransitionOut(TransitionTo::Dead)),
                Render(SECOND, TransitionOut(TransitionTo::Dead))
            ]
        );
        // wait for transition to finish
        for _ in 0..4 {
            tick(&mut states, &mut context)?;
            assert_eq!(
                context.take_log(),
                vec![
                    Transition(SECOND, TransitionOut(TransitionTo::Dead)),
                    Update(SECOND, TransitionOut(TransitionTo::Dead)),
                    Render(SECOND, TransitionOut(TransitionTo::Dead))
                ]
            );
        }
        // second state finished transitioning out, now dies. first state wakes up again.
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(SECOND, Dead, TransitionOut(TransitionTo::Dead))]);
        // first state will transition in
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, TransitionIn, Paused),
                Transition(FIRST, TransitionIn),
                Update(FIRST, TransitionIn),
                Render(FIRST, TransitionIn)
            ]
        );
        // wait for transition to finish
        for _ in 0..2 {
            tick(&mut states, &mut context)?;
            assert_eq!(
                context.take_log(),
                vec![
                    Transition(FIRST, TransitionIn),
                    Update(FIRST, TransitionIn),
                    Render(FIRST, TransitionIn)
                ]
            );
        }
        // first state finished transitioning in, now moves to active
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, Active, TransitionIn),
                Update(FIRST, Active),
                Render(FIRST, Active)
            ]
        );

        // pop first state

        states.pop()?;
        assert_eq!(context.take_log(), vec![]);
        // first state begins to transition out to 'dead'
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, TransitionOut(TransitionTo::Dead), Active),
                Transition(FIRST, TransitionOut(TransitionTo::Dead)),
                Update(FIRST, TransitionOut(TransitionTo::Dead)),
                Render(FIRST, TransitionOut(TransitionTo::Dead))
            ]
        );
        // wait for transition to finish
        for _ in 0..2 {
            tick(&mut states, &mut context)?;
            assert_eq!(
                context.take_log(),
                vec![
                    Transition(FIRST, TransitionOut(TransitionTo::Dead)),
                    Update(FIRST, TransitionOut(TransitionTo::Dead)),
                    Render(FIRST, TransitionOut(TransitionTo::Dead))
                ]
            );
        }
        // first state finished transitioning out, now dies
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(FIRST, Dead, TransitionOut(TransitionTo::Dead))]);

        // nothing! no states anymore!
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![]);

        Ok(())
    }

    struct SelfPushPopState {
        id: u32,
        counter: u32,
        push_after: Option<u32>,
        pop_after: u32,
    }

    impl SelfPushPopState {
        pub fn new(id: u32, push_after: Option<u32>, pop_after: u32) -> Self {
            SelfPushPopState {
                id,
                counter: 0,
                push_after,
                pop_after
            }
        }
    }

    impl GameState<TestContext> for SelfPushPopState {
        fn update(&mut self, state: State, context: &mut TestContext) -> Option<StateChange<TestContext>> {
            context.log(LogEntry::Update(self.id, state));
            if state == State::Active {
                self.counter += 1;
                if self.push_after == Some(self.counter) {
                    return Some(StateChange::Push(Box::new(SelfPushPopState::new(self.id + 1, None, self.pop_after))));
                } else if self.pop_after ==  self.counter {
                    return Some(StateChange::Pop);
                }
            }
            None
        }

        fn render(&mut self, state: State, context: &mut TestContext) {
            context.log(LogEntry::Render(self.id, state));
        }

        fn transition(&mut self, state: State, context: &mut TestContext) -> bool {
            context.log(LogEntry::Transition(self.id, state));
            true
        }

        fn state_change(&mut self, new_state: State, old_state: State, context: &mut TestContext) {
            context.log(LogEntry::StateChange(self.id, new_state, old_state));
        }
    }


    #[test]
    fn state_can_push_and_pop_states_itself() -> Result<(), StateError> {
        use LogEntry::*;
        use State::*;

        const FIRST: u32 = 1;
        const SECOND: u32 = 2;

        let mut states = States::<TestContext>::new();
        let mut context = TestContext::new();

        // pop first state. it will do the rest this time ...

        states.push(SelfPushPopState::new(FIRST, Some(5), 10))?;
        assert_eq!(context.take_log(), vec![]);
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(FIRST, Pending, Dead)]);
        // first state will transition in
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, TransitionIn, Pending),
                Transition(FIRST, TransitionIn),
                Update(FIRST, TransitionIn),
                Render(FIRST, TransitionIn)
            ]
        );
        // first state finished transitioning in, now moves to active
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, Active, TransitionIn),
                Update(FIRST, Active),
                Render(FIRST, Active)
            ]
        );
        // wait for first state's counter to count up to where it should push the second state
        for _ in 0..4 {
            tick(&mut states, &mut context)?;
            assert_eq!(
                context.take_log(),
                vec![
                    Update(FIRST, Active),
                    Render(FIRST, Active)
                ]
            );
        }

        // first state begins to transition out to 'paused' state because it pushed the second state
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, TransitionOut(TransitionTo::Paused), Active),
                Transition(FIRST, TransitionOut(TransitionTo::Paused)),
                Update(FIRST, TransitionOut(TransitionTo::Paused)),
                Render(FIRST, TransitionOut(TransitionTo::Paused))
            ]
        );
        // first state finished transitioning out, now is paused
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, Paused, TransitionOut(TransitionTo::Paused)),
                StateChange(SECOND, Pending, Dead)
            ]
        );
        // second state starts up
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![]);
        // second state will transition in
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(SECOND, TransitionIn, Pending),
                Transition(SECOND, TransitionIn),
                Update(SECOND, TransitionIn),
                Render(SECOND, TransitionIn)
            ]
        );
        // second state finished transitioning in, now moves to active
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(SECOND, Active, TransitionIn),
                Update(SECOND, Active),
                Render(SECOND, Active)
            ]
        );
        // wait for second state's counter to count up to where it should pop itself
        for _ in 0..9 {
            tick(&mut states, &mut context)?;
            assert_eq!(
                context.take_log(),
                vec![
                    Update(SECOND, Active),
                    Render(SECOND, Active)
                ]
            );
        }

        // second state begins to transition out to 'dead'
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(SECOND, TransitionOut(TransitionTo::Dead), Active),
                Transition(SECOND, TransitionOut(TransitionTo::Dead)),
                Update(SECOND, TransitionOut(TransitionTo::Dead)),
                Render(SECOND, TransitionOut(TransitionTo::Dead))
            ]
        );
        // second state finished transitioning out, now dies. first state wakes up again.
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(SECOND, Dead, TransitionOut(TransitionTo::Dead))]);
        // first state will transition in
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, TransitionIn, Paused),
                Transition(FIRST, TransitionIn),
                Update(FIRST, TransitionIn),
                Render(FIRST, TransitionIn)
            ]
        );
        // first state finished transitioning in, now moves to active
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, Active, TransitionIn),
                Update(FIRST, Active),
                Render(FIRST, Active)
            ]
        );
        // wait for first state's counter to count up to where it should pop itself
        for _ in 0..4 {
            tick(&mut states, &mut context)?;
            assert_eq!(
                context.take_log(),
                vec![
                    Update(FIRST, Active),
                    Render(FIRST, Active)
                ]
            );
        }

        // first state begins to transition out to 'dead'
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FIRST, TransitionOut(TransitionTo::Dead), Active),
                Transition(FIRST, TransitionOut(TransitionTo::Dead)),
                Update(FIRST, TransitionOut(TransitionTo::Dead)),
                Render(FIRST, TransitionOut(TransitionTo::Dead))
            ]
        );
        // first state finished transitioning out, now dies
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(FIRST, Dead, TransitionOut(TransitionTo::Dead))]);

        // nothing! no states anymore!
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![]);

        Ok(())
    }

    #[test]
    fn cannot_push_or_pop_states_when_current_state_not_active() -> Result<(), StateError> {
        use LogEntry::*;
        use State::*;

        const FOO: u32 = 1;

        let mut states = States::<TestContext>::new();
        let mut context = TestContext::new();

        states.push(TestState::new(FOO))?;
        assert_eq!(context.take_log(), vec![]);
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(FOO, Pending, Dead)]);

        assert_matches!(states.push(TestState::new(123)), Err(StateError::HasPendingStateChange));
        assert_matches!(states.pop(), Err(StateError::HasPendingStateChange));

        // state will transition in
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FOO, TransitionIn, Pending),
                Transition(FOO, TransitionIn),
                Update(FOO, TransitionIn),
                Render(FOO, TransitionIn)
            ]
        );

        assert_matches!(states.push(TestState::new(123)), Err(StateError::HasPendingStateChange));
        assert_matches!(states.pop(), Err(StateError::HasPendingStateChange));

        // state finished transitioning in, now moves to active
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FOO, Active, TransitionIn),
                Update(FOO, Active),
                Render(FOO, Active)
            ]
        );

        states.pop()?;
        assert_eq!(context.take_log(), vec![]);
        // state begins to transition out to 'dead'
        tick(&mut states, &mut context)?;
        assert_eq!(
            context.take_log(),
            vec![
                StateChange(FOO, TransitionOut(TransitionTo::Dead), Active),
                Transition(FOO, TransitionOut(TransitionTo::Dead)),
                Update(FOO, TransitionOut(TransitionTo::Dead)),
                Render(FOO, TransitionOut(TransitionTo::Dead))
            ]
        );

        assert_matches!(states.push(TestState::new(123)), Err(StateError::HasPendingStateChange));
        assert_matches!(states.pop(), Err(StateError::HasPendingStateChange));

        // state finished transitioning out, now dies
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![StateChange(FOO, Dead, TransitionOut(TransitionTo::Dead))]);

        states.pop()?;

        // nothing! no states anymore!
        tick(&mut states, &mut context)?;
        assert_eq!(context.take_log(), vec![]);

        Ok(())
    }
}
