// KNHK Production Deployment Example - Fortune 500 Configuration
// This example demonstrates how to deploy KNHK in a production environment
// with full monitoring, persistence, recovery, and cost tracking

use knhk::{
    ProductionPlatform, PlatformConfig,
    initialize_production,
};
use std::time::Duration;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting KNHK Production Deployment");
    info!("Target: Fortune 500 Enterprise");
    info!("SLA: 99.99% uptime");
    info!("Scale: 1000+ workflows/second");

    // Production configuration
    let config = PlatformConfig {
        // Workflow execution
        max_concurrent_workflows: 10_000,
        workflow_timeout: Duration::from_secs(300),

        // Features
        enable_auto_scaling: true,
        enable_learning: true,
        enable_cost_tracking: true,

        // Infrastructure
        persistence_path: "/var/lib/knhk/data".to_string(),
        cluster_mode: true,
        node_id: format!("knhk-prod-{}", hostname::get()?.to_string_lossy()),

        // Telemetry (connect to your OTLP endpoint)
        telemetry_endpoint: Some("http://otel-collector:4317".to_string()),

        // Health checks
        health_check_port: 9090,
    };

    // Initialize platform
    let mut platform = ProductionPlatform::new(config)?;

    // Start all subsystems
    info!("Starting production platform...");
    platform.start().await?;

    info!("KNHK Production Platform is running");
    info!("Health check endpoint: http://0.0.0.0:9090/health");
    info!("Metrics endpoint: http://0.0.0.0:9090/metrics");

    // Example: Submit a production workflow
    let workflow_descriptor = r#"
        apiVersion: knhk/v1
        kind: Workflow
        metadata:
          name: payment-processing
          department: finance
          cost_center: operations
        spec:
          covenant: Sigma  # Observable composition
          timeout: 60s
          steps:
            - name: validate-payment
              action: validate
              params:
                schema: payment-v2
                strict: true

            - name: check-fraud
              action: fraud-detection
              params:
                threshold: 0.95
                model: ml-fraud-v3

            - name: process-payment
              action: payment-processor
              params:
                provider: stripe
                currency: USD

            - name: update-ledger
              action: ledger-update
              params:
                journal: general-ledger

            - name: send-notification
              action: notify
              params:
                channel: email
                template: payment-success
    "#;

    // Submit workflow
    info!("Submitting example workflow");
    let workflow_id = platform.submit_workflow(workflow_descriptor.to_string()).await?;
    info!("Workflow submitted: {}", workflow_id);

    // Monitor workflow execution
    tokio::spawn(async move {
        loop {
            // In production, you would monitor via observability tools
            tokio::time::sleep(Duration::from_secs(5)).await;

            // Check workflow status
            if let Some(state) = platform.workflows.get(&workflow_id) {
                info!("Workflow {} status: {:?}", workflow_id, state.status);

                if matches!(
                    state.status,
                    knhk::production::platform::WorkflowStatus::Completed |
                    knhk::production::platform::WorkflowStatus::Failed
                ) {
                    break;
                }
            }
        }
    });

    // Set up signal handlers for graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Shutdown signal received");
    };

    // Run until shutdown signal
    shutdown_signal.await;

    // Graceful shutdown
    info!("Initiating graceful shutdown...");
    platform.shutdown().await?;

    info!("KNHK Production Platform shutdown complete");
    Ok(())
}

/// Example: Configure production monitoring
async fn configure_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    use knhk::production::monitoring::{AlertConfig, AlertChannel, AlertThresholds};

    let alert_config = AlertConfig {
        enable_alerts: true,
        alert_channels: vec![
            AlertChannel::PagerDuty {
                service_key: "your-pagerduty-key".to_string(),
            },
            AlertChannel::Slack {
                webhook_url: "https://hooks.slack.com/services/...".to_string(),
            },
            AlertChannel::Email {
                addresses: vec![
                    "ops-team@company.com".to_string(),
                    "on-call@company.com".to_string(),
                ],
            },
        ],
        thresholds: AlertThresholds {
            downtime_seconds: 30,
            latency_p50_ms: 200,
            latency_p99_ms: 2000,
            error_rate_percent: 0.5,
            cpu_percent: 85.0,
            memory_percent: 85.0,
            disk_percent: 90.0,
        },
        cooldown_period: Duration::from_secs(300),
    };

    // Apply configuration
    info!("Monitoring configured with PagerDuty, Slack, and Email alerts");
    Ok(())
}

/// Example: Configure auto-scaling
async fn configure_autoscaling() -> Result<(), Box<dyn std::error::Error>> {
    use knhk::production::scaling::ScalingPolicy;

    let scaling_policy = ScalingPolicy {
        enabled: true,
        min_replicas: 3,  // Minimum for HA
        max_replicas: 100, // Maximum scale
        target_cpu: 70.0,
        target_memory: 70.0,
        target_rps: 1000.0,
        scale_up_increment: 3,    // Add 3 nodes at a time
        scale_down_increment: 1,  // Remove 1 node at a time
        predictive_scaling: true, // Enable ML-based prediction
    };

    info!("Auto-scaling configured: 3-100 replicas, 70% target utilization");
    Ok(())
}

/// Example: Configure cost tracking
async fn configure_cost_tracking() -> Result<(), Box<dyn std::error::Error>> {
    use knhk::production::cost_tracking::PricingModel;

    // Configure pricing based on your cloud provider
    let pricing = PricingModel {
        cpu_per_core_hour: 0.0416,      // AWS m5.large equivalent
        memory_per_gb_hour: 0.00465,
        storage_per_gb_month: 0.023,    // S3 Standard
        network_per_gb: 0.09,           // Internet egress
        io_per_million: 0.10,
        api_per_million: 0.40,
        currency: "USD".to_string(),
    };

    info!("Cost tracking configured with AWS pricing model");
    Ok(())
}

/// Example: Disaster recovery setup
async fn setup_disaster_recovery() -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting up disaster recovery:");
    info!("  - Snapshots every 5 minutes");
    info!("  - Retention: 7 days");
    info!("  - Replication: Cross-region to us-west-2");
    info!("  - RTO: < 15 minutes");
    info!("  - RPO: < 5 minutes");
    Ok(())
}

/// Example: Health check endpoint
async fn health_check_handler() -> impl axum::response::IntoResponse {
    use axum::Json;
    use serde_json::json;

    // In production, check all subsystems
    let health = json!({
        "status": "healthy",
        "checks": {
            "persistence": "ok",
            "observability": "ok",
            "monitoring": "ok",
            "recovery": "ok",
            "scaling": "ok",
            "learning": "ok",
            "cost_tracking": "ok"
        },
        "metrics": {
            "uptime_percentage": 99.99,
            "active_workflows": 42,
            "cpu_usage": 35.2,
            "memory_usage": 48.7
        }
    });

    Json(health)
}

/// Example: Metrics endpoint for Prometheus
async fn metrics_handler() -> String {
    // Return Prometheus-formatted metrics
    r#"
# HELP knhk_workflows_total Total number of workflows processed
# TYPE knhk_workflows_total counter
knhk_workflows_total 1234567

# HELP knhk_workflows_active Number of currently active workflows
# TYPE knhk_workflows_active gauge
knhk_workflows_active 42

# HELP knhk_workflow_duration_seconds Workflow execution duration
# TYPE knhk_workflow_duration_seconds histogram
knhk_workflow_duration_seconds_bucket{le="0.1"} 45000
knhk_workflow_duration_seconds_bucket{le="0.5"} 89000
knhk_workflow_duration_seconds_bucket{le="1.0"} 98000
knhk_workflow_duration_seconds_bucket{le="5.0"} 99900
knhk_workflow_duration_seconds_bucket{le="+Inf"} 100000

# HELP knhk_uptime_percentage Current uptime percentage
# TYPE knhk_uptime_percentage gauge
knhk_uptime_percentage 99.994

# HELP knhk_cost_per_workflow Average cost per workflow in USD
# TYPE knhk_cost_per_workflow gauge
knhk_cost_per_workflow 0.0823
    "#.to_string()
}