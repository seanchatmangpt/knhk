//! Type system integration tests
//!
//! Comprehensive tests for type-system-v2 features including GATs, phantom types,
//! newtypes, HRTBs, and zero-cost abstractions.

#![cfg(test)]

mod gat_tests;
mod hrtb_tests;
mod newtype_tests;
mod phantom_tests;
mod type_state_tests;
mod zero_cost_tests;
