# CLI First Steps

## 1. Initialize System

Initialize Σ (schema) and Q (invariants) registries:

```bash
knhk boot init schema.ttl invariants.sparql
```

This creates the configuration directory (`~/.knhk/` on Unix) and stores:
- `sigma.ttl` - Schema registry
- `q.sparql` - Invariant registry

## 2. Register a Connector

Register a Kafka connector:

```bash
knhk connect register kafka-prod \
  urn:knhk:schema:default \
  kafka://localhost:9092/triples
```

## 3. Define a Cover

Define a cover over the knowledge graph:

```bash
knhk cover define \
  "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" \
  "max_run_len 8"
```

**Note**: `max_run_len` must be ≤ 8 (Chatman Constant).

## 4. Declare a Reflex

Declare a hot path reflex:

```bash
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8
```

**Note**: Operation must be in H_hot set, run length ≤ 8.

## 5. Run Pipeline

Execute the ETL pipeline:

```bash
knhk pipeline run --connectors kafka-prod
```

## 6. Check Status

View pipeline status and metrics:

```bash
knhk pipeline status
knhk metrics get
```

## Common Issues

### Guard Violations

If you see "Guard violation" errors:
- Check `max_run_len ≤ 8` for covers and reflexes
- Check `τ ≤ 8` for epochs
- Verify operation is in H_hot set for reflexes

### Configuration Errors

If configuration files are missing:
- Run `knhk boot init` first
- Check `~/.knhk/` directory exists
- Verify file permissions

## Next Steps

- [Commands Reference](commands.md) - Complete command list
- [Configuration](configuration.md) - Configuration guide

