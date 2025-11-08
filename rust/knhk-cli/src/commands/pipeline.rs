// rust/knhk-cli/src/commands/pipeline.rs
// Pipeline commands - ETL pipeline operations

use crate::connector::ConnectorRegistry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[cfg(feature = "std")]
use knhk_etl::integration::IntegratedPipeline;

/// Pipeline execution status
#[derive(Debug, Serialize, Deserialize)]
struct PipelineStatus {
    last_execution: Option<u64>, // timestamp_ms
    last_result: Option<PipelineExecutionResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PipelineExecutionResult {
    receipts_written: usize,
    actions_sent: usize,
    lockchain_hashes: Vec<String>,
    timestamp_ms: u64,
}

/// Execute full ETL pipeline (Ingest → Emit)
pub fn run(connectors: Option<String>, schema: Option<String>) -> Result<(), String> {
    #[cfg(feature = "otel")]
    let _span = tracing::span!(
        tracing::Level::INFO,
        "knhk.pipeline.run",
        knhk.operation.name = "pipeline.run",
        knhk.operation.type = "etl"
    );
    #[cfg(feature = "otel")]
    let _enter = _span.enter();

    println!("Executing ETL pipeline...");

    // Use ConnectorRegistry to get connectors
    let connector_registry = ConnectorRegistry::new()?;

    // Determine connectors to use
    let connector_ids: Vec<String> = if let Some(ref conns) = connectors {
        conns.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        // Use all registered connectors
        connector_registry.list()?
    };

    if connector_ids.is_empty() {
        return Err("No connectors specified or registered".to_string());
    }

    // Get schema (default or from config)
    let schema_iri = schema.unwrap_or_else(|| "urn:knhk:schema:default".to_string());

    println!("  Connectors: {}", connector_ids.join(", "));
    println!("  Schema: {}", schema_iri);

    // Verify connectors exist
    for conn_id in &connector_ids {
        connector_registry.get(conn_id)?;
    }

    #[cfg(feature = "std")]
    {
        // Create integrated pipeline with actual connectors
        let mut pipeline = IntegratedPipeline::new(
            connector_ids,
            schema_iri,
            true,   // lockchain enabled
            vec![], // downstream endpoints (empty for now)
        );

        // Execute pipeline
        match pipeline.execute() {
            Ok(result) => {
                #[cfg(feature = "otel")]
                {
                    use knhk_otel::{MetricsHelper, Tracer};
                    let mut tracer = Tracer::new();
                    MetricsHelper::record_operation(&mut tracer, "pipeline.run", true);
                    MetricsHelper::record_connector_throughput(
                        &mut tracer,
                        "pipeline",
                        result.receipts_written,
                    );
                }

                println!("✓ Pipeline execution completed");
                println!("  Receipts written: {}", result.receipts_written);
                println!("  Actions sent: {}", result.actions_sent);
                println!("  Lockchain hashes: {}", result.lockchain_hashes.len());

                // Save execution status
                let status = PipelineStatus {
                    last_execution: Some(get_current_timestamp_ms()),
                    last_result: Some(PipelineExecutionResult {
                        receipts_written: result.receipts_written,
                        actions_sent: result.actions_sent,
                        lockchain_hashes: result.lockchain_hashes,
                        timestamp_ms: get_current_timestamp_ms(),
                    }),
                };
                save_pipeline_status(&status)?;

                Ok(())
            }
            Err(e) => {
                #[cfg(feature = "otel")]
                {
                    use knhk_otel::{MetricsHelper, Tracer};
                    let mut tracer = Tracer::new();
                    MetricsHelper::record_operation(&mut tracer, "pipeline.run", false);
                }
                Err(format!("Pipeline execution failed: {:?}", e))
            }
        }
    }

    #[cfg(not(feature = "std"))]
    {
        Err("Pipeline execution requires std feature".to_string())
    }
}

/// Show pipeline execution status and metrics
pub fn status() -> Result<String, String> {
    let status = load_pipeline_status()?;

    if let Some(ref result) = status.last_result {
        let status_str = format!(
            "Last execution: {} ms ago\nReceipts written: {}\nActions sent: {}\nLockchain hashes: {}",
            get_current_timestamp_ms().saturating_sub(result.timestamp_ms),
            result.receipts_written,
            result.actions_sent,
            result.lockchain_hashes.len()
        );
        Ok(status_str)
    } else {
        Ok("Pipeline Status: No executions recorded".to_string())
    }
}

fn get_config_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let mut path = PathBuf::from(std::env::var("APPDATA").map_err(|_| "APPDATA not set")?);
        path.push("knhk");
        Ok(path)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").map_err(|_| "HOME not set")?;
        let mut path = PathBuf::from(home);
        path.push(".knhk");
        Ok(path)
    }
}

fn get_current_timestamp_ms() -> u64 {
    #[cfg(feature = "std")]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    #[cfg(not(feature = "std"))]
    {
        0
    }
}

fn load_pipeline_status() -> Result<PipelineStatus, String> {
    let config_dir = get_config_dir()?;
    let status_file = config_dir.join("pipeline_status.json");

    if !status_file.exists() {
        return Ok(PipelineStatus {
            last_execution: None,
            last_result: None,
        });
    }

    let content = fs::read_to_string(&status_file)
        .map_err(|e| format!("Failed to read pipeline status file: {}", e))?;

    let status: PipelineStatus = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse pipeline status file: {}", e))?;

    Ok(status)
}

fn save_pipeline_status(status: &PipelineStatus) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let status_file = config_dir.join("pipeline_status.json");
    let content = serde_json::to_string_pretty(status)
        .map_err(|e| format!("Failed to serialize pipeline status: {}", e))?;

    fs::write(&status_file, content)
        .map_err(|e| format!("Failed to write pipeline status file: {}", e))?;

    Ok(())
}
