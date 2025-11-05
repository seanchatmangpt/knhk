# KNHK Integration Guide

**Version**: 0.4.0  
**Integration Guide**: Essential patterns for integrating KNHK into your system

## Overview

This guide explains how to integrate KNHK into your system. The integration focuses on the critical path features that deliver 80% of enterprise value.

## End-to-End Integration

Full pipeline: Connector → ETL → Hot Path → Lockchain → OTEL

### Integration Flow

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Connector  │───>│  ETL Pipeline│───>│  Hot Path   │───>│  Lockchain  │
│  (Kafka/etc)│    │  (5 stages) │    │  (≤8 ticks) │    │  (Receipts) │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                                                              │
                                                              ▼
                                                        ┌─────────────┐
                                                        │    OTEL     │
                                                        │ (Observable)│
                                                        └─────────────┘
```

## Connector Integration

### Supported Connectors (v0.4.0)

**Production Ready**:
- ✅ **Kafka** - Real rdkafka integration with delivery confirmation
- ✅ **HTTP/Webhook** - Real reqwest integration with retry logic
- ✅ **gRPC** - HTTP gateway fallback

**Connector Framework**:
- Circuit breaker pattern for resilience
- Health checking and metrics
- Guard validation (max_run_len ≤ 8, max_batch_size, max_lag_ms)

### Registering a Connector

```bash
# Register Kafka connector
knhk connect register kafka-prod \
  urn:knhk:schema:default \
  kafka://localhost:9092/triples

# Register HTTP connector
knhk connect register http-api \
  urn:knhk:schema:enterprise \
  https://api.example.com/triples

# List connectors
knhk connect list
```

### Using Connectors in Pipeline

```bash
# Run pipeline with specific connectors
knhk pipeline run --connectors kafka-prod,http-api

# Check pipeline status
knhk pipeline status
```

## Hook Integration

### Supported Operations (H_hot Set)

**Read Operations** (18 operations, all ≤8 ticks):
- ASK_SP, ASK_SPO, ASK_OP - Existence checks
- COUNT_SP_GE/LE/EQ, COUNT_OP variants - Cardinality validation
- COMPARE_O_EQ/GT/LT/GE/LE - Value comparisons
- VALIDATE_DATATYPE_SP/SPO - Property validation
- UNIQUE_SP - Uniqueness check
- SELECT_SP - Limited to 4 results

**Write Operations**:
- ⚠️ CONSTRUCT8 - Exceeds 8-tick budget (41-83 ticks), move to warm path in v0.5.0

### Declaring Reflexes

```bash
# Declare ASK reflex
knhk reflex declare check-auth ASK_SP 0xC0FFEE 0 8

# Declare COUNT reflex
knhk reflex declare check-count COUNT_SP_GE 0xC0FFEE 1 8

# List reflexes
knhk reflex list
```

### Creating Epochs

```bash
# Create epoch with reflexes
knhk epoch create epoch1 8 "check-auth,check-count"

# Run epoch
knhk epoch run epoch1

# List epochs
knhk epoch list
```

## ETL Pipeline Integration

### Pipeline Stages

1. **Ingest** - Connector polling, RDF/Turtle parsing, JSON-LD support
2. **Transform** - Schema validation (O ⊨ Σ), IRI hashing (FNV-1a), typed triples
3. **Load** - Predicate run grouping, SoA conversion, 64-byte alignment verification
4. **Reflex** - Hot path execution (≤8 ticks), receipt generation, receipt merging (⊕)
5. **Emit** - Lockchain writing (Merkle-linked), downstream APIs (webhooks, Kafka, gRPC)

### Running Pipeline

```bash
# Run full pipeline
knhk pipeline run --connectors kafka-prod

# Run with specific schema
knhk pipeline run --connectors kafka-prod --schema urn:knhk:schema:enterprise

# Check status
knhk pipeline status
```

## Lockchain Integration

### Receipt Operations

```bash
# Get receipt
knhk receipt get <receipt-id>

# Merge receipts
knhk receipt merge <id1>,<id2>,<id3>

# List receipts
knhk receipt list

# Verify receipt
knhk receipt verify <receipt-id>
```

### Receipt Properties

- **URDNA2015 + SHA-256** - Cryptographic integrity
- **Merkle-linked** - Immutable audit trail
- **Git-based storage** - Version-controlled receipts
- **OTEL span IDs** - Observability correlation

## OTEL Integration

### Metrics

```bash
# Get metrics
knhk metrics get
```

### Span Generation

All operations generate OTEL-compatible span IDs:
- Real span ID generation (no placeholders)
- OTEL-compatible format
- Provenance tracking (hash(A) = hash(μ(O)))

## Guard Constraints

All integrations enforce guard constraints:

- **max_run_len ≤ 8** - Predicate run size limit
- **τ ≤ 8 ticks** - Execution time limit
- **max_batch_size** - Batch size validation
- **Schema validation** - IRI format checking
- **Operation validation** - H_hot set membership

## Integration Examples

### Complete Workflow

```bash
# 1. Initialize system
knhk boot init schema.ttl invariants.sparql

# 2. Register connector
knhk connect register kafka-prod \
  urn:knhk:schema:default \
  kafka://localhost:9092/triples

# 3. Define cover
knhk cover define \
  "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" \
  "max_run_len 8"

# 4. Declare reflex
knhk reflex declare check-auth ASK_SP 0xC0FFEE 0 8

# 5. Create epoch
knhk epoch create epoch1 8 "check-auth"

# 6. Run pipeline
knhk pipeline run --connectors kafka-prod

# 7. Check metrics
knhk metrics get
knhk coverage get
```

## Best Practices

1. **Guard Validation** - Always enforce guard constraints (max_run_len ≤ 8)
2. **Error Handling** - All operations return `Result<(), String>` for proper error handling
3. **Receipt Verification** - Verify receipts for provenance tracking
4. **OTEL Integration** - Use metrics and spans for observability
5. **80/20 Focus** - Use hot path operations for critical path queries

## Known Limitations (v0.4.0)

- ⚠️ **CONSTRUCT8**: Exceeds 8-tick budget (41-83 ticks) - Move to warm path in v0.5.0
- ⚠️ **Configuration Management**: TOML config incomplete - Deferred to v0.5.0
- ⚠️ **CLI Documentation**: Comprehensive docs pending - Deferred to v0.5.0
- ⚠️ **Examples Directory**: Missing examples - Deferred to v0.5.0

See [v0.4.0 Status](archived/v0.4.0/v0.4.0-status.md) for complete details.

## See Also

- [Architecture](architecture.md) - System architecture
- [API Reference](api.md) - API documentation
- [CLI Guide](cli.md) - Command-line interface
- [Deployment Guide](deployment.md) - Deployment instructions
