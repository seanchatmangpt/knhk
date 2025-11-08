# ggen Integration

RDF-driven template generation for workflows.

## Overview

ggen integration enables RDF-driven code generation:

- **Workflow Specs**: Generate from RDF ontologies
- **Tests**: Generate Chicago TDD tests
- **Documentation**: Generate Markdown docs
- **SPARQL Support**: Query RDF graphs in templates

## Basic Usage

```rust
use knhk_workflow_engine::ggen::generate_workflow_from_rdf;

generate_workflow_from_rdf(
    "templates/workflow.tmpl",
    "domain/workflow.ttl",
    "src/generated.rs",
)?;
```

## Template Generation

Generate tests:

```rust
use knhk_workflow_engine::ggen::generate_tests_from_spec;

generate_tests_from_spec(
    &spec,
    "templates/test.tmpl",
    "tests/generated.rs",
)?;
```

Generate documentation:

```rust
use knhk_workflow_engine::ggen::generate_documentation_from_spec;

generate_documentation_from_spec(
    &spec,
    "templates/doc.tmpl",
    "docs/generated.md",
)?;
```

## Next Steps

- [ggen Use Cases](use-cases/ggen.md) - Comprehensive examples
- [GGEN Integration Guide](../docs/GGEN_INTEGRATION.md) - Detailed guide

