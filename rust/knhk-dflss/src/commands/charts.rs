// SPC Control Charts Management
// Update and manage Statistical Process Control charts

use crate::internal::chart::{ChartData, ChartManager};
use crate::internal::rules::check_western_electric_rules;
use crate::internal::statistics::*;
use anyhow::Context;
use chrono::Utc;
use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use std::path::PathBuf;
use tracing::{error, info};

// Helper to convert anyhow::Error to clap_noun_verb error
fn to_cnv_error(e: anyhow::Error) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(e.to_string())
}

#[derive(Debug, Serialize)]
pub struct ChartUpdateResult {
    pub chart_type: String,
    pub value: f64,
    pub ucl: f64,
    pub cl: f64,
    pub lcl: f64,
    pub special_causes: Vec<crate::internal::chart::SpecialCause>,
}

/// Update X-bar and R charts from performance results
#[verb("charts update-xbar-r")]
pub fn update_xbar_r(results: PathBuf, output_dir: PathBuf) -> CnvResult<ChartUpdateResult> {
    info!("Updating X-bar and R charts from: {}", results.display());

    // Parse performance results
    let content = std::fs::read_to_string(&results)
        .context("Failed to read results file")
        .map_err(to_cnv_error)?;
    let operations = parse_performance_results(&content)
        .context("Failed to parse results")
        .map_err(to_cnv_error)?;

    if operations.is_empty() {
        return Err(to_cnv_error(anyhow::anyhow!(
            "No performance data found in results file"
        )));
    }

    // Calculate X-bar and R
    let tick_counts: Vec<f64> = operations.values().copied().collect();
    let x_bar = mean(&tick_counts);
    let r = range(&tick_counts);

    info!("X-bar (Mean): {:.2} ticks", x_bar);
    info!("R (Range): {:.2} ticks", r);
    info!("Operations: {}", operations.len());

    // Update charts
    let timestamp = Utc::now().to_rfc3339();
    let year_month = Utc::now().format("%Y_%m").to_string();

    let manager = ChartManager::new(output_dir.clone());

    // Update X-bar chart
    let xbar_file = format!("x_bar_chart_{}.csv", year_month);
    let xbar_data = manager
        .read_chart(&xbar_file)
        .context("Failed to read X-bar chart")
        .map_err(to_cnv_error)?;
    let (ucl_x, cl_x, lcl_x) = calculate_control_limits(&xbar_data, x_bar);

    let new_xbar = ChartData {
        timestamp: timestamp.clone(),
        value: x_bar,
        ucl: ucl_x,
        cl: cl_x,
        lcl: lcl_x,
        subgroup_data: Some(tick_counts.clone()),
    };
    manager
        .append_chart(&xbar_file, new_xbar)
        .context("Failed to update X-bar chart")
        .map_err(to_cnv_error)?;
    info!(
        "X-bar chart updated: UCL={:.2}, CL={:.2}, LCL={:.2}",
        ucl_x, cl_x, lcl_x
    );

    // Update R chart
    let r_file = format!("r_chart_{}.csv", year_month);
    let r_data = manager
        .read_chart(&r_file)
        .context("Failed to read R chart")
        .map_err(to_cnv_error)?;
    let (ucl_r, cl_r, lcl_r) = calculate_control_limits(&r_data, r);

    let new_r = ChartData {
        timestamp,
        value: r,
        ucl: ucl_r,
        cl: cl_r,
        lcl: lcl_r,
        subgroup_data: None,
    };
    manager
        .append_chart(&r_file, new_r)
        .context("Failed to update R chart")
        .map_err(to_cnv_error)?;
    info!(
        "R chart updated: UCL={:.2}, CL={:.2}, LCL={:.2}",
        ucl_r, cl_r, lcl_r
    );

    // Check for special causes
    let all_xbar = manager
        .read_chart(&xbar_file)
        .context("Failed to read X-bar chart for checking")
        .map_err(to_cnv_error)?;
    let alerts = check_western_electric_rules(&all_xbar);

    if !alerts.is_empty() {
        error!("Special causes detected: {} alerts", alerts.len());
        for alert in &alerts {
            error!("  {:?}", alert);
        }
    } else {
        info!("No special causes detected. Process is in control.");
    }

    Ok(ChartUpdateResult {
        chart_type: "xbar-r".to_string(),
        value: x_bar,
        ucl: ucl_x,
        cl: cl_x,
        lcl: lcl_x,
        special_causes: alerts,
    })
}

/// Update p-chart for Weaver validation pass rate
#[verb("charts update-p")]
pub fn update_p(
    result: String,
    validations: u32,
    failures: u32,
    output_dir: PathBuf,
) -> CnvResult<serde_json::Value> {
    info!(
        "Updating p-chart: result={}, validations={}, failures={}",
        result, validations, failures
    );

    // Determine failures from result
    let actual_failures = if result == "FAIL" { failures } else { 0 };

    // Calculate proportion
    let p = if validations > 0 {
        actual_failures as f64 / validations as f64
    } else {
        0.0
    };

    info!("Weaver Validation Metrics: Result={}, Validations={}, Failures={}, Proportion={:.4} ({:.2}%)", 
          result, validations, actual_failures, p, p * 100.0);

    // Update chart
    let timestamp = Utc::now().to_rfc3339();
    let year_month = Utc::now().format("%Y_%m").to_string();

    let manager = ChartManager::new(output_dir.clone());
    let chart_file = format!("p_chart_{}.csv", year_month);
    let chart_data = manager
        .read_chart(&chart_file)
        .context("Failed to read p-chart")
        .map_err(to_cnv_error)?;

    // Calculate control limits for p-chart
    let (ucl, cl, lcl) = calculate_p_chart_limits(&chart_data, validations as f64);

    // Create new data point
    let new_data = ChartData {
        timestamp: timestamp.clone(),
        value: p,
        ucl,
        cl,
        lcl,
        subgroup_data: Some(vec![validations as f64, actual_failures as f64]),
    };

    manager
        .append_chart(&chart_file, new_data)
        .context("Failed to update p-chart")
        .map_err(to_cnv_error)?;

    info!(
        "p-chart updated: UCL={:.4}, CL={:.4}, LCL={:.4}",
        ucl, cl, lcl
    );

    // Check for special causes
    let all_data = manager
        .read_chart(&chart_file)
        .context("Failed to read p-chart for checking")
        .map_err(to_cnv_error)?;
    let alerts = check_western_electric_rules(&all_data);

    if !alerts.is_empty() {
        error!(
            "Special causes detected in p-chart: {} alerts",
            alerts.len()
        );
        for alert in &alerts {
            error!("  {:?}", alert);
        }
    } else {
        info!("No special causes detected. Process is in control.");
    }

    Ok(serde_json::json!({
        "chart_type": "p",
        "timestamp": timestamp,
        "proportion": p,
        "validations": validations,
        "failures": actual_failures,
        "ucl": ucl,
        "cl": cl,
        "lcl": lcl,
        "special_causes": alerts.len()
    }))
}

/// Update c-chart for code quality defect count
#[verb("charts update-c")]
pub fn update_c(data: PathBuf, output_dir: PathBuf) -> CnvResult<serde_json::Value> {
    info!("Updating c-chart from: {}", data.display());

    // Load code quality data
    let content = std::fs::read_to_string(&data)
        .context("Failed to read code quality data file")
        .map_err(to_cnv_error)?;
    let quality_data: crate::internal::quality::QualityMetrics = serde_json::from_str(&content)
        .context("Failed to parse code quality data")
        .map_err(to_cnv_error)?;

    let weighted_total = quality_data.weighted_total;
    let categories = &quality_data.categories;

    info!(
        "Code Quality Metrics: Weighted Defects={}, Critical={}, High={}, Medium={}, Low={}",
        weighted_total, categories.critical, categories.high, categories.medium, categories.low
    );

    // Update chart
    let timestamp = Utc::now().to_rfc3339();
    let year_month = Utc::now().format("%Y_%m").to_string();

    let manager = ChartManager::new(output_dir.clone());
    let chart_file = format!("c_chart_{}.csv", year_month);
    let chart_data = manager
        .read_chart(&chart_file)
        .context("Failed to read c-chart")
        .map_err(to_cnv_error)?;

    // Calculate control limits for c-chart
    let (ucl, cl, lcl) = calculate_c_chart_limits(&chart_data);

    // Create new data point
    let new_data = ChartData {
        timestamp: timestamp.clone(),
        value: weighted_total as f64,
        ucl,
        cl,
        lcl,
        subgroup_data: Some(vec![
            categories.critical as f64,
            categories.high as f64,
            categories.medium as f64,
            categories.low as f64,
        ]),
    };

    manager
        .append_chart(&chart_file, new_data)
        .context("Failed to update c-chart")
        .map_err(to_cnv_error)?;

    info!(
        "c-chart updated: UCL={:.2}, CL={:.2}, LCL={:.2}",
        ucl, cl, lcl
    );

    // Check for special causes
    let all_data = manager
        .read_chart(&chart_file)
        .context("Failed to read c-chart for checking")
        .map_err(to_cnv_error)?;
    let alerts = check_western_electric_rules(&all_data);

    if !alerts.is_empty() {
        error!(
            "Special causes detected in c-chart: {} alerts",
            alerts.len()
        );
        for alert in &alerts {
            error!("  {:?}", alert);
        }
    } else {
        info!("No special causes detected. Process is in control.");
    }

    Ok(serde_json::json!({
        "chart_type": "c",
        "timestamp": timestamp,
        "weighted_total": weighted_total,
        "categories": {
            "critical": categories.critical,
            "high": categories.high,
            "medium": categories.medium,
            "low": categories.low,
        },
        "ucl": ucl,
        "cl": cl,
        "lcl": lcl,
        "special_causes": alerts.len()
    }))
}

/// Check all charts for special causes
#[verb("charts check-special-causes")]
pub fn check_special_causes(
    xbar_chart: PathBuf,
    r_chart: PathBuf,
    p_chart: PathBuf,
    c_chart: PathBuf,
    output: Option<PathBuf>,
) -> CnvResult<serde_json::Value> {
    info!("Checking all charts for special causes...");

    let mut all_alerts = Vec::new();
    let mut chart_results = serde_json::Map::new();

    // Check X-bar chart
    if xbar_chart.exists() {
        let manager = ChartManager::new(xbar_chart.parent().unwrap().to_path_buf());
        let chart_name = xbar_chart
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let data = manager
            .read_chart(&chart_name)
            .context("Failed to read X-bar chart")
            .map_err(to_cnv_error)?;
        let alerts = check_western_electric_rules(&data);
        let alert_count = alerts.len();
        chart_results.insert(
            "xbar".to_string(),
            serde_json::json!({
                "alerts": alert_count,
                "status": if alerts.is_empty() { "in_control" } else { "out_of_control" }
            }),
        );
        all_alerts.extend(alerts);
        info!("X-bar chart: {} special cause(s) detected", alert_count);
    }

    // Check R chart
    if r_chart.exists() {
        let manager = ChartManager::new(r_chart.parent().unwrap().to_path_buf());
        let chart_name = r_chart.file_name().unwrap().to_string_lossy().to_string();
        let data = manager
            .read_chart(&chart_name)
            .context("Failed to read R chart")
            .map_err(to_cnv_error)?;
        let alerts = check_western_electric_rules(&data);
        let alert_count = alerts.len();
        chart_results.insert(
            "r".to_string(),
            serde_json::json!({
                "alerts": alert_count,
                "status": if alerts.is_empty() { "in_control" } else { "out_of_control" }
            }),
        );
        all_alerts.extend(alerts);
        info!("R chart: {} special cause(s) detected", alert_count);
    }

    // Check p-chart
    if p_chart.exists() {
        let manager = ChartManager::new(p_chart.parent().unwrap().to_path_buf());
        let chart_name = p_chart.file_name().unwrap().to_string_lossy().to_string();
        let data = manager
            .read_chart(&chart_name)
            .context("Failed to read p-chart")
            .map_err(to_cnv_error)?;
        let alerts = check_western_electric_rules(&data);
        let alert_count = alerts.len();
        chart_results.insert(
            "p".to_string(),
            serde_json::json!({
                "alerts": alert_count,
                "status": if alerts.is_empty() { "in_control" } else { "out_of_control" }
            }),
        );
        all_alerts.extend(alerts);
        info!("p-chart: {} special cause(s) detected", alert_count);
    }

    // Check c-chart
    if c_chart.exists() {
        let manager = ChartManager::new(c_chart.parent().unwrap().to_path_buf());
        let chart_name = c_chart.file_name().unwrap().to_string_lossy().to_string();
        let data = manager
            .read_chart(&chart_name)
            .context("Failed to read c-chart")
            .map_err(to_cnv_error)?;
        let alerts = check_western_electric_rules(&data);
        let alert_count = alerts.len();
        chart_results.insert(
            "c".to_string(),
            serde_json::json!({
                "alerts": alert_count,
                "status": if alerts.is_empty() { "in_control" } else { "out_of_control" }
            }),
        );
        all_alerts.extend(alerts);
        info!("c-chart: {} special cause(s) detected", alert_count);
    }

    let alerts_json: Vec<serde_json::Value> = all_alerts
        .iter()
        .map(|a| serde_json::to_value(a).unwrap_or(serde_json::Value::Null))
        .collect();

    let result = serde_json::json!({
        "total_alerts": all_alerts.len(),
        "charts": chart_results,
        "alerts": alerts_json
    });

    if let Some(output_path) = output {
        let json_str = serde_json::to_string_pretty(&result)
            .context("Failed to serialize JSON")
            .map_err(to_cnv_error)?;
        std::fs::write(&output_path, json_str)
            .context("Failed to write output file")
            .map_err(to_cnv_error)?;
        info!("Results saved to: {}", output_path.display());
    }

    if all_alerts.is_empty() {
        info!("All charts are in control. No special causes detected.");
    } else {
        error!("Total special causes detected: {}", all_alerts.len());
    }

    Ok(result)
}

fn parse_performance_results(
    content: &str,
) -> anyhow::Result<std::collections::HashMap<String, f64>> {
    use regex::Regex;
    let re = Regex::new(r"(\w+):\s+(\d+(?:\.\d+)?)\s+ticks")?;
    let mut operations = std::collections::HashMap::new();

    for cap in re.captures_iter(content) {
        let op = cap.get(1).unwrap().as_str().to_string();
        let ticks: f64 = cap.get(2).unwrap().as_str().parse()?;
        operations.insert(op, ticks);
    }

    Ok(operations)
}

fn calculate_control_limits(data: &[ChartData], _value: f64) -> (f64, f64, f64) {
    if data.len() > 20 {
        let values: Vec<f64> = data.iter().map(|d| d.value).collect();
        let process_mean = mean(&values);
        let process_std = std_dev(&values);

        let ucl = process_mean + 3.0 * process_std;
        let cl = process_mean;
        let lcl = (process_mean - 3.0 * process_std).max(0.0);

        (ucl, cl, lcl)
    } else {
        // Initial baseline
        let ucl = 7.2;
        let cl = 6.1;
        let lcl = 5.0;
        (ucl, cl, lcl)
    }
}

fn calculate_p_chart_limits(data: &[ChartData], n_bar: f64) -> (f64, f64, f64) {
    let p_bar = if data.len() < 20 {
        // Use baseline (target 0% defects, but allow 1% for chart stability)
        0.01
    } else {
        // Calculate from historical data
        let total_failures: f64 = data
            .iter()
            .filter_map(|d| d.subgroup_data.as_ref())
            .map(|sub| sub.get(1).copied().unwrap_or(0.0))
            .sum();
        let total_validations: f64 = data
            .iter()
            .filter_map(|d| d.subgroup_data.as_ref())
            .map(|sub| sub.get(0).copied().unwrap_or(0.0))
            .sum();
        if total_validations > 0.0 {
            total_failures / total_validations
        } else {
            0.01
        }
    };

    // p-chart control limits: p̄ ± 3√(p̄(1-p̄)/n̄)
    let std_dev = (p_bar * (1.0 - p_bar) / n_bar).sqrt();

    let ucl = (p_bar + 3.0 * std_dev).min(1.0); // Cannot exceed 100%
    let cl = p_bar;
    let lcl = (p_bar - 3.0 * std_dev).max(0.0); // Cannot be negative

    (ucl, cl, lcl)
}

fn calculate_c_chart_limits(data: &[ChartData]) -> (f64, f64, f64) {
    let c_bar = if data.len() < 20 {
        // Use baseline
        5.0
    } else {
        // Calculate from historical data
        mean(&data.iter().map(|d| d.value).collect::<Vec<f64>>())
    };

    // c-chart control limits: c̄ ± 3√c̄ (Poisson distribution)
    let std_dev = c_bar.sqrt();

    let ucl = c_bar + 3.0 * std_dev;
    let cl = c_bar;
    let lcl = (c_bar - 3.0 * std_dev).max(0.0); // Cannot be negative

    (ucl, cl, lcl)
}
