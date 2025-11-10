//! Soundness Verification CLI Commands
//!
//! Implements Van der Aalst's three fundamental soundness properties:
//! - Option to Complete: Every case can reach completion
//! - Proper Completion: Only output condition marked when case completes
//! - No Dead Tasks: Every task is reachable and executable

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
#[cfg(feature = "workflow")]
use knhk_workflow_engine::validation::ShaclValidator;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Soundness verification report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundnessReport {
    /// Option to complete property
    pub option_to_complete: bool,
    /// Proper completion property
    pub proper_completion: bool,
    /// No dead tasks property
    pub no_dead_tasks: bool,
    /// Overall soundness (all properties must be true)
    pub is_sound: bool,
    /// Violations found
    pub violations: Vec<String>,
}

/// Verify all three soundness properties
#[verb]
pub fn verify(workflow_file: PathBuf, json: bool) -> CnvResult<()> {
    let workflow_content = std::fs::read_to_string(&workflow_file).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to read workflow file: {}",
            e
        ))
    })?;

    let validator = ShaclValidator::new().map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Failed to create validator: {}", e))
    })?;

    let report = validator
        .validate_soundness(&workflow_content)
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to validate soundness: {}",
                e
            ))
        })?;

    // Extract soundness properties from SHACL report
    let option_to_complete = !report
        .violations
        .iter()
        .any(|v| v.message.contains("option to complete") || v.message.contains("reachable"));
    let proper_completion = !report
        .violations
        .iter()
        .any(|v| v.message.contains("proper completion") || v.message.contains("output"));
    let no_dead_tasks = !report
        .violations
        .iter()
        .any(|v| v.message.contains("dead task") || v.message.contains("unreachable"));

    let violations: Vec<String> = report
        .violations
        .iter()
        .map(|v| format!("{:?}: {}", v.severity, v.message))
        .collect();

    let soundness_report = SoundnessReport {
        option_to_complete,
        proper_completion,
        no_dead_tasks,
        is_sound: report.conforms && option_to_complete && proper_completion && no_dead_tasks,
        violations,
    };

    if json {
        let json_output = serde_json::to_string_pretty(&soundness_report).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to serialize report: {}",
                e
            ))
        })?;
        println!("{}", json_output);
    } else {
        println!("Soundness Verification Report");
        println!("============================");
        println!(
            "Option to Complete: {}",
            if soundness_report.option_to_complete {
                "✓"
            } else {
                "✗"
            }
        );
        println!(
            "Proper Completion: {}",
            if soundness_report.proper_completion {
                "✓"
            } else {
                "✗"
            }
        );
        println!(
            "No Dead Tasks: {}",
            if soundness_report.no_dead_tasks {
                "✓"
            } else {
                "✗"
            }
        );
        println!(
            "Overall Soundness: {}",
            if soundness_report.is_sound {
                "✓ SOUND"
            } else {
                "✗ UNSOUND"
            }
        );

        if !soundness_report.violations.is_empty() {
            println!("\nViolations:");
            for violation in &soundness_report.violations {
                println!("  - {}", violation);
            }
        }
    }

    if !soundness_report.is_sound {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "Workflow is not sound".to_string(),
        ));
    }

    Ok(())
}

/// Verify option to complete property
#[verb]
pub fn option_to_complete(workflow_file: PathBuf, json: bool) -> CnvResult<()> {
    let workflow_content = std::fs::read_to_string(&workflow_file).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to read workflow file: {}",
            e
        ))
    })?;

    let validator = ShaclValidator::new().map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Failed to create validator: {}", e))
    })?;

    let report = validator
        .validate_soundness(&workflow_content)
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to validate soundness: {}",
                e
            ))
        })?;

    let option_to_complete = !report
        .violations
        .iter()
        .any(|v| v.message.contains("option to complete") || v.message.contains("reachable"));

    if json {
        let result = serde_json::json!({
            "property": "option_to_complete",
            "satisfied": option_to_complete,
            "violations": report.violations.iter().filter(|v| v.message.contains("option to complete") || v.message.contains("reachable")).map(|v| format!("{:?}: {}", v.severity, v.message)).collect::<Vec<_>>()
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to serialize result: {}",
                    e
                ))
            })?
        );
    } else if option_to_complete {
        println!("✓ Option to Complete: SATISFIED");
        println!("  Every case can reach completion (Van der Aalst Property 1)");
    } else {
        println!("✗ Option to Complete: VIOLATED");
        println!("  Some cases cannot reach completion");
        for violation in report
            .violations
            .iter()
            .filter(|v| v.message.contains("option to complete") || v.message.contains("reachable"))
        {
            println!("  - {:?}: {}", violation.severity, violation.message);
        }
    }

    if !option_to_complete {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "Option to complete property violated".to_string(),
        ));
    }

    Ok(())
}

/// Verify proper completion property
#[verb]
pub fn proper_completion(workflow_file: PathBuf, json: bool) -> CnvResult<()> {
    let workflow_content = std::fs::read_to_string(&workflow_file).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to read workflow file: {}",
            e
        ))
    })?;

    let validator = ShaclValidator::new().map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Failed to create validator: {}", e))
    })?;

    let report = validator
        .validate_soundness(&workflow_content)
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to validate soundness: {}",
                e
            ))
        })?;

    let proper_completion = !report
        .violations
        .iter()
        .any(|v| v.message.contains("proper completion") || v.message.contains("output"));

    if json {
        let result = serde_json::json!({
            "property": "proper_completion",
            "satisfied": proper_completion,
            "violations": report.violations.iter().filter(|v| v.message.contains("proper completion") || v.message.contains("output")).map(|v| format!("{:?}: {}", v.severity, v.message)).collect::<Vec<_>>()
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to serialize result: {}",
                    e
                ))
            })?
        );
    } else if proper_completion {
        println!("✓ Proper Completion: SATISFIED");
        println!("  Only output condition marked when case completes (Van der Aalst Property 2)");
    } else {
        println!("✗ Proper Completion: VIOLATED");
        println!("  Output condition not properly marked on completion");
        for violation in report
            .violations
            .iter()
            .filter(|v| v.message.contains("proper completion") || v.message.contains("output"))
        {
            println!("  - {:?}: {}", violation.severity, violation.message);
        }
    }

    if !proper_completion {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "Proper completion property violated".to_string(),
        ));
    }

    Ok(())
}

/// Verify no dead tasks property
#[verb]
pub fn no_dead_tasks(workflow_file: PathBuf, json: bool) -> CnvResult<()> {
    let workflow_content = std::fs::read_to_string(&workflow_file).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to read workflow file: {}",
            e
        ))
    })?;

    let validator = ShaclValidator::new().map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Failed to create validator: {}", e))
    })?;

    let report = validator
        .validate_soundness(&workflow_content)
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to validate soundness: {}",
                e
            ))
        })?;

    let no_dead_tasks = !report
        .violations
        .iter()
        .any(|v| v.message.contains("dead task") || v.message.contains("unreachable"));

    if json {
        let result = serde_json::json!({
            "property": "no_dead_tasks",
            "satisfied": no_dead_tasks,
            "violations": report.violations.iter().filter(|v| v.message.contains("dead task") || v.message.contains("unreachable")).map(|v| format!("{:?}: {}", v.severity, v.message)).collect::<Vec<_>>()
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to serialize result: {}",
                    e
                ))
            })?
        );
    } else if no_dead_tasks {
        println!("✓ No Dead Tasks: SATISFIED");
        println!("  Every task is reachable and executable (Van der Aalst Property 3)");
    } else {
        println!("✗ No Dead Tasks: VIOLATED");
        println!("  Some tasks are unreachable or cannot be executed");
        for violation in report
            .violations
            .iter()
            .filter(|v| v.message.contains("dead task") || v.message.contains("unreachable"))
        {
            println!("  - {:?}: {}", violation.severity, violation.message);
        }
    }

    if !no_dead_tasks {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "No dead tasks property violated".to_string(),
        ));
    }

    Ok(())
}

/// Generate detailed soundness report
#[verb]
pub fn report(workflow_file: PathBuf, output: Option<PathBuf>, json: bool) -> CnvResult<()> {
    let workflow_content = std::fs::read_to_string(&workflow_file).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to read workflow file: {}",
            e
        ))
    })?;

    let validator = ShaclValidator::new().map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Failed to create validator: {}", e))
    })?;

    let report = validator
        .validate_soundness(&workflow_content)
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to validate soundness: {}",
                e
            ))
        })?;

    // Extract soundness properties
    let option_to_complete = !report
        .violations
        .iter()
        .any(|v| v.message.contains("option to complete") || v.message.contains("reachable"));
    let proper_completion = !report
        .violations
        .iter()
        .any(|v| v.message.contains("proper completion") || v.message.contains("output"));
    let no_dead_tasks = !report
        .violations
        .iter()
        .any(|v| v.message.contains("dead task") || v.message.contains("unreachable"));

    let violations: Vec<String> = report
        .violations
        .iter()
        .map(|v| format!("{:?}: {}", v.severity, v.message))
        .collect();

    let soundness_report = SoundnessReport {
        option_to_complete,
        proper_completion,
        no_dead_tasks,
        is_sound: report.conforms && option_to_complete && proper_completion && no_dead_tasks,
        violations,
    };

    let output_text = if json {
        serde_json::to_string_pretty(&soundness_report).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to serialize report: {}",
                e
            ))
        })?
    } else {
        format!(
            "Soundness Verification Report\n\
            ============================\n\
            Workflow: {}\n\n\
            Van der Aalst Soundness Properties:\n\
            - Option to Complete: {}\n\
            - Proper Completion: {}\n\
            - No Dead Tasks: {}\n\n\
            Overall Soundness: {}\n\n\
            Violations ({}):\n\
            {}\n",
            workflow_file.display(),
            if soundness_report.option_to_complete {
                "✓ SATISFIED"
            } else {
                "✗ VIOLATED"
            },
            if soundness_report.proper_completion {
                "✓ SATISFIED"
            } else {
                "✗ VIOLATED"
            },
            if soundness_report.no_dead_tasks {
                "✓ SATISFIED"
            } else {
                "✗ VIOLATED"
            },
            if soundness_report.is_sound {
                "✓ SOUND"
            } else {
                "✗ UNSOUND"
            },
            soundness_report.violations.len(),
            if soundness_report.violations.is_empty() {
                "  (none)".to_string()
            } else {
                soundness_report
                    .violations
                    .iter()
                    .map(|v| format!("  - {}", v))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        )
    };

    if let Some(output_path) = output {
        std::fs::write(&output_path, output_text).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to write report file: {}",
                e
            ))
        })?;
        println!("Report saved to: {}", output_path.display());
    } else {
        println!("{}", output_text);
    }

    Ok(())
}
