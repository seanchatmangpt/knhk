# KNHK Workflow Engine

**Enterprise workflow engine with full 43-pattern YAWL support**

The KNHK Workflow Engine is a production-ready, enterprise-grade workflow execution engine that supports all 43 Van der Aalst workflow patterns and provides full YAWL (Yet Another Workflow Language) compatibility.

## Features

- ✅ **Full Pattern Support**: All 43 Van der Aalst workflow patterns
- ✅ **YAWL Compatibility**: Parse and execute YAWL workflow definitions
- ✅ **Enterprise APIs**: REST and gRPC interfaces
- ✅ **State Persistence**: Sled-based state store
- ✅ **Observability**: OTEL integration for tracing
- ✅ **Provenance**: Lockchain integration for audit trails
- ✅ **Fortune 5 Ready**: Enterprise-grade features for Fortune 5 deployments
- ✅ **Chicago TDD**: Comprehensive testing framework
- ✅ **ggen Integration**: RDF-driven template generation

## Quick Start

```rust
use knhk_workflow_engine::{WorkflowEngine, WorkflowParser, StateStore};

// Create state store
let state_store = StateStore::new("./workflow_db")?;

// Create engine
let engine = WorkflowEngine::new(state_store);

// Parse workflow from Turtle
let mut parser = WorkflowParser::new()?;
let spec = parser.parse_file("workflow.ttl")?;

// Register workflow
let spec_id = engine.register_workflow(spec).await?;

// Create and execute case
let case_id = engine.create_case(spec_id, serde_json::json!({})).await?;
engine.start_case(case_id).await?;
engine.execute_case(case_id).await?;
```

## Documentation

**📖 [80/20 Consolidated Guide](../../docs/WORKFLOW_ENGINE.md)** - Single source of truth for workflow engine documentation

This guide covers:
- Quick start (80% use cases)
- Core API reference
- Critical patterns (5 most important)
- Production readiness status
- Troubleshooting

### Additional Resources

- **[Getting Started](getting-started/introduction.md)** - Learn the basics
- **[Core Features](core/patterns.md)** - Understand workflow patterns and execution
- **[Advanced Features](advanced/fortune5.md)** - Enterprise features and integrations
- **[API Reference](api/rest.md)** - Complete API documentation
- **[Use Cases](use-cases/swift-fibo.md)** - Real-world examples
- **[Guides](guides/workflow-design.md)** - Best practices and tutorials

## Testing

The workflow engine uses Chicago TDD methodology with comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run Chicago TDD integration tests
cargo test --test chicago_tdd_tools_integration

# Run with output
cargo test --test chicago_tdd_tools_integration -- --nocapture
```

See [Chicago TDD Documentation](docs/CHICAGO_TDD_WORKFLOW_ENGINE_TESTS.md) for details.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
knhk-workflow-engine = "1.0.0"
```

## License

MIT License - see [LICENSE](appendix/license.md) for details.

## Links

- [GitHub](https://github.com/yourusername/knhk)
- [docs.rs](https://docs.rs/knhk-workflow-engine)
- [crates.io](https://crates.io/crates/knhk-workflow-engine)
