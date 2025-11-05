# Code Organization

## Project Structure

```
vendors/knhk/
├── src/              # C hot path implementation
├── include/          # C headers
├── rust/             # Rust warm path crates
│   ├── knhk-cli/    # CLI tool
│   ├── knhk-etl/    # ETL pipeline
│   ├── knhk-connectors/  # Connector framework
│   ├── knhk-lockchain/   # Provenance lockchain
│   └── knhk-otel/   # OTEL integration
├── erlang/           # Erlang cold path
├── tests/            # Test suite
├── docs/             # Documentation
└── Makefile          # Build system
```

## Header Structure

```
include/
├── knhk.h              # Main umbrella header
└── knhk/
    ├── types.h          # Type definitions
    ├── eval.h           # Query evaluation
    ├── receipts.h       # Receipt operations
    └── utils.h          # Utility functions
```

## Source Structure

```
src/
├── simd.h               # SIMD umbrella header
├── simd/
│   ├── existence.h      # ASK operations
│   ├── count.h           # COUNT operations
│   ├── compare.h         # Comparison operations
│   ├── select.h          # SELECT operations
│   ├── validate.h        # Datatype validation
│   └── construct.h      # CONSTRUCT8 operations
├── core.c               # Core operations
├── rdf.c                # RDF parsing
└── clock.c              # Timing utilities
```

