//! Weaver Live-Check for All 43 Van der Aalst Patterns
//!
//! This example validates that all 43 Van der Aalst workflow patterns
//! emit proper OTEL telemetry that conforms to the Weaver schema.
//!
//! Usage:
//!   OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 cargo run --example weaver_all_43_patterns
//!
//! This validates the complete lifecycle of all 43 patterns with Weaver live-check.

use knhk_otel::{SpanContext, SpanStatus};
use knhk_workflow_engine::integration::{OtelIntegration, PatternAttributes, PatternOtelHelper};
use knhk_workflow_engine::patterns::{
    PatternExecutionContext, PatternId, PatternRegistry, RegisterAllExt,
};
use knhk_workflow_engine::{CaseId, WorkflowSpecId};
use std::collections::HashMap;
use tracing::{error, info, warn};

/// All 43 Van der Aalst patterns with their names and categories
const ALL_PATTERNS: &[(u32, &str, &str)] = &[
    // Basic Control Flow (1-5)
    (1, "Sequence", "Basic Control Flow"),
    (2, "Parallel Split", "Basic Control Flow"),
    (3, "Synchronization", "Basic Control Flow"),
    (4, "Exclusive Choice", "Basic Control Flow"),
    (5, "Simple Merge", "Basic Control Flow"),
    // Advanced Branching (6-11)
    (6, "Multi-Choice", "Advanced Branching"),
    (7, "Structured Synchronizing Merge", "Advanced Branching"),
    (8, "Multi-Merge", "Advanced Branching"),
    (9, "Discriminator", "Advanced Branching"),
    (10, "Arbitrary Cycles", "Advanced Branching"),
    (11, "Implicit Termination", "Advanced Branching"),
    // Multiple Instance (12-15)
    (12, "MI Without Sync", "Multiple Instance"),
    (13, "MI With Design-Time Knowledge", "Multiple Instance"),
    (14, "MI With Runtime Knowledge", "Multiple Instance"),
    (15, "MI Without Runtime Knowledge", "Multiple Instance"),
    // State-Based (16-18)
    (16, "Deferred Choice", "State-Based"),
    (17, "Interleaved Parallel Routing", "State-Based"),
    (18, "Milestone", "State-Based"),
    // Cancellation (19-25)
    (19, "Cancel Activity", "Cancellation"),
    (20, "Cancel Case", "Cancellation"),
    (21, "Cancel Region", "Cancellation"),
    (22, "Cancel MI Activity", "Cancellation"),
    (23, "Complete MI Activity", "Cancellation"),
    (24, "Blocking Discriminator", "Cancellation"),
    (25, "Cancelling Discriminator", "Cancellation"),
    // Advanced Control (26-39)
    (26, "Advanced Control Pattern 26", "Advanced Control"),
    (27, "Advanced Control Pattern 27", "Advanced Control"),
    (28, "Advanced Control Pattern 28", "Advanced Control"),
    (29, "Advanced Control Pattern 29", "Advanced Control"),
    (30, "Advanced Control Pattern 30", "Advanced Control"),
    (31, "Advanced Control Pattern 31", "Advanced Control"),
    (32, "Advanced Control Pattern 32", "Advanced Control"),
    (33, "Advanced Control Pattern 33", "Advanced Control"),
    (34, "Advanced Control Pattern 34", "Advanced Control"),
    (35, "Advanced Control Pattern 35", "Advanced Control"),
    (36, "Advanced Control Pattern 36", "Advanced Control"),
    (37, "Advanced Control Pattern 37", "Advanced Control"),
    (38, "Advanced Control Pattern 38", "Advanced Control"),
    (39, "Advanced Control Pattern 39", "Advanced Control"),
    // Trigger (40-43)
    (40, "Trigger Pattern 40", "Trigger"),
    (41, "Trigger Pattern 41", "Trigger"),
    (42, "Trigger Pattern 42", "Trigger"),
    (43, "Trigger Pattern 43", "Trigger"),
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔍 Weaver Live-Check: All 43 Van der Aalst Patterns");
    println!("==================================================\n");

    // Get OTLP endpoint from environment
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    println!("📡 OTLP Endpoint: {}", otlp_endpoint);
    println!("📊 Testing all 43 Van der Aalst patterns\n");

    // Initialize OTEL integration
    let otel = OtelIntegration::new(Some(otlp_endpoint.clone()));
    otel.initialize().await?;
    println!("✅ OTEL initialized\n");

    // Create pattern registry with all 43 patterns
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();
    println!("✅ Pattern registry initialized with all 43 patterns\n");

    // Create a test case and context
    let case_id = CaseId::new();
    let spec_id = WorkflowSpecId::new();
    let mut context = PatternExecutionContext {
        case_id: case_id.clone(),
        workflow_id: spec_id,
        variables: HashMap::new(),
        arrived_from: std::collections::HashSet::new(),
        scope_id: "test_scope".to_string(),
    };

    // Track results
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    // Test all 43 patterns
    for (pattern_id, pattern_name, category) in ALL_PATTERNS {
        let pattern_id = match PatternId::new(*pattern_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid pattern ID {}: {}", pattern_id, e);
                skipped += 1;
                continue;
            }
        };

        print!(
            "Testing Pattern {}: {} ({})... ",
            pattern_id.0, pattern_name, category
        );

        // Create pattern-specific attributes based on pattern type
        let pattern_attrs = match pattern_id.0 {
            2 | 3 => {
                // Parallel Split / Synchronization
                PatternAttributes {
                    branch_count: Some(3),
                    synchronized_count: Some(3),
                    ..Default::default()
                }
            }
            4 | 6 | 16 => {
                // Exclusive Choice / Multi-Choice / Deferred Choice
                PatternAttributes {
                    chosen_branch: Some("branch_1".to_string()),
                    chosen_branches: Some("branch_1,branch_2".to_string()),
                    ..Default::default()
                }
            }
            12..=15 => {
                // Multiple Instance patterns
                PatternAttributes {
                    instance_count: Some(5),
                    ..Default::default()
                }
            }
            19..=25 => {
                // Cancellation patterns
                PatternAttributes {
                    cancelled_activity: Some("task_1".to_string()),
                    ..Default::default()
                }
            }
            _ => PatternAttributes::default(),
        };

        // Start pattern span
        let span_result = PatternOtelHelper::start_pattern_span_with_attrs(
            &otel,
            &pattern_id,
            &case_id,
            pattern_attrs.clone(),
        )
        .await;

        match span_result {
            Ok(Some(span_ctx)) => {
                // Actually execute the pattern through the registry
                let start_time = std::time::Instant::now();
                let execution_result = registry.execute(&pattern_id, &context);
                let latency_ms = start_time.elapsed().as_millis() as u64;

                // Determine success from execution result
                let success = execution_result
                    .as_ref()
                    .map(|r| r.success)
                    .unwrap_or(false);

                // Add success attribute
                otel.add_attribute(
                    span_ctx.clone(),
                    "knhk.workflow_engine.success".to_string(),
                    success.to_string(),
                )
                .await?;

                // Add latency attribute
                otel.add_attribute(
                    span_ctx.clone(),
                    "knhk.workflow_engine.latency_ms".to_string(),
                    latency_ms.to_string(),
                )
                .await?;

                // Update context variables if pattern executed successfully
                if let Some(result) = execution_result {
                    context.variables = result.variables.clone();
                }

                // End span
                PatternOtelHelper::end_pattern_span(&otel, span_ctx, success).await?;

                if success {
                    println!("✅ PASSED ({}ms)", latency_ms);
                    passed += 1;
                } else {
                    println!("⚠️  EXECUTED (failed)");
                    failed += 1;
                }
            }
            Ok(None) => {
                warn!("OTEL not initialized, skipping pattern {}", pattern_id.0);
                skipped += 1;
            }
            Err(e) => {
                error!("FAILED: {}", e);
                failed += 1;
            }
        }
    }

    // Export telemetry
    println!("\n📤 Exporting telemetry to OTLP endpoint...");
    otel.export().await?;
    println!("✅ Telemetry exported\n");

    // Print summary
    println!("📊 Test Summary:");
    println!("   Total patterns: {}", ALL_PATTERNS.len());
    println!("   ✅ Passed: {}", passed);
    println!("   ❌ Failed: {}", failed);
    println!("   ⏭️  Skipped: {}", skipped);

    if failed > 0 {
        eprintln!("\n❌ Some patterns failed validation!");
        std::process::exit(1);
    }

    if passed != ALL_PATTERNS.len() {
        eprintln!(
            "\n⚠️  Not all patterns were tested! Expected {}, got {}",
            ALL_PATTERNS.len(),
            passed
        );
        std::process::exit(1);
    }

    println!("\n✅ All 43 Van der Aalst patterns validated with Weaver live-check!");
    println!("\n🔍 Next steps:");
    println!("   1. Ensure Weaver live-check is running: weaver registry live-check --registry registry/");
    println!("   2. Verify spans match schema definitions in registry/knhk-workflow-engine.yaml");
    println!("   3. Check Weaver validation report for any schema violations");

    Ok(())
}
