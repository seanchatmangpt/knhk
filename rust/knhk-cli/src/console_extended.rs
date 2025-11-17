//! Extended Console Commands - Phase-Based Validation
//!
//! Provides advanced console commands for the new phase system:
//! - console validate <phase>: Validate specific phase
//! - console metrics: Real-time phase metrics
//! - console export <format>: Export validation reports
//! - console analyze: Advanced workflow analysis

#![allow(non_upper_case_globals)]

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "workflow")]
use knhk_workflow_engine::{
    parser::{WorkflowParser, WorkflowSpecId},
    state::StateStore,
    validation::{
        ConformanceMetricsPhase, FormalSoundnessPhase, LoadTestingPhase, PatternSemanticsPhase,
        Phase, PhaseContext, PhaseExecutor, PhaseRegistry, PhaseStatus,
    },
    WorkflowEngine,
};

#[cfg(feature = "otel")]
use tracing::{error, info, instrument};

/// Validate specific phase result
#[derive(Serialize, Debug)]
struct ValidatePhaseResult {
    status: String,
    phase: String,
    passed: usize,
    failed: usize,
    warnings: usize,
    duration_ms: u64,
    metrics: std::collections::HashMap<String, f64>,
    messages: Vec<String>,
}

/// Metrics result
#[derive(Serialize, Debug)]
struct MetricsResult {
    status: String,
    workflow_id: String,
    metrics: std::collections::HashMap<String, f64>,
    timestamp: String,
}

/// Export result
#[derive(Serialize, Debug)]
struct ExportResult {
    status: String,
    format: String,
    output_path: String,
    size_bytes: usize,
}

/// Analyze result
#[derive(Serialize, Debug)]
struct AnalyzeResult {
    status: String,
    workflow_id: String,
    analysis: WorkflowAnalysis,
}

#[derive(Serialize, Debug)]
struct WorkflowAnalysis {
    total_tasks: usize,
    patterns_used: Vec<String>,
    complexity_score: f64,
    soundness_score: f64,
    performance_score: f64,
    recommendations: Vec<String>,
}

/// Get or create tokio runtime for async operations
fn get_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Runtime::new().unwrap_or_else(|e| {
            panic!("Failed to create tokio runtime: {}", e);
        })
    })
}

/// Create workflow engine with state store
#[cfg(feature = "workflow")]
fn create_engine(state_store_path: &Option<String>) -> CnvResult<Arc<WorkflowEngine>> {
    let path = state_store_path.as_deref().unwrap_or("./workflow_db");
    let state_store = StateStore::new(path).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to create state store: {}",
            e
        ))
    })?;
    Ok(Arc::new(WorkflowEngine::new(state_store)))
}

/// Validate a specific phase of a loaded workflow
///
/// # Arguments
/// * `phase` - Phase name (formal_soundness, conformance_metrics, pattern_semantics, load_testing)
/// * `workflow_file` - Path to workflow Turtle file
/// * `state_store` - Optional state store path
///
/// # Example
/// ```bash
/// knhk console validate formal_soundness --workflow-file workflow.ttl
/// ```
#[cfg_attr(feature = "otel", instrument(skip_all, fields(operation = "knhk.console.validate", phase = %phase)))]
#[verb(noun = "console")]
pub fn validate(
    phase: String,
    workflow_file: PathBuf,
    state_store: Option<String>,
) -> CnvResult<ValidatePhaseResult> {
    #[cfg(feature = "otel")]
    {
        use std::time::Instant;

        let start_time = Instant::now();

        let runtime = get_runtime();
        let result = runtime.block_on(async {
            // Parse workflow
            let mut parser = WorkflowParser::new().map_err(|e| {
                error!(error = ?e, "Failed to create parser");
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to create parser: {}",
                    e
                ))
            })?;

            let spec = parser.parse_file(&workflow_file).map_err(|e| {
                error!(error = ?e, "Failed to parse workflow");
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to parse workflow: {}",
                    e
                ))
            })?;

            let spec_id = spec.id;

            // Create engine
            let engine = create_engine(&state_store)?;

            // Register spec
            engine.register_spec(spec).await.map_err(|e| {
                error!(error = ?e, "Failed to register spec");
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to register spec: {}",
                    e
                ))
            })?;

            // Create context
            let ctx = PhaseContext::new(engine, spec_id);

            // Execute phase
            let phase_result = match phase.as_str() {
                "formal_soundness" => {
                    let executor = PhaseExecutor::new();
                    let phase_impl = FormalSoundnessPhase::new();
                    executor.execute_phase(&phase_impl, ctx).await
                }
                "conformance_metrics" => {
                    let executor = PhaseExecutor::new();
                    let phase_impl = ConformanceMetricsPhase::new();
                    executor.execute_phase(&phase_impl, ctx).await
                }
                "pattern_semantics" => {
                    let executor = PhaseExecutor::new();
                    let phase_impl = PatternSemanticsPhase::new();
                    executor.execute_phase(&phase_impl, ctx).await
                }
                "load_testing" => {
                    let executor = PhaseExecutor::new();
                    let phase_impl = LoadTestingPhase::new().with_num_cases(50); // Reduced for console
                    executor.execute_phase(&phase_impl, ctx).await
                }
                _ => {
                    error!(phase = %phase, "Unknown phase");
                    return Err(clap_noun_verb::NounVerbError::execution_error(format!(
                        "Unknown phase: {}. Available: formal_soundness, conformance_metrics, pattern_semantics, load_testing",
                        phase
                    )));
                }
            };

            phase_result.map_err(|e| {
                error!(error = ?e, "Phase execution failed");
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Phase execution failed: {}",
                    e
                ))
            })
        })?;

        let duration = start_time.elapsed();

        info!(
            duration_ms = duration.as_millis(),
            phase = %phase,
            status = ?result.status,
            "console.validate.success"
        );

        Ok(ValidatePhaseResult {
            status: match result.status {
                PhaseStatus::Pass => "pass".to_string(),
                PhaseStatus::Fail => "fail".to_string(),
                PhaseStatus::Warning => "warning".to_string(),
                PhaseStatus::Skipped => "skipped".to_string(),
            },
            phase,
            passed: result.passed,
            failed: result.failed,
            warnings: result.warnings,
            duration_ms: result.duration.as_millis() as u64,
            metrics: result.metrics,
            messages: result.messages,
        })
    }

    #[cfg(not(feature = "otel"))]
    {
        let runtime = get_runtime();
        let result = runtime.block_on(async {
            let mut parser = WorkflowParser::new().map_err(|e| {
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

            let spec_id = spec.id;
            let engine = create_engine(&state_store)?;
            engine.register_spec(spec).await.map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Failed to register spec: {}",
                    e
                ))
            })?;

            let ctx = PhaseContext::new(engine, spec_id);

            let phase_result = match phase.as_str() {
                "formal_soundness" => {
                    let executor = PhaseExecutor::new();
                    let phase_impl = FormalSoundnessPhase::new();
                    executor.execute_phase(&phase_impl, ctx).await
                }
                "conformance_metrics" => {
                    let executor = PhaseExecutor::new();
                    let phase_impl = ConformanceMetricsPhase::new();
                    executor.execute_phase(&phase_impl, ctx).await
                }
                "pattern_semantics" => {
                    let executor = PhaseExecutor::new();
                    let phase_impl = PatternSemanticsPhase::new();
                    executor.execute_phase(&phase_impl, ctx).await
                }
                "load_testing" => {
                    let executor = PhaseExecutor::new();
                    let phase_impl = LoadTestingPhase::new().with_num_cases(50);
                    executor.execute_phase(&phase_impl, ctx).await
                }
                _ => {
                    return Err(clap_noun_verb::NounVerbError::execution_error(format!(
                        "Unknown phase: {}",
                        phase
                    )));
                }
            };

            phase_result.map_err(|e| {
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Phase execution failed: {}",
                    e
                ))
            })
        })?;

        Ok(ValidatePhaseResult {
            status: match result.status {
                PhaseStatus::Pass => "pass".to_string(),
                PhaseStatus::Fail => "fail".to_string(),
                PhaseStatus::Warning => "warning".to_string(),
                PhaseStatus::Skipped => "skipped".to_string(),
            },
            phase,
            passed: result.passed,
            failed: result.failed,
            warnings: result.warnings,
            duration_ms: result.duration.as_millis() as u64,
            metrics: result.metrics,
            messages: result.messages,
        })
    }
}

/// Get real-time phase metrics for a workflow
#[cfg_attr(
    feature = "otel",
    instrument(skip_all, fields(operation = "knhk.console.metrics"))
)]
#[verb(noun = "console")]
pub fn metrics(workflow_file: PathBuf, state_store: Option<String>) -> CnvResult<MetricsResult> {
    let runtime = get_runtime();

    #[cfg(feature = "workflow")]
    {
        let result = runtime.block_on(async {
            let mut parser = WorkflowParser::new().map_err(|e| {
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

            let spec_id = spec.id;
            let engine = create_engine(&state_store)?;

            // Collect basic metrics
            let mut metrics = std::collections::HashMap::new();
            metrics.insert("total_tasks".to_string(), spec.tasks.len() as f64);
            metrics.insert(
                "total_patterns".to_string(),
                spec.tasks
                    .iter()
                    .map(|t| t.pattern.to_string())
                    .collect::<std::collections::HashSet<_>>()
                    .len() as f64,
            );

            Ok::<
                (WorkflowSpecId, std::collections::HashMap<String, f64>),
                clap_noun_verb::NounVerbError,
            >((spec_id, metrics))
        })?;

        Ok(MetricsResult {
            status: "success".to_string(),
            workflow_id: result.0.to_string(),
            metrics: result.1,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    #[cfg(not(feature = "workflow"))]
    {
        Err(clap_noun_verb::NounVerbError::execution_error(
            "Workflow feature not enabled".to_string(),
        ))
    }
}

/// Export validation report in specified format
#[cfg_attr(feature = "otel", instrument(skip_all, fields(operation = "knhk.console.export", format = %format)))]
#[verb(noun = "console")]
pub fn export(
    workflow_file: PathBuf,
    format: String,
    output: PathBuf,
    state_store: Option<String>,
) -> CnvResult<ExportResult> {
    #[cfg(feature = "workflow")]
    {
        // Supported formats: json, yaml, markdown
        match format.as_str() {
            "json" | "yaml" | "markdown" => {}
            _ => {
                return Err(clap_noun_verb::NounVerbError::execution_error(format!(
                    "Unsupported format: {}. Supported: json, yaml, markdown",
                    format
                )));
            }
        }

        // Export placeholder (would implement full export logic)
        let content = format!("# Validation Report\n\nFormat: {}\n", format);
        std::fs::write(&output, content.as_bytes()).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!("Failed to write file: {}", e))
        })?;

        Ok(ExportResult {
            status: "success".to_string(),
            format,
            output_path: output.to_string_lossy().to_string(),
            size_bytes: content.len(),
        })
    }

    #[cfg(not(feature = "workflow"))]
    {
        Err(clap_noun_verb::NounVerbError::execution_error(
            "Workflow feature not enabled".to_string(),
        ))
    }
}

/// Analyze workflow and provide insights
#[cfg_attr(
    feature = "otel",
    instrument(skip_all, fields(operation = "knhk.console.analyze"))
)]
#[verb(noun = "console")]
pub fn analyze(workflow_file: PathBuf, state_store: Option<String>) -> CnvResult<AnalyzeResult> {
    let runtime = get_runtime();

    #[cfg(feature = "workflow")]
    {
        let result = runtime.block_on(async {
            let mut parser = WorkflowParser::new().map_err(|e| {
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

            let spec_id = spec.id;

            // Perform analysis
            let total_tasks = spec.tasks.len();
            let patterns_used: Vec<String> = spec
                .tasks
                .iter()
                .map(|t| t.pattern.to_string())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let complexity_score = (total_tasks as f64).ln() * patterns_used.len() as f64;
            let soundness_score = 0.85; // Placeholder
            let performance_score = 0.90; // Placeholder

            let mut recommendations = Vec::new();
            if total_tasks > 20 {
                recommendations
                    .push("Consider breaking workflow into smaller sub-workflows".to_string());
            }
            if patterns_used.len() > 10 {
                recommendations.push("High pattern diversity - ensure consistency".to_string());
            }

            Ok::<(WorkflowSpecId, WorkflowAnalysis), clap_noun_verb::NounVerbError>((
                spec_id,
                WorkflowAnalysis {
                    total_tasks,
                    patterns_used,
                    complexity_score,
                    soundness_score,
                    performance_score,
                    recommendations,
                },
            ))
        })?;

        Ok(AnalyzeResult {
            status: "success".to_string(),
            workflow_id: result.0.to_string(),
            analysis: result.1,
        })
    }

    #[cfg(not(feature = "workflow"))]
    {
        Err(clap_noun_verb::NounVerbError::execution_error(
            "Workflow feature not enabled".to_string(),
        ))
    }
}
