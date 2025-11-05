# ETL Pipeline Example

This example demonstrates a complete ETL pipeline execution.

## Files

- `pipeline-config.toml` - Pipeline configuration
- `run.sh` - Pipeline execution script
- `README.md` - This file

## Usage

```bash
./run.sh
```

## Pipeline Stages

1. **Ingest** - Connector polling, RDF/Turtle parsing
2. **Transform** - Schema validation, IRI hashing
3. **Load** - Predicate run grouping, SoA conversion
4. **Reflex** - Hot path execution (≤8 ticks), receipt generation
5. **Emit** - Lockchain writing, downstream APIs

## Expected Output

- Pipeline execution status
- Metrics and timing information
- Receipts generated
