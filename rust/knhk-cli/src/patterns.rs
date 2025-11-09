//! Pattern Testing CLI Commands
//!
//! Implements Van der Aalst's 43 workflow patterns:
//! - List all patterns
//! - Test individual patterns
//! - Test all patterns
//! - Verify pattern in workflow
//! - Show pattern coverage

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use knhk_workflow_engine::{
    patterns::{PatternId, PatternRegistry, RegisterAllExt},
    state::StateStore,
    WorkflowEngine,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Get or create tokio runtime for async operations
fn get_runtime() -> &'static Runtime {
    static RUNTIME: std::sync::OnceLock<Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        Runtime::new().unwrap_or_else(|e| {
            panic!("Failed to create tokio runtime: {}", e);
        })
    })
}

/// Get workflow engine instance
fn get_engine(state_store_path: Option<&str>) -> CnvResult<Arc<WorkflowEngine>> {
    let path = state_store_path.unwrap_or("./workflow_db");
    let state_store = StateStore::new(path).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to create state store: {}",
            e
        ))
    })?;
    Ok(Arc::new(WorkflowEngine::new(state_store)))
}

/// Pattern information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternInfo {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub description: String,
}

/// List all 43 workflow patterns
#[verb]
pub fn list(json: bool) -> CnvResult<()> {
    let runtime = get_runtime();
    let _engine = get_engine(None)?;

    runtime.block_on(async {
        // Register all patterns
        let mut registry = PatternRegistry::new();
        registry.register_all_patterns();

        // Use pattern metadata instead of executor methods
        use knhk_workflow_engine::patterns::rdf::metadata::get_all_pattern_metadata;
        let metadata = get_all_pattern_metadata();
        let patterns: Vec<PatternInfo> = metadata
            .iter()
            .map(|m| PatternInfo {
                id: m.pattern_id,
                name: m.name.clone(),
                category: m.category.clone(),
                description: m.description.clone(),
            })
            .collect();

        if json {
            let json_output = serde_json::to_string_pretty(&patterns).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to serialize patterns: {}",
                    e
                ))
            })?;
            println!("{}", json_output);
        } else {
            println!("Van der Aalst Workflow Patterns (43 total)");
            println!("==========================================");
            for pattern in &patterns {
                println!("\nPattern {}: {}", pattern.id, pattern.name);
                println!("  Category: {}", pattern.category);
                println!("  Description: {}", pattern.description);
            }
        }

        Ok(())
    })
}

/// Test a specific pattern
#[verb]
pub fn test(pattern_id: u32, state_store: Option<String>, json: bool) -> CnvResult<()> {
    if !(1..=43).contains(&pattern_id) {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "Pattern ID must be between 1 and 43, got: {}",
            pattern_id
        )));
    }

    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        let pattern_id_enum = PatternId(pattern_id);

        // Register all patterns
        let mut registry = PatternRegistry::new();
        registry.register_all_patterns();

        let pattern = registry.get(&pattern_id_enum).ok_or_else(|| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Pattern {} not found",
                pattern_id
            ))
        })?;

        // Get pattern metadata
        use knhk_workflow_engine::patterns::rdf::metadata::get_all_pattern_metadata;
        let metadata = get_all_pattern_metadata();
        let pattern_meta = metadata.iter().find(|m| m.pattern_id == pattern_id);

        // Create execution context
        let context = knhk_workflow_engine::patterns::PatternExecutionContext::default();

        // Execute pattern (not async)
        let result = pattern.execute(&context);

        if json {
            let result_json = serde_json::json!({
                "pattern_id": pattern_id,
                "category": pattern_meta.map(|m| m.category.clone()).unwrap_or_else(|| "Unknown".to_string()),
                "execution_result": if result.success { "success" } else { "error" },
                "next_activities": result.next_activities,
                "terminates": result.terminates
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&result_json).map_err(|e| {
                    clap_noun_verb::NounVerbError::execution_error(format!(
                        "Failed to serialize result: {}",
                        e
                    ))
                })?
            );
        } else {
            println!("Pattern Test: Pattern {}", pattern_id);
            println!("==============");
            println!("ID: {}", pattern_id);
            if result.success {
                println!("Execution: ✓ SUCCESS");
                if !result.next_activities.is_empty() {
                    println!("Next activities: {:?}", result.next_activities);
                }
                if result.terminates {
                    println!("Workflow terminates");
                }
            } else {
                println!("Execution: ✗ ERROR");
            }
        }

        if !result.success {
            return Err(clap_noun_verb::NounVerbError::execution_error(
                "Pattern execution failed".to_string(),
            ));
        }

        Ok(())
    })
}

/// Test all 43 patterns
#[verb]
pub fn test_all(state_store: Option<String>, json: bool) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        // Register all patterns
        let mut registry = PatternRegistry::new();
        registry.register_all_patterns();

        let mut results: Vec<(u32, bool, Option<String>)> = Vec::new();

        for id in 1..=43 {
            let pattern_id = PatternId(id);
            if let Some(pattern) = registry.get(&pattern_id) {
                let context = knhk_workflow_engine::patterns::PatternExecutionContext::default();
                let result = pattern.execute(&context);
                if result.success {
                    results.push((id, true, None));
                } else {
                    results.push((id, false, Some("Pattern execution failed".to_string())));
                }
            } else {
                results.push((id, false, Some("Pattern not found".to_string())));
            }
        }

        let passed = results.iter().filter(|(_, success, _)| *success).count();
        let failed = results.len() - passed;

        if json {
            let result_json = serde_json::json!({
                "total_patterns": 43,
                "passed": passed,
                "failed": failed,
                "results": results.iter().map(|(id, success, error)| {
                    serde_json::json!({
                        "pattern_id": id,
                        "success": success,
                        "error": error
                    })
                }).collect::<Vec<_>>()
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&result_json).map_err(|e| {
                    clap_noun_verb::NounVerbError::execution_error(format!(
                        "Failed to serialize result: {}",
                        e
                    ))
                })?
            );
        } else {
            println!("Pattern Test Results");
            println!("===================");
            println!("Total Patterns: 43");
            println!("Passed: {}", passed);
            println!("Failed: {}", failed);
            println!("\nDetailed Results:");
            for (id, success, error) in &results {
                let status = if *success { "✓" } else { "✗" };
                println!(
                    "  {} Pattern {}: {}",
                    status,
                    id,
                    if *success { "PASS" } else { "FAIL" }
                );
                if let Some(err) = error {
                    println!("    Error: {}", err);
                }
            }
        }

        if failed > 0 {
            return Err(clap_noun_verb::NounVerbError::execution_error(format!(
                "{} patterns failed",
                failed
            )));
        }

        Ok(())
    })
}

/// Verify pattern in workflow
#[verb]
pub fn verify(
    pattern_id: u32,
    workflow_file: std::path::PathBuf,
    state_store: Option<String>,
    json: bool,
) -> CnvResult<()> {
    if !(1..=43).contains(&pattern_id) {
        return Err(clap_noun_verb::NounVerbError::execution_error(format!(
            "Pattern ID must be between 1 and 43, got: {}",
            pattern_id
        )));
    }

    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        // Parse workflow
        let mut parser = knhk_workflow_engine::parser::WorkflowParser::new().map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to create parser: {}",
                e
            ))
        })?;

        let spec = parser.parse_file(&workflow_file).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to parse workflow: {}",
                e
            ))
        })?;

        // Register workflow
        engine.register_workflow(spec.clone()).await.map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to register workflow: {}",
                e
            ))
        })?;

        // Get pattern
        let pattern_id_enum = PatternId(pattern_id);
        let mut registry = PatternRegistry::new();
        registry.register_all_patterns();

        let pattern = registry.get(&pattern_id_enum).ok_or_else(|| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Pattern {} not found",
                pattern_id
            ))
        })?;

        // For now, just verify that the pattern exists and can be executed
        // In a full implementation, we would analyze the workflow structure to detect pattern usage
        let context = knhk_workflow_engine::patterns::PatternExecutionContext::default();
        let result = pattern.execute(&context);

        if json {
            let result_json = serde_json::json!({
                "pattern_id": pattern_id,
                "workflow": workflow_file.display().to_string(),
                "pattern_executable": result.success,
                "next_activities": result.next_activities,
                "terminates": result.terminates
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&result_json).map_err(|e| {
                    clap_noun_verb::NounVerbError::execution_error(format!(
                        "Failed to serialize result: {}",
                        e
                    ))
                })?
            );
        } else {
            println!("Pattern Verification");
            println!("===================");
            println!("Pattern: {}", pattern_id);
            println!("Workflow: {}", workflow_file.display());
            if result.success {
                println!("Pattern Status: ✓ Executable");
                if !result.next_activities.is_empty() {
                    println!("Next activities: {:?}", result.next_activities);
                }
            } else {
                println!("Pattern Status: ✗ Error");
            }
        }

        if !result.success {
            return Err(clap_noun_verb::NounVerbError::execution_error(
                "Pattern verification failed".to_string(),
            ));
        }

        Ok(())
    })
}

/// Show pattern coverage for a workflow
#[verb]
pub fn coverage(
    workflow_file: std::path::PathBuf,
    state_store: Option<String>,
    json: bool,
) -> CnvResult<()> {
    let runtime = get_runtime();
    let engine = get_engine(state_store.as_deref())?;

    runtime.block_on(async {
        // Parse workflow
        let mut parser = knhk_workflow_engine::parser::WorkflowParser::new().map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to create parser: {}",
                e
            ))
        })?;

        let spec = parser.parse_file(&workflow_file).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to parse workflow: {}",
                e
            ))
        })?;

        // Register workflow
        engine.register_workflow(spec.clone()).await.map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to register workflow: {}",
                e
            ))
        })?;

        // For now, provide a basic coverage report
        // In a full implementation, we would analyze the workflow structure to detect which patterns are used
        let mut registry = PatternRegistry::new();
        registry.register_all_patterns();

        let total_patterns = 43;
        // Placeholder: would analyze workflow to find actual pattern usage
        let detected_patterns = Vec::<u32>::new();

        if json {
            let result_json = serde_json::json!({
                "workflow": workflow_file.display().to_string(),
                "total_patterns": total_patterns,
                "detected_patterns": detected_patterns,
                "coverage_percent": if total_patterns > 0 {
                    (detected_patterns.len() as f64 / total_patterns as f64) * 100.0
                } else {
                    0.0
                },
                "note": "Pattern detection requires workflow structure analysis (basic implementation)"
            });
            println!("{}", serde_json::to_string_pretty(&result_json).map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to serialize result: {}",
                    e
                ))
            })?);
        } else {
            println!("Pattern Coverage Report");
            println!("======================");
            println!("Workflow: {}", workflow_file.display());
            println!("Total Patterns: {}", total_patterns);
            println!("Detected Patterns: {}", detected_patterns.len());
            println!("Coverage: {:.1}%", if total_patterns > 0 {
                (detected_patterns.len() as f64 / total_patterns as f64) * 100.0
            } else {
                0.0
            });
            println!("\nNote: Pattern detection requires workflow structure analysis");
        }

        Ok(())
    })
}
