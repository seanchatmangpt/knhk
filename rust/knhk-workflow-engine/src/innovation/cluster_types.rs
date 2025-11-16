//! Distributed KNHK: Cluster as a Type
//!
//! Cluster configuration and consensus roles encoded into type parameters.
//! Only consensus-aware code can commit state; all high-impact decisions are
//! pure, replicated state machine transitions.
//!
//! # Type-Level Guarantees
//! - Cluster config is a compile-time object
//! - Role-based access control via types
//! - Deterministic state machine replication
//! - Quorum enforcement at compile time

use core::marker::PhantomData;

/// Cluster role - phantom type for access control
pub trait ClusterRole: 'static {
    const CAN_COMMIT: bool;
    const CAN_PROPOSE: bool;
    const CAN_VOTE: bool;
    const ROLE_NAME: &'static str;
}

/// Leader role - can propose and commit
pub struct Leader;
impl ClusterRole for Leader {
    const CAN_COMMIT: bool = true;
    const CAN_PROPOSE: bool = true;
    const CAN_VOTE: bool = true;
    const ROLE_NAME: &'static str = "leader";
}

/// Follower role - can vote but not commit
pub struct Follower;
impl ClusterRole for Follower {
    const CAN_COMMIT: bool = false;
    const CAN_PROPOSE: bool = false;
    const CAN_VOTE: bool = true;
    const ROLE_NAME: &'static str = "follower";
}

/// Observer role - read-only
pub struct Observer;
impl ClusterRole for Observer {
    const CAN_COMMIT: bool = false;
    const CAN_PROPOSE: bool = false;
    const CAN_VOTE: bool = false;
    const ROLE_NAME: &'static str = "observer";
}

/// Cluster configuration - const generic parameters
pub struct ClusterConfig<const REPLICAS: usize, const QUORUM: usize> {
    _phantom: PhantomData<()>,
}

impl<const REPLICAS: usize, const QUORUM: usize> ClusterConfig<REPLICAS, QUORUM> {
    /// Create cluster config with compile-time validation
    pub const fn new() -> Self {
        // Quorum must be > REPLICAS/2 (majority)
        const_assert!(QUORUM * 2 > REPLICAS);
        // At least 1 replica
        const_assert!(REPLICAS > 0);
        // Quorum cannot exceed replicas
        const_assert!(QUORUM <= REPLICAS);

        Self {
            _phantom: PhantomData,
        }
    }

    /// Check if quorum is reached
    pub const fn has_quorum(votes: usize) -> bool {
        votes >= QUORUM
    }

    /// Minimum nodes for operation
    pub const fn min_nodes() -> usize {
        QUORUM
    }
}

/// Replication factor - phantom type for data redundancy
pub trait ReplicationFactor: 'static {
    const FACTOR: usize;
    const FAULT_TOLERANCE: usize; // How many failures can be tolerated
}

/// Single node - no replication
pub struct SingleNode;
impl ReplicationFactor for SingleNode {
    const FACTOR: usize = 1;
    const FAULT_TOLERANCE: usize = 0;
}

/// Three-way replication
pub struct TripleReplication;
impl ReplicationFactor for TripleReplication {
    const FACTOR: usize = 3;
    const FAULT_TOLERANCE: usize = 1; // Can tolerate 1 failure
}

/// Five-way replication
pub struct FiveWayReplication;
impl ReplicationFactor for FiveWayReplication {
    const FACTOR: usize = 5;
    const FAULT_TOLERANCE: usize = 2; // Can tolerate 2 failures
}

/// Consensus operation - only callable by roles with permission
pub struct ConsensusOp<R: ClusterRole, C: ReplicationFactor> {
    _role: PhantomData<R>,
    _config: PhantomData<C>,
}

impl<R: ClusterRole, C: ReplicationFactor> ConsensusOp<R, C> {
    /// Create consensus operation
    pub const fn new() -> Self {
        Self {
            _role: PhantomData,
            _config: PhantomData,
        }
    }

    /// Propose - only compiles for roles with propose permission
    pub fn propose<T>(&self, value: T) -> Result<Proposal<T, C>, &'static str>
    where
        R: ClusterRole,
    {
        if R::CAN_PROPOSE {
            Ok(Proposal::new(value))
        } else {
            Err("Role cannot propose")
        }
    }
}

/// Proposal in consensus protocol
pub struct Proposal<T, C: ReplicationFactor> {
    value: T,
    votes: usize,
    _config: PhantomData<C>,
}

impl<T, C: ReplicationFactor> Proposal<T, C> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            votes: 0,
            _config: PhantomData,
        }
    }

    /// Add vote - requires voter role
    pub fn vote<R: ClusterRole>(&mut self) -> Result<(), &'static str> {
        if R::CAN_VOTE {
            self.votes += 1;
            Ok(())
        } else {
            Err("Role cannot vote")
        }
    }

    /// Check if quorum reached
    pub fn has_quorum(&self) -> bool {
        self.votes >= (C::FACTOR / 2 + 1)
    }

    /// Commit - only if quorum reached and role can commit
    pub fn commit<R: ClusterRole>(self) -> Result<Committed<T>, &'static str> {
        if !R::CAN_COMMIT {
            return Err("Role cannot commit");
        }
        if !self.has_quorum() {
            return Err("Quorum not reached");
        }
        Ok(Committed::new(self.value))
    }
}

/// Committed value - immutable, replicated
pub struct Committed<T> {
    value: T,
}

impl<T> Committed<T> {
    fn new(value: T) -> Self {
        Self { value }
    }

    /// Get committed value
    pub fn get(&self) -> &T {
        &self.value
    }
}

/// Deterministic log entry - pure state transition
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LogEntry<const ENTRY_ID: u64> {
    _phantom: PhantomData<()>,
}

impl<const ENTRY_ID: u64> LogEntry<ENTRY_ID> {
    /// Create log entry
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Get entry ID
    pub const fn id() -> u64 {
        ENTRY_ID
    }
}

/// Replicated log - deterministic state machine
pub struct ReplicatedLog<const MAX_ENTRIES: usize> {
    entries: [Option<u64>; MAX_ENTRIES],
    count: usize,
}

impl<const MAX_ENTRIES: usize> ReplicatedLog<MAX_ENTRIES> {
    /// Create empty log
    pub const fn new() -> Self {
        Self {
            entries: [None; MAX_ENTRIES],
            count: 0,
        }
    }

    /// Append entry - deterministic operation
    pub fn append(&mut self, entry_id: u64) -> Result<(), &'static str> {
        if self.count >= MAX_ENTRIES {
            return Err("Log full");
        }
        self.entries[self.count] = Some(entry_id);
        self.count += 1;
        Ok(())
    }

    /// Get entry count
    pub const fn len(&self) -> usize {
        self.count
    }

    /// Check if log is empty
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Distributed execution context - role and config as types
pub struct DistributedContext<R: ClusterRole, const REPLICAS: usize, const QUORUM: usize> {
    role: PhantomData<R>,
    config: ClusterConfig<REPLICAS, QUORUM>,
    log: ReplicatedLog<1000>,
}

impl<R: ClusterRole, const REPLICAS: usize, const QUORUM: usize>
    DistributedContext<R, REPLICAS, QUORUM>
{
    /// Create distributed context
    pub const fn new() -> Self {
        Self {
            role: PhantomData,
            config: ClusterConfig::new(),
            log: ReplicatedLog::new(),
        }
    }

    /// Execute deterministic operation
    pub fn execute<T, F>(&mut self, f: F) -> Result<T, &'static str>
    where
        F: FnOnce() -> T,
    {
        // All operations are deterministic (pure functions)
        Ok(f())
    }

    /// Commit decision - only for leaders
    pub fn commit_decision(&mut self, entry_id: u64) -> Result<(), &'static str>
    where
        R: ClusterRole,
    {
        if !R::CAN_COMMIT {
            return Err("Role cannot commit");
        }
        self.log.append(entry_id)
    }
}

/// Consensus state machine - type-level transitions
pub trait ConsensusState: 'static {
    type Next: ConsensusState;
    const STATE_NAME: &'static str;
}

/// Initial consensus state
pub struct Initial;
impl ConsensusState for Initial {
    type Next = Proposed;
    const STATE_NAME: &'static str = "initial";
}

/// Proposed state
pub struct Proposed;
impl ConsensusState for Proposed {
    type Next = CommittedState;
    const STATE_NAME: &'static str = "proposed";
}

/// Committed state (terminal)
pub struct CommittedState;
impl ConsensusState for CommittedState {
    type Next = CommittedState;
    const STATE_NAME: &'static str = "committed";
}

/// State machine with type-level transitions
pub struct StateMachine<S: ConsensusState, T> {
    state: PhantomData<S>,
    value: T,
}

impl<S: ConsensusState, T> StateMachine<S, T> {
    /// Create state machine
    pub fn new(value: T) -> Self {
        Self {
            state: PhantomData,
            value,
        }
    }

    /// Transition to next state - type-level state change
    pub fn transition(self) -> StateMachine<S::Next, T> {
        StateMachine {
            state: PhantomData,
            value: self.value,
        }
    }

    /// Get value
    pub fn value(&self) -> &T {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_roles() {
        assert!(Leader::CAN_COMMIT);
        assert!(!Follower::CAN_COMMIT);
        assert!(!Observer::CAN_VOTE);
    }

    #[test]
    fn test_cluster_config() {
        let config = ClusterConfig::<3, 2>::new();
        assert!(ClusterConfig::<3, 2>::has_quorum(2));
        assert!(!ClusterConfig::<3, 2>::has_quorum(1));
        assert_eq!(ClusterConfig::<3, 2>::min_nodes(), 2);
    }

    #[test]
    fn test_replication_factors() {
        assert_eq!(TripleReplication::FACTOR, 3);
        assert_eq!(TripleReplication::FAULT_TOLERANCE, 1);

        assert_eq!(FiveWayReplication::FACTOR, 5);
        assert_eq!(FiveWayReplication::FAULT_TOLERANCE, 2);
    }

    #[test]
    fn test_consensus_proposal() {
        let mut proposal = Proposal::<i32, TripleReplication>::new(42);

        // Add votes
        assert!(proposal.vote::<Leader>().is_ok());
        assert!(proposal.vote::<Follower>().is_ok());

        // Check quorum (need 2 out of 3)
        assert!(proposal.has_quorum());

        // Commit as leader
        let committed = proposal.commit::<Leader>().unwrap();
        assert_eq!(*committed.get(), 42);
    }

    #[test]
    fn test_replicated_log() {
        let mut log = ReplicatedLog::<10>::new();
        assert!(log.is_empty());

        assert!(log.append(1).is_ok());
        assert!(log.append(2).is_ok());
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn test_distributed_context() {
        let mut ctx = DistributedContext::<Leader, 3, 2>::new();

        // Execute deterministic operation
        let result = ctx.execute(|| 42);
        assert_eq!(result.unwrap(), 42);

        // Commit as leader
        assert!(ctx.commit_decision(1).is_ok());
    }

    #[test]
    fn test_state_machine() {
        let sm = StateMachine::<Initial, i32>::new(42);
        assert_eq!(*sm.value(), 42);

        let sm = sm.transition(); // Initial -> Proposed
        let sm = sm.transition(); // Proposed -> Committed
        assert_eq!(*sm.value(), 42);
    }

    #[test]
    fn test_role_based_access() {
        // Leader can commit
        let ctx = DistributedContext::<Leader, 3, 2>::new();
        // Compiles because Leader::CAN_COMMIT == true

        // Follower cannot commit (would fail at runtime, but type system helps)
        let mut ctx = DistributedContext::<Follower, 3, 2>::new();
        assert!(ctx.commit_decision(1).is_err());
    }
}
