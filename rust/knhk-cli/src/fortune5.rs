//! Fortune 5 commands - Fortune 5 readiness validation

// Allow non_upper_case_globals - #[verb] macro generates static vars with lowercase names
#![allow(non_upper_case_globals)]

#[cfg(feature = "fortune5")]
use crate::commands::fortune5 as fortune5_impl;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// Re-export types from implementation
// TestResult and TestSummary are defined in commands/fortune5.rs
#[cfg(feature = "fortune5")]
pub use fortune5_impl::{TestResult, TestSummary};

/// Run all Fortune 5 tests
#[verb] // Noun "fortune5" auto-inferred from filename "fortune5.rs"
#[cfg(feature = "fortune5")]
fn test() -> Result<TestSummary> {
    #[cfg(feature = "fortune5")]
    {
        fortune5_impl::run_all_tests().map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to run Fortune 5 tests: {}",
                e
            ))
        })
    }
    #[cfg(not(feature = "fortune5"))]
    {
        Err(clap_noun_verb::NounVerbError::execution_error(
            "Fortune 5 feature not enabled. Build with --features fortune5".to_string(),
        ))
    }
}

/// Run tests for a specific category
#[verb]
#[cfg(feature = "fortune5")]
fn test_category(category: String) -> Result<TestSummary> {
    #[cfg(feature = "fortune5")]
    {
        fortune5_impl::run_category_tests(&category).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to run {} tests: {}",
                category, e
            ))
        })
    }
    #[cfg(not(feature = "fortune5"))]
    {
        Err(clap_noun_verb::NounVerbError::execution_error(
            "Fortune 5 feature not enabled. Build with --features fortune5".to_string(),
        ))
    }
}

/// Validate Fortune 5 configuration
#[verb]
#[cfg(feature = "fortune5")]
fn validate() -> Result<String> {
    #[cfg(feature = "fortune5")]
    {
        fortune5_impl::validate_config()
            .map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Fortune 5 validation failed: {}",
                    e
                ))
            })
            .map(|msg| format!("✓ {}", msg))
    }
    #[cfg(not(feature = "fortune5"))]
    {
        Err(clap_noun_verb::NounVerbError::execution_error(
            "Fortune 5 feature not enabled. Build with --features fortune5".to_string(),
        ))
    }
}

/// Show Fortune 5 status
#[verb]
fn status() -> Result<String> {
    #[cfg(feature = "fortune5")]
    {
        fortune5::show_status().map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to get Fortune 5 status: {}",
                e
            ))
        })
    }
    #[cfg(not(feature = "fortune5"))]
    {
        Err(clap_noun_verb::NounVerbError::execution_error(
            "Fortune 5 feature not enabled. Build with --features fortune5".to_string(),
        ))
    }
}
