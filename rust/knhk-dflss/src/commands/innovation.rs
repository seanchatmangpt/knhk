// Innovation Tracking
use anyhow::Context;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

// Helper to convert anyhow::Error to clap_noun_verb error
fn to_cnv_error(e: anyhow::Error) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(e.to_string())
}
// Track TRIZ ideality scores, contradictions, and MGPP progress

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdealityScore {
    pub version: String,
    pub score: f64,
    pub benefits: u32,
    pub costs: u32,
    pub harms: u32,
}

/// Calculate TRIZ ideality score
#[verb("innovation ideality")]
pub fn calculate_ideality(version: String) -> CnvResult<IdealityScore> {
    info!("Calculating TRIZ ideality score for version: {}", version);

    // Load innovation data (JSON format)
    let data_path = PathBuf::from("docs/evidence/innovation.json");
    let (benefits, costs, harms) = if data_path.exists() {
        let content = fs::read_to_string(&data_path)
            .context("Failed to read innovation data")
            .map_err(to_cnv_error)?;

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let version_data = json.get(&version).or_else(|| json.get("latest"));
            (
                version_data
                    .and_then(|v| v.get("benefits"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as u32,
                version_data
                    .and_then(|v| v.get("costs"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as u32,
                version_data
                    .and_then(|v| v.get("harms"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(2) as u32,
            )
        } else {
            (10, 5, 2) // Default values
        }
    } else {
        (10, 5, 2) // Default values
    };

    // TRIZ Ideality = Benefits / (Costs + Harms)
    let score = if (costs + harms) > 0 {
        benefits as f64 / (costs + harms) as f64
    } else {
        f64::INFINITY
    };

    Ok(IdealityScore {
        version,
        score,
        benefits,
        costs,
        harms,
    })
}

/// Track TRIZ contradictions resolved
#[verb("innovation contradictions")]
pub fn track_contradictions(
    from: Option<String>,
    to: Option<String>,
) -> CnvResult<serde_json::Value> {
    info!("Tracking TRIZ contradictions from: {:?} to: {:?}", from, to);

    // Load contradictions data
    let data_path = PathBuf::from("docs/evidence/contradictions.json");
    let mut contradictions = Vec::new();

    if data_path.exists() {
        let content = fs::read_to_string(&data_path)
            .context("Failed to read contradictions data")
            .map_err(to_cnv_error)?;

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(contradictions_array) =
                json.get("contradictions").and_then(|v| v.as_array())
            {
                for contradiction in contradictions_array {
                    let resolved = contradiction
                        .get("resolved")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let date = contradiction
                        .get("date")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    // Filter by date range if specified
                    if let (Some(from_date), Some(to_date)) = (&from, &to) {
                        if date >= from_date.as_str() && date <= to_date.as_str() {
                            contradictions.push(contradiction.clone());
                        }
                    } else {
                        contradictions.push(contradiction.clone());
                    }
                }
            }
        }
    }

    // Calculate statistics
    let total = contradictions.len();
    let resolved = contradictions
        .iter()
        .filter(|c| c.get("resolved").and_then(|v| v.as_bool()).unwrap_or(false))
        .count();
    let resolution_rate = if total > 0 {
        (resolved as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let result = serde_json::json!({
        "total_contradictions": total,
        "resolved": resolved,
        "unresolved": total - resolved,
        "resolution_rate": resolution_rate,
        "contradictions": contradictions,
        "timestamp": Utc::now().to_rfc3339(),
    });

    Ok(result)
}

/// Analyze innovation level (1-5)
#[verb("innovation analyze-level")]
pub fn analyze_level() -> CnvResult<serde_json::Value> {
    info!("Analyzing innovation level");

    // Calculate innovation level based on ideality, contradictions, and patents
    let ideality_result = calculate_ideality("latest".to_string())?;
    let contradictions_result = track_contradictions(None, None)?;

    let ideality_score = ideality_result.score;
    let resolution_rate = contradictions_result
        .get("resolution_rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    // Innovation level scoring (1-5)
    let mut level = 1;
    let mut score = 0.0;

    // Ideality contribution (0-2 points)
    if ideality_score >= 2.0 {
        score += 2.0;
    } else if ideality_score >= 1.0 {
        score += 1.0;
    }

    // Contradiction resolution contribution (0-2 points)
    if resolution_rate >= 80.0 {
        score += 2.0;
    } else if resolution_rate >= 50.0 {
        score += 1.0;
    }

    // Patents/innovations contribution (0-1 point)
    // In production, this would load actual patent data
    score += 0.5; // Placeholder

    // Map score to level (1-5)
    level = if score >= 4.5 {
        5
    } else if score >= 3.5 {
        4
    } else if score >= 2.5 {
        3
    } else if score >= 1.5 {
        2
    } else {
        1
    };

    let result = serde_json::json!({
        "level": level,
        "score": score,
        "ideality": ideality_score,
        "contradiction_resolution_rate": resolution_rate,
        "assessment": match level {
            5 => "World-class innovation",
            4 => "Advanced innovation",
            3 => "Moderate innovation",
            2 => "Basic innovation",
            _ => "Limited innovation",
        },
        "timestamp": Utc::now().to_rfc3339(),
    });

    Ok(result)
}

/// Generate innovation roadmap
#[verb("innovation roadmap")]
pub fn generate_roadmap(target_ideality: f64) -> CnvResult<serde_json::Value> {
    info!(
        "Generating innovation roadmap with target ideality: {:.2}",
        target_ideality
    );

    // Get current ideality
    let current = calculate_ideality("latest".to_string())?;
    let current_ideality = current.score;

    // Calculate gap
    let gap = target_ideality - current_ideality;

    // Generate roadmap phases
    let mut phases = Vec::new();

    if gap > 0.0 {
        // Phase 1: Reduce costs
        phases.push(serde_json::json!({
            "phase": 1,
            "name": "Cost Reduction",
            "target": "Reduce costs by 20%",
            "expected_ideality": current_ideality + gap * 0.3,
            "duration_months": 3,
        }));

        // Phase 2: Increase benefits
        phases.push(serde_json::json!({
            "phase": 2,
            "name": "Benefit Enhancement",
            "target": "Increase benefits by 30%",
            "expected_ideality": current_ideality + gap * 0.6,
            "duration_months": 6,
        }));

        // Phase 3: Eliminate harms
        phases.push(serde_json::json!({
            "phase": 3,
            "name": "Harm Elimination",
            "target": "Eliminate all harms",
            "expected_ideality": target_ideality,
            "duration_months": 9,
        }));
    }

    let result = serde_json::json!({
        "current_ideality": current_ideality,
        "target_ideality": target_ideality,
        "gap": gap,
        "phases": phases,
        "total_duration_months": phases.iter()
            .map(|p| p.get("duration_months").and_then(|v| v.as_u64()).unwrap_or(0))
            .sum::<u64>(),
        "timestamp": Utc::now().to_rfc3339(),
    });

    Ok(result)
}

/// Track MGPP (Multi-Generation Product Plan) progress
#[verb("innovation mgpp")]
pub fn track_mgpp(generation: Option<String>) -> CnvResult<serde_json::Value> {
    info!("Tracking MGPP progress for generation: {:?}", generation);

    // Load MGPP data
    let data_path = PathBuf::from("docs/evidence/mgpp.json");
    let mut generations = Vec::new();

    if data_path.exists() {
        let content = fs::read_to_string(&data_path)
            .context("Failed to read MGPP data")
            .map_err(to_cnv_error)?;

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(generations_array) = json.get("generations").and_then(|v| v.as_array()) {
                for gen in generations_array {
                    if let Some(gen_name) = gen.get("name").and_then(|v| v.as_str()) {
                        if generation.is_none() || generation.as_ref().unwrap() == gen_name {
                            generations.push(gen.clone());
                        }
                    }
                }
            }
        }
    } else {
        // Default generations if no data file
        generations = vec![
            serde_json::json!({
                "name": "Gen1",
                "status": "completed",
                "progress": 100,
                "ideality": 1.5,
            }),
            serde_json::json!({
                "name": "Gen2",
                "status": "in_progress",
                "progress": 60,
                "ideality": 2.0,
            }),
            serde_json::json!({
                "name": "Gen3",
                "status": "planned",
                "progress": 0,
                "ideality": 2.5,
            }),
        ];
    }

    // Calculate overall progress
    let total_progress = generations
        .iter()
        .map(|g| g.get("progress").and_then(|v| v.as_u64()).unwrap_or(0))
        .sum::<u64>() as f64
        / generations.len() as f64;

    let result = serde_json::json!({
        "generations": generations,
        "total_generations": generations.len(),
        "overall_progress": total_progress,
        "timestamp": Utc::now().to_rfc3339(),
    });

    Ok(result)
}
