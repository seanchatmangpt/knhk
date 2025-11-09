// Process Capability Analysis
use anyhow::Context;

// Helper to convert anyhow::Error to clap_noun_verb error
fn to_cnv_error(e: anyhow::Error) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(e.to_string())
}
// Calculate Cp, Cpk, Sigma level, and DPMO

use crate::internal::capability::ProcessCapability;
use chrono::Utc;
use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Serialize)]
pub struct CapabilityResult {
    pub capability: ProcessCapability,
}

/// Calculate process capability from performance data
#[verb("capability calculate")]
pub fn calculate(
    data: PathBuf,
    usl: f64,
    lsl: f64,
    output: Option<PathBuf>,
) -> CnvResult<CapabilityResult> {
    info!("Calculating process capability from: {}", data.display());

    // Parse data file (CSV or JSON)
    let content = std::fs::read_to_string(&data)
        .context("Failed to read data file")
        .map_err(to_cnv_error)?;

    let values: Vec<f64> = if data.extension().and_then(|s| s.to_str()) == Some("json") {
        // JSON format: array of numbers or object with "values" key
        let json: serde_json::Value = serde_json::from_str(&content)
            .context("Failed to parse JSON data")
            .map_err(to_cnv_error)?;
        match json {
            serde_json::Value::Array(arr) => arr.iter().filter_map(|v| v.as_f64()).collect(),
            serde_json::Value::Object(obj) => {
                if let Some(serde_json::Value::Array(arr)) = obj.get("values") {
                    arr.iter().filter_map(|v| v.as_f64()).collect()
                } else {
                    return Err(to_cnv_error(anyhow::anyhow!(
                        "JSON must be an array or object with 'values' key"
                    )));
                }
            }
            _ => {
                return Err(to_cnv_error(anyhow::anyhow!(
                    "JSON must be an array or object"
                )))
            }
        }
    } else {
        // CSV format: single column or first column
        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let mut values = Vec::new();
        for result in reader.records() {
            let record = result
                .context("Failed to parse CSV record")
                .map_err(to_cnv_error)?;
            if let Some(first_field) = record.get(0) {
                if let Ok(value) = first_field.parse::<f64>() {
                    values.push(value);
                }
            }
        }
        values
    };

    if values.is_empty() {
        return Err(to_cnv_error(anyhow::anyhow!(
            "No data points found in file"
        )));
    }

    // Calculate capability
    let capability = crate::internal::capability::ProcessCapability::calculate(&values, usl, lsl)
        .map_err(|e| anyhow::anyhow!("Capability calculation failed: {}", e))
        .map_err(to_cnv_error)?;

    info!(
        "Process Capability: Cp={:.3}, Cpk={:.3}, Sigma={:.2}σ, DPMO={:.1}",
        capability.cp, capability.cpk, capability.sigma_level, capability.dpmo
    );

    // Save to output file if specified
    if let Some(output_path) = output {
        let json = serde_json::to_string_pretty(&capability)
            .context("Failed to serialize capability")
            .map_err(to_cnv_error)?;
        std::fs::write(&output_path, json)
            .context("Failed to write output file")
            .map_err(to_cnv_error)?;
        info!("Capability results saved to: {}", output_path.display());
    }

    Ok(CapabilityResult { capability })
}

/// Calculate capability per operation
#[verb("capability calculate-per-operation")]
pub fn calculate_per_operation(
    data: PathBuf,
    usl: f64,
    output: Option<PathBuf>,
) -> CnvResult<serde_json::Value> {
    info!(
        "Calculating per-operation capability from: {}",
        data.display()
    );

    // Load performance data
    let content = std::fs::read_to_string(&data)
        .context("Failed to read data file")
        .map_err(to_cnv_error)?;

    // Parse performance data (format: "operation: X.XX ticks")
    use regex::Regex;
    let re = Regex::new(r"(\w+):\s+(\d+(?:\.\d+)?)\s+ticks")
        .context("Failed to compile regex")
        .map_err(to_cnv_error)?;

    let mut operation_data: std::collections::HashMap<String, Vec<f64>> =
        std::collections::HashMap::new();

    for cap in re.captures_iter(&content) {
        let op = cap.get(1).unwrap().as_str().to_string();
        let ticks: f64 = cap
            .get(2)
            .unwrap()
            .as_str()
            .parse()
            .context("Failed to parse tick count")
            .map_err(to_cnv_error)?;
        operation_data
            .entry(op)
            .or_insert_with(Vec::new)
            .push(ticks);
    }

    if operation_data.is_empty() {
        return Err(to_cnv_error(anyhow::anyhow!("No performance data found")));
    }

    // Calculate capability for each operation
    let mut per_operation = serde_json::Map::new();
    for (op_name, values) in &operation_data {
        match crate::internal::capability::ProcessCapability::calculate(values, usl, 0.0) {
            Ok(capability) => {
                per_operation.insert(
                    op_name.clone(),
                    serde_json::json!({
                        "cp": capability.cp,
                        "cpk": capability.cpk,
                        "sigma_level": capability.sigma_level,
                        "dpmo": capability.dpmo,
                        "mean": capability.mean,
                        "std_dev": capability.std_dev,
                        "usl": capability.usl,
                        "lsl": capability.lsl,
                    }),
                );
                info!(
                    "{}: Cp={:.3}, Cpk={:.3}, Sigma={:.2}σ",
                    op_name, capability.cp, capability.cpk, capability.sigma_level
                );
            }
            Err(e) => {
                info!("{}: Failed to calculate capability: {}", op_name, e);
            }
        }
    }

    let result = serde_json::json!({
        "operations": per_operation,
        "total_operations": operation_data.len(),
        "usl": usl,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    if let Some(output_path) = output {
        let json_str = serde_json::to_string_pretty(&result)
            .context("Failed to serialize results")
            .map_err(to_cnv_error)?;
        std::fs::write(&output_path, json_str)
            .context("Failed to write output file")
            .map_err(to_cnv_error)?;
        info!(
            "Per-operation capability results saved to: {}",
            output_path.display()
        );
    }

    Ok(result)
}

/// Generate capability report
#[verb("capability report")]
pub fn generate_report(
    data: PathBuf,
    usl: f64,
    format: String,
    output: Option<PathBuf>,
) -> CnvResult<String> {
    info!(
        "Generating capability report from: {} (format: {})",
        data.display(),
        format
    );

    // Calculate capability
    let capability_result = calculate(data, usl, 0.0, None)?;
    let capability = &capability_result.capability;

    // Generate report based on format
    let report = match format.as_str() {
        "markdown" | "md" => {
            format!(
                "# Process Capability Report\n\n\
                **Generated**: {}\n\n\
                ## Summary\n\n\
                - **Cp**: {:.3}\n\
                - **Cpk**: {:.3}\n\
                - **Sigma Level**: {:.2}σ\n\
                - **DPMO**: {:.1}\n\n\
                ## Process Statistics\n\n\
                - **Mean (μ)**: {:.2}\n\
                - **Std Dev (σ)**: {:.2}\n\
                - **USL**: {:.2}\n\
                - **LSL**: {:.2}\n\n\
                ## Interpretation\n\n\
                - **Cp ≥ 1.33**: Process is capable ✅\n\
                - **Cpk ≥ 1.67**: Process is centered and capable ✅\n\
                - **Sigma ≥ 4.5**: World-class quality ✅\n",
                chrono::Utc::now().to_rfc3339(),
                capability.cp,
                capability.cpk,
                capability.sigma_level,
                capability.dpmo,
                capability.mean,
                capability.std_dev,
                capability.usl,
                capability.lsl
            )
        }
        "json" => serde_json::to_string_pretty(&capability)
            .context("Failed to serialize capability")
            .map_err(to_cnv_error)?,
        _ => {
            format!(
                "Process Capability Report\n\
                ========================\n\n\
                Generated: {}\n\n\
                Summary:\n\
                  Cp: {:.3}\n\
                  Cpk: {:.3}\n\
                  Sigma Level: {:.2}σ\n\
                  DPMO: {:.1}\n\n\
                Process Statistics:\n\
                  Mean (μ): {:.2}\n\
                  Std Dev (σ): {:.2}\n\
                  USL: {:.2}\n\
                  LSL: {:.2}\n",
                chrono::Utc::now().to_rfc3339(),
                capability.cp,
                capability.cpk,
                capability.sigma_level,
                capability.dpmo,
                capability.mean,
                capability.std_dev,
                capability.usl,
                capability.lsl
            )
        }
    };

    if let Some(output_path) = output {
        std::fs::write(&output_path, &report)
            .context("Failed to write report file")
            .map_err(to_cnv_error)?;
        info!("Capability report saved to: {}", output_path.display());
    }

    Ok(report)
}
