# CLI Usage Examples

This example demonstrates common CLI usage patterns and workflows.

## Overview

This directory contains examples for:
- Basic command usage
- Configuration management
- Workflow automation
- Error handling

## Examples

### Basic Commands

```bash
# Initialize system
knhk boot init schema.ttl invariants.sparql

# Register connector
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples

# Define cover
knhk cover define "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" "max_run_len 8"

# Declare reflex
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8

# Create epoch
knhk epoch create epoch1 8 "check-count"

# Run pipeline
knhk pipeline run --connectors kafka-prod
```

### Configuration Management

```bash
# Show current context
knhk context current

# Create new context
knhk context create prod1 Production urn:knhk:schema:enterprise

# Switch context
knhk context use prod1

# List contexts
knhk context list
```

### Hook Management

```bash
# Create hook
knhk hook create auth-check ASK_SP 0xC0FFEE 0 8

# List hooks
knhk hook list

# Evaluate hook
knhk hook eval auth-check

# Show hook details
knhk hook show auth-check
```

### Receipt Operations

```bash
# Get receipt
knhk receipt get receipt_1234567890abcdef

# Verify receipt
knhk receipt verify receipt_1234567890abcdef

# Merge receipts
knhk receipt merge receipt_1,receipt_2,receipt_3

# List receipts
knhk receipt list
```

### Route Management

```bash
# Install webhook route
knhk route install webhook1 webhook https://api.example.com/webhook

# Install Kafka route
knhk route install kafka1 kafka kafka://localhost:9092/actions

# List routes
knhk route list
```

## Running Examples

```bash
# Make script executable
chmod +x run.sh

# Run example
./run.sh
```

## Files

- `basic-commands.sh` - Basic command examples
- `config-management.sh` - Configuration examples
- `hook-management.sh` - Hook examples
- `receipt-operations.sh` - Receipt examples
- `route-management.sh` - Route examples
- `README.md` - This file

