# Rust API Reference

## Crates

### knhk-hot
Rust FFI wrapper for C hot path:
```rust
use knhk_hot::{Engine, Op, Ir, Receipt, Run};

let engine = Engine::new(s.as_ptr(), p.as_ptr(), o.as_ptr());
let run = Run { pred: 0xC0FFEE, off: 0, len: 8 };
engine.pin_run(run)?;
let mut ir = Ir { op: Op::AskSp, s: 0, p: 0xC0FFEE, o: 0, k: 0, .. };
let mut receipt = Receipt::default();
engine.eval_bool(&mut ir, &mut receipt)?;
```

### knhk-etl
ETL Pipeline implementation:
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

### knhk-connectors
Connector framework:
```rust
use knhk_connectors::{Connector, KafkaConnector, DataFormat};

let connector = KafkaConnector::new(
    "kafka-prod".to_string(),
    "kafka://localhost:9092/triples".to_string(),
    DataFormat::JsonLd,
)?;
let delta = connector.fetch_delta()?;
```

### knhk-lockchain
Merkle-linked provenance storage:
```rust
use knhk_lockchain::{Lockchain, LockchainEntry};

let mut lockchain = Lockchain::new();
lockchain.with_git_repo("./receipts".to_string());
let entry = LockchainEntry { /* ... */ };
lockchain.append(&entry)?;
```

### knhk-otel
OTEL integration:
```rust
use knhk_otel::{Tracer, Span, OtlpExporter};

let tracer = Tracer::new();
let mut span = tracer.start_span("operation");
// ... operation ...
span.end();
let exporter = OtlpExporter::new("http://localhost:4318".to_string());
exporter.export_spans(&vec![span])?;
```

## See Also

- [C API](c-api.md) - C API reference
- [Erlang API](erlang-api.md) - Erlang API reference

