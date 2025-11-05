# Examples Directory

This directory contains working examples for common KNHK use cases.

## Examples

- **[basic-hook](basic-hook/)** - Basic hook execution example
- **[kafka-connector](kafka-connector/)** - Kafka connector setup example
- **[etl-pipeline](etl-pipeline/)** - Full ETL pipeline example
- **[receipt-verification](receipt-verification/)** - Receipt verification example
- **[cli-usage](cli-usage/)** - CLI usage examples

## Running Examples

Each example directory contains:
- `README.md` - Example documentation
- Example files (`.ttl`, `.toml`, `.sh`, etc.)
- `run.sh` - Execution script

To run an example:
```bash
cd examples/<example-name>
./run.sh
```

## Prerequisites

- KNHK CLI installed (`cargo install --path rust/knhk-cli`)
- Configuration directory initialized (`~/.knhk/`)
- Required dependencies (Kafka, HTTP endpoints, etc.) as specified in each example

