// DFLSS Report Generation
use anyhow::Context;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

// Helper to convert anyhow::Error to clap_noun_verb error
fn to_cnv_error(e: anyhow::Error) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(e.to_string())
}
// Generate comprehensive DFLSS reports

use crate::internal::chart::ChartManager;
use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use tracing::info;

/// Generate DFLSS metrics report
#[verb("report metrics")]
pub fn metrics_report(
    metrics: PathBuf,
    format: String,
    output: Option<PathBuf>,
) -> CnvResult<String> {
    info!(
        "Generating metrics report from: {} (format: {})",
        metrics.display(),
        format
    );

    // Load metrics JSON
    let content = fs::read_to_string(&metrics)
        .context("Failed to read metrics file")
        .map_err(to_cnv_error)?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse metrics JSON")
        .map_err(to_cnv_error)?;

    // Generate report based on format
    let report = match format.as_str() {
        "markdown" | "md" => {
            format!(
                "# DFLSS Metrics Report\n\n\
                **Generated**: {}\n\n\
                ## Quality Metrics\n\n\
                - **Clippy Errors**: {}\n\
                - **Clippy Warnings**: {}\n\
                - **Unwrap Count**: {}\n\
                - **Expect Count**: {}\n\
                - **Println Count**: {}\n\
                - **Unimplemented Count**: {}\n\
                - **Defect Count**: {}\n\n\
                ## Performance Metrics\n\n\
                {}\n\n\
                ## Weaver Metrics\n\n\
                {}\n",
                Utc::now().to_rfc3339(),
                json.get("quality")
                    .and_then(|q| q.get("clippy_errors"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                json.get("quality")
                    .and_then(|q| q.get("clippy_warnings"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                json.get("quality")
                    .and_then(|q| q.get("unwrap_count"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                json.get("quality")
                    .and_then(|q| q.get("expect_count"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                json.get("quality")
                    .and_then(|q| q.get("println_count"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                json.get("quality")
                    .and_then(|q| q.get("unimplemented_count"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                json.get("quality")
                    .and_then(|q| q.get("defect_count"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                if let Some(perf) = json.get("performance") {
                    format!(
                        "- **Median Ticks**: {:.2}\n- **P95 Ticks**: {:.2}\n- **P99 Ticks**: {:.2}",
                        perf.get("median_ticks")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0),
                        perf.get("p95_ticks")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0),
                        perf.get("p99_ticks")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0)
                    )
                } else {
                    "No performance data available".to_string()
                },
                if let Some(weaver) = json.get("weaver") {
                    format!("- **Static Validations**: {}\n- **Live Validations**: {}\n- **Pass Rate**: {:.1}%",
                        weaver.get("static_validations").and_then(|v| v.as_u64()).unwrap_or(0),
                        weaver.get("live_validations").and_then(|v| v.as_u64()).unwrap_or(0),
                        weaver.get("pass_rate").and_then(|v| v.as_f64()).unwrap_or(0.0))
                } else {
                    "No Weaver data available".to_string()
                }
            )
        }
        "json" => serde_json::to_string_pretty(&json)
            .context("Failed to serialize metrics")
            .map_err(to_cnv_error)?,
        _ => {
            format!(
                "DFLSS Metrics Report\n\
                ===================\n\n\
                Generated: {}\n\n\
                Quality Metrics:\n\
                  Clippy Errors: {}\n\
                  Clippy Warnings: {}\n\
                  Unwrap Count: {}\n\
                  Expect Count: {}\n\
                  Println Count: {}\n\
                  Unimplemented Count: {}\n\
                  Defect Count: {:.2}\n",
                Utc::now().to_rfc3339(),
                json.get("quality")
                    .and_then(|q| q.get("clippy_errors"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                json.get("quality")
                    .and_then(|q| q.get("clippy_warnings"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                json.get("quality")
                    .and_then(|q| q.get("unwrap_count"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                json.get("quality")
                    .and_then(|q| q.get("expect_count"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                json.get("quality")
                    .and_then(|q| q.get("println_count"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                json.get("quality")
                    .and_then(|q| q.get("unimplemented_count"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                json.get("quality")
                    .and_then(|q| q.get("defect_count"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            )
        }
    };

    if let Some(output_path) = output {
        fs::write(&output_path, &report)
            .context("Failed to write report file")
            .map_err(to_cnv_error)?;
        info!("Metrics report saved to: {}", output_path.display());
    }

    Ok(report)
}

/// Generate SPC charts summary report
#[verb("report charts")]
pub fn charts_report(dir: PathBuf, format: String, output: Option<PathBuf>) -> CnvResult<String> {
    info!(
        "Generating charts report from: {} (format: {})",
        dir.display(),
        format
    );

    let chart_manager = ChartManager::new(dir.clone());
    let mut chart_summaries = Vec::new();

    // Read all chart files
    let chart_files = vec![
        "xbar_chart.csv",
        "r_chart.csv",
        "p_chart.csv",
        "c_chart.csv",
    ];

    for chart_file in &chart_files {
        match chart_manager.read_chart(chart_file) {
            Ok(data) => {
                if !data.is_empty() {
                    let latest = &data[data.len() - 1];
                    chart_summaries.push(format!(
                        "{}: Value={:.2}, UCL={:.2}, CL={:.2}, LCL={:.2}, Status={}",
                        chart_file,
                        latest.value,
                        latest.ucl,
                        latest.cl,
                        latest.lcl,
                        if latest.value > latest.ucl || latest.value < latest.lcl {
                            "OUT OF CONTROL"
                        } else {
                            "IN CONTROL"
                        }
                    ));
                }
            }
            Err(_) => {
                chart_summaries.push(format!("{}: No data available", chart_file));
            }
        }
    }

    // Generate report based on format
    let report = match format.as_str() {
        "markdown" | "md" => {
            format!(
                "# SPC Charts Summary Report\n\n\
                **Generated**: {}\n\n\
                ## Chart Status\n\n\
                {}\n\n\
                ## Summary\n\n\
                Total Charts: {}\n",
                Utc::now().to_rfc3339(),
                chart_summaries
                    .iter()
                    .map(|s| format!("- {}", s))
                    .collect::<Vec<_>>()
                    .join("\n"),
                chart_summaries.len()
            )
        }
        "json" => serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "charts": chart_summaries,
            "total_charts": chart_summaries.len()
        })
        .to_string(),
        _ => {
            format!(
                "SPC Charts Summary Report\n\
                =========================\n\n\
                Generated: {}\n\n\
                Chart Status:\n\
                {}\n\n\
                Total Charts: {}\n",
                Utc::now().to_rfc3339(),
                chart_summaries
                    .iter()
                    .map(|s| format!("  {}", s))
                    .collect::<Vec<_>>()
                    .join("\n"),
                chart_summaries.len()
            )
        }
    };

    if let Some(output_path) = output {
        fs::write(&output_path, &report)
            .context("Failed to write report file")
            .map_err(to_cnv_error)?;
        info!("Charts report saved to: {}", output_path.display());
    }

    Ok(report)
}

/// Generate process capability report
#[verb("report capability")]
pub fn capability_report(
    data: PathBuf,
    format: String,
    output: Option<PathBuf>,
) -> CnvResult<String> {
    info!(
        "Generating capability report from: {} (format: {})",
        data.display(),
        format
    );

    // Reuse capability::generate_report logic
    use crate::commands::capability::generate_report;
    generate_report(data, 8.0, format, output)
}

/// Generate comprehensive DFLSS status report
#[verb("report status")]
pub fn status_report(output: Option<PathBuf>) -> CnvResult<String> {
    info!("Generating comprehensive DFLSS status report...");

    // Collect all available data
    let mut sections = Vec::new();

    // Try to load metrics
    let metrics_path = PathBuf::from("docs/evidence/dflss_metrics_summary.json");
    if metrics_path.exists() {
        match fs::read_to_string(&metrics_path) {
            Ok(content) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    sections.push(format!(
                        "## Metrics Status\n\n\
                        Quality: {}\n\
                        Performance: {}\n\
                        Weaver: {}\n",
                        if json.get("quality").is_some() {
                            "Available"
                        } else {
                            "Not available"
                        },
                        if json.get("performance").is_some() {
                            "Available"
                        } else {
                            "Not available"
                        },
                        if json.get("weaver").is_some() {
                            "Available"
                        } else {
                            "Not available"
                        }
                    ));
                }
            }
            Err(_) => {}
        }
    }

    // Try to load charts
    let charts_dir = PathBuf::from("docs/evidence/spc");
    if charts_dir.exists() {
        let chart_manager = ChartManager::new(charts_dir);
        let mut chart_count = 0;
        for chart_file in &[
            "xbar_chart.csv",
            "r_chart.csv",
            "p_chart.csv",
            "c_chart.csv",
        ] {
            if chart_manager.read_chart(chart_file).is_ok() {
                chart_count += 1;
            }
        }
        sections.push(format!(
            "## Charts Status\n\n\
            Active Charts: {}/4\n",
            chart_count
        ));
    }

    // Try to load validation results
    let validation_path = PathBuf::from("docs/evidence/validation_results.json");
    if validation_path.exists() {
        match fs::read_to_string(&validation_path) {
            Ok(content) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    sections.push(format!(
                        "## Validation Status\n\n\
                        Overall Status: {}\n",
                        json.get("overall_status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                    ));
                }
            }
            Err(_) => {}
        }
    }

    // Generate comprehensive report
    let report = format!(
        "# DFLSS Status Report\n\n\
        **Generated**: {}\n\n\
        {}\n\n\
        ## Summary\n\n\
        This report aggregates all available DFLSS data including metrics, charts, and validation results.\n",
        Utc::now().to_rfc3339(),
        sections.join("\n")
    );

    if let Some(output_path) = output {
        fs::write(&output_path, &report)
            .context("Failed to write report file")
            .map_err(to_cnv_error)?;
        info!("Status report saved to: {}", output_path.display());
    }

    Ok(report)
}
