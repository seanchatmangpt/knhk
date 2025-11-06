# Weaver Pattern Alignment Analysis for KNHK Sidecar

## Executive Summary

The KNHK sidecar now integrates the ETL pipeline directly into service methods, aligning with several Weaver architectural patterns. This document analyzes the current implementation against Weaver's patterns and identifies opportunities for improvement.

## Current Implementation Analysis

### ✅ Patterns Already Implemented

#### 1. **Ingester Pattern** (Weaver Pattern: Multiple Input Sources)
**Current State**: ✅ Implemented
- `IngestStage` handles RDF/Turtle parsing from multiple sources
- Unified interface for different input types (RDF data in gRPC requests)
- Similar to Weaver's `Ingester` trait pattern

**Code Example**:
```rust
let ingest = knhk_etl::IngestStage::new(vec!["grpc".to_string()], "turtle".to_string());
let triples = ingest.parse_rdf_turtle(&turtle_data)?;
```

**Weaver Equivalent**: `JsonFileIngester`, `OtlpIngester`, etc.

#### 2. **Pipeline Pattern** (Weaver Pattern: Stage-Based Processing)
**Current State**: ✅ Fully Implemented
- Clear stage separation: Ingest → Transform → Load → Reflex → Emit
- Each stage has well-defined inputs/outputs
- Error propagation through stages

**Code Example**:
```rust
let ingest_result = ingest.parse_rdf_turtle(&turtle_data)?;
let transform_result = transform.transform(ingest_result)?;
let load_result = load.load(transform_result)?;
let reflex_result = reflex.reflex(load_result)?;
let emit_result = emit.emit(reflex_result)?;
```

**Weaver Equivalent**: Weaver's validation pipeline (ingest → validate → report)

#### 3. **Error Handling** (Weaver Pattern: Structured Diagnostics)
**Current State**: ✅ Good Foundation
- Using `thiserror` for error types
- Clear error variants: `TransactionFailed`, `QueryFailed`, `ValidationFailed`, `HookEvaluationFailed`
- Error context preserved through stages

**Improvement Opportunity**: Add `#[non_exhaustive]` for extensibility (Weaver pattern)

#### 4. **Telemetry Integration** (Weaver Pattern: Observability by Design)
**Current State**: ✅ Implemented
- Continuous telemetry export to Weaver
- Spans for each operation with attributes
- Metrics recording (latency, counts)
- Integration with Weaver live-check

**Code Example**:
```rust
self.export_telemetry(
    "knhk.sidecar.transaction",
    "apply_transaction",
    success,
    latency_ms,
    vec![
        ("transaction_id", transaction_id.clone()),
        ("method", "ApplyTransaction".to_string()),
        ("rdf_bytes", req.rdf_data.len().to_string()),
    ],
).await;
```

**Weaver Equivalent**: Weaver's telemetry validation and semantic convention checking

### 🔄 Patterns Partially Implemented

#### 1. **Policy Engine** (Weaver Pattern: Rego-Based Policies)
**Current State**: 🔄 Partial
- Schema validation exists (`TransformStage` with schema validation)
- Guard constraints enforced (max_run_len ≤ 8)
- **Missing**: Rego-based policy engine for custom rules

**Improvement Opportunity**: 
- Integrate Rego policy engine (like Weaver's `weaver_checker`)
- Apply to guard constraints, performance budgets, receipt validation
- Custom validation rules via policies

#### 2. **Schema Resolution** (Weaver Pattern: Resolved Schemas)
**Current State**: 🔄 Partial
- Schema IRI passed to `TransformStage`
- Schema validation enabled/disabled flag
- **Missing**: Schema catalog, version management, dependency tracking

**Improvement Opportunity**:
- Implement resolved schema pattern (like Weaver's `ResolvedTelemetrySchema`)
- Schema catalog for shared definitions
- Version management and dependency tracking

#### 3. **Streaming Processing** (Weaver Pattern: Streaming Ingesters)
**Current State**: 🔄 Partial
- ETL pipeline processes data in stages
- **Missing**: True streaming support for large datasets
- **Missing**: Real-time validation during streaming

**Improvement Opportunity**:
- Implement streaming ingesters for RDF parsing
- Real-time pipeline execution with streaming validation
- Streaming receipt validation

### ❌ Patterns Not Yet Implemented

#### 1. **Template Engine** (Weaver Pattern: Jinja2-Based Templates)
**Current State**: ❌ Not Implemented
- No template-based code generation
- No embedded templates

**Improvement Opportunity**:
- Enhance AOT template engine with Jinja2-like features
- Embedded default templates
- Better code generation capabilities

#### 2. **Diagnostic System** (Weaver Pattern: Structured Diagnostics)
**Current State**: ❌ Basic Implementation
- Using `thiserror` but missing rich context
- No structured diagnostic output (JSON/ANSI)
- No integration with OTEL spans for error tracking

**Improvement Opportunity**:
- Adopt `miette`-style diagnostics for better error messages
- Structured error context with OTEL span integration
- JSON output for CI/CD integration

#### 3. **CLI Improvements** (Weaver Pattern: Subcommand Architecture)
**Current State**: ❌ Not Applicable (Sidecar is a service)
- Sidecar is a gRPC service, not a CLI
- **Note**: This pattern applies to `knhk-cli`, not sidecar

## Specific Alignment with Weaver Patterns

### 1. **Modular Architecture** ✅
- **Weaver**: Workspace-based crate organization (`crates/*`)
- **KNHK**: Already following (`rust/knhk-*`)
- **Status**: ✅ Aligned

### 2. **Stage-Based Processing** ✅
- **Weaver**: Validation pipeline with stages
- **KNHK**: ETL pipeline with 5 stages
- **Status**: ✅ Aligned, even better (5 stages vs Weaver's 3)

### 3. **Telemetry Validation** ✅
- **Weaver**: Live-check validation
- **KNHK**: Integrated Weaver live-check
- **Status**: ✅ Fully integrated

### 4. **Error Handling** 🔄
- **Weaver**: Structured diagnostics with context
- **KNHK**: Using `thiserror`, missing rich context
- **Status**: 🔄 Good foundation, can improve

### 5. **Schema Management** 🔄
- **Weaver**: Resolved schemas with catalog
- **KNHK**: Schema IRI-based, missing catalog
- **Status**: 🔄 Partial implementation

## Recommendations

### High Priority (P0)

1. **Add `#[non_exhaustive]` to Error Enums**
   ```rust
   #[derive(Debug, Error, Clone)]
   #[non_exhaustive]
   pub enum SidecarError {
       // ... existing variants
   }
   ```
   **Benefit**: Allows extensibility without breaking changes

2. **Improve Error Context**
   - Add structured error context (like Weaver's `DiagnosticMessage`)
   - Integrate with OTEL spans for error tracking
   - JSON output for CI/CD

### Medium Priority (P1)

3. **Policy Engine Integration**
   - Integrate Rego-based policy engine
   - Apply to guard constraints and performance validation
   - Custom validation rules via policies

4. **Schema Resolution**
   - Implement resolved schema pattern
   - Schema catalog for shared definitions
   - Version management and dependencies

### Low Priority (P2)

5. **Streaming Processing**
   - Streaming ingesters for RDF
   - Real-time pipeline execution
   - Streaming validation

## Conclusion

The KNHK sidecar implementation **strongly aligns** with Weaver's architectural patterns:

✅ **Fully Aligned**:
- Modular architecture
- Stage-based processing
- Telemetry validation
- Ingester pattern

🔄 **Partially Aligned** (with improvement opportunities):
- Error handling (good foundation, can add rich context)
- Schema management (basic implementation, can add catalog)
- Policy engine (validation exists, can add Rego policies)

❌ **Not Applicable/Not Implemented**:
- Template engine (applies to AOT, not sidecar)
- CLI improvements (sidecar is a service, not CLI)

The current implementation demonstrates **excellent architectural alignment** with Weaver's patterns, particularly in the pipeline architecture and telemetry integration. The improvements identified are enhancements rather than fundamental gaps.

## Next Steps

1. **Immediate**: Add `#[non_exhaustive]` to error enums
2. **Short-term**: Improve error diagnostics with structured context
3. **Medium-term**: Integrate Rego policy engine for custom validation
4. **Long-term**: Implement schema catalog and resolution system

