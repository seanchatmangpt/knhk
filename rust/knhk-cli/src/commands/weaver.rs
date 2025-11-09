//! Weaver Live-Check CLI Commands
//!
//! Provides CLI interface for Weaver live-check validation:
//! - weaver start: Start Weaver live-check server
//! - weaver stop: Stop Weaver live-check server
//! - weaver validate: Run Weaver validation on telemetry
//! - weaver check: Check Weaver binary availability

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use std::path::PathBuf;

#[cfg(feature = "otel")]
use knhk_otel::WeaverLiveCheck;

/// Start Weaver live-check server
///
/// Usage: knhk weaver start [--registry <path>] [--otlp-port <port>] [--admin-port <port>] [--output <path>]
#[verb("weaver start")]
#[cfg(feature = "otel")]
pub fn weaver_start(
    registry: Option<PathBuf>,
    otlp_port: Option<u16>,
    admin_port: Option<u16>,
    output: Option<PathBuf>,
) -> CnvResult<()> {
    use crate::commands::metrics::weaver_start;

    let registry_str = registry
        .as_ref()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string());

    let output_str = output
        .as_ref()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string());

    let (endpoint, admin_port, _process_id) = weaver_start(
        registry_str,
        otlp_port,
        admin_port,
        Some("json".to_string()),
        output_str,
    )
    .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e))?;

    println!("✅ Weaver live-check started");
    println!("   OTLP endpoint: {}", endpoint);
    println!("   Admin port: {}", admin_port);
    println!("\n📡 Export telemetry to: {}", endpoint);
    println!(
        "   Set environment variable: OTEL_EXPORTER_OTLP_ENDPOINT={}",
        endpoint
    );
    println!("\n💡 Run workflow operations to generate telemetry");
    println!("   Press Ctrl+C to stop Weaver and view validation report");

    Ok(())
}

#[verb("weaver start")]
#[cfg(not(feature = "otel"))]
pub fn weaver_start(
    _registry: Option<PathBuf>,
    _otlp_port: Option<u16>,
    _admin_port: Option<u16>,
    _output: Option<PathBuf>,
) -> CnvResult<()> {
    Err(clap_noun_verb::NounVerbError::execution_error(
        "Weaver live-check requires OTEL feature. Build with --features otel".to_string(),
    ))
}

/// Stop Weaver live-check server
///
/// Usage: knhk weaver stop [--admin-port <port>]
#[verb("weaver stop")]
#[cfg(feature = "otel")]
pub fn weaver_stop(admin_port: Option<u16>) -> CnvResult<()> {
    use crate::commands::metrics::weaver_stop;

    weaver_stop(admin_port).map_err(|e| clap_noun_verb::NounVerbError::execution_error(e))?;
    println!("✅ Weaver live-check stopped");
    println!("   Check ./weaver-reports for validation results");

    Ok(())
}

#[verb("weaver stop")]
#[cfg(not(feature = "otel"))]
pub fn weaver_stop(_admin_port: Option<u16>) -> CnvResult<()> {
    Err(clap_noun_verb::NounVerbError::execution_error(
        "Weaver live-check requires OTEL feature. Build with --features otel".to_string(),
    ))
}

/// Validate telemetry with Weaver
///
/// Usage: knhk weaver validate [--registry <path>] [--otlp-port <port>] [--admin-port <port>] [--timeout <seconds>]
#[verb("weaver validate")]
#[cfg(feature = "otel")]
pub fn weaver_validate(
    registry: Option<PathBuf>,
    otlp_port: Option<u16>,
    admin_port: Option<u16>,
    timeout: Option<u64>,
) -> CnvResult<()> {
    use crate::commands::metrics::weaver_validate;

    let registry_str = registry
        .as_ref()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string());

    let (compliant, violations, message) =
        weaver_validate(registry_str, otlp_port, admin_port, timeout)
            .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e))?;

    if compliant {
        println!("✅ Weaver validation PASSED");
        println!("   {}", message);
    } else {
        println!("❌ Weaver validation FAILED");
        println!("   Violations: {}", violations);
        println!("   {}", message);
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "Weaver validation failed with {} violations",
            violations
        )));
    }

    Ok(())
}

#[verb("weaver validate")]
#[cfg(not(feature = "otel"))]
pub fn weaver_validate(
    _registry: Option<PathBuf>,
    _otlp_port: Option<u16>,
    _admin_port: Option<u16>,
    _timeout: Option<u64>,
) -> CnvResult<()> {
    Err(clap_noun_verb::NounVerbError::execution_error(
        "Weaver live-check requires OTEL feature. Build with --features otel".to_string(),
    ))
}

/// Check if Weaver binary is available
///
/// Usage: knhk weaver check
#[verb("weaver check")]
#[cfg(feature = "otel")]
pub fn weaver_check() -> CnvResult<()> {
    match WeaverLiveCheck::check_weaver_available() {
        Ok(_) => {
            println!("✅ Weaver binary is available");
            Ok(())
        }
        Err(e) => {
            println!("❌ Weaver binary not found: {}", e);
            println!("   Install with: cargo install weaver");
            println!("   Or run: ./scripts/install-weaver.sh");
            Err(clap_noun_verb::NounVerbError::execution_error(format!(
                "Weaver binary not available: {}",
                e
            )))
        }
    }
}

#[verb("weaver check")]
#[cfg(not(feature = "otel"))]
pub fn weaver_check() -> CnvResult<()> {
    Err(clap_noun_verb::NounVerbError::execution_error(
        "Weaver live-check requires OTEL feature. Build with --features otel".to_string(),
    ))
}
