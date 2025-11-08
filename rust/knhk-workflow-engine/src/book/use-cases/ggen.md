# ggen Use Cases

Comprehensive use cases for ggen integration.

## Use Cases

### 1. Generate Workflow Specs from RDF

Generate Rust workflow specifications from RDF ontologies:

```rust
generate_workflow_from_rdf(
    "templates/workflow.tmpl",
    "domain/workflow.ttl",
    "src/generated.rs",
)?;
```

### 2. Generate Tests

Generate Chicago TDD test suites:

```rust
generate_tests_from_spec(
    &spec,
    "templates/test.tmpl",
    "tests/generated.rs",
)?;
```

### 3. Generate Documentation

Generate Markdown documentation:

```rust
generate_documentation_from_spec(
    &spec,
    "templates/doc.tmpl",
    "docs/generated.md",
)?;
```

## Complete Examples

See the full use cases documentation for:

- API client generation
- Configuration file generation
- Database migration generation
- OpenAPI spec generation
- And more!

## Next Steps

- [Full Use Cases](../docs/GGEN_USE_CASES.md) - Complete documentation
- [ggen Integration](../advanced/ggen.md) - Integration guide

