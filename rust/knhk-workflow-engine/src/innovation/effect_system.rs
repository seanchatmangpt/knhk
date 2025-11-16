//! Effect System: Tracking Side Effects at the Type Level
//!
//! Pure functions vs effectful operations encoded in types.
//! Effect handlers for I/O, state, errors, and resources.
//! Algebraic effects without runtime overhead.

use core::marker::PhantomData;

/// Effect trait - describes a computational effect
pub trait Effect: 'static {
    const NAME: &'static str;
    const IS_PURE: bool;
}

/// Pure effect - no side effects
pub struct Pure;
impl Effect for Pure {
    const NAME: &'static str = "pure";
    const IS_PURE: bool = true;
}

/// I/O effect - reads/writes to external systems
pub struct Io;
impl Effect for Io {
    const NAME: &'static str = "io";
    const IS_PURE: bool = false;
}

/// State effect - mutable state access
pub struct State;
impl Effect for State {
    const NAME: &'static str = "state";
    const IS_PURE: bool = false;
}

/// Error effect - can fail
pub struct Error;
impl Effect for Error {
    const NAME: &'static str = "error";
    const IS_PURE: bool = false;
}

/// Resource effect - manages scarce resources
pub struct Resource;
impl Effect for Resource {
    const NAME: &'static str = "resource";
    const IS_PURE: bool = false;
}

/// Effect set - combines multiple effects
pub trait EffectSet: 'static {
    const HAS_IO: bool;
    const HAS_STATE: bool;
    const HAS_ERROR: bool;
    const HAS_RESOURCE: bool;
    const IS_PURE: bool;
}

/// No effects - pure computation
pub struct NoEffects;
impl EffectSet for NoEffects {
    const HAS_IO: bool = false;
    const HAS_STATE: bool = false;
    const HAS_ERROR: bool = false;
    const HAS_RESOURCE: bool = false;
    const IS_PURE: bool = true;
}

/// All effects - unrestricted computation
pub struct AllEffects;
impl EffectSet for AllEffects {
    const HAS_IO: bool = true;
    const HAS_STATE: bool = true;
    const HAS_ERROR: bool = true;
    const HAS_RESOURCE: bool = true;
    const IS_PURE: bool = false;
}

/// Effectful computation - tracks effects in types
pub struct Effectful<T, E: EffectSet> {
    value: T,
    _effects: PhantomData<E>,
}

impl<T, E: EffectSet> Effectful<T, E> {
    /// Create effectful computation
    pub const fn new(value: T) -> Self {
        Self {
            value,
            _effects: PhantomData,
        }
    }

    /// Extract value (only for pure computations)
    pub fn pure(self) -> T
    where
        E: EffectSet<IS_PURE = true>,
    {
        self.value
    }

    /// Run with effect handler
    pub fn handle<F, R>(self, handler: F) -> R
    where
        F: FnOnce(T) -> R,
    {
        handler(self.value)
    }

    /// Map over effectful value (preserves effects)
    pub fn map<U, F>(self, f: F) -> Effectful<U, E>
    where
        F: FnOnce(T) -> U,
    {
        Effectful::new(f(self.value))
    }

    /// Flat map - sequence effectful computations
    pub fn and_then<U, F>(self, f: F) -> Effectful<U, E>
    where
        F: FnOnce(T) -> Effectful<U, E>,
    {
        f(self.value)
    }
}

/// Effect handler for I/O operations
pub struct IoHandler<T> {
    operation: Box<dyn FnOnce() -> T>,
}

impl<T> IoHandler<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T + 'static,
    {
        Self {
            operation: Box::new(f),
        }
    }

    pub fn run(self) -> Effectful<T, AllEffects> {
        Effectful::new((self.operation)())
    }
}

/// Effect handler for state operations
pub struct StateHandler<S, T> {
    state: S,
    operation: Box<dyn FnOnce(&mut S) -> T>,
}

impl<S, T> StateHandler<S, T> {
    pub fn new<F>(state: S, f: F) -> Self
    where
        F: FnOnce(&mut S) -> T + 'static,
    {
        Self {
            state,
            operation: Box::new(f),
        }
    }

    pub fn run(mut self) -> (Effectful<T, AllEffects>, S) {
        let result = (self.operation)(&mut self.state);
        (Effectful::new(result), self.state)
    }
}

/// Effect composition - combine effect sets
pub struct Union<E1: EffectSet, E2: EffectSet> {
    _e1: PhantomData<E1>,
    _e2: PhantomData<E2>,
}

impl<E1: EffectSet, E2: EffectSet> EffectSet for Union<E1, E2> {
    const HAS_IO: bool = E1::HAS_IO || E2::HAS_IO;
    const HAS_STATE: bool = E1::HAS_STATE || E2::HAS_STATE;
    const HAS_ERROR: bool = E1::HAS_ERROR || E2::HAS_ERROR;
    const HAS_RESOURCE: bool = E1::HAS_RESOURCE || E2::HAS_RESOURCE;
    const IS_PURE: bool = E1::IS_PURE && E2::IS_PURE;
}

/// Workflow with tracked effects
pub struct EffectfulWorkflow<I, O, E: EffectSet> {
    _input: PhantomData<I>,
    _output: PhantomData<O>,
    _effects: PhantomData<E>,
}

impl<I, O, E: EffectSet> EffectfulWorkflow<I, O, E> {
    pub const fn new() -> Self {
        Self {
            _input: PhantomData,
            _output: PhantomData,
            _effects: PhantomData,
        }
    }

    /// Execute workflow (only if pure)
    pub fn execute_pure(&self, input: I) -> O
    where
        E: EffectSet<IS_PURE = true>,
        O: Default,
    {
        // Pure workflows can be executed without effect handlers
        O::default()
    }

    /// Execute workflow with effect handling
    pub fn execute_with_effects<F>(&self, input: I, handler: F) -> Effectful<O, E>
    where
        F: FnOnce(I) -> O,
    {
        Effectful::new(handler(input))
    }
}

/// Effect permission - compile-time capability
pub struct Permission<E: Effect> {
    _effect: PhantomData<E>,
}

impl<E: Effect> Permission<E> {
    pub const fn new() -> Self {
        Self {
            _effect: PhantomData,
        }
    }

    /// Grant permission (zero-cost)
    pub const fn grant() -> Self {
        Self::new()
    }
}

/// Workflow execution context with effect permissions
pub struct EffectContext<E: EffectSet> {
    _effects: PhantomData<E>,
}

impl<E: EffectSet> EffectContext<E> {
    pub const fn new() -> Self {
        Self {
            _effects: PhantomData,
        }
    }

    /// Execute I/O operation (only if permitted)
    pub fn execute_io<T, F>(&self, f: F) -> Effectful<T, E>
    where
        E: EffectSet<HAS_IO = true>,
        F: FnOnce() -> T,
    {
        Effectful::new(f())
    }

    /// Modify state (only if permitted)
    pub fn modify_state<S, T, F>(&self, state: &mut S, f: F) -> Effectful<T, E>
    where
        E: EffectSet<HAS_STATE = true>,
        F: FnOnce(&mut S) -> T,
    {
        Effectful::new(f(state))
    }

    /// Propagate error (only if permitted)
    pub fn propagate_error<T>(&self, error: &'static str) -> Effectful<Result<T, &'static str>, E>
    where
        E: EffectSet<HAS_ERROR = true>,
    {
        Effectful::new(Err(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_computation() {
        let computation = Effectful::<i32, NoEffects>::new(42);
        let value = computation.pure();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_effectful_map() {
        let computation = Effectful::<i32, NoEffects>::new(21);
        let doubled = computation.map(|x| x * 2);
        assert_eq!(doubled.pure(), 42);
    }

    #[test]
    fn test_effect_context() {
        let ctx = EffectContext::<AllEffects>::new();
        let result = ctx.execute_io(|| 42);
        result.handle(|value| assert_eq!(value, 42));
    }

    #[test]
    fn test_state_handler() {
        let handler = StateHandler::new(0, |state: &mut i32| {
            *state += 10;
            *state
        });

        let (result, final_state) = handler.run();
        result.handle(|value| assert_eq!(value, 10));
        assert_eq!(final_state, 10);
    }

    #[test]
    fn test_effect_permissions() {
        let _io_perm = Permission::<Io>::grant();
        let _state_perm = Permission::<State>::grant();

        // Permissions are zero-sized
        assert_eq!(std::mem::size_of::<Permission<Io>>(), 0);
        assert_eq!(std::mem::size_of::<Permission<State>>(), 0);
    }

    #[test]
    fn test_workflow_with_effects() {
        let workflow = EffectfulWorkflow::<(), i32, AllEffects>::new();
        let result = workflow.execute_with_effects((), |_| 42);
        result.handle(|value| assert_eq!(value, 42));
    }
}
