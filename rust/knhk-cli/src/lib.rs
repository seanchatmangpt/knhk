//! KNHK CLI Library - Exposed for testing
//!
//! This library exposes the CLI modules for testing purposes.
//! The main binary is in `main.rs`.

// CRITICAL: Enforce proper error handling - no unwrap/expect in production code
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
// Allow acceptable warnings for clean build
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(deprecated)] // Some dependencies use deprecated APIs (will be updated)
#![allow(unexpected_cfgs)] // Some cfg values are informational (policy-engine, network, tempfile)

pub mod commands;
pub mod connector;
pub mod dependency;
pub mod error;
pub mod hook_registry;
pub mod lockchain;
pub mod receipt_store;
pub mod state;
pub mod tracing;
pub mod validation;

// Re-export command modules for testing
pub mod admit;
pub mod boot;
pub mod config;
pub mod connect;
pub mod context;
pub mod cover;
pub mod coverage;
pub mod epoch;
#[cfg(feature = "fortune5")]
pub mod fortune5;
pub mod metrics;
pub mod pipeline;
pub mod reflex;
pub mod route;
