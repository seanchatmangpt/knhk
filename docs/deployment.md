# KNHK Deployment Guide

**Version**: 0.4.0  
**Deployment Guide**: Essential steps for production deployment

## Overview

This guide covers production deployment of KNHK v0.4.0. The deployment focuses on the critical path features that deliver 80% of enterprise value.

## Prerequisites

### System Requirements

- **OS**: Linux (primary), macOS (development)
- **C Compiler**: GCC or Clang with SIMD support (ARM NEON / x86 AVX2)
- **Rust Toolchain**: Latest stable (for CLI and warm path)
- **Build Tools**: Make or CMake

### Dependencies

**C Library**:
- Standard C library (no external dependencies)
- SIMD support (ARM NEON / x86 AVX2)

**Rust Crates**:
- `clap-noun-verb` - CLI framework
- `rdkafka` - Kafka integration (optional, feature-gated)
- `reqwest` - HTTP client (optional, feature-gated)
- `sha2` - SHA-256 hashing
- `serde_json` - JSON serialization

## Build

### Building C Library

```bash
# Build static library
make lib

# Build with tests
make test
```

### Building CLI Tool

```bash
# Build CLI
cd rust/knhk-cli
cargo build --release

# Install CLI
cargo install --path .
```

### Building All Components

```bash
# Build everything
make all

# Run tests
make test
```

## Configuration

### Configuration Directory

Configuration is stored in:
- **Unix**: `~/.knhk/`
- **Windows**: `%APPDATA%/knhk/`

### Configuration Files

Current configuration files (JSON-based):
- `sigma.ttl` - Schema registry
- `q.sparql` - Invariant registry
- `connectors.json` - Connector registry
- `covers.json` - Cover definitions
- `reflexes.json` - Reflex definitions
- `epochs.json` - Epoch definitions
- `routes.json` - Route definitions

**Note**: TOML configuration management is incomplete in v0.4.0 and deferred to v0.5.0. Configuration must be done via CLI commands.

## Initialization

### System Bootstrap

```bash
# Initialize Σ and Q registries
knhk boot init schema.ttl invariants.sparql
```

This creates the configuration directory and initializes schema and invariant registries.

## Deployment Scenarios

### Single Node Deployment

**Use Case**: Development, testing, small-scale production

```bash
# 1. Initialize system
knhk boot init schema.ttl invariants.sparql

# 2. Register connectors
knhk connect register kafka-prod \
  urn:knhk:schema:default \
  kafka://localhost:9092/triples

# 3. Start pipeline
knhk pipeline run --connectors kafka-prod
```

### Docker Deployment

**Dockerfile** (example):
```dockerfile
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cd rust/knhk-cli && cargo build --release

FROM ubuntu:latest
RUN apt-get update && apt-get install -y libssl-dev
COPY --from=builder /app/rust/knhk-cli/target/release/knhk /usr/local/bin/
COPY --from=builder /app/libknhk.a /usr/local/lib/
CMD ["knhk", "pipeline", "run"]
```

**Build and Run**:
```bash
# Build image
docker build -t knhk:v0.4.0 .

# Run container
docker run -v ~/.knhk:/root/.knhk knhk:v0.4.0
```

### Kubernetes Deployment

**Deployment YAML** (example):
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: knhk
spec:
  replicas: 1
  selector:
    matchLabels:
      app: knhk
  template:
    metadata:
      labels:
        app: knhk
    spec:
      containers:
      - name: knhk
        image: knhk:v0.4.0
        volumeMounts:
        - name: config
          mountPath: /root/.knhk
      volumes:
      - name: config
        configMap:
          name: knhk-config
```

**Note**: Kubernetes deployment details are basic in v0.4.0. Full deployment guide deferred to v0.5.0.

## Monitoring

### Metrics

```bash
# Get OTEL metrics
knhk metrics get

# Get coverage metrics
knhk coverage get
```

### Observability

All operations generate OTEL-compatible span IDs:
- Real span ID generation (no placeholders)
- OTEL-compatible format
- Provenance tracking (hash(A) = hash(μ(O)))

## Guard Constraints

Deployment must enforce guard constraints:

- **max_run_len ≤ 8** - Predicate run size limit
- **τ ≤ 8 ticks** - Execution time limit
- **max_batch_size** - Batch size validation
- **Schema validation** - IRI format checking
- **Operation validation** - H_hot set membership

## Performance Considerations

### Hot Path Performance

- **Target**: ≤2ns (Chatman Constant) for hot path operations
- **Measurement**: External timing by Rust framework (C hot path contains zero timing code)
- **Status**: 18/19 operations meet ≤8 tick constraint

**Known Limitation**: CONSTRUCT8 exceeds 8-tick budget (41-83 ticks). Move to warm path in v0.5.0.

### Resource Requirements

- **Memory**: Minimal (hot path data fits in L1 cache)
- **CPU**: SIMD support required (ARM NEON / x86 AVX2)
- **Storage**: Config directory, receipt storage (Git lockchain)

## Troubleshooting

### Common Issues

**Issue**: Connector registration fails
- **Solution**: Verify connector source format (kafka://, http://, etc.)

**Issue**: Guard violation errors
- **Solution**: Ensure max_run_len ≤ 8, check predicate run sizes

**Issue**: Pipeline not running
- **Solution**: Check connector status, verify schema validation

**Issue**: Performance degradation
- **Solution**: Verify SIMD support, check cache alignment, review guard constraints

## Security Considerations

### Guard Enforcement

- All operations enforce guard constraints at runtime
- Schema validation prevents invalid data ingestion
- Receipt verification ensures provenance integrity

### Receipt Security

- URDNA2015 + SHA-256 hashing
- Merkle-linked receipts in Git lockchain
- Tamper detection via receipt verification

## Backup and Recovery

### Configuration Backup

```bash
# Backup configuration directory
tar -czf knhk-config-backup.tar.gz ~/.knhk/

# Restore configuration
tar -xzf knhk-config-backup.tar.gz -C ~/
```

### Receipt Backup

Receipts are stored in Git lockchain. Backup Git repository:
```bash
# Backup lockchain
cd ~/.knhk/lockchain
git bundle create ../lockchain-backup.bundle --all
```

## Known Limitations (v0.4.0)

- ⚠️ **CONSTRUCT8**: Exceeds 8-tick budget (41-83 ticks) - Move to warm path in v0.5.0
- ⚠️ **Configuration Management**: TOML config incomplete - Deferred to v0.5.0
- ⚠️ **CLI Documentation**: Comprehensive docs pending - Deferred to v0.5.0
- ⚠️ **Examples Directory**: Missing examples - Deferred to v0.5.0

See [v0.4.0 Status](archived/v0.4.0/v0.4.0-status.md) for complete details.

## See Also

- [Architecture](architecture.md) - System architecture
- [Integration Guide](integration.md) - Integration examples
- [CLI Guide](cli.md) - Command-line interface
- [API Reference](api.md) - API documentation
