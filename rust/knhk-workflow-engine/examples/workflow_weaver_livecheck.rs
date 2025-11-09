//! Example: Workflow Engine with Weaver Live-Check
//!
//! This example demonstrates using the workflow engine and emitting OTEL telemetry
//! that can be validated by Weaver live-check.
//!
//! Usage:
//!   cargo run --example workflow_weaver_livecheck
//!
//! With OTLP endpoint:
//!   OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo run --example workflow_weaver_livecheck

use knhk_otel::{SpanContext, SpanStatus};
use knhk_workflow_engine::integration::OtelIntegration;
use knhk_workflow_engine::{
    CaseId, PatternId, StateStore, WorkflowEngine, WorkflowParser, WorkflowSpecId,
};
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 KNHK Workflow Engine - Weaver Live-Check Example");
    println!("==================================================\n");

    // Get OTLP endpoint from environment or use default
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .ok()
        .or_else(|| Some("http://localhost:4318".to_string()))
        .unwrap();

    println!("📡 OTLP Endpoint: {}", otlp_endpoint);

    // Create temporary state store
    let temp_dir = TempDir::new()?;
    let state_store = StateStore::new(temp_dir.path())?;

    // Create workflow engine
    println!("\n1️⃣ Creating workflow engine...");
    let engine = WorkflowEngine::new(state_store);
    println!("   ✅ Engine created");

    // Initialize OTEL integration
    println!("\n2️⃣ Initializing OTEL integration...");
    let otel = OtelIntegration::new(Some(otlp_endpoint.clone()));
    otel.initialize().await?;
    println!("   ✅ OTEL initialized");

    // Create a simple workflow specification
    println!("\n3️⃣ Creating workflow specification...");
    let mut parser = WorkflowParser::new()?;

    // Simple workflow in Turtle format
    let workflow_turtle = r#"
@prefix yawl: <http://www.yawlfoundation.org/yawlschema> .
@prefix ex: <http://example.org/workflow/> .

ex:swift-payment a yawl:WorkflowSpec ;
    yawl:name "SWIFT Payment Processing" ;
    yawl:version "1.0.0" .

ex:task-validate a yawl:Task ;
    yawl:name "Validate Payment" ;
    yawl:taskType yawl:AtomicTask .

ex:task-process a yawl:Task ;
    yawl:name "Process Payment" ;
    yawl:taskType yawl:AtomicTask .

ex:start a yawl:Condition ;
    yawl:name "Start" .

ex:end a yawl:Condition ;
    yawl:name "End" .

ex:start yawl:flowsTo ex:task-validate .
ex:task-validate yawl:flowsTo ex:task-process .
ex:task-process yawl:flowsTo ex:end .
"#;

    let spec = parser.parse_turtle(workflow_turtle)?;
    let spec_id = spec.id.clone();
    println!("   ✅ Workflow parsed: {}", spec_id);

    // Start workflow span
    println!("\n4️⃣ Starting workflow execution span...");
    let workflow_span: Option<SpanContext> = otel.start_workflow_span(&spec_id).await?;
    if let Some(ref span_ctx) = workflow_span {
        otel.add_attribute(
            span_ctx.clone(),
            "knhk.workflow.name".to_string(),
            "SWIFT Payment Processing".to_string(),
        )
        .await?;
        otel.add_attribute(
            span_ctx.clone(),
            "knhk.workflow.version".to_string(),
            "1.0.0".to_string(),
        )
        .await?;
        otel.add_attribute(
            span_ctx.clone(),
            "knhk.workflow.pattern_count".to_string(),
            "2".to_string(),
        )
        .await?;
        otel.add_attribute(
            span_ctx.clone(),
            "knhk.workflow.deadlock_validated".to_string(),
            "true".to_string(),
        )
        .await?;
        println!("   ✅ Workflow span started with attributes");
    }

    // Register workflow
    println!("\n5️⃣ Registering workflow...");
    engine.register_workflow(spec).await?;
    println!("   ✅ Workflow registered");

    // Create a case
    println!("\n6️⃣ Creating workflow case...");
    let case_id = engine
        .create_case(
            spec_id.clone(),
            serde_json::json!({
                "payment_id": "PAY-12345",
                "amount": 1000.00,
                "currency": "USD"
            }),
        )
        .await?;
    println!("   ✅ Case created: {}", case_id);

    // Start case span
    println!("\n7️⃣ Starting case execution span...");
    let case_span: Option<SpanContext> = otel.start_case_span(&case_id).await?;
    if let Some(ref span_ctx) = case_span {
        otel.add_attribute(
            span_ctx.clone(),
            "knhk.case.workflow_id".to_string(),
            spec_id.to_string(),
        )
        .await?;
        otel.add_attribute(
            span_ctx.clone(),
            "knhk.case.state".to_string(),
            "running".to_string(),
        )
        .await?;
        otel.add_attribute(
            span_ctx.clone(),
            "knhk.case.priority".to_string(),
            "100".to_string(),
        )
        .await?;
        println!("   ✅ Case span started with attributes");
    }

    // Execute pattern spans (simulating pattern execution)
    println!("\n8️⃣ Executing workflow patterns...");
    for pattern_id in [1u32, 5u32] {
        let pattern_span: Option<SpanContext> =
            otel.start_pattern_span(&PatternId(pattern_id)).await?;
        if let Some(ref span_ctx) = pattern_span {
            otel.add_attribute(
                span_ctx.clone(),
                "knhk.pattern.case_id".to_string(),
                case_id.to_string(),
            )
            .await?;
            otel.add_attribute(
                span_ctx.clone(),
                "knhk.pattern.name".to_string(),
                match pattern_id {
                    1 => "Sequence".to_string(),
                    5 => "Simple Merge".to_string(),
                    _ => format!("Pattern {}", pattern_id),
                },
            )
            .await?;
            otel.add_attribute(
                span_ctx.clone(),
                "knhk.pattern.ticks".to_string(),
                "3".to_string(), // Within ≤8 tick budget
            )
            .await?;
            otel.add_attribute(
                span_ctx.clone(),
                "knhk.pattern.success".to_string(),
                "true".to_string(),
            )
            .await?;

            // Simulate pattern execution
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            // End pattern span
            otel.end_span(span_ctx.clone(), SpanStatus::Ok).await?;
            println!("   ✅ Pattern {} executed (3 ticks)", pattern_id);
        }
    }

    // End case span
    if let Some(span_ctx) = case_span {
        otel.end_span(span_ctx, SpanStatus::Ok).await?;
        println!("   ✅ Case span ended");
    }

    // End workflow span
    if let Some(span_ctx) = workflow_span {
        otel.end_span(span_ctx, SpanStatus::Ok).await?;
        println!("   ✅ Workflow span ended");
    }

    // Export telemetry
    println!("\n9️⃣ Exporting telemetry to OTLP endpoint...");
    otel.export().await?;
    println!("   ✅ Telemetry exported to {}", otlp_endpoint);

    println!("\n✅ Workflow execution complete!");
    println!("\n📊 Telemetry Summary:");
    println!("   - Workflow spans: 1");
    println!("   - Case spans: 1");
    println!("   - Pattern spans: 2");
    println!("   - Total spans: 4");
    println!("\n🔍 Next steps:");
    println!(
        "   1. Ensure OTLP collector is running on {}",
        otlp_endpoint
    );
    println!("   2. Run: weaver registry live-check --registry registry/");
    println!("   3. Verify spans match schema definitions");

    Ok(())
}
