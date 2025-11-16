//! Integration Test Suite for KNHK Self-Executing Workflows
//!
//! End-to-end integration tests validating complete workflow lifecycle:
//! - Ontology → Code → Execution → Telemetry pipeline
//! - MAPE-K feedback loops
//! - Process mining integration
//!
//! ## Test Organization
//!
//! - `workflow_end_to_end_test` - Complete workflow execution pipeline
//! - `mape_k_feedback_loop_test` - Autonomic MAPE-K cycles
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all integration tests
//! cargo test --test integration
//!
//! # Run specific integration test
//! cargo test --test workflow_end_to_end_test
//! cargo test --test mape_k_feedback_loop_test
//!
//! # Run with output
//! cargo test --test integration -- --nocapture
//! ```

mod mape_k_feedback_loop_test;
mod workflow_end_to_end_test;
