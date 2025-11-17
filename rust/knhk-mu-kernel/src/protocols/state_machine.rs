//! Type-Level State Machines
//!
//! This module implements the typestate pattern for compile-time protocol enforcement.
//! State machines are encoded entirely in the type system, providing:
//! - Compile-time validation of state transitions
//! - Impossible to be in invalid state
//! - Impossible to call wrong method for state
//! - Zero runtime overhead
//!
//! ## Example
//! ```no_run
//! use knhk_mu_kernel::protocols::state_machine::*;
//!
//! // Create state machine in Initial state
//! let machine = StateMachine::<Initial>::new();
//!
//! // Transition to Running state
//! let machine = machine.start();
//!
//! // Can only call methods available in Running state
//! let machine = machine.pause();
//!
//! // Invalid transitions are compile errors:
//! // machine.start(); // ERROR: no method `start` on StateMachine<Paused>
//! ```

use core::marker::PhantomData;

/// Generic state machine parameterized by current state
///
/// The state parameter S is a zero-sized marker type that encodes
/// the current state in the type system. This ensures:
/// - State is tracked at compile time
/// - Invalid transitions are type errors
/// - Zero runtime cost
pub struct StateMachine<S> {
    /// Phantom marker for state (zero-sized)
    ///
    /// We use fn() -> S instead of S to avoid requiring S: Sized
    /// and to ensure the marker is invariant in S.
    _state: PhantomData<fn() -> S>,
}

// Ensure StateMachine is always zero-sized
const _: () = {
    assert!(core::mem::size_of::<StateMachine<()>>() == 0);
};

impl<S> StateMachine<S> {
    /// Internal constructor (private to this module)
    ///
    /// This prevents arbitrary state creation.
    /// Each state must explicitly provide a way to enter it.
    #[inline(always)]
    const fn new() -> Self {
        Self {
            _state: PhantomData,
        }
    }
}

/// Common state definitions
///
/// These are zero-sized marker types that exist purely for type-level state tracking.

/// Initial state - machine not yet started
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Initial;

/// Running state - machine is executing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Running;

/// Paused state - machine is paused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Paused;

/// Stopped state - machine has stopped (terminal)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stopped;

/// Error state - machine encountered error (terminal)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error;

// State transitions

impl StateMachine<Initial> {
    /// Create a new state machine in Initial state
    ///
    /// This is the only public constructor.
    #[inline(always)]
    pub const fn new() -> Self {
        Self::new()
    }

    /// Start the machine (Initial -> Running)
    #[inline(always)]
    pub fn start(self) -> StateMachine<Running> {
        StateMachine::new()
    }
}

impl Default for StateMachine<Initial> {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine<Running> {
    /// Pause the machine (Running -> Paused)
    #[inline(always)]
    pub fn pause(self) -> StateMachine<Paused> {
        StateMachine::new()
    }

    /// Stop the machine (Running -> Stopped)
    #[inline(always)]
    pub fn stop(self) -> StateMachine<Stopped> {
        StateMachine::new()
    }

    /// Encounter error (Running -> Error)
    #[inline(always)]
    pub fn error(self) -> StateMachine<Error> {
        StateMachine::new()
    }
}

impl StateMachine<Paused> {
    /// Resume the machine (Paused -> Running)
    #[inline(always)]
    pub fn resume(self) -> StateMachine<Running> {
        StateMachine::new()
    }

    /// Stop the machine (Paused -> Stopped)
    #[inline(always)]
    pub fn stop(self) -> StateMachine<Stopped> {
        StateMachine::new()
    }
}

// Terminal states (Stopped, Error) have no transitions

/// Stateful machine with data
///
/// This variant carries actual data along with the type-level state.
/// Useful when you need to track values during state transitions.
pub struct StatefulMachine<S, T> {
    /// The actual data
    data: T,
    /// Type-level state marker
    _state: PhantomData<fn() -> S>,
}

impl<S, T> StatefulMachine<S, T> {
    /// Create stateful machine in state S with data
    #[inline(always)]
    fn with_data(data: T) -> Self {
        Self {
            data,
            _state: PhantomData,
        }
    }

    /// Get reference to data
    #[inline(always)]
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Get mutable reference to data
    #[inline(always)]
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Consume and return data
    #[inline(always)]
    pub fn into_data(self) -> T {
        self.data
    }
}

impl<T> StatefulMachine<Initial, T> {
    /// Create new stateful machine
    #[inline(always)]
    pub fn new(data: T) -> Self {
        Self::with_data(data)
    }

    /// Start with transformation
    #[inline(always)]
    pub fn start<F>(self, f: F) -> StatefulMachine<Running, T>
    where
        F: FnOnce(&mut T),
    {
        let mut data = self.data;
        f(&mut data);
        StatefulMachine::with_data(data)
    }
}

impl<T> StatefulMachine<Running, T> {
    /// Pause with transformation
    #[inline(always)]
    pub fn pause<F>(self, f: F) -> StatefulMachine<Paused, T>
    where
        F: FnOnce(&mut T),
    {
        let mut data = self.data;
        f(&mut data);
        StatefulMachine::with_data(data)
    }

    /// Stop with transformation
    #[inline(always)]
    pub fn stop<F>(self, f: F) -> StatefulMachine<Stopped, T>
    where
        F: FnOnce(&mut T),
    {
        let mut data = self.data;
        f(&mut data);
        StatefulMachine::with_data(data)
    }
}

impl<T> StatefulMachine<Paused, T> {
    /// Resume with transformation
    #[inline(always)]
    pub fn resume<F>(self, f: F) -> StatefulMachine<Running, T>
    where
        F: FnOnce(&mut T),
    {
        let mut data = self.data;
        f(&mut data);
        StatefulMachine::with_data(data)
    }
}

/// State machine builder pattern
///
/// Provides a fluent API for constructing and transitioning state machines.
pub struct Builder<S, T> {
    machine: StatefulMachine<S, T>,
}

impl<T> Builder<Initial, T> {
    /// Create new builder
    #[inline(always)]
    pub fn new(data: T) -> Self {
        Self {
            machine: StatefulMachine::new(data),
        }
    }

    /// Build and start
    #[inline(always)]
    pub fn build_and_start<F>(self, f: F) -> Builder<Running, T>
    where
        F: FnOnce(&mut T),
    {
        Builder {
            machine: self.machine.start(f),
        }
    }
}

impl<S, T> Builder<S, T> {
    /// Get inner machine
    #[inline(always)]
    pub fn into_machine(self) -> StatefulMachine<S, T> {
        self.machine
    }

    /// Get data reference
    #[inline(always)]
    pub fn data(&self) -> &T {
        self.machine.data()
    }
}

/// Conditional state transitions
///
/// Allows branching based on runtime conditions while maintaining type safety.
pub enum ConditionalTransition<S1, S2> {
    /// Transition to first state
    First(StateMachine<S1>),
    /// Transition to second state
    Second(StateMachine<S2>),
}

impl<S1, S2> ConditionalTransition<S1, S2> {
    /// Create first transition
    #[inline(always)]
    pub fn first() -> Self {
        ConditionalTransition::First(StateMachine::new())
    }

    /// Create second transition
    #[inline(always)]
    pub fn second() -> Self {
        ConditionalTransition::Second(StateMachine::new())
    }

    /// Match on transition
    #[inline(always)]
    pub fn match_transition<F, G, R>(self, f: F, g: G) -> R
    where
        F: FnOnce(StateMachine<S1>) -> R,
        G: FnOnce(StateMachine<S2>) -> R,
    {
        match self {
            ConditionalTransition::First(m) => f(m),
            ConditionalTransition::Second(m) => g(m),
        }
    }
}

/// Parallel state machines
///
/// Run two state machines in parallel.
pub struct Parallel<S1, S2> {
    _machine1: PhantomData<fn() -> S1>,
    _machine2: PhantomData<fn() -> S2>,
}

impl<S1, S2> Parallel<S1, S2> {
    /// Create parallel machines
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            _machine1: PhantomData,
            _machine2: PhantomData,
        }
    }
}

impl<S1, S2> Default for Parallel<S1, S2> {
    fn default() -> Self {
        Self::new()
    }
}

/// State machine with guards
///
/// Enforces preconditions before allowing transitions.
pub struct Guarded<S, T> {
    machine: StatefulMachine<S, T>,
}

impl<S, T> Guarded<S, T> {
    /// Create guarded machine
    #[inline(always)]
    fn new(machine: StatefulMachine<S, T>) -> Self {
        Self { machine }
    }

    /// Get data
    #[inline(always)]
    pub fn data(&self) -> &T {
        self.machine.data()
    }
}

impl<T> Guarded<Initial, T> {
    /// Create guarded machine
    #[inline(always)]
    pub fn create(data: T) -> Self {
        Self::new(StatefulMachine::new(data))
    }

    /// Guarded start - only transitions if guard passes
    #[inline(always)]
    pub fn start_if<F, G>(
        self,
        guard: F,
        transform: G,
    ) -> Result<Guarded<Running, T>, Guarded<Error, T>>
    where
        F: FnOnce(&T) -> bool,
        G: FnOnce(&mut T),
    {
        if guard(self.machine.data()) {
            let machine = self.machine.start(transform);
            Ok(Guarded::new(machine))
        } else {
            let machine = StatefulMachine::with_data(self.machine.into_data());
            Err(Guarded::new(machine))
        }
    }
}

/// Timed state machine
///
/// Tracks timing information alongside state.
pub struct TimedMachine<S> {
    _state: PhantomData<fn() -> S>,
    /// Tick count in this state
    ticks: u64,
}

impl<S> TimedMachine<S> {
    /// Create timed machine
    #[inline(always)]
    fn new() -> Self {
        Self {
            _state: PhantomData,
            ticks: 0,
        }
    }

    /// Get tick count
    #[inline(always)]
    pub fn ticks(&self) -> u64 {
        self.ticks
    }
}

impl TimedMachine<Initial> {
    /// Create new timed machine
    #[inline(always)]
    pub fn new() -> Self {
        Self::new()
    }

    /// Start timing
    #[inline(always)]
    pub fn start(self) -> TimedMachine<Running> {
        TimedMachine::new()
    }
}

impl Default for TimedMachine<Initial> {
    fn default() -> Self {
        Self::new()
    }
}

impl TimedMachine<Running> {
    /// Tick the machine
    #[inline(always)]
    pub fn tick(mut self) -> Self {
        self.ticks = self.ticks.saturating_add(1);
        self
    }

    /// Stop timing
    #[inline(always)]
    pub fn stop(self) -> TimedMachine<Stopped> {
        TimedMachine {
            _state: PhantomData,
            ticks: self.ticks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_state_transitions() {
        let machine = StateMachine::<Initial>::new();
        let machine = machine.start();
        let machine = machine.pause();
        let machine = machine.resume();
        let _machine = machine.stop();
    }

    #[test]
    fn test_stateful_machine() {
        let machine = StatefulMachine::new(42u32);
        let machine = machine.start(|x| *x += 1);
        assert_eq!(*machine.data(), 43);

        let machine = machine.pause(|x| *x *= 2);
        assert_eq!(*machine.data(), 86);
    }

    #[test]
    fn test_builder_pattern() {
        let builder = Builder::new(vec![1, 2, 3]);
        let builder = builder.build_and_start(|v| v.push(4));

        assert_eq!(builder.data(), &vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_conditional_transition() {
        let transition: ConditionalTransition<Running, Error> = if true {
            ConditionalTransition::first()
        } else {
            ConditionalTransition::second()
        };

        let _result = transition.match_transition(|m| m.pause(), |_m| StateMachine::<Error>::new());
    }

    #[test]
    fn test_guarded_transition() {
        let machine = Guarded::create(10u32);

        let result = machine.start_if(|x| *x > 5, |x| *x *= 2);

        assert!(result.is_ok());
        let machine = result.unwrap();
        assert_eq!(*machine.data(), 20);
    }

    #[test]
    fn test_guarded_transition_fails() {
        let machine = Guarded::create(3u32);

        let result = machine.start_if(|x| *x > 5, |x| *x *= 2);

        assert!(result.is_err());
    }

    #[test]
    fn test_timed_machine() {
        let machine = TimedMachine::<Initial>::new();
        let machine = machine.start();
        let machine = machine.tick().tick().tick();

        assert_eq!(machine.ticks(), 3);
    }

    #[test]
    fn test_zero_size_state_machine() {
        // Verify zero runtime overhead for stateless machines
        assert_eq!(core::mem::size_of::<StateMachine<Initial>>(), 0);
        assert_eq!(core::mem::size_of::<StateMachine<Running>>(), 0);
        assert_eq!(core::mem::size_of::<StateMachine<Stopped>>(), 0);
    }
}
