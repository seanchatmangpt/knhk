# ETL Library Split Plan

The `rust/knhk-etl/src/lib.rs` file (1578 lines) has been split into modules:

## Completed Modules:
- `types.rs` - Common types (PipelineStage, PipelineMetrics, RawTriple, TypedTriple, SoAArrays, PredRun, Action, Receipt)
- `error.rs` - PipelineError enum
- `ingest/mod.rs` - IngestStage and IngestResult (lines 41-195 from original)

## Remaining Modules to Create:

### `transform/mod.rs` (lines 197-308)
- TransformStage struct and impl
- TransformResult struct
- TypedTriple struct (move to types.rs if not already there)
- hash_iri function
- validate_schema function

### `load/mod.rs` (lines 310-442)
- LoadStage struct and impl
- LoadResult struct
- SoAArrays struct (move to types.rs if not already there)
- PredRun struct (move to types.rs if not already there)

### `reflex/mod.rs` (lines 444-785)
- ReflexStage struct and impl
- ReflexResult struct
- Action struct (move to types.rs if not already there)
- Receipt struct (move to types.rs if not already there)
- execute_hook function
- merge_receipts function
- generate_span_id functions
- compute_a_hash function

### `emit/mod.rs` (lines 787-1221)
- EmitStage struct and impl
- EmitResult struct
- write_receipt_to_lockchain functions
- send_action_to_endpoint functions
- send_http_webhook function
- send_kafka_action function
- send_grpc_action function
- compute_receipt_hash function

## Test Migration:
Move tests from original lib.rs (lines 1283-1577) to appropriate module test files or keep in lib.rs tests module.

## Dependencies:
Each module should:
- Import from `crate::types::*`
- Import from `crate::error::PipelineError`
- Import from previous stage modules (e.g., transform uses ingest::IngestResult)

