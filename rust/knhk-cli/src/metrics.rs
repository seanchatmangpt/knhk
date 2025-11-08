//! Metrics commands - Metrics operations

// Allow non_upper_case_globals - #[verb] macro generates static vars with lowercase names
#![allow(non_upper_case_globals)]

use crate::commands::metrics as metrics_impl;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[cfg(feature = "otel")]
use tracing::instrument;

#[derive(Serialize, Debug)]
struct MetricsResult {
    metrics: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Debug)]
struct WeaverStartResult {
    endpoint: String,
    admin_port: u16,
    process_id: Option<u32>,
}

#[derive(Serialize, Debug)]
struct WeaverValidateResult {
    compliant: bool,
    violations: u32,
    message: String,
}

/// Get metrics
#[cfg_attr(
    feature = "otel",
    instrument(skip_all, fields(operation = "knhk.metrics.get"))
)]
#[verb] // Noun "metrics" auto-inferred from filename "metrics.rs"
fn get() -> Result<MetricsResult> {
    #[cfg(feature = "otel")]
    {
        use std::time::Instant;
        use tracing::info;

        let start = Instant::now();
        let result = metrics_impl::get().map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to get metrics: {}", e))
        });

        let duration = start.elapsed();
        if let Ok(ref metrics) = &result {
            info!(
                duration_ms = duration.as_millis(),
                metric_count = metrics.len(),
                "metrics.get.success"
            );
        }

        result.map(|metrics| MetricsResult { metrics })
    }

    #[cfg(not(feature = "otel"))]
    {
        metrics_impl::get()
            .map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to get metrics: {}",
                    e
                ))
            })
            .map(|metrics| MetricsResult { metrics })
    }
}

/// Start Weaver live-check
#[cfg_attr(
    feature = "otel",
    instrument(skip_all, fields(operation = "knhk.metrics.weaver.start"))
)]
#[verb]
fn weaver_start(
    registry: Option<String>,
    otlp_port: Option<u16>,
    admin_port: Option<u16>,
    format: Option<String>,
    output: Option<String>,
) -> Result<WeaverStartResult> {
    #[cfg(feature = "otel")]
    {
        use std::time::Instant;
        use tracing::{error, info};

        let start = Instant::now();
        let result = metrics_impl::weaver_start(registry, otlp_port, admin_port, format, output);

        let duration = start.elapsed();
        match &result {
            Ok((ref endpoint, ref admin_port, ref process_id)) => {
                info!(
                    duration_ms = duration.as_millis(),
                    endpoint = %endpoint,
                    admin_port = admin_port,
                    "weaver.start.success"
                );

                Ok(WeaverStartResult {
                    endpoint: endpoint.clone(),
                    admin_port: *admin_port,
                    process_id: *process_id,
                })
            }
            Err(ref e) => {
                error!(error = %e, duration_ms = duration.as_millis(), "weaver.start.failed");
                Err(clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to start Weaver: {}",
                    e
                )))
            }
        }
    }

    #[cfg(not(feature = "otel"))]
    {
        metrics_impl::weaver_start(registry, otlp_port, admin_port, format, output)
            .map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to start Weaver: {}",
                    e
                ))
            })
            .map(|(endpoint, admin_port, process_id)| WeaverStartResult {
                endpoint,
                admin_port,
                process_id,
            })
    }
}

/// Stop Weaver live-check
#[cfg_attr(
    feature = "otel",
    instrument(skip_all, fields(operation = "knhk.metrics.weaver.stop"))
)]
#[verb]
fn weaver_stop(admin_port: Option<u16>) -> Result<()> {
    #[cfg(feature = "otel")]
    {
        use std::time::Instant;
        use tracing::{error, info};

        let start = Instant::now();
        let result = metrics_impl::weaver_stop(admin_port);

        let duration = start.elapsed();
        match &result {
            Ok(_) => {
                info!(duration_ms = duration.as_millis(), "weaver.stop.success");
            }
            Err(ref e) => {
                error!(error = %e, duration_ms = duration.as_millis(), "weaver.stop.failed");
            }
        }

        result.map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to stop Weaver: {}", e))
        })
    }

    #[cfg(not(feature = "otel"))]
    {
        metrics_impl::weaver_stop(admin_port).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to stop Weaver: {}", e))
        })
    }
}

/// Validate telemetry with Weaver
#[cfg_attr(
    feature = "otel",
    instrument(skip_all, fields(operation = "knhk.metrics.weaver.validate"))
)]
#[verb]
fn weaver_validate(
    registry: Option<String>,
    otlp_port: Option<u16>,
    admin_port: Option<u16>,
    timeout: Option<u64>,
) -> Result<WeaverValidateResult> {
    #[cfg(feature = "otel")]
    {
        use std::time::Instant;
        use tracing::{error, info};

        let start = Instant::now();
        let result = metrics_impl::weaver_validate(registry, otlp_port, admin_port, timeout);

        let duration = start.elapsed();
        match &result {
            Ok((ref compliant, ref violations, ref message)) => {
                info!(
                    duration_ms = duration.as_millis(),
                    compliant = compliant,
                    violations = violations,
                    "weaver.validate.completed"
                );

                Ok(WeaverValidateResult {
                    compliant: *compliant,
                    violations: *violations,
                    message: message.clone(),
                })
            }
            Err(ref e) => {
                error!(error = %e, duration_ms = duration.as_millis(), "weaver.validate.failed");
                Err(clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to validate with Weaver: {}",
                    e
                )))
            }
        }
    }

    #[cfg(not(feature = "otel"))]
    {
        metrics_impl::weaver_validate(registry, otlp_port, admin_port, timeout)
            .map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to validate with Weaver: {}",
                    e
                ))
            })
            .map(|(compliant, violations, message)| WeaverValidateResult {
                compliant,
                violations,
                message,
            })
    }
}
