# Code Files Index

**Last Updated**: January 2025  
**Total Files**: 407 code files extracted from YAWL documentation

## File Type Breakdown

| Type | Count | Extension | Description |
|------|-------|-----------|-------------|
| Rust | 168 | `.rs` | Pattern executors, engine components, utilities |
| Markdown | 93 | `.md` | Documentation snippets and examples |
| YAML | 31 | `.yaml`, `.yml` | Configuration files, Weaver configs |
| C | 30 | `.c` | Hot path implementations, kernels |
| JSON | 21 | `.json` | API examples, test data |
| Shell | 19 | `.sh` | Build scripts, utilities |
| TOML | 11 | `.toml` | Cargo configs, project configs |
| XML | 8 | `.xml` | YAWL workflow definitions |
| Text | 7 | `.txt` | Plain text examples |
| Turtle/RDF | 7 | `.ttl` | RDF workflow specifications |
| C Headers | 6 | `.h` | C header files |

## Key Files by Category

### Pattern Executors (Rust)

Core pattern implementation files extracted from the YAWL workflow engine scaffold:

- `src_lib.rs` - Main engine library with pattern registry integration
- `src_ids.rs` - Pattern ID definitions (1-43)
- `src_types.rs` - Pattern execution context and result types
- `src_registry.rs` - Pattern registry implementation
- `src_exec/mod.rs` - Pattern executor implementations (P01-P43)
- `rust_001.rs` through `rust_173.rs` - Various Rust code examples

**Usage**: Reference these for pattern implementation patterns and API usage.

### Hot Path (C)

C implementations for performance-critical hot path operations:

- `beat.c` - Beat scheduler implementation
- `kernels.c` - Hot path kernels
- `ring_buffer.c` - Ring buffer implementation
- `warm_path.c` - Warm path implementation
- `workflow_patterns.c` - Workflow pattern C implementations
- `c_001.c` through `c_035.c` - Various C code examples
- `aot_guard.h` - AOT guard header
- `knhk.h` - Main KNHK C API header

**Usage**: Reference for hot path optimization patterns and C API usage.

### Configuration Files

#### TOML (Cargo/Project Configs)
- `Cargo.toml` - Workspace Cargo configuration
- `toolchain.toml` - Rust toolchain specification
- `book.toml` - mdBook configuration
- `clippy.toml` - Clippy lint configuration
- `cliff.toml` - Changelog generation config
- `Makefile.toml` - Makefile configuration

#### YAML (Weaver/Service Configs)
- `weaver.yaml` - Weaver integration configuration
- `weaver_checks.yaml` - Weaver validation checks
- `weaver_registry.yaml` - Weaver registry config
- `app.yaml` - Application configuration
- `client.yaml` - Client configuration
- `server.yaml` - Server configuration
- `database.yaml` - Database configuration
- `metrics.yaml` - Metrics configuration
- `ci.yml` - CI/CD configuration

**Usage**: Reference for project setup and configuration patterns.

### State Management

- `src_store.rs` - Sled-based state store implementation
- `src_types.rs` - State types (Case, WorkflowSpec, etc.)
- `state.rs` - State management utilities

**Usage**: Reference for state persistence patterns.

### Integration Components

- `src_reflex.rs` - Reflex bridge implementation
- `src_error.rs` - Error handling types
- `crates_timebase_src_lib.rs` - Timebase integration
- `rdf_src_lib.rs` - RDF/Oxigraph integration

**Usage**: Reference for integration patterns with external systems.

### Workflow Definitions

#### Turtle/RDF
- `workflow.ttl` - Workflow specification example
- `sequence.ttl` - Sequence pattern example
- `osys.ttl` - Ontology system definitions
- `turtle_006.ttl` through `turtle_009.ttl` - Various RDF examples

#### XML
- `xml_001.xml` through `xml_008.xml` - YAWL XML workflow definitions

**Usage**: Reference for workflow specification formats.

### API Examples

- `json_001.json` through `json_021.json` - JSON API request/response examples
- `basic_usage.rs` - Basic API usage examples
- `cli.rs` - CLI implementation examples

**Usage**: Reference for API integration patterns.

### Build & Development

- `build.rs` - Build script examples
- `build.sh` - Build shell scripts
- `run.sh` - Run scripts
- `weaver.sh` - Weaver validation scripts
- `validate_reflex_capabilities.sh` - Validation scripts

**Usage**: Reference for build and development workflows.

## Search Tips

### Finding Pattern Implementations
```bash
# Find pattern executor files
grep -r "PatternId\|PatternExecutor" code/*.rs

# Find specific pattern (e.g., P16 Deferred Choice)
grep -r "P16\|DeferredChoice" code/
```

### Finding Hot Path Code
```bash
# Find C hot path implementations
ls code/*.c code/*.h

# Find SIMD optimizations
grep -r "SIMD\|simd" code/*.c code/*.rs
```

### Finding Configuration Examples
```bash
# Find all TOML configs
ls code/*.toml

# Find Weaver configs
ls code/*weaver*.yaml
```

### Finding Workflow Definitions
```bash
# Find RDF/Turtle workflows
ls code/*.ttl

# Find XML workflows
ls code/*.xml
```

## File Naming Conventions

- `rust_*.rs` - Sequential Rust code examples from documentation
- `c_*.c` - Sequential C code examples
- `src_*.rs` - Source files from workspace scaffold
- `*_*.yaml` - Categorized YAML configs (weaver, app, client, etc.)
- `json_*.json` - Sequential JSON examples
- `xml_*.xml` - Sequential XML examples
- `turtle_*.ttl` - Sequential Turtle/RDF examples

## Common Patterns

### Pattern Registry Usage
See `src_registry.rs` for pattern registration and dispatch patterns.

### State Management
See `src_store.rs` for Sled-based persistence patterns.

### Error Handling
See `src_error.rs` for error type definitions and handling patterns.

### Integration Examples
See `src_reflex.rs` for reflex bridge patterns and `crates_timebase_src_lib.rs` for timebase integration.

## Notes

- Files are extracted from documentation examples and may be incomplete or illustrative
- Some files represent scaffold/template code rather than complete implementations
- File numbers (e.g., `rust_001.rs`) correspond to extraction order, not importance
- Source files (e.g., `src_*.rs`) represent complete workspace scaffold structure

## Related Documentation

- [README.md](README.md) - Main documentation
- [DIAGRAMS_README.md](DIAGRAMS_README.md) - Architecture diagrams
- [ARCHITECTURE_IMPROVEMENTS.md](ARCHITECTURE_IMPROVEMENTS.md) - Architecture documentation

