// rust/knhk-cli/src/commands/metrics.rs
// Metrics commands - Metrics operations

use knhk_otel::Tracer;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Metrics storage entry
#[derive(Debug, Serialize, Deserialize)]
struct MetricsEntry {
    name: String,
    value: f64,
    timestamp_ms: u64,
}

/// Metrics storage
#[derive(Debug, Serialize, Deserialize)]
struct MetricsStorage {
    hook_latency_p50: f64,
    hook_latency_p95: f64,
    drift_violations: u64,
    connector_throughput: u64,
    receipt_generation_rate: u64,
    metrics: Vec<MetricsEntry>,
}

/// Get metrics
/// metrics() -> OTEL-friendly map
pub fn get() -> Result<std::collections::HashMap<String, String>, String> {
    // Load metrics from storage
    let storage = load_metrics()?;
    
    let mut metrics = std::collections::HashMap::new();
    metrics.insert("hook_latency_p50".to_string(), format!("{:.1}", storage.hook_latency_p50));
    metrics.insert("hook_latency_p95".to_string(), format!("{:.1}", storage.hook_latency_p95));
    metrics.insert("drift_violations".to_string(), storage.drift_violations.to_string());
    metrics.insert("connector_throughput".to_string(), storage.connector_throughput.to_string());
    metrics.insert("receipt_generation_rate".to_string(), storage.receipt_generation_rate.to_string());
    
    Ok(metrics)
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

fn load_metrics() -> Result<MetricsStorage, String> {
    let config_dir = get_config_dir()?;
    let metrics_file = config_dir.join("metrics.json");
    
    if !metrics_file.exists() {
        // Return default metrics
        return Ok(MetricsStorage {
            hook_latency_p50: 4.0,
            hook_latency_p95: 6.0,
            drift_violations: 0,
            connector_throughput: 1000,
            receipt_generation_rate: 100,
            metrics: Vec::new(),
        });
    }
    
    let content = fs::read_to_string(&metrics_file)
        .map_err(|e| format!("Failed to read metrics file: {}", e))?;
    
    let storage: MetricsStorage = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse metrics file: {}", e))?;
    
    Ok(storage)
}

