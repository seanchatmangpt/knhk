// DFLSS Validation Checks
use anyhow::Context;

// Helper to convert anyhow::Error to clap_noun_verb error
fn to_cnv_error(e: anyhow::Error) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(e.to_string())
}
// Validate metrics against CTQ requirements

use chrono::Utc;
use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub status: String,
    pub message: String,
}

/// Validate Weaver compliance (100% static + live)
#[verb("validation check-weaver")]
pub fn check_weaver(registry: Option<PathBuf>) -> CnvResult<ValidationResult> {
    info!("Validating Weaver compliance...");

    // Collect Weaver metrics
    let registry_path = registry
        .as_ref()
        .map(|p| p.as_os_str().to_string_lossy().to_string())
        .unwrap_or_else(|| "registry/".to_string());

    let rt = tokio::runtime::Runtime::new()
        .context("Failed to create runtime")
        .map_err(to_cnv_error)?;

    // Run Weaver static check
    let weaver_check = rt.block_on(
        tokio::process::Command::new("weaver")
            .args(&["registry", "check", "-r", &registry_path])
            .output(),
    );

    let static_pass = match weaver_check {
        Ok(output) => output.status.success(),
        Err(_) => {
            info!("Weaver command not found");
            false
        }
    };

    // Run Weaver live check
    let weaver_live = rt.block_on(
        tokio::process::Command::new("weaver")
            .args(&["registry", "live-check", "--registry", &registry_path])
            .output(),
    );

    let live_pass = match weaver_live {
        Ok(output) => Some(output.status.success()),
        Err(_) => None,
    };

    // CTQ requirement: 100% static + live
    let compliant = static_pass && live_pass.unwrap_or(false);

    let status = if compliant { "PASS" } else { "FAIL" };

    let message = if compliant {
        "Weaver validation: 100% compliant (static + live)".to_string()
    } else {
        format!(
            "Weaver validation: Static={}, Live={:?}",
            static_pass, live_pass
        )
    };

    info!("Weaver Validation: {}", message);

    Ok(ValidationResult {
        status: status.to_string(),
        message,
    })
}

/// Validate performance compliance (≤8 ticks)
#[verb("validation check-performance")]
pub fn check_performance(data: PathBuf, threshold: f64) -> CnvResult<ValidationResult> {
    info!(
        "Validating performance compliance (threshold: {} ticks)...",
        threshold
    );

    // Load performance data
    let content = std::fs::read_to_string(&data)
        .context("Failed to read performance data file")
        .map_err(to_cnv_error)?;

    // Parse performance data
    use regex::Regex;
    let re = Regex::new(r"(\w+):\s+(\d+(?:\.\d+)?)\s+ticks")
        .context("Failed to compile regex")
        .map_err(to_cnv_error)?;

    let mut operations_under_threshold = 0u32;
    let mut total_operations = 0u32;
    let mut violations = Vec::new();

    for cap in re.captures_iter(&content) {
        let op = cap.get(1).unwrap().as_str().to_string();
        let ticks: f64 = cap
            .get(2)
            .unwrap()
            .as_str()
            .parse()
            .context("Failed to parse tick count")
            .map_err(to_cnv_error)?;

        total_operations += 1;
        if ticks <= threshold {
            operations_under_threshold += 1;
        } else {
            violations.push(format!("{}: {:.2} ticks", op, ticks));
        }
    }

    if total_operations == 0 {
        return Err(to_cnv_error(anyhow::anyhow!("No performance data found")));
    }

    let compliance_rate = (operations_under_threshold as f64 / total_operations as f64) * 100.0;
    let compliant = compliance_rate >= 100.0;

    let status = if compliant { "PASS" } else { "FAIL" };

    let message = if compliant {
        format!(
            "Performance: 100% compliant ({} operations ≤ {} ticks)",
            operations_under_threshold, threshold
        )
    } else {
        format!(
            "Performance: {:.1}% compliant ({} violations: {})",
            compliance_rate,
            violations.len(),
            violations.join(", ")
        )
    };

    info!("Performance Validation: {}", message);

    Ok(ValidationResult {
        status: status.to_string(),
        message,
    })
}

/// Validate code quality (zero unwrap, zero clippy errors)
#[verb("validation check-quality")]
pub fn check_quality(data: Option<PathBuf>) -> CnvResult<ValidationResult> {
    info!("Validating code quality...");

    // Collect quality metrics if not provided
    let quality_metrics = if let Some(data_path) = data {
        let content = std::fs::read_to_string(&data_path)
            .context("Failed to read quality data file")
            .map_err(to_cnv_error)?;
        serde_json::from_str::<crate::internal::quality::QualityMetrics>(&content)
            .context("Failed to parse quality data")
            .map_err(to_cnv_error)?
    } else {
        // Collect metrics
        let collector = crate::internal::quality::QualityCollector::new(PathBuf::from("rust"));
        let rt = tokio::runtime::Runtime::new()
            .context("Failed to create runtime")
            .map_err(to_cnv_error)?;
        rt.block_on(collector.collect())
            .context("Failed to collect quality metrics")
            .map_err(to_cnv_error)?
    };

    // CTQ requirements: zero unwrap, zero clippy errors
    let clippy_compliant = quality_metrics.clippy_errors == 0;
    let unwrap_compliant = quality_metrics.unwrap_count == 0;
    let compliant = clippy_compliant && unwrap_compliant;

    let status = if compliant { "PASS" } else { "FAIL" };

    let mut issues = Vec::new();
    if !clippy_compliant {
        issues.push(format!("{} clippy errors", quality_metrics.clippy_errors));
    }
    if !unwrap_compliant {
        issues.push(format!("{} unwrap() calls", quality_metrics.unwrap_count));
    }

    let message = if compliant {
        "Code quality: 100% compliant (zero clippy errors, zero unwrap)".to_string()
    } else {
        format!(
            "Code quality: {} violations ({})",
            issues.len(),
            issues.join(", ")
        )
    };

    info!("Quality Validation: {}", message);

    Ok(ValidationResult {
        status: status.to_string(),
        message,
    })
}

/// Validate DoD compliance (≥85%)
#[verb("validation check-dod")]
pub fn check_dod(metrics: PathBuf) -> CnvResult<ValidationResult> {
    info!("Validating DoD compliance...");

    // Load metrics file
    let content = std::fs::read_to_string(&metrics)
        .context("Failed to read metrics file")
        .map_err(to_cnv_error)?;

    // Parse metrics (could be DFLSS metrics or JSON with dod_compliance)
    let json: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse metrics JSON")
        .map_err(to_cnv_error)?;

    let dod_compliance = json
        .get("dod_compliance")
        .and_then(|v| v.get("percentage"))
        .and_then(|v| v.as_f64())
        .or_else(|| {
            // Try alternative structure
            json.get("dod")
                .and_then(|v| v.get("percentage"))
                .and_then(|v| v.as_f64())
        })
        .unwrap_or(0.0);

    // CTQ requirement: ≥85% DoD compliance
    let compliant = dod_compliance >= 85.0;

    let status = if compliant { "PASS" } else { "FAIL" };

    let message = if compliant {
        format!("DoD compliance: {:.1}% (≥85% required)", dod_compliance)
    } else {
        format!(
            "DoD compliance: {:.1}% (≥85% required, gap: {:.1}%)",
            dod_compliance,
            85.0 - dod_compliance
        )
    };

    info!("DoD Validation: {}", message);

    Ok(ValidationResult {
        status: status.to_string(),
        message,
    })
}

/// Run all validations
#[verb("validation check-all")]
pub fn check_all(output: Option<PathBuf>) -> CnvResult<serde_json::Value> {
    info!("Running all DFLSS validations...");

    let mut results = serde_json::Map::new();
    let mut all_passed = true;

    // Validate Weaver
    match check_weaver(None) {
        Ok(weaver_result) => {
            results.insert(
                "weaver".to_string(),
                serde_json::json!({
                    "status": weaver_result.status,
                    "message": weaver_result.message
                }),
            );
            if weaver_result.status != "PASS" {
                all_passed = false;
            }
        }
        Err(e) => {
            results.insert(
                "weaver".to_string(),
                serde_json::json!({
                    "status": "ERROR",
                    "message": e.to_string()
                }),
            );
            all_passed = false;
        }
    }

    // Validate code quality
    match check_quality(None) {
        Ok(quality_result) => {
            results.insert(
                "quality".to_string(),
                serde_json::json!({
                    "status": quality_result.status,
                    "message": quality_result.message
                }),
            );
            if quality_result.status != "PASS" {
                all_passed = false;
            }
        }
        Err(e) => {
            results.insert(
                "quality".to_string(),
                serde_json::json!({
                    "status": "ERROR",
                    "message": e.to_string()
                }),
            );
            all_passed = false;
        }
    }

    let summary = serde_json::json!({
        "overall_status": if all_passed { "PASS" } else { "FAIL" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "validations": results
    });

    if let Some(output_path) = output {
        let json_str = serde_json::to_string_pretty(&summary)
            .context("Failed to serialize results")
            .map_err(to_cnv_error)?;
        std::fs::write(&output_path, json_str)
            .context("Failed to write output file")
            .map_err(to_cnv_error)?;
        info!("Validation results saved to: {}", output_path.display());
    }

    if all_passed {
        info!("All validations PASSED");
    } else {
        error!("Some validations FAILED");
    }

    Ok(summary)
}
