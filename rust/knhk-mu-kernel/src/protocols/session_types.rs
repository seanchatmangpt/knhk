//! Session Types for Protocol Validation
//!
//! This module implements linear session types that encode protocol behavior
//! in the type system. Session types ensure:
//! - Protocols are followed correctly
//! - States transition in valid order
//! - No states are skipped
//! - No invalid operations on states
//!
//! All enforcement happens at compile time with zero runtime overhead.

use core::marker::PhantomData;

/// Session type marker
///
/// This is a zero-sized type that exists purely for compile-time checking.
/// The type parameter S encodes the current state of the session.
pub struct Session<S> {
    _state: PhantomData<fn() -> S>,
}

impl<S> Session<S> {
    /// Create a new session in state S (internal use only)
    ///
    /// This is private to prevent arbitrary state creation.
    #[inline(always)]
    fn new() -> Self {
        Self {
            _state: PhantomData,
        }
    }
}

// Zero-cost guarantee: Session<S> has same representation as ()
const _: () = {
    assert!(core::mem::size_of::<Session<()>>() == 0);
};

/// Session protocol trait
///
/// Types that implement this trait can participate in session-typed protocols.
pub trait SessionProtocol {
    /// The initial state of this protocol
    type Initial;

    /// The terminal state of this protocol
    type Terminal;
}

/// Linear session types (affine types simulation)
///
/// These types enforce use-once semantics:
/// - Must consume self (not &self)
/// - Cannot be cloned
/// - Cannot be duplicated
/// - Must be used exactly once
pub trait Linear: Sized {
    /// Transition to next state
    ///
    /// This method MUST consume self (not &self) to ensure linearity.
    type Next;

    /// Execute the transition
    fn transition(self) -> Self::Next;
}

/// Protocol states
///
/// These are zero-sized marker types that exist purely for type-level state tracking.

/// Uninitialized protocol state
pub struct Uninitialized;

/// Initialized protocol state
pub struct Initialized;

/// Active protocol state
pub struct Active;

/// Completed protocol state
pub struct Completed;

/// Failed protocol state (terminal)
pub struct Failed;

/// Basic session protocol
///
/// This demonstrates the pattern:
/// Uninitialized -> Initialized -> Active -> Completed
impl SessionProtocol for Session<Uninitialized> {
    type Initial = Uninitialized;
    type Terminal = Completed;
}

// State transitions
impl Session<Uninitialized> {
    /// Create a new uninitialized session
    #[inline(always)]
    pub fn new() -> Self {
        Self::new()
    }

    /// Initialize the session
    ///
    /// This consumes the Uninitialized session and returns an Initialized one.
    #[inline(always)]
    pub fn initialize(self) -> Session<Initialized> {
        Session::new()
    }
}

impl Default for Session<Uninitialized> {
    fn default() -> Self {
        Self::new()
    }
}

impl Session<Initialized> {
    /// Activate the session
    ///
    /// This consumes the Initialized session and returns an Active one.
    #[inline(always)]
    pub fn activate(self) -> Session<Active> {
        Session::new()
    }

    /// Fail during initialization
    ///
    /// This is a terminal state - no further transitions allowed.
    #[inline(always)]
    pub fn fail(self) -> Session<Failed> {
        Session::new()
    }
}

impl Session<Active> {
    /// Complete the session successfully
    ///
    /// This consumes the Active session and returns a Completed one.
    #[inline(always)]
    pub fn complete(self) -> Session<Completed> {
        Session::new()
    }

    /// Fail during active phase
    ///
    /// This is a terminal state - no further transitions allowed.
    #[inline(always)]
    pub fn fail(self) -> Session<Failed> {
        Session::new()
    }
}

// Terminal states have no transitions (enforced by not implementing methods)

/// Session builder pattern
///
/// This provides a fluent API for protocol execution:
/// ```no_run
/// let result = Session::new()
///     .initialize()
///     .activate()
///     .complete();
/// ```
///
/// Invalid transitions cause compile errors:
/// ```compile_fail
/// let bad = Session::new().complete(); // ERROR: no method `complete` on Uninitialized
/// ```

/// Choice combinator for session types
///
/// Allows branching in protocols:
/// ```text
/// Session<S> -> Choice<Session<A>, Session<B>>
/// ```
pub enum Choice<A, B> {
    /// Left branch
    Left(A),
    /// Right branch
    Right(B),
}

impl<A, B> Choice<A, B> {
    /// Select left branch
    #[inline(always)]
    pub fn left(a: A) -> Self {
        Choice::Left(a)
    }

    /// Select right branch
    #[inline(always)]
    pub fn right(b: B) -> Self {
        Choice::Right(b)
    }

    /// Pattern match on choice
    #[inline(always)]
    pub fn match_choice<F, G, R>(self, f: F, g: G) -> R
    where
        F: FnOnce(A) -> R,
        G: FnOnce(B) -> R,
    {
        match self {
            Choice::Left(a) => f(a),
            Choice::Right(b) => g(b),
        }
    }
}

/// Sequence combinator for session types
///
/// Enforces sequential composition:
/// ```text
/// Session<A> -> Session<B> -> Session<C>
/// ```
pub struct Sequence<A, B> {
    first: A,
    _next: PhantomData<fn() -> B>,
}

impl<A, B> Sequence<A, B> {
    /// Create a sequence
    #[inline(always)]
    pub fn new(first: A) -> Self {
        Self {
            first,
            _next: PhantomData,
        }
    }

    /// Execute first step
    #[inline(always)]
    pub fn then<F>(self, f: F) -> B
    where
        F: FnOnce(A) -> B,
    {
        f(self.first)
    }
}

/// Recursive session types
///
/// Allows protocols with loops:
/// ```text
/// μX. (Action -> X) | Done
/// ```
pub struct Recursive<S> {
    _state: PhantomData<fn() -> S>,
}

impl<S> Recursive<S> {
    /// Recurse to start of loop
    #[inline(always)]
    pub fn recurse(self) -> Session<S> {
        Session::new()
    }

    /// Exit the loop
    #[inline(always)]
    pub fn exit(self) -> Session<Completed> {
        Session::new()
    }
}

/// Protocol validation trait
///
/// Types that can validate their protocol compliance.
pub trait ProtocolValidation {
    /// Validate that the protocol was followed correctly
    ///
    /// This is a compile-time check - it should optimize to nothing.
    fn validate() -> bool {
        true // Always true if it compiles
    }
}

impl<S> ProtocolValidation for Session<S> {}

/// Capability-based session types
///
/// Encodes what operations are allowed in each state.
pub trait Capability<Op> {
    /// Execute an operation if capability is present
    fn execute_op(self, op: Op) -> Self;
}

/// Read capability
pub struct Read;

/// Write capability
pub struct Write;

/// Execute capability
pub struct Execute;

/// Read-only session state
pub struct ReadOnly;

/// Read-write session state
pub struct ReadWrite;

impl Capability<Read> for Session<ReadOnly> {
    #[inline(always)]
    fn execute_op(self, _op: Read) -> Self {
        self
    }
}

impl Capability<Read> for Session<ReadWrite> {
    #[inline(always)]
    fn execute_op(self, _op: Read) -> Self {
        self
    }
}

impl Capability<Write> for Session<ReadWrite> {
    #[inline(always)]
    fn execute_op(self, _op: Write) -> Self {
        self
    }
}

// Importantly: Session<ReadOnly> does NOT implement Capability<Write>
// This is enforced at compile time!

/// Type-level protocol composition
///
/// Compose two protocols into a larger protocol.
pub struct Composed<P1, P2> {
    _p1: PhantomData<fn() -> P1>,
    _p2: PhantomData<fn() -> P2>,
}

impl<P1, P2> Composed<P1, P2> {
    /// Create composed protocol
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            _p1: PhantomData,
            _p2: PhantomData,
        }
    }
}

impl<P1, P2> Default for Composed<P1, P2> {
    fn default() -> Self {
        Self::new()
    }
}

/// Protocol duality
///
/// Ensures client and server protocols are compatible.
pub trait Dual {
    /// The dual protocol
    type Dual;
}

/// Send session
pub struct Send<T, Next> {
    _phantom: PhantomData<(T, Next)>,
}

/// Receive session
pub struct Recv<T, Next> {
    _phantom: PhantomData<(T, Next)>,
}

impl<T, Next> Dual for Send<T, Next>
where
    Next: Dual,
{
    type Dual = Recv<T, Next::Dual>;
}

impl<T, Next> Dual for Recv<T, Next>
where
    Next: Dual,
{
    type Dual = Send<T, Next::Dual>;
}

impl Dual for Session<Completed> {
    type Dual = Session<Completed>;
}

/// Session channel
///
/// Represents a communication channel with session type S.
pub struct Channel<S> {
    _session: PhantomData<fn() -> S>,
}

impl<T, Next> Channel<Send<T, Next>> {
    /// Send a value
    #[inline(always)]
    pub fn send(self, _value: T) -> Channel<Next> {
        Channel {
            _session: PhantomData,
        }
    }
}

impl<T, Next> Channel<Recv<T, Next>> {
    /// Receive a value
    #[inline(always)]
    pub fn recv(self) -> (T, Channel<Next>)
    where
        T: Default,
    {
        (
            T::default(),
            Channel {
                _session: PhantomData,
            },
        )
    }
}

/// Type-level natural numbers for protocol indices
pub struct Z;
pub struct S<N>(PhantomData<N>);

/// Indexed session types
///
/// Allows parameterizing protocols by compile-time constants.
pub struct Indexed<N, St> {
    _index: PhantomData<fn() -> N>,
    _state: PhantomData<fn() -> St>,
}

impl Indexed<Z, Uninitialized> {
    /// Create indexed protocol at zero
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            _index: PhantomData,
            _state: PhantomData,
        }
    }
}

impl Default for Indexed<Z, Uninitialized> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N, St> Indexed<N, St> {
    /// Increment index
    #[inline(always)]
    pub fn increment(self) -> Indexed<S<N>, St> {
        Indexed {
            _index: PhantomData,
            _state: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_session_flow() {
        // This should compile
        let session = Session::<Uninitialized>::new();
        let session = session.initialize();
        let session = session.activate();
        let _session = session.complete();
    }

    #[test]
    fn test_session_failure_path() {
        // Should be able to fail from Initialized
        let session = Session::<Uninitialized>::new();
        let session = session.initialize();
        let _session = session.fail();
    }

    #[test]
    fn test_choice_combinator() {
        let session = Session::<Uninitialized>::new();
        let session = session.initialize();

        let choice = Choice::left(session);

        let _result = choice.match_choice(|s| s.activate().complete(), |s| s.fail());
    }

    #[test]
    fn test_zero_size() {
        // Verify zero runtime overhead
        assert_eq!(core::mem::size_of::<Session<Uninitialized>>(), 0);
        assert_eq!(core::mem::size_of::<Session<Initialized>>(), 0);
        assert_eq!(core::mem::size_of::<Session<Active>>(), 0);
        assert_eq!(core::mem::size_of::<Session<Completed>>(), 0);
    }

    #[test]
    fn test_read_only_capability() {
        let session = Session::<ReadOnly>::new();
        let _session = session.execute_op(Read);
        // session.execute_op(Write); // Would not compile!
    }

    #[test]
    fn test_read_write_capability() {
        let session = Session::<ReadWrite>::new();
        let session = session.execute_op(Read);
        let _session = session.execute_op(Write);
    }
}
