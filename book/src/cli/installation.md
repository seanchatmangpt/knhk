# CLI Installation

## Prerequisites

- Rust toolchain (stable or nightly)
- C compiler (clang or gcc)
- Make

## Installation Steps

### 1. Build C Library

```bash
make lib
```

This builds the C hot path library (`libknhk.a`).

### 2. Build CLI Tool

```bash
cd rust/knhk-cli
cargo build --release
```

### 3. Install CLI (Optional)

```bash
cargo install --path .
```

This installs `knhk` to your cargo bin directory.

## Verify Installation

```bash
knhk --help
```

You should see the CLI help output with all available nouns and verbs.

## Next Steps

- [First Steps](first-steps.md) - Initialize and configure KNHK
- [Commands Reference](commands.md) - Complete command list

