// rust/knhk-cli/src/main.rs
// KNHKS CLI - Main Entry Point
// Noun-Verb Command Interface based on CONVO.txt API

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

mod commands;
mod connector;
mod dependency;
mod error;
mod hook_registry;
mod lockchain;
mod receipt_store;
mod state;
mod tracing;
mod validation;

// Import all noun modules so their verbs are auto-discovered
mod admit;
mod boot;
mod config;
mod conformance;
mod connect;
mod context;
mod cover;
mod coverage;
mod epoch;
mod fortune5;
mod insights;
mod metrics;
mod mining;
mod patterns;
mod pipeline;
mod reflex;
mod route;
mod soundness;
mod workflow;

use clap_noun_verb::Result as CnvResult;
use knhk_config::{load_config, Config};

// Global configuration (loaded at startup)
static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();

fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        match load_config(None) {
            Ok(config) => {
                // Record configuration load metric
                #[cfg(feature = "otel")]
                {
                    use knhk_otel::{MetricsHelper, Tracer};
                    let mut tracer = Tracer::new();
                    MetricsHelper::record_config_load(&mut tracer, "file");
                }
                config
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to load configuration: {}. Using defaults.",
                    e
                );
                // Record configuration error metric
                #[cfg(feature = "otel")]
                {
                    use knhk_otel::{MetricsHelper, Tracer};
                    let mut tracer = Tracer::new();
                    MetricsHelper::record_config_error(&mut tracer, "load_failed");
                }
                Config::default()
            }
        }
    })
}

fn main() -> CnvResult<()> {
    // Initialize OpenTelemetry tracing first (before any other operations)
    // The guard is kept alive for the duration of the program
    let _otel_guard = match tracing::init_tracing() {
        Ok(guard) => {
            // Guard is dropped at end of main, which flushes telemetry
            guard
        }
        Err(e) => {
            eprintln!("Warning: Failed to initialize tracing: {}", e);
            None
        }
    };

    // Load configuration at startup
    let _ = get_config();

    // Auto-discover all registered commands and run
    // CNV v3.3.0 automatically discovers all #[verb] functions
    // Nouns are auto-inferred from filenames (boot.rs → "boot", etc.)
    clap_noun_verb::run()
}
