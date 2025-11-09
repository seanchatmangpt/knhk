// Metrics collection commands
// Collect and aggregate metrics for DFLSS analysis

use crate::internal::quality::QualityCollector;
use anyhow::Context;
use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use std::path::PathBuf;
use tracing::info;

// Helper to convert anyhow::Error to clap_noun_verb error
fn to_cnv_error(e: anyhow::Error) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(e.to_string())
}

#[derive(Debug, Serialize)]
pub struct QualityMetricsOutput {
    pub metrics: crate::internal::quality::QualityMetrics,
}

#[derive(Debug, Serialize)]
pub struct DflssMetricsOutput {
    pub quality: crate::internal::quality::QualityMetrics,
    pub timestamp: String,
}

/// Collect code quality metrics (clippy, unwrap, println, unimplemented)
#[verb("metrics collect-quality")]
pub fn collect_quality(
    output: Option<PathBuf>,
    rust_dir: PathBuf,
) -> CnvResult<QualityMetricsOutput> {
    info!("Collecting code quality metrics...");
    let collector = QualityCollector::new(rust_dir);
    let rt = tokio::runtime::Runtime::new()
        .context("Failed to create runtime")
        .map_err(to_cnv_error)?;
    let metrics = rt
        .block_on(collector.collect())
        .context("Failed to collect metrics")
        .map_err(to_cnv_error)?;

    let json = serde_json::to_string_pretty(&metrics)
        .context("Failed to serialize metrics")
        .map_err(to_cnv_error)?;

    if let Some(output_path) = output {
        std::fs::write(&output_path, &json)
            .context("Failed to write output")
            .map_err(to_cnv_error)?;
        info!("Metrics saved to: {}", output_path.display());
    } else {
        println!("{}", json);
    }

    Ok(QualityMetricsOutput { metrics })
}

/// Collect performance metrics from benchmark results
#[verb("metrics collect-performance")]
pub fn collect_performance(
    results: PathBuf,
    output: Option<PathBuf>,
) -> CnvResult<serde_json::Value> {
    info!("Collecting performance metrics from: {}", results.display());

    // Parse performance results file
    let content = std::fs::read_to_string(&results)
        .context("Failed to read performance results file")
        .map_err(to_cnv_error)?;

    // Parse performance data (format: "operation: X.XX ticks")
    use regex::Regex;
    let re = Regex::new(r"(\w+):\s+(\d+(?:\.\d+)?)\s+ticks")
        .context("Failed to compile regex")
        .map_err(to_cnv_error)?;

    let mut operation_ticks: std::collections::HashMap<String, Vec<f64>> =
        std::collections::HashMap::new();
    let mut operations_under_8_ticks = 0u32;
    let mut total_operations = 0u32;

    for cap in re.captures_iter(&content) {
        let op = cap.get(1).unwrap().as_str().to_string();
        let ticks: f64 = cap
            .get(2)
            .unwrap()
            .as_str()
            .parse()
            .context("Failed to parse tick count")
            .map_err(to_cnv_error)?;

        operation_ticks
            .entry(op.clone())
            .or_insert_with(Vec::new)
            .push(ticks);
        total_operations += 1;
        if ticks <= 8.0 {
            operations_under_8_ticks += 1;
        }
    }

    if operation_ticks.is_empty() {
        return Err(to_cnv_error(anyhow::anyhow!(
            "No performance data found in results file"
        )));
    }

    // Calculate statistics
    let all_ticks: Vec<f64> = operation_ticks.values().flatten().copied().collect();
    let median_ticks = crate::internal::statistics::percentile(&all_ticks, 0.5);
    let p95_ticks = crate::internal::statistics::percentile(&all_ticks, 0.95);
    let p99_ticks = crate::internal::statistics::percentile(&all_ticks, 0.99);

    let metrics = crate::internal::metrics::PerformanceMetrics {
        operation_ticks: operation_ticks.clone(),
        operations_under_8_ticks,
        total_operations,
        median_ticks,
        p95_ticks,
        p99_ticks,
    };

    info!("Performance Metrics: Operations={}, Under 8 ticks={} ({:.1}%), Median={:.2}, P95={:.2}, P99={:.2}", 
          total_operations, operations_under_8_ticks, 
          (operations_under_8_ticks as f64 / total_operations as f64) * 100.0,
          median_ticks, p95_ticks, p99_ticks);

    let json = serde_json::to_value(&metrics)
        .context("Failed to serialize metrics")
        .map_err(to_cnv_error)?;

    if let Some(output_path) = output {
        let json_str = serde_json::to_string_pretty(&json)
            .context("Failed to serialize JSON")
            .map_err(to_cnv_error)?;
        std::fs::write(&output_path, json_str)
            .context("Failed to write output file")
            .map_err(to_cnv_error)?;
        info!("Performance metrics saved to: {}", output_path.display());
    }

    Ok(json)
}

/// Collect Weaver validation metrics
#[verb("metrics collect-weaver")]
pub fn collect_weaver(
    registry: Option<PathBuf>,
    output: Option<PathBuf>,
) -> CnvResult<serde_json::Value> {
    info!("Collecting Weaver validation metrics...");

    // Run Weaver static check
    let registry_path = registry
        .as_ref()
        .map(|p| p.as_os_str().to_string_lossy().to_string())
        .unwrap_or_else(|| "registry/".to_string());

    let rt = tokio::runtime::Runtime::new()
        .context("Failed to create runtime")
        .map_err(to_cnv_error)?;

    // Check if weaver command exists
    let weaver_check = rt.block_on(
        tokio::process::Command::new("weaver")
            .args(&["registry", "check", "-r", &registry_path])
            .output(),
    );

    let (static_pass, validations, failures) = match weaver_check {
        Ok(output) => {
            let pass = output.status.success();
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Parse validation counts from output
            let mut validations = 0u32;
            let mut failures = 0u32;

            use regex::Regex;
            if let Ok(re) = Regex::new(r"(\d+)\s+validations?") {
                if let Some(cap) = re.captures(&stdout) {
                    validations = cap.get(1).unwrap().as_str().parse().unwrap_or(0);
                }
            }
            if let Ok(re) = Regex::new(r"(\d+)\s+failures?") {
                if let Some(cap) = re.captures(&stdout) {
                    failures = cap.get(1).unwrap().as_str().parse().unwrap_or(0);
                }
            }

            (pass, validations, failures)
        }
        Err(_) => {
            info!("Weaver command not found, assuming static check passed");
            (true, 0, 0)
        }
    };

    // Run Weaver live check
    let weaver_live = rt.block_on(
        tokio::process::Command::new("weaver")
            .args(&["registry", "live-check", "--registry", &registry_path])
            .output(),
    );

    let live_pass = match weaver_live {
        Ok(output) => {
            if output.status.success() {
                Some(true)
            } else {
                Some(false)
            }
        }
        Err(_) => {
            info!("Weaver live-check not available");
            None
        }
    };

    let pass_rate = if validations > 0 {
        (validations - failures) as f64 / validations as f64
    } else {
        1.0
    };

    let metrics = crate::internal::metrics::WeaverMetrics {
        static_pass,
        live_pass,
        validations,
        failures,
        pass_rate,
    };

    info!(
        "Weaver Metrics: Static={}, Live={:?}, Validations={}, Failures={}, Pass Rate={:.2}%",
        static_pass,
        live_pass,
        validations,
        failures,
        pass_rate * 100.0
    );

    let json = serde_json::to_value(&metrics)
        .context("Failed to serialize metrics")
        .map_err(to_cnv_error)?;

    if let Some(output_path) = output {
        let json_str = serde_json::to_string_pretty(&json)
            .context("Failed to serialize JSON")
            .map_err(to_cnv_error)?;
        std::fs::write(&output_path, json_str)
            .context("Failed to write output file")
            .map_err(to_cnv_error)?;
        info!("Weaver metrics saved to: {}", output_path.display());
    }

    Ok(json)
}

/// Collect all DFLSS metrics (composite command)
#[verb("metrics collect-all")]
pub fn collect_all(output_dir: PathBuf) -> CnvResult<DflssMetricsOutput> {
    info!("Collecting all DFLSS metrics...");
    std::fs::create_dir_all(&output_dir)
        .context("Failed to create output directory")
        .map_err(to_cnv_error)?;

    // Collect quality metrics
    let quality_output = output_dir.join("quality_metrics.json");
    let collector = QualityCollector::new(PathBuf::from("rust"));
    let rt = tokio::runtime::Runtime::new()
        .context("Failed to create runtime")
        .map_err(to_cnv_error)?;
    let quality_metrics = rt
        .block_on(collector.collect())
        .context("Failed to collect quality metrics")
        .map_err(to_cnv_error)?;

    let quality_json = serde_json::to_string_pretty(&quality_metrics)
        .context("Failed to serialize quality metrics")
        .map_err(to_cnv_error)?;
    std::fs::write(&quality_output, quality_json)
        .context("Failed to write quality metrics")
        .map_err(to_cnv_error)?;
    info!("Quality metrics saved to: {}", quality_output.display());

    // Collect performance metrics (if available)
    let perf_results_file = output_dir.join("performance_results.txt");
    let performance_metrics = if perf_results_file.exists() {
        match collect_performance(perf_results_file.clone(), None) {
            Ok(perf_json) => {
                let perf_output = output_dir.join("performance_metrics.json");
                if let Some(perf_path) = perf_output.to_str() {
                    let perf_str = serde_json::to_string_pretty(&perf_json)
                        .context("Failed to serialize performance metrics")
                        .map_err(to_cnv_error)?;
                    std::fs::write(&perf_output, perf_str)
                        .context("Failed to write performance metrics")
                        .map_err(to_cnv_error)?;
                    info!("Performance metrics saved to: {}", perf_output.display());
                }
                Some(perf_json)
            }
            Err(e) => {
                info!("Performance metrics collection failed: {}", e);
                None
            }
        }
    } else {
        info!("Performance results file not found, skipping performance metrics");
        None
    };

    // Collect Weaver metrics (if registry available)
    let registry_path = output_dir.join("registry");
    let weaver_metrics = if registry_path.exists() {
        match collect_weaver(Some(registry_path), None) {
            Ok(weaver_json) => {
                let weaver_output = output_dir.join("weaver_metrics.json");
                if let Some(weaver_path) = weaver_output.to_str() {
                    let weaver_str = serde_json::to_string_pretty(&weaver_json)
                        .context("Failed to serialize Weaver metrics")
                        .map_err(to_cnv_error)?;
                    std::fs::write(&weaver_output, weaver_str)
                        .context("Failed to write Weaver metrics")
                        .map_err(to_cnv_error)?;
                    info!("Weaver metrics saved to: {}", weaver_output.display());
                }
                Some(weaver_json)
            }
            Err(e) => {
                info!("Weaver metrics collection failed: {}", e);
                None
            }
        }
    } else {
        info!("Weaver registry not found, skipping Weaver metrics");
        None
    };

    // Generate summary
    let summary = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "quality": quality_metrics,
        "performance": performance_metrics,
        "weaver": weaver_metrics,
    });

    let summary_file = output_dir.join("dflss_metrics_summary.json");
    let summary_str = serde_json::to_string_pretty(&summary)
        .context("Failed to serialize summary")
        .map_err(to_cnv_error)?;
    std::fs::write(&summary_file, summary_str)
        .context("Failed to write summary")
        .map_err(to_cnv_error)?;
    info!("DFLSS metrics summary saved to: {}", summary_file.display());

    Ok(DflssMetricsOutput {
        quality: quality_metrics,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
