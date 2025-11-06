// rust/knhk-sidecar/src/main.rs
// Main entry point for KGC Sidecar server

use knhk_sidecar::{run, SidecarConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SidecarConfig::from_env();
    run(config).await
}

