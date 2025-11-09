# knhk-unrdf Documentation

unrdf integration for cold path operations.

## File Structure

```
rust/knhk-unrdf/
├── src/
│   ├── lib.rs              # Module exports and public API
│   ├── ffi.rs              # FFI bindings to Node.js process
│   ├── query.rs            # SPARQL query execution
│   ├── query_native.rs     # Native query implementation
│   ├── store.rs            # RDF store operations
│   ├── hooks.rs            # Hook execution
│   ├── shacl.rs            # SHACL validation
│   ├── transaction.rs      # Transaction management
│   ├── serialize.rs        # Serialization utilities
│   ├── canonicalize.rs     # IRI canonicalization
│   ├── cache.rs            # Caching layer
│   ├── state.rs            # State management
│   ├── script.rs           # JavaScript code generation
│   ├── template.rs         # Tera template management
│   ├── types.rs            # Type definitions
│   ├── error.rs            # Error types
│   ├── errors.rs           # Error handling utilities
│   └── utils.rs            # Utility functions
├── templates/
│   ├── query-only.tera           # Query-only template
│   ├── query-with-data.tera      # Query with data template
│   ├── hook-execute.tera         # Hook execution template
│   ├── hook-execute-with-data.tera # Hook execution with data
│   ├── hook-register.tera        # Hook registration template
│   ├── store.tera                # Store operations template
│   ├── serialize.tera            # Serialization template
│   ├── shacl-validate.tera       # SHACL validation template
│   └── transaction-commit.tera   # Transaction commit template
└── Cargo.toml
```

## Core Components

### FFI Layer (`src/ffi.rs`)
- Manages Node.js process lifecycle
- Executes JavaScript code via Node.js
- Parses JavaScript errors to Rust errors
- Error handling and process management

### Query Module (`src/query.rs`)
- `query_sparql()` - Execute SPARQL SELECT queries
- `query_sparql_ask()` - Execute ASK queries
- `query_sparql_construct()` - Execute CONSTRUCT queries
- `query_sparql_describe()` - Execute DESCRIBE queries

### Store Module (`src/store.rs`)
- RDF data storage operations
- Turtle data loading
- Triple management

### Hooks Module (`src/hooks.rs`)
- Hook execution via unrdf
- Hook registration and management

### SHACL Module (`src/shacl.rs`)
- SHACL validation operations
- Shape constraint validation

### Template System (`src/template.rs`, `templates/`)
- Tera templates for JavaScript code generation
- Template-based code generation for unrdf operations

## Dependencies

- `libloading` - Dynamic library loading
- `tera` - Template engine
- Node.js runtime (external dependency)

## Key Features

- **Complex SPARQL**: Full SPARQL compliance
- **SHACL Validation**: Shape constraint validation
- **OWL Reasoning**: OWL inference capabilities
- **Template-Based**: Code generation via templates
- **Error Handling**: Robust error parsing and handling

## Related Documentation

- [unrdf Integration Status](../../../docs/unrdf-integration-status.md) - Integration status
- [unrdf Integration DoD](../../../docs/unrdf-integration-dod.md) - DoD validation
- [Architecture Guide](../../../docs/ARCHITECTURE.md) - 🆕 Consolidated 80/20 guide (System architecture)
- [Architecture Reference](../../../docs/architecture.md) - Detailed architecture reference

