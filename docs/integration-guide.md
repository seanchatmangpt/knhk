# KNHK Integration Guide

**⚠️ This document has been consolidated. See the [80/20 Integration Guide](INTEGRATION.md) for the single source of truth.**

This file is kept for backward compatibility. All new documentation should reference the consolidated guide.

---

# KNHK Integration Guide

**Version**: 1.0.0
**Date**: 2025-11-06
**Audience**: Developers integrating KNHK components

## Overview

This guide explains how to integrate the four core KNHK crates:

- **knhk-hot** (v1.0.0): Ultra-fast hot path (≤8 ticks)
- **knhk-etl** (v0.1.0): ETL pipeline and data processing
- **knhk-warm** (v0.1.0): Warm path operations (≤500ms)
- **knhk-sidecar** (v0.5.0): gRPC proxy with enterprise features

## Table of Contents

1. [Quick Start](#quick-start)
2. [Integration Patterns](#integration-patterns)
3. [Feature Flags](#feature-flags)
4. [Error Handling](#error-handling)
5. [Performance Considerations](#performance-considerations)
6. [Examples](#examples)
7. [Testing](#testing)
8. [Troubleshooting](#troubleshooting)

## Quick Start

### Minimal Integration

```toml
# Cargo.toml
[dependencies]
knhk-hot = { path = "../knhk-hot", version = "1.0.0" }
knhk-etl = { path = "../knhk-etl", version = "0.1.0" }
```

```rust
use knhk_etl::Pipeline;
use knhk_hot::{Engine, Run, Ir, Receipt, Op, Aligned};

fn main() {
    // Create ETL pipeline
    let mut pipeline = Pipeline::new(
        vec!["my_connector".to_string()],
        "urn:my:schema".to_string(),
        false,
        vec![],
    );

    // Execute pipeline
    let result = pipeline.execute().expect("Pipeline execution failed");
    println!("Receipts: {}, Actions: {}",
        result.receipts_written, result.actions_sent);

    // Use hot path directly for critical operations
    let s = Aligned([1u64; 8]);
    let p = Aligned([10u64; 8]);
    let o = Aligned([100u64; 8]);

    let mut engine = Engine::new(s.0.as_ptr(), p.0.as_ptr(), o.0.as_ptr());
    let run = Run { pred: 10, off: 0, len: 8 };
    engine.pin_run(run).expect("Failed to pin run");

    let mut ir = Ir {
        op: Op::AskSp,
        s: 1, p: 10, o: 0, k: 0,
        out_S: std::ptr::null_mut(),
        out_P: std::ptr::null_mut(),
        out_O: std::ptr::null_mut(),
        out_mask: 0,
    };

    let mut receipt = Receipt::default();
    let found = engine.eval_bool(&mut ir, &mut receipt);

    assert!(receipt.ticks <= 8, "Hot path violated tick budget!");
}
```

## Integration Patterns

### 1. ETL → Hot Path Integration

The most common pattern: ETL pipeline calls hot path for critical operations.

```rust
use knhk_etl::Pipeline;
use knhk_hot::{Engine, Run, Ir, Receipt, Op, Aligned, NROWS};

fn integrate_etl_with_hot_path() -> Result<(), Box<dyn std::error::Error>> {
    // Prepare aligned data (required for hot path)
    let subjects = Aligned([1u64, 2, 3, 4, 5, 6, 7, 8]);
    let predicates = Aligned([100u64, 100, 100, 100, 200, 200, 200, 200]);
    let objects = Aligned([1000u64, 2000, 3000, 4000, 5000, 6000, 7000, 8000]);

    // Create hot path engine
    let mut engine = Engine::new(
        subjects.0.as_ptr(),
        predicates.0.as_ptr(),
        objects.0.as_ptr()
    );

    // Pin a run (defines predicate window)
    let run = Run {
        pred: 100,  // predicate ID
        off: 0,     // offset in arrays
        len: 4,     // number of triples (≤8)
    };

    engine.pin_run(run)?;

    // Execute hot path operation
    let mut ir = Ir {
        op: Op::AskSp,      // Ask if (subject, predicate) exists
        s: 1,               // subject ID to search
        p: 100,             // predicate ID
        o: 0,               // not used for AskSp
        k: 0,               // not used for AskSp
        out_S: std::ptr::null_mut(),
        out_P: std::ptr::null_mut(),
        out_O: std::ptr::null_mut(),
        out_mask: 0,
    };

    let mut receipt = Receipt::default();
    let found = engine.eval_bool(&mut ir, &mut receipt);

    println!("Found: {}, Ticks: {}, Lanes: {}",
        found, receipt.ticks, receipt.lanes);

    // CRITICAL: Verify performance budget
    assert!(receipt.ticks <= 8, "Hot path violated Chatman Constant!");

    Ok(())
}
```

**Key Points**:
- Always use `Aligned<[u64; 8]>` for SoA data (64-byte alignment required)
- Run length must be ≤8 (Hatman Constant H)
- Check `receipt.ticks <= 8` after every operation
- Receipts can be merged using `Receipt::merge()`

### 2. ETL → Warm Path Integration

Integrate warm path SPARQL query execution with ETL pipeline.

```rust
use knhk_etl::integration::{IntegratedPipeline, WarmPathQueryExecutor, WarmPathQueryResult};
use std::collections::BTreeMap;

// Implement warm path executor
struct MyWarmPathExecutor {
    // Your warm path implementation (e.g., knhk_warm::WarmPathExecutor)
}

impl WarmPathQueryExecutor for MyWarmPathExecutor {
    fn execute_query(&self, sparql: &str) -> Result<WarmPathQueryResult, String> {
        // Execute SPARQL query using knhk-warm
        // This is where you'd integrate with knhk_warm::WarmPathExecutor

        // Example mock result
        let mut solution = BTreeMap::new();
        solution.insert("subject".to_string(), "http://example.org/s1".to_string());
        solution.insert("predicate".to_string(), "http://example.org/p1".to_string());
        solution.insert("object".to_string(), "value1".to_string());

        Ok(WarmPathQueryResult::Solutions(vec![solution]))
    }
}

fn integrate_etl_with_warm_path() -> Result<(), Box<dyn std::error::Error>> {
    // Create integrated pipeline
    let mut pipeline = IntegratedPipeline::new(
        vec!["kafka_connector".to_string()],
        "urn:my:schema".to_string(),
        true, // lockchain enabled
        vec!["http://localhost:8080/webhook".to_string()],
    );

    // Set warm path executor
    pipeline.set_warm_path_executor(Box::new(MyWarmPathExecutor {}));

    // Execute pipeline (includes hot path operations)
    let result = pipeline.execute()?;
    println!("Pipeline executed: {} receipts, {} actions",
        result.receipts_written, result.actions_sent);

    // Execute warm path query
    let sparql = "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10";
    let query_result = pipeline.execute_warm_path_query(sparql)?;

    match query_result {
        WarmPathQueryResult::Solutions(solutions) => {
            println!("Query returned {} solutions", solutions.len());
            for solution in solutions {
                println!("  Solution: {:?}", solution);
            }
        }
        WarmPathQueryResult::Boolean(b) => {
            println!("ASK query result: {}", b);
        }
        WarmPathQueryResult::Graph(triples) => {
            println!("CONSTRUCT returned {} triples", triples.len());
        }
    }

    Ok(())
}
```

**Key Points**:
- Implement `WarmPathQueryExecutor` trait for your warm path engine
- Use `IntegratedPipeline` instead of plain `Pipeline`
- Warm path operations have 500ms budget (vs 8 ticks for hot path)
- Query results support SELECT, ASK, and CONSTRUCT query types

### 3. Sidecar Integration

Integrate gRPC sidecar service with ETL pipeline and telemetry.

```rust
use knhk_sidecar::{SidecarConfig, run};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure sidecar
    let mut config = SidecarConfig::default();
    config.listen_address = "0.0.0.0:50051".to_string();
    config.batch_size = 100;
    config.batch_timeout_ms = 1000;
    config.retry_max_attempts = 3;
    config.circuit_breaker_failure_threshold = 5;

    // Enable TLS
    config.tls_enabled = true;
    config.tls_cert_path = Some("/path/to/cert.pem".to_string());
    config.tls_key_path = Some("/path/to/key.pem".to_string());

    // Enable Weaver validation
    config.weaver_enabled = true;
    config.weaver_registry_path = Some("./registry".to_string());
    config.weaver_output_path = Some("./weaver-reports".to_string());
    config.enable_otel = true;

    // Start sidecar (blocks until shutdown)
    run(config).await?;

    Ok(())
}
```

**Key Points**:
- Sidecar automatically integrates with knhk-etl
- Weaver live-check validates telemetry against schema
- Supports batching, retries, circuit breaking
- TLS optional but recommended for production

### 4. Full Stack Integration

Complete integration of all components.

```rust
use knhk_etl::integration::IntegratedPipeline;
use knhk_hot::{Engine, Run, Ir, Receipt, Op, Aligned};
use knhk_sidecar::{SidecarConfig, SidecarClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Start sidecar in background
    let config = SidecarConfig::default();
    tokio::spawn(async move {
        knhk_sidecar::run(config).await
    });

    // 2. Create integrated pipeline
    let mut pipeline = IntegratedPipeline::new(
        vec!["kafka_connector".to_string()],
        "urn:my:schema".to_string(),
        true,
        vec!["http://localhost:8080/webhook".to_string()],
    );

    // 3. Execute pipeline (uses hot path internally)
    let result = pipeline.execute()?;

    // 4. Use hot path directly for critical operations
    let s = Aligned([1u64; 8]);
    let p = Aligned([10u64; 8]);
    let o = Aligned([100u64; 8]);

    let mut engine = Engine::new(s.0.as_ptr(), p.0.as_ptr(), o.0.as_ptr());
    let run = Run { pred: 10, off: 0, len: 8 };
    engine.pin_run(run)?;

    let mut ir = Ir {
        op: Op::CountSpGe,
        s: 1, p: 10, o: 0, k: 5,
        out_S: std::ptr::null_mut(),
        out_P: std::ptr::null_mut(),
        out_O: std::ptr::null_mut(),
        out_mask: 0,
    };

    let mut receipt = Receipt::default();
    let count_result = engine.eval_bool(&mut ir, &mut receipt);

    println!("Pipeline receipts: {}", result.receipts_written);
    println!("Hot path ticks: {}", receipt.ticks);

    Ok(())
}
```

## Feature Flags

### Available Features

| Crate | Feature | Default | Purpose | Dependencies |
|-------|---------|---------|---------|--------------|
| knhk-etl | `std` | ✅ | Enables std library features | reqwest |
| knhk-etl | `knhk-otel` | ❌ | Enables telemetry export | knhk-otel |
| knhk-etl | `knhk-lockchain` | ❌ | Enables immutable audit log | knhk-lockchain |
| knhk-etl | `kafka` | ❌ | Enables Kafka connector | rdkafka |
| knhk-etl | `grpc` | ❌ | Enables gRPC connector | - |
| knhk-warm | `otel` | ❌ | Enables telemetry | knhk-otel |
| knhk-warm | `unrdf` | ❌ | Enables RDF utilities | knhk-unrdf |
| knhk-sidecar | `otel` | ✅ | Enables telemetry | knhk-otel |

### Feature Combinations

#### Development Setup
```toml
[dependencies]
knhk-etl = { path = "../knhk-etl", features = ["std"] }
knhk-hot = { path = "../knhk-hot" }
```

#### Production Setup with Telemetry
```toml
[dependencies]
knhk-etl = { path = "../knhk-etl", features = ["std", "knhk-otel", "knhk-lockchain"] }
knhk-warm = { path = "../knhk-warm", features = ["otel"] }
knhk-sidecar = { path = "../knhk-sidecar" }  # otel enabled by default
knhk-hot = { path = "../knhk-hot" }
```

#### Kafka Integration
```toml
[dependencies]
knhk-etl = { path = "../knhk-etl", features = ["std", "kafka", "knhk-otel"] }
knhk-connectors = { path = "../knhk-connectors", features = ["kafka"] }
```

## Error Handling

### Error Propagation

```rust
use knhk_etl::{Pipeline, PipelineError};
use knhk_hot::Engine;

fn handle_errors() -> Result<(), Box<dyn std::error::Error>> {
    // ETL errors
    let mut pipeline = Pipeline::new(vec![], "urn:test".to_string(), false, vec![]);

    match pipeline.execute() {
        Ok(result) => println!("Success: {} receipts", result.receipts_written),
        Err(PipelineError::ConnectorError(msg)) => {
            eprintln!("Connector failed: {}", msg);
            return Err(msg.into());
        }
        Err(PipelineError::ReflexError(msg)) => {
            eprintln!("Reflex execution failed: {}", msg);
            return Err(msg.into());
        }
        Err(e) => {
            eprintln!("Pipeline error: {:?}", e);
            return Err(e.into());
        }
    }

    // Hot path errors
    let s = knhk_hot::Aligned([1u64; 8]);
    let p = knhk_hot::Aligned([10u64; 8]);
    let o = knhk_hot::Aligned([100u64; 8]);

    let mut engine = Engine::new(s.0.as_ptr(), p.0.as_ptr(), o.0.as_ptr());

    // Run length validation
    let invalid_run = knhk_hot::Run { pred: 10, off: 0, len: 100 };
    match engine.pin_run(invalid_run) {
        Ok(_) => println!("Run pinned"),
        Err(e) => {
            eprintln!("Hot path error: {}", e);
            assert_eq!(e, "H: run.len > 8 blocked");
            return Err(e.into());
        }
    }

    Ok(())
}
```

## Performance Considerations

### Hot Path Budget (≤8 ticks)

```rust
use knhk_hot::{Engine, Run, Ir, Receipt, Op, Aligned};

fn verify_performance() {
    let s = Aligned([1u64; 8]);
    let p = Aligned([10u64; 8]);
    let o = Aligned([100u64; 8]);

    let mut engine = Engine::new(s.0.as_ptr(), p.0.as_ptr(), o.0.as_ptr());
    let run = Run { pred: 10, off: 0, len: 8 };
    engine.pin_run(run).unwrap();

    let mut ir = Ir {
        op: Op::AskSp,
        s: 1, p: 10, o: 0, k: 0,
        out_S: std::ptr::null_mut(),
        out_P: std::ptr::null_mut(),
        out_O: std::ptr::null_mut(),
        out_mask: 0,
    };

    let mut receipt = Receipt::default();
    engine.eval_bool(&mut ir, &mut receipt);

    // CRITICAL: Always verify performance budget
    if receipt.ticks > 8 {
        panic!("Hot path violated Chatman Constant! {} ticks > 8", receipt.ticks);
    }

    println!("✅ Hot path completed in {} ticks (within budget)", receipt.ticks);
}
```

### Warm Path Budget (≤500ms)

```rust
use std::time::Instant;

fn verify_warm_path_performance() {
    let start = Instant::now();

    // Execute warm path query
    // (your SPARQL query execution here)

    let elapsed = start.elapsed();

    if elapsed.as_millis() > 500 {
        eprintln!("⚠️ Warm path exceeded 500ms budget: {:?}", elapsed);
    } else {
        println!("✅ Warm path completed in {:?} (within budget)", elapsed);
    }
}
```

## Examples

See `rust/tests/integration_complete.rs` for comprehensive examples:

- ETL → Hot path integration
- ETL → Warm path integration
- Sidecar → ETL integration
- Feature flag testing
- Error propagation
- Receipt merging
- Concurrent execution

## Testing

### Unit Tests

```bash
# Test individual crates
cd rust/knhk-etl && cargo test
cd rust/knhk-hot && cargo test
cd rust/knhk-warm && cargo test
cd rust/knhk-sidecar && cargo test
```

### Integration Tests

```bash
# Run comprehensive integration tests
cargo test --test integration_complete

# Test specific integration
cargo test --test integration_complete test_etl_hot_path_integration

# Test with features
cargo test --test integration_complete --features knhk-otel
```

### Testcontainer Integration

```bash
# Run testcontainer-based tests
cd rust/knhk-integration-tests
cargo test
```

## Troubleshooting

### Issue: "H: run.len > 8 blocked"

**Cause**: Attempting to pin a run with length > 8 (violates Hatman Constant)

**Solution**: Ensure `Run::len <= 8`:
```rust
let run = Run { pred: 10, off: 0, len: 8 }; // ✅ OK
let run = Run { pred: 10, off: 0, len: 9 }; // ❌ ERROR
```

### Issue: Hot path ticks > 8

**Cause**: Operation exceeded performance budget

**Solution**: Profile operation and optimize, or move to warm path:
```rust
// Check receipt after every operation
let mut receipt = Receipt::default();
engine.eval_bool(&mut ir, &mut receipt);

if receipt.ticks > 8 {
    // Move this operation to warm path
    eprintln!("⚠️ Operation took {} ticks, moving to warm path", receipt.ticks);
}
```

### Issue: "Warm path executor not configured"

**Cause**: Attempting to execute warm path query without setting executor

**Solution**: Set executor before querying:
```rust
let mut pipeline = IntegratedPipeline::new(...);
pipeline.set_warm_path_executor(Box::new(MyExecutor {}));
// Now queries will work
```

### Issue: Feature flag compilation errors

**Cause**: Mismatched feature flags across crates

**Solution**: Align features in Cargo.toml:
```toml
knhk-etl = { features = ["knhk-otel"] }
knhk-warm = { features = ["otel"] }
knhk-sidecar = { features = ["otel"] }  # default enabled
```

### Issue: Alignment errors

**Cause**: Not using `Aligned<[u64; N]>` for SoA data

**Solution**: Always wrap arrays in `Aligned`:
```rust
use knhk_hot::Aligned;

let s = Aligned([1u64, 2, 3, 4, 5, 6, 7, 8]);
let p = Aligned([10u64; 8]);
let o = Aligned([100u64; 8]);

let engine = Engine::new(s.0.as_ptr(), p.0.as_ptr(), o.0.as_ptr());
```

## Additional Resources

- [Integration Analysis Report](./integration-analysis-report.md)
- [Performance Compliance Report](./performance-compliance-report.md)
- [KNHK Architecture Documentation](../README.md)
- [Weaver Integration Guide](../rust/knhk-sidecar/docs/WEAVER_INTEGRATION.md)

---

**Version**: 1.0.0
**Last Updated**: 2025-11-06
**Maintainer**: KNHK Team
