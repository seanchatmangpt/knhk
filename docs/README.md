# KNHK Documentation

Welcome to the KNHK documentation directory. This directory contains all documentation for the KNHK knowledge graph query system.

## Quick Navigation

- **[INDEX.md](INDEX.md)** - Comprehensive documentation index
- **[QUICK_START.md](QUICK_START.md)** - Get started in 5 minutes
- **[Architecture](architecture.md)** - System architecture overview
- **[API Reference](api.md)** - Complete API documentation
- **[Testing](testing.md)** - Testing documentation and coverage

## Documentation Structure

### Core Documentation
- **Architecture** - System design and components
- **API** - Programming interfaces (C, Rust, Erlang)
- **CLI** - Command-line interface
- **Integration** - Integration guides and examples
- **Performance** - Performance characteristics and optimization
- **Testing** - Test coverage and methodology

### Integration Guides
- **ggen Integration** - oxigraph integration guide
- **unrdf Integration** - unrdf integration status and guides
- **Weaver Integration** - Weaver.ai integration

### Planning & Requirements
- **v1.0 Requirements** - Forward-looking requirements
- **v1.0 Integration Plans** - Integration planning documents

### Archived Documentation
Historical and version-specific documentation is archived in `archived/`:
- `archived/v0.4.0/` - Version 0.4.0 specific docs
- `archived/status/` - Implementation status reports
- `archived/planning/` - Project planning documents
- `archived/versions/` - Historical version docs
- `archived/analysis/` - Analysis documents
- `archived/implementation-details/` - Implementation details

## Documentation Standards

All documentation follows these principles:
- **Production-ready**: No placeholders, real implementations
- **Chicago TDD**: Test-driven development methodology
- **80/20 Focus**: Critical path implementations
- **Performance**: Hot path ≤2ns, warm path ≤500ms

## Contributing

When adding new documentation:
1. Update `INDEX.md` with new document entry
2. Follow existing documentation structure
3. Include examples and code snippets
4. Link to related documentation
5. Archive outdated versions appropriately

## Subproject Documentation

Each subproject has its own `docs/` directory:
- `rust/knhk-*/docs/` - Rust crate documentation
- `c/docs/` - C hot path documentation
- `erlang/docs/` - Erlang cold path documentation
- `playground/*/docs/` - Playground project documentation

See [INDEX.md](INDEX.md) for links to all subproject documentation.

