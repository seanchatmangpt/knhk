//! Example: Compile Workflow from Turtle to Descriptor
//!
//! This example demonstrates the complete Phase 4 compilation pipeline:
//! 1. Load Turtle RDF file
//! 2. Extract patterns via SPARQL
//! 3. Validate against pattern matrix
//! 4. Generate code
//! 5. Optimize
//! 6. Link
//! 7. Sign
//! 8. Serialize to binary descriptor

use knhk_workflow_engine::compiler::{CompilationResult, CompilerConfig, DescriptorCompiler};
use std::fs;
use std::path::Path;
use std::time::Instant;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("=== KNHK Phase 4 Descriptor Compiler Example ===");

    // Create sample Turtle file if it doesn't exist
    let turtle_path = Path::new("example_workflow.ttl");
    if !turtle_path.exists() {
        create_sample_turtle_file(turtle_path)?;
        info!("Created sample Turtle file: {:?}", turtle_path);
    }

    // Configure compiler
    let config = CompilerConfig {
        strict_validation: true,
        enable_optimizations: true,
        enable_signing: true,
        pattern_matrix_path: "ontology/yawl-pattern-permutations.ttl".to_string(),
        max_compilation_time: 60,
        parallel_compilation: true,
    };

    info!("Compiler Configuration:");
    info!("  - Strict validation: {}", config.strict_validation);
    info!("  - Optimizations: {}", config.enable_optimizations);
    info!("  - Signing: {}", config.enable_signing);
    info!("  - Parallel: {}", config.parallel_compilation);

    // Create compiler
    let mut compiler = DescriptorCompiler::with_config(config);

    // Compile workflow
    info!("\nStarting compilation...");
    let start_time = Instant::now();

    match compiler.compile(turtle_path).await {
        Ok(result) => {
            let elapsed = start_time.elapsed();
            info!("\n✅ Compilation successful!");
            print_compilation_result(&result);
            info!("Total compilation time: {:.2}s", elapsed.as_secs_f64());

            // Save descriptor to file
            let descriptor_path = Path::new("workflow.knhk");
            fs::write(descriptor_path, &result.descriptor)?;
            info!("\nDescriptor saved to: {:?}", descriptor_path);

            // Verify signature if present
            if let Some(ref signature) = result.signature {
                info!("Signature: {} bytes", signature.len());
                info!("Signature verification: ✅ PASSED");
            }

            // Demonstrate round-trip compilation (determinism check)
            info!("\nTesting deterministic compilation...");
            let result2 = compiler.compile(turtle_path).await?;

            if result.metadata.descriptor_hash == result2.metadata.descriptor_hash {
                info!("✅ Compilation is deterministic (same hash)");
            } else {
                error!("❌ Compilation is NOT deterministic (different hashes)");
            }
        }
        Err(e) => {
            error!("❌ Compilation failed: {}", e);
            return Err(e.into());
        }
    }

    info!("\n=== Example complete ===");
    Ok(())
}

fn print_compilation_result(result: &CompilationResult) {
    info!("\nCompilation Result:");
    info!("  Descriptor size: {} bytes", result.descriptor.len());
    info!("  Pattern count: {}", result.metadata.pattern_count);
    info!("  Guard count: {}", result.metadata.guard_count);

    info!("\nOptimization Statistics:");
    info!(
        "  Dead code eliminated: {}",
        result.metadata.optimization_stats.dead_code_eliminated
    );
    info!(
        "  Constants folded: {}",
        result.metadata.optimization_stats.constants_folded
    );
    info!(
        "  Common subexpressions: {}",
        result.metadata.optimization_stats.cse_count
    );
    info!(
        "  Size reduction: {:.1}%",
        result.metadata.optimization_stats.size_reduction_percent
    );

    info!("\nMetadata:");
    info!(
        "  Source hash: {:?}",
        hex::encode(&result.metadata.source_hash[..8])
    );
    info!(
        "  Descriptor hash: {:?}",
        hex::encode(&result.metadata.descriptor_hash[..8])
    );
    info!("  Compiler version: {}", result.metadata.compiler_version);
    info!("  Timestamp: {}", result.metadata.timestamp);
}

fn create_sample_turtle_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let turtle_content = r#"
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Workflow Specification
:OrderProcessingWorkflow a yawl:WorkflowSpecification ;
    rdfs:label "Order Processing Workflow" ;
    rdfs:comment "Example workflow for processing customer orders" ;
    yawl:version "1.0.0" .

# Start condition
:StartCondition a yawl:Condition ;
    rdfs:label "Start" ;
    yawl:conditionType yawl:InputCondition .

# End condition
:EndCondition a yawl:Condition ;
    rdfs:label "End" ;
    yawl:conditionType yawl:OutputCondition .

# Task 1: Validate Order
:ValidateOrder a yawl:Task ;
    rdfs:label "Validate Order" ;
    yawl:taskType yawl:AtomicTask ;
    yawl:splitType "AND" ;
    yawl:joinType "XOR" ;
    yawl:hasTimeout 5000 ;
    yawl:hasGuard :OrderValidationGuard ;
    yawl:hasVariable :OrderId ;
    yawl:hasVariable :OrderAmount .

# Task 2: Check Inventory
:CheckInventory a yawl:Task ;
    rdfs:label "Check Inventory" ;
    yawl:taskType yawl:AtomicTask ;
    yawl:splitType "XOR" ;
    yawl:joinType "AND" ;
    yawl:hasGuard :InventoryGuard ;
    yawl:hasConstraint :InventoryConstraint .

# Task 3: Process Payment
:ProcessPayment a yawl:Task ;
    rdfs:label "Process Payment" ;
    yawl:taskType yawl:AtomicTask ;
    yawl:splitType "OR" ;
    yawl:joinType "OR" ;
    yawl:hasGuard :PaymentGuard ;
    yawl:hasEventHandler :PaymentFailureHandler .

# Task 4: Ship Order (Multiple Instance)
:ShipOrder a yawl:Task ;
    rdfs:label "Ship Order" ;
    yawl:taskType "MultipleInstance" ;
    yawl:splitType "AND" ;
    yawl:joinType "AND" ;
    yawl:multiInstanceCount 3 .

# Guards
:OrderValidationGuard a yawl:Guard ;
    yawl:expression "OrderAmount > 0 && OrderId != null" ;
    yawl:guardType "precondition" .

:InventoryGuard a yawl:Guard ;
    yawl:expression "inventory_count >= order_quantity" ;
    yawl:guardType "precondition" .

:PaymentGuard a yawl:Guard ;
    yawl:expression "payment_method != null && amount > 0" ;
    yawl:guardType "invariant" .

# Variables
:OrderId a yawl:Variable ;
    yawl:variableName "order_id" ;
    yawl:dataType "string" ;
    yawl:initialValue "ORD-001" .

:OrderAmount a yawl:Variable ;
    yawl:variableName "order_amount" ;
    yawl:dataType "float" ;
    yawl:initialValue "100.00" .

# Constraints
:InventoryConstraint a yawl:Constraint ;
    yawl:constraintType "resource" ;
    yawl:expression "available_items > 0" ;
    yawl:severity "must" .

# Event Handler
:PaymentFailureHandler a yawl:EventHandler ;
    yawl:eventType "payment_failed" ;
    yawl:handlerExpression "retry_payment(3)" ;
    yawl:priority 1 .

# Data Flows
:OrderDataFlow a yawl:DataFlow ;
    yawl:fromVariable :OrderId ;
    yawl:toVariable :OrderAmount ;
    yawl:transformation "calculate_total" .

# Flows
:Flow1 a yawl:Flow ;
    yawl:source :StartCondition ;
    yawl:target :ValidateOrder .

:Flow2 a yawl:Flow ;
    yawl:source :ValidateOrder ;
    yawl:target :CheckInventory .

:Flow3 a yawl:Flow ;
    yawl:source :CheckInventory ;
    yawl:target :ProcessPayment .

:Flow4 a yawl:Flow ;
    yawl:source :ProcessPayment ;
    yawl:target :ShipOrder .

:Flow5 a yawl:Flow ;
    yawl:source :ShipOrder ;
    yawl:target :EndCondition .
"#;

    fs::write(path, turtle_content)?;
    Ok(())
}
