//! Chicago TDD Tools
//!
//! A comprehensive testing framework for Chicago TDD (Classicist Test-Driven Development)
//! methodology in Rust. Provides fixtures, builders, helpers, and advanced testing
//! capabilities including property-based testing and mutation testing.
//!
//! ## Features
//!
//! - **Test Fixtures**: Reusable test fixtures with automatic cleanup
//! - **Builders**: Fluent builders for test data and workflows
//! - **Assertion Helpers**: Comprehensive assertion utilities
//! - **Property-Based Testing**: QuickCheck-style random test generation
//! - **Mutation Testing**: Test quality validation through mutations
//! - **Coverage Analysis**: Test coverage reporting and analysis
//!
//! ## Chicago TDD Principles
//!
//! This framework enforces Chicago TDD principles:
//!
//! 1. **State-Based Testing**: Tests verify outputs and state, not implementation
//! 2. **Real Collaborators**: Uses actual dependencies, not mocks
//! 3. **Behavior Verification**: Tests verify what code does, not how
//! 4. **AAA Pattern**: All tests follow Arrange-Act-Assert structure
//!
//! ## Usage
//!
//! ```rust,no_run
//! use chicago_tdd_tools::prelude::*;
//!
//! #[tokio::test]
//! async fn test_example() {
//!     // Arrange: Create fixture
//!     let fixture = TestFixture::new().unwrap_or_else(|e| panic!("Failed to create fixture: {}", e));
//!
//!     // Act: Execute test
//!     let counter = fixture.test_counter();
//!
//!     // Assert: Verify state
//!     assert!(counter >= 0);
//! }
//! ```
//!
//! ## Modules
//!
//! - `fixture`: Test fixtures and setup utilities
//! - `builders`: Fluent builders for test data
//! - `assertions`: Assertion helpers and utilities
//! - `property`: Property-based testing framework
//! - `mutation`: Mutation testing framework
//! - `coverage`: Test coverage analysis
//! - `generator`: Test code generation

#![deny(clippy::unwrap_used)]
#![warn(missing_docs)]

pub mod assertions;
pub mod builders;
pub mod coverage;
pub mod fixture;
pub mod generator;
pub mod mutation;
pub mod property;

/// Prelude module - import commonly used items
pub mod prelude {
    pub use crate::assertions::*;
    pub use crate::builders::*;
    pub use crate::fixture::*;

    #[cfg(feature = "property-testing")]
    pub use crate::property::*;

    #[cfg(feature = "mutation-testing")]
    pub use crate::mutation::*;
}
