//! KNHK LaTeX CLI
//!
//! A CLI tool for compiling and managing LaTeX documents using clap-noun-verb.

// CRITICAL: Enforce proper error handling - no unwrap/expect in production code
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
// Allow acceptable warnings for clean build
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

pub mod cleaner;
pub mod compiler;
pub mod mermaid;
pub mod validator;

// Import noun module so verbs are auto-discovered
mod latex;

use clap_noun_verb::Result;

fn main() -> Result<()> {
    clap_noun_verb::run()
}
