# ETL Pipeline Example

This example demonstrates a complete ETL pipeline execution.

## Overview

Shows full pipeline: Ingest → Transform → Load → Reflex → Emit

## Files

- `pipeline-config.toml` - Pipeline configuration
- `run.sh` - Execution script
- `README.md` - This file

## Usage

```bash
# Run full pipeline
./run.sh
```

## Pipeline Stages

1. **Ingest** - Read data from connectors
2. **Transform** - Validate and hash IRIs
3. **Load** - Convert to SoA format
4. **Reflex** - Execute hooks (hot/warm path)
5. **Emit** - Write receipts and actions

