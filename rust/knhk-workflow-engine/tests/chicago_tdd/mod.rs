//! Chicago TDD Test Suite for KNHK Autonomic Governance Layer
//!
//! This module contains comprehensive tests following the Chicago School of TDD:
//! - **State-based testing**: Verify system behavior through observable state
//! - **Real collaborators**: Use actual components, not mocks
//! - **Behavior verification**: Test what the system does, not how
//! - **AAA pattern**: Arrange, Act, Assert
//!
//! # Test Organization
//!
//! - `policy_lattice_properties`: Property-based tests for lattice algebra
//! - `counterfactual_snapshots`: Snapshot tests for deterministic replay
//! - `session_concurrency_tests`: Concurrency tests with real atomics
//! - `mode_policy_transitions`: State machine tests for mode transitions
//! - `governance_mutation_tests`: Mutation testing to verify test quality
//! - `performance_constraints`: Performance SLA enforcement tests
//! - `governance_integration_tests`: End-to-end integration scenarios
//!
//! # Testing Philosophy: Chicago vs London
//!
//! **Chicago School (This Suite)**:
//! - Tests verify final state after operations
//! - Uses real objects and their actual behavior
//! - Focuses on system output and observable effects
//! - Example: Create session, increment counter, assert count == 1
//!
//! **London School (Alternative)**:
//! - Tests verify interactions between objects
//! - Uses mocks to isolate units
//! - Focuses on message passing and collaborations
//! - Example: Verify session.increment() was called once
//!
//! # Why Chicago TDD for KNHK?
//!
//! 1. **KNHK's mission**: Eliminate false positives in testing
//!    - Chicago tests verify actual behavior, not test doubles
//!    - Reduces risk of passing tests with broken features
//!
//! 2. **Real performance validation**:
//!    - Tests measure actual latency, not mocked timers
//!    - Verifies Chatman Constant compliance with real code
//!
//! 3. **Concurrency correctness**:
//!    - Tests real atomic operations and race conditions
//!    - Catches actual synchronization bugs
//!
//! 4. **Integration confidence**:
//!    - Real components working together
//!    - Detects interface mismatches and integration bugs
//!
//! # Test Quality Metrics
//!
//! - **Coverage**: ≥90% line coverage
//! - **Mutation Score**: ≥80% (mutation tests detect buggy mutations)
//! - **Performance**: All hot paths within SLA budgets
//! - **Concurrency**: Zero data races (tested with real threads)
//!
//! # Running Tests
//!
//! ```bash
//! # Run all governance tests
//! cargo test --test chicago_tdd
//!
//! # Run specific test module
//! cargo test --test chicago_tdd policy_lattice
//!
//! # Run with output
//! cargo test --test chicago_tdd -- --nocapture
//!
//! # Run performance tests
//! cargo test --test chicago_tdd performance --release
//! ```
//!
//! # AAA Pattern Example
//!
//! ```rust
//! #[test]
//! fn test_session_lifecycle() {
//!     // Arrange: Set up test data
//!     let table = SessionTable::new();
//!     let session_id = SessionId::new();
//!
//!     // Act: Perform operation
//!     table.create_session(session_id, case_id, tenant_id);
//!
//!     // Assert: Verify outcome
//!     assert!(table.session_exists(session_id));
//! }
//! ```

// ============================================================================
// Test Modules
// ============================================================================

mod policy_lattice_properties;
mod counterfactual_snapshots;
mod session_concurrency_tests;
mod mode_policy_transitions;
mod governance_mutation_tests;
mod performance_constraints;
mod governance_integration_tests;

// ============================================================================
// Re-exports for Convenience
// ============================================================================

pub use policy_lattice_properties::*;
pub use counterfactual_snapshots::*;
pub use session_concurrency_tests::*;
pub use mode_policy_transitions::*;
pub use governance_mutation_tests::*;
pub use performance_constraints::*;
pub use governance_integration_tests::*;
