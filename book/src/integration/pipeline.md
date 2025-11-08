# ETL Pipeline

## Pipeline Stages

1. **Ingest**: Connector polling, RDF/Turtle parsing, JSON-LD support
2. **Transform**: Schema validation (O ⊨ Σ), IRI hashing (FNV-1a), typed triples
3. **Load**: Predicate run grouping, SoA conversion, 64-byte alignment
4. **Reflex**: Hot path execution (≤8 ticks), receipt generation, receipt merging (⊕)
5. **Emit**: Lockchain writing (Merkle-linked), downstream APIs (webhooks, Kafka, gRPC)

## Usage

```rust
use knhk_etl::{Pipeline, PipelineStage};

let mut pipeline = Pipeline::new();
pipeline.add_stage(PipelineStage::Ingest { connectors: vec!["kafka-prod"] });
pipeline.add_stage(PipelineStage::Transform { schema: "urn:knhk:schema:default".to_string() });
pipeline.add_stage(PipelineStage::Load);
pipeline.add_stage(PipelineStage::Reflex { hooks: vec![] });
pipeline.add_stage(PipelineStage::Emit { lockchain_enabled: true });
pipeline.run()?;
```

## Pattern-Based Hook Execution

The Reflex stage supports pattern-based hook orchestration:

```rust
use knhk_etl::hook_orchestration::{HookOrchestrator, HookExecutionContext, HookExecutionPattern};
use knhk_etl::hook_registry::HookRegistry;
use knhk_etl::ReflexStage;

let reflex = ReflexStage::new();
let load_result = pipeline.execute_to_load()?;

// Execute with parallel pattern
let pattern = HookExecutionPattern::Parallel(vec![pred1, pred2]);
let result = reflex.reflex_with_patterns(load_result, pattern)?;
```

**Pattern Types:**
- **Sequence**: Sequential hook execution
- **Parallel**: Parallel hook execution (SIMD-optimized)
- **Choice**: Conditional routing based on context
- **Retry**: Retry logic for transient failures

See [Hook Integration Guide](../../rust/knhk-patterns/HOOK_INTEGRATION.md) for details.

## Guard Constraints

- **max_run_len ≤ 8**: Enforced at Load stage
- **τ ≤ 8**: Enforced at Reflex stage
- **Schema validation**: Enforced at Transform stage

