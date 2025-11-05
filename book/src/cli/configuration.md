# CLI Configuration

## Configuration Directory

Configuration is stored in:
- **Unix/Linux/macOS**: `~/.knhk/`
- **Windows**: `%APPDATA%/knhk/`

## Configuration Files

### Schema Registry (`sigma.ttl`)

RDF/Turtle file defining the schema (Σ). Loaded during `boot init`.

### Invariant Registry (`q.sparql`)

SPARQL queries defining invariants (Q). Loaded during `boot init`.

### Connector Registry (`connectors.json`)

Stores registered connectors:

```json
{
  "connectors": [
    {
      "name": "kafka-prod",
      "schema": "urn:knhk:schema:default",
      "source": "kafka://localhost:9092/triples",
      "status": "registered"
    }
  ]
}
```

### Cover Definitions (`covers.json`)

Stores defined covers:

```json
{
  "covers": [
    {
      "id": "cover_1",
      "select": "SELECT ?s ?p ?o WHERE { ?s ?p ?o }",
      "shard": "max_run_len 8"
    }
  ]
}
```

### Reflex Definitions (`reflexes.json`)

Stores declared reflexes:

```json
{
  "reflexes": [
    {
      "id": "reflex_1",
      "name": "check-count",
      "op": "ASK_SP",
      "pred": 12648430,
      "off": 0,
      "len": 8
    }
  ]
}
```

### Epoch Definitions (`epochs.json`)

Stores epoch definitions:

```json
{
  "epochs": [
    {
      "id": "epoch1",
      "tau": 8,
      "lambda": ["reflex1", "reflex2"],
      "status": "scheduled"
    }
  ]
}
```

### Route Definitions (`routes.json`)

Stores installed routes:

```json
{
  "routes": [
    {
      "id": "route_1",
      "name": "webhook1",
      "kind": "webhook",
      "target": "https://api.example.com/webhook"
    }
  ]
}
```

## Environment Variables

Currently, configuration is file-based. Environment variable support may be added in future versions.

## Configuration Validation

All configuration is validated:
- Guard constraints enforced (max_run_len ≤ 8, τ ≤ 8)
- Schema IRI format validated
- Endpoint URLs validated
- Operation names validated (must be in H_hot set)

## See Also

- [Commands Reference](commands.md) - Command usage
- [First Steps](first-steps.md) - Initial setup

