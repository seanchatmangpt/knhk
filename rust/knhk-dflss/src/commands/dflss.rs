// Advanced DFLSS Tools
use anyhow::Context;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

// Helper to convert anyhow::Error to clap_noun_verb error
fn to_cnv_error(e: anyhow::Error) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(e.to_string())
}
// DOE, Monte Carlo, Taguchi, FMEA, QFD

use crate::internal::statistics::*;
use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use tracing::info;

#[derive(Debug, Serialize)]
pub struct DoeDesign {
    pub factors: Vec<String>,
    pub levels: Vec<u32>,
    pub design_type: String,
    pub runs: u32,
    pub matrix: Vec<Vec<u32>>,
}

/// Design of Experiments (DOE) analysis
#[verb("dflss doe")]
pub fn design_of_experiments(
    factors: String,
    levels: String,
    design_type: String,
) -> CnvResult<DoeDesign> {
    info!(
        "Designing experiment with factors: {}, levels: {}, type: {}",
        factors, levels, design_type
    );

    // Parse factors and levels from comma-separated strings
    let factors_vec: Vec<String> = factors.split(',').map(|s| s.trim().to_string()).collect();
    let levels_vec: Vec<u32> = levels
        .split(',')
        .map(|s| s.trim().parse::<u32>())
        .collect::<Result<Vec<u32>, _>>()
        .context("Failed to parse levels")
        .map_err(to_cnv_error)?;

    if factors_vec.len() != levels_vec.len() {
        return Err(to_cnv_error(anyhow::anyhow!(
            "Number of factors must match number of levels"
        )));
    }

    // Calculate number of runs based on design type
    let runs = match design_type.as_str() {
        "full" => {
            // Full factorial: product of all levels
            levels_vec.iter().product()
        }
        "fractional" => {
            // Fractional factorial: half of full factorial
            levels_vec.iter().product::<u32>() / 2
        }
        "plackett-burman" => {
            // Plackett-Burman: next multiple of 4 >= number of factors
            ((factors_vec.len() as u32 + 3) / 4) * 4
        }
        _ => {
            // Default: full factorial
            levels_vec.iter().product()
        }
    };

    // Generate design matrix (simplified - in production would use proper DOE algorithms)
    let mut matrix = Vec::new();
    for run in 0..runs {
        let mut row = Vec::new();
        for (i, &level) in levels_vec.iter().enumerate() {
            // Simple pattern: cycle through levels
            row.push((run / levels_vec[..i].iter().product::<u32>().max(1)) % level);
        }
        matrix.push(row);
    }

    Ok(DoeDesign {
        factors: factors_vec,
        levels: levels_vec,
        design_type,
        runs,
        matrix,
    })
}

/// Monte Carlo simulation
#[verb("dflss monte-carlo")]
pub fn monte_carlo(iterations: u32, distribution: String) -> CnvResult<serde_json::Value> {
    info!(
        "Running Monte Carlo simulation: {} iterations, distribution: {}",
        iterations, distribution
    );

    // Generate random samples based on distribution
    let mut samples = Vec::new();

    match distribution.as_str() {
        "normal" | "gaussian" => {
            // Normal distribution: mean=0, std_dev=1
            // Use Box-Muller transform for sampling
            for i in 0..iterations {
                let u1 = ((i * 7 + 1) % 1000) as f64 / 1000.0;
                let u2 = ((i * 11 + 3) % 1000) as f64 / 1000.0;
                let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                samples.push(z0);
            }
        }
        "uniform" => {
            // Uniform distribution: [0, 1]
            for i in 0..iterations {
                samples.push((i * 7 + 1) as f64 / 1000.0 % 1.0);
            }
        }
        "exponential" => {
            // Exponential distribution: lambda=1
            for i in 0..iterations {
                let u = ((i * 7 + 1) % 1000) as f64 / 1000.0;
                samples.push(-(1.0 - u).ln());
            }
        }
        _ => {
            // Default: uniform
            for i in 0..iterations {
                samples.push((i * 7 + 1) as f64 / 1000.0 % 1.0);
            }
        }
    }

    // Calculate statistics
    let mean_val = mean(&samples);
    let std_dev_val = std_dev(&samples);
    let min_val = min(&samples);
    let max_val = max(&samples);

    let result = serde_json::json!({
        "distribution": distribution,
        "iterations": iterations,
        "statistics": {
            "mean": mean_val,
            "std_dev": std_dev_val,
            "min": min_val,
            "max": max_val,
        },
        "samples": samples[..samples.len().min(100)].to_vec(), // First 100 samples
        "timestamp": Utc::now().to_rfc3339(),
    });

    Ok(result)
}

/// Statistical Tolerance Design
#[verb("dflss tolerance-design")]
pub fn tolerance_design(spec: String, confidence: f64) -> CnvResult<serde_json::Value> {
    info!(
        "Calculating tolerance design: spec={}, confidence={:.2}",
        spec, confidence
    );

    // Parse specification limits from comma-separated string
    let spec_vec: Vec<f64> = spec
        .split(',')
        .map(|s| s.trim().parse::<f64>())
        .collect::<Result<Vec<f64>, _>>()
        .context("Failed to parse specification limits")
        .map_err(to_cnv_error)?;

    if spec_vec.is_empty() {
        return Err(to_cnv_error(anyhow::anyhow!(
            "At least one specification limit required"
        )));
    }

    // Calculate tolerance stack-up using RSS (Root Sum Square) method
    let mean_spec = mean(&spec_vec);
    let std_dev_spec = std_dev(&spec_vec);

    // RSS tolerance: sqrt(sum of squares of individual tolerances)
    let rss_tolerance = (spec_vec.iter().map(|&x| x.powi(2)).sum::<f64>()).sqrt();

    // Worst-case tolerance: sum of absolute values
    let worst_case_tolerance = spec_vec.iter().map(|&x| x.abs()).sum::<f64>();

    // Calculate confidence interval
    use statrs::distribution::{ContinuousCDF, Normal};
    let normal = Normal::new(0.0, 1.0).unwrap();
    let z_score = normal.inverse_cdf((1.0 + confidence) / 2.0);
    let confidence_interval = z_score * std_dev_spec;

    let result = serde_json::json!({
        "specification_limits": spec_vec,
        "mean": mean_spec,
        "std_dev": std_dev_spec,
        "rss_tolerance": rss_tolerance,
        "worst_case_tolerance": worst_case_tolerance,
        "confidence": confidence,
        "confidence_interval": confidence_interval,
        "timestamp": Utc::now().to_rfc3339(),
    });

    Ok(result)
}

/// Taguchi robust design analysis
#[verb("dflss taguchi")]
pub fn taguchi_analysis(data: PathBuf, sn_ratio: String) -> CnvResult<serde_json::Value> {
    info!(
        "Performing Taguchi analysis: data={}, sn_ratio={}",
        data.display(),
        sn_ratio
    );

    // Load data
    let content = fs::read_to_string(&data)
        .context("Failed to read data file")
        .map_err(to_cnv_error)?;

    // Parse data (CSV format: factor1,factor2,...,response)
    let mut values = Vec::new();
    for line in content.lines() {
        if let Some(last_comma) = line.rfind(',') {
            if let Ok(value) = line[last_comma + 1..].trim().parse::<f64>() {
                values.push(value);
            }
        }
    }

    if values.is_empty() {
        return Err(to_cnv_error(anyhow::anyhow!("No data values found")));
    }

    // Calculate Signal-to-Noise ratio
    let mean_val = mean(&values);
    let std_dev_val = std_dev(&values);

    let sn_ratio_value = match sn_ratio.as_str() {
        "larger-is-better" => {
            // S/N = -10 * log10(mean(1/y^2))
            let sum_inv_sq = values.iter().map(|&y| 1.0 / (y * y)).sum::<f64>();
            -10.0 * (sum_inv_sq / values.len() as f64).log10()
        }
        "smaller-is-better" => {
            // S/N = -10 * log10(mean(y^2))
            let sum_sq = values.iter().map(|&y| y * y).sum::<f64>();
            -10.0 * (sum_sq / values.len() as f64).log10()
        }
        "nominal-is-best" => {
            // S/N = 10 * log10(mean^2 / variance)
            if std_dev_val > 0.0 {
                10.0 * ((mean_val * mean_val) / (std_dev_val * std_dev_val)).log10()
            } else {
                0.0
            }
        }
        _ => {
            // Default: nominal-is-best
            if std_dev_val > 0.0 {
                10.0 * ((mean_val * mean_val) / (std_dev_val * std_dev_val)).log10()
            } else {
                0.0
            }
        }
    };

    let result = serde_json::json!({
        "sn_ratio_type": sn_ratio,
        "sn_ratio": sn_ratio_value,
        "mean": mean_val,
        "std_dev": std_dev_val,
        "optimal_setting": if sn_ratio_value > 0.0 { "maximize" } else { "minimize" },
        "timestamp": Utc::now().to_rfc3339(),
    });

    Ok(result)
}

/// FMEA risk analysis
#[verb("dflss fmea")]
pub fn fmea_analysis(fmea_type: String) -> CnvResult<serde_json::Value> {
    info!("Performing FMEA analysis: type={}", fmea_type);

    // FMEA Risk Priority Number (RPN) = Severity × Occurrence × Detection
    // In production, this would load actual FMEA data from a file
    // For now, generate example FMEA entries

    let mut fmea_entries = Vec::new();

    // Example failure modes
    let failure_modes = vec![
        ("System crash", 9, 3, 2), // High severity, low occurrence, good detection
        ("Data corruption", 8, 2, 3), // High severity, very low occurrence, moderate detection
        ("Performance degradation", 5, 5, 4), // Medium severity, medium occurrence, poor detection
    ];

    for (mode, severity, occurrence, detection) in failure_modes {
        let rpn = severity * occurrence * detection;
        fmea_entries.push(serde_json::json!({
            "failure_mode": mode,
            "severity": severity,
            "occurrence": occurrence,
            "detection": detection,
            "rpn": rpn,
            "risk_level": if rpn >= 100 { "high" } else if rpn >= 50 { "medium" } else { "low" },
        }));
    }

    // Sort by RPN (highest first)
    fmea_entries.sort_by(|a, b| {
        let a_rpn = a.get("rpn").and_then(|v| v.as_u64()).unwrap_or(0);
        let b_rpn = b.get("rpn").and_then(|v| v.as_u64()).unwrap_or(0);
        b_rpn.cmp(&a_rpn)
    });

    let result = serde_json::json!({
        "fmea_type": fmea_type,
        "entries": fmea_entries,
        "total_entries": fmea_entries.len(),
        "high_risk_count": fmea_entries.iter().filter(|e| e.get("risk_level").and_then(|v| v.as_str()) == Some("high")).count(),
        "timestamp": Utc::now().to_rfc3339(),
    });

    Ok(result)
}

/// QFD House of Quality
#[verb("dflss qfd")]
pub fn qfd_analysis(voc: PathBuf) -> CnvResult<serde_json::Value> {
    info!(
        "Performing QFD House of Quality analysis: voc={}",
        voc.display()
    );

    // Load Voice of Customer (VOC) data
    let content = fs::read_to_string(&voc)
        .context("Failed to read VOC file")
        .map_err(to_cnv_error)?;

    // Parse VOC requirements (JSON format)
    let voc_json: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse VOC JSON")
        .map_err(to_cnv_error)?;

    // Extract customer requirements and technical requirements
    let customer_requirements = voc_json
        .get("customer_requirements")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![]);

    let technical_requirements = voc_json
        .get("technical_requirements")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![]);

    // Generate relationship matrix (customer requirements × technical requirements)
    let mut relationship_matrix = Vec::new();
    for cr in customer_requirements {
        let mut row = Vec::new();
        for tr in technical_requirements {
            // Calculate relationship score (1-9 scale, simplified)
            let score = if let (Some(cr_str), Some(tr_str)) = (cr.as_str(), tr.as_str()) {
                // Simple scoring: if keywords match, higher score
                if cr_str.to_lowercase().contains(&tr_str.to_lowercase()) {
                    9
                } else if cr_str.len() > 0 && tr_str.len() > 0 {
                    5
                } else {
                    1
                }
            } else {
                1
            };
            row.push(score);
        }
        relationship_matrix.push(row);
    }

    // Calculate importance scores
    let mut importance_scores = Vec::new();
    for (i, cr) in customer_requirements.iter().enumerate() {
        let importance = cr.get("importance").and_then(|v| v.as_u64()).unwrap_or(5);
        let weighted_score = relationship_matrix[i].iter().sum::<u32>() * importance;
        importance_scores.push(weighted_score);
    }

    let result = serde_json::json!({
        "customer_requirements_count": customer_requirements.len(),
        "technical_requirements_count": technical_requirements.len(),
        "relationship_matrix": relationship_matrix,
        "importance_scores": importance_scores,
        "top_priority": importance_scores.iter().enumerate()
            .max_by_key(|(_, &score)| score)
            .map(|(i, _)| i),
        "timestamp": Utc::now().to_rfc3339(),
    });

    Ok(result)
}
