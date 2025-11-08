#![allow(clippy::unwrap_used)] // CLI tool - unwrap() acceptable for user-facing errors
//! Workflow Engine CLI Commands
//!
//! Provides CLI interface for workflow engine operations:
//! - workflow create: Create a new workflow case
//! - workflow start: Start a workflow case
//! - workflow execute: Execute a workflow case
//! - workflow cancel: Cancel a workflow case
//! - workflow list: List all workflow cases
//! - workflow pattern: Execute a workflow pattern
//! - workflow patterns: List all 43 patterns

use clap_noun_verb::{noun, verb};
use knhk_workflow_engine::{
    case::{Case, CaseId, CaseState},
    error::WorkflowResult,
    parser::WorkflowSpecId,
    patterns::{PatternExecutionContext, PatternId, PatternRegistry},
    state::StateStore,
    WorkflowEngine,
};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

/// Global workflow engine instance
static ENGINE: std::sync::OnceLock<WorkflowEngine> = std::sync::OnceLock::new();

fn get_engine() -> WorkflowResult<&'static WorkflowEngine> {
    ENGINE.get_or_try_init(|| {
        let temp_dir = TempDir::new().map_err(|e| {
            knhk_workflow_engine::error::WorkflowError::StatePersistence(format!(
                "Failed to create temp directory: {}",
                e
            ))
        })?;
        let state_store = StateStore::new(temp_dir.path()).map_err(|e| {
            knhk_workflow_engine::error::WorkflowError::StatePersistence(format!(
                "Failed to create state store: {}",
                e
            ))
        })?;
        Ok(WorkflowEngine::new(state_store))
    })
}

/// Create a new workflow case
#[verb]
pub fn create(
    spec_id: Option<String>,
    data: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let engine = get_engine()?;
    let spec_id = if let Some(id_str) = spec_id {
        WorkflowSpecId::parse_str(&id_str)?
    } else {
        WorkflowSpecId::new()
    };

    let input_data = if let Some(data_str) = data {
        serde_json::from_str(&data_str)?
    } else {
        json!({})
    };

    let case_id = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(engine.create_case(spec_id, input_data))?;

    println!("Created workflow case: {}", case_id);
    Ok(())
}

/// Start a workflow case
#[verb]
pub fn start(case_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let engine = get_engine()?;
    let case_id = CaseId::parse_str(&case_id)?;

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(engine.start_case(case_id))?;

    println!("Started workflow case: {}", case_id);
    Ok(())
}

/// Execute a workflow case
#[verb]
pub fn execute(case_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let engine = get_engine()?;
    let case_id = CaseId::parse_str(&case_id)?;

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(engine.execute_case(case_id))?;

    println!("Executed workflow case: {}", case_id);
    Ok(())
}

/// Cancel a workflow case
#[verb]
pub fn cancel(case_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let engine = get_engine()?;
    let case_id = CaseId::parse_str(&case_id)?;

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(engine.cancel_case(case_id))?;

    println!("Cancelled workflow case: {}", case_id);
    Ok(())
}

/// Get workflow case status
#[verb]
pub fn get(case_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let engine = get_engine()?;
    let case_id = CaseId::parse_str(&case_id)?;

    let case = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(engine.get_case(case_id))?;

    println!("Case ID: {}", case.id);
    println!("State: {:?}", case.state);
    println!("Created: {}", case.created_at);
    if let Some(started) = case.started_at {
        println!("Started: {}", started);
    }
    if let Some(completed) = case.completed_at {
        println!("Completed: {}", completed);
    }
    if let Some(error) = case.error {
        println!("Error: {}", error);
    }

    Ok(())
}

/// List all workflow patterns
#[verb]
pub fn patterns() -> Result<(), Box<dyn std::error::Error>> {
    let engine = get_engine()?;
    let registry = engine.pattern_registry();
    let pattern_ids = registry.list();

    println!(
        "Registered workflow patterns ({} total):",
        pattern_ids.len()
    );
    for pattern_id in pattern_ids {
        println!("  Pattern {}: {}", pattern_id.0, pattern_id);
    }

    Ok(())
}

/// Execute a workflow pattern
#[verb]
pub fn pattern(
    pattern_id: u32,
    case_id: Option<String>,
    spec_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let engine = get_engine()?;
    let pattern_id = PatternId::new(pattern_id)?;
    let registry = engine.pattern_registry();

    let executor = registry
        .get(&pattern_id)
        .ok_or_else(|| format!("Pattern {} not found", pattern_id.0))?;

    let case_id = if let Some(id_str) = case_id {
        CaseId::parse_str(&id_str)?
    } else {
        CaseId::new()
    };

    let spec_id = if let Some(id_str) = spec_id {
        WorkflowSpecId::parse_str(&id_str)?
    } else {
        WorkflowSpecId::new()
    };

    let context = PatternExecutionContext {
        case_id,
        workflow_id: spec_id,
        variables: HashMap::new(),
    };

    let result = executor.execute(&context);

    if result.success {
        println!("Pattern {} executed successfully", pattern_id.0);
        if let Some(next_state) = result.next_state {
            println!("Next state: {}", next_state);
        }
        if !result.variables.is_empty() {
            println!("Output variables:");
            for (key, value) in result.variables {
                println!("  {}: {}", key, value);
            }
        }
    } else {
        eprintln!("Pattern {} execution failed", pattern_id.0);
        return Err("Pattern execution failed".into());
    }

    Ok(())
}
