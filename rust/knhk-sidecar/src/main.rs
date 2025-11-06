// rust/knhk-sidecar/src/main.rs
// KGC Sidecar main entry point

use knhk_sidecar::ServerConfig;
use knhk_sidecar::start_server;
use std::env;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    // Load configuration from environment
    let listen_addr = env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string());
    
    let warm_endpoint = env::var("WARM_ORCHESTRATOR_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:50052".to_string());
    
    let batch_size = env::var("BATCH_SIZE")
        .unwrap_or_else(|_| "8".to_string())
        .parse::<usize>()
        .unwrap_or(8);
    
    let batch_timeout_ms = env::var("BATCH_TIMEOUT_MS")
        .unwrap_or_else(|_| "100".to_string())
        .parse::<u64>()
        .unwrap_or(100);
    
    // Create server configuration
    let config = ServerConfig {
        listen_addr,
        batch_config: knhk_sidecar::BatchConfig {
            max_batch_size: batch_size,
            batch_timeout_ms,
        },
        retry_config: knhk_sidecar::RetryConfig::default(),
        warm_client_config: knhk_sidecar::WarmClientConfig {
            endpoint: warm_endpoint,
            ..Default::default()
        },
    };
    
    info!("Starting KGC Sidecar server");
    info!("Listen address: {}", config.listen_addr);
    info!("Warm orchestrator endpoint: {}", config.warm_client_config.endpoint);
    info!("Batch size: {}", config.batch_config.max_batch_size);
    info!("Batch timeout: {}ms", config.batch_config.batch_timeout_ms);
    
    // Start server
    match start_server(config).await {
        Ok(_) => {
            info!("Server stopped");
            Ok(())
        }
        Err(e) => {
            error!("Server error: {}", e);
            Err(Box::new(e))
        }
    }
}

