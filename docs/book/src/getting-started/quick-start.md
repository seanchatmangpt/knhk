# Quick Start Guide

**Version**: 0.4.0  
**Status**: ✅ Production Ready  
**Quick Start**: Essential steps to get KNHK running.

**Formal Foundation**: KNHK's behavior is defined through 17 foundational laws (the Constitution) ensuring deterministic execution, safe parallelism, and cryptographic verification. See [Formal Mathematical Foundations](formal-foundations.md) for complete treatment.

**See [v0.4.0 Status](archived/v0.4.0/v0.4.0-status.md) for complete status and limitations.**

## 5-Minute Setup

### 1. Build

```bash
make lib
cd rust/knhk-cli && cargo build --release
```

### 2. Initialize

```bash
knhk boot init schema.ttl invariants.sparql
```

### 3. Register Connector

```bash
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples
```

### 4. Run Pipeline

```bash
knhk pipeline run --connectors kafka-prod
```

## Common Commands

```bash
# List connectors
knhk connect list

# Define cover
knhk cover define "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" "max_run_len 8"

# Declare reflex
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8

# Run epoch
knhk epoch create epoch1 8 "check-count"
knhk epoch run epoch1

# Check status
knhk pipeline status
knhk metrics get
```

## Next Steps

- [CLI Guide](cli.md) - Complete command reference
- [Architecture](architecture.md) - System architecture
- [Integration Guide](integration.md) - Integration examples
- [Formal Mathematical Foundations](formal-foundations.md) - Deep formal insights and emergent properties
- [Repository Overview](../REPOSITORY_OVERVIEW.md) - Complete system overview

---

**Formal Guarantee**: All operations satisfy **A = μ(O)** - Actions are deterministic projections of observations, verified through formal laws and test validation.
