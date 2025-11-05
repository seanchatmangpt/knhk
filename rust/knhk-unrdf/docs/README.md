# knhk-unrdf Documentation

unrdf integration for cold path operations.

## Overview

The `knhk-unrdf` crate provides Rust FFI bindings to unrdf (Node.js) knowledge engine:
- SPARQL query execution
- SHACL validation
- OWL reasoning
- Schema/invariant management

## Architecture

- **Rust FFI Layer**: Safe wrappers around Node.js process
- **Error Handling**: Parses JavaScript errors to Rust errors
- **Template System**: Tera templates for unrdf code generation

## Key Features

- **Complex SPARQL**: Full SPARQL compliance
- **SHACL Validation**: Shape constraint validation
- **OWL Reasoning**: OWL inference capabilities
- **Schema Management**: Schema registry integration

## Related Documentation

- [unrdf Integration Status](../../../docs/unrdf-integration-status.md) - Integration status
- [unrdf Integration DoD](../../../docs/unrdf-integration-dod.md) - DoD validation
- [Architecture](../../../docs/architecture.md) - System architecture

