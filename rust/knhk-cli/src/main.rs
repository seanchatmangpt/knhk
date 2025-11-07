// rust/knhk-cli/src/main.rs
// KNHKS CLI - Main Entry Point
// Noun-Verb Command Interface based on CONVO.txt API

// CRITICAL: Enforce proper error handling - no unwrap/expect in production code
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

mod commands;
mod error;
mod tracing;

// Import all noun modules so their verbs are auto-discovered
mod boot;
mod connect;
mod cover;
mod admit;
mod reflex;
mod epoch;
mod route;
mod receipt;
mod pipeline;
mod metrics;
mod coverage;
mod context;
mod config;
mod hook;

use clap_noun_verb::Result as CnvResult;
use knhk_config::{load_config, config::KnhkConfig, Config};

// Global configuration (loaded at startup)
static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();

fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        match load_config(None) {
            Ok(config) => {
                // Record configuration load metric
                #[cfg(feature = "otel")]
                {
                    use knhk_otel::{Tracer, MetricsHelper};
                    let mut tracer = Tracer::new();
                    MetricsHelper::record_config_load(&mut tracer, "file");
                }
                config
            }
            Err(e) => {
                eprintln!("Warning: Failed to load configuration: {}. Using defaults.", e);
                // Record configuration error metric
                #[cfg(feature = "otel")]
                {
                    use knhk_otel::{Tracer, MetricsHelper};
                    let mut tracer = Tracer::new();
                    MetricsHelper::record_config_error(&mut tracer, "load_failed");
                }
                Config::default()
            }
        }
    })
}

fn main() -> CnvResult<()> {
    // Initialize tracing first (before any other operations)
    if let Err(e) = tracing::init_tracing() {
        eprintln!("Warning: Failed to initialize tracing: {}", e);
    }

    // Load configuration at startup
    let _ = get_config();
    
    // Auto-discover all registered commands and run
    // CNV v3.3.0 automatically discovers all #[verb] functions
    // Nouns are auto-inferred from filenames (boot.rs → "boot", etc.)
    clap_noun_verb::run()
}
