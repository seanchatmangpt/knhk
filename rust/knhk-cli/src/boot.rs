//! Boot commands - System initialization

// Allow non_upper_case_globals - #[verb] macro generates static vars with lowercase names
#![allow(non_upper_case_globals)]

use crate::commands::boot as boot_impl;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[cfg(feature = "otel")]
use tracing::instrument;

#[derive(Serialize, Debug)]
struct InitResult {
    sigma: String,
    q: String,
    config_dir: String,
}

/// Initialize Σ and Q
#[cfg_attr(feature = "otel", instrument(skip_all, fields(operation = "knhk.boot.init", sigma = %sigma, q = %q)))]
#[verb] // Noun "boot" auto-inferred from filename "boot.rs"
fn init(sigma: String, q: String) -> Result<InitResult> {
    #[cfg(feature = "otel")]
    {
        use std::time::Instant;
        use tracing::{error, info};

        let start = Instant::now();
        let result = boot_impl::init(sigma.clone(), q.clone()).map_err(|e| {
            error!(error = %e, "boot.init.failed");
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to initialize: {}", e))
        });

        let duration = start.elapsed();
        match &result {
            Ok(ref config_dir) => {
                info!(
                    duration_ms = duration.as_millis(),
                    config_dir = %config_dir.display(),
                    "boot.init.success"
                );
            }
            Err(_) => {
                error!(duration_ms = duration.as_millis(), "boot.init.failed");
            }
        }

        let config_dir = result?;
        Ok(InitResult {
            sigma,
            q,
            config_dir: config_dir.to_string_lossy().to_string(),
        })
    }

    #[cfg(not(feature = "otel"))]
    {
        let config_dir = boot_impl::init(sigma.clone(), q.clone()).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to initialize: {}", e))
        })?;

        Ok(InitResult {
            sigma,
            q,
            config_dir: config_dir.to_string_lossy().to_string(),
        })
    }
}
