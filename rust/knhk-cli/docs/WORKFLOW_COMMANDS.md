# Workflow Engine CLI Commands

## Quick Start

```bash
# Parse and register workflow
knhk workflow parse workflow.ttl
knhk workflow register workflow.ttl

# Create and execute case
knhk workflow create <spec-id> --data '{"input":"test"}'
knhk workflow start <case-id>
knhk workflow execute <case-id>

# List patterns
knhk workflow patterns
```

## Commands

### `parse` - Parse workflow from Turtle file
```bash
knhk workflow parse <file> [--output <file>]
```
Parse workflow and optionally save as JSON.

### `register` - Register workflow specification
```bash
knhk workflow register <file> [--state-store <path>]
```
Register workflow in state store (validates for deadlocks).

### `create` - Create workflow case
```bash
knhk workflow create <spec-id> [--data <json>] [--state-store <path>]
```
Create new case with optional input data.

### `start` - Start case execution
```bash
knhk workflow start <case-id> [--state-store <path>]
```

### `execute` - Execute workflow case
```bash
knhk workflow execute <case-id> [--state-store <path>]
```

### `get` - Get case status
```bash
knhk workflow get <case-id> [--state-store <path>]
```

### `cancel` - Cancel case
```bash
knhk workflow cancel <case-id> [--state-store <path>]
```

### `list` - List cases
```bash
knhk workflow list [<spec-id>] [--state-store <path>]
```

### `patterns` - List all 43 patterns
```bash
knhk workflow patterns
```

### `serve` - Start REST API server
```bash
knhk workflow serve [--port <port>] [--host <host>] [--state-store <path>]
```

## Complete Example

```bash
# 1. Parse workflow
knhk workflow parse examples/simple-sequence.ttl

# 2. Register workflow
knhk workflow register examples/simple-sequence.ttl
# Output: Workflow registered: Simple Sequence Workflow (<spec-id>)

# 3. Create case
knhk workflow create <spec-id> --data '{"input":"test"}'
# Output: Case created: <case-id>

# 4. Start and execute
knhk workflow start <case-id>
knhk workflow execute <case-id>

# 5. Check status
knhk workflow get <case-id>
```

## REST API

Start server: `knhk workflow serve --port 8080`

Then use HTTP:
```bash
curl -X POST http://localhost:8080/workflows -H "Content-Type: application/json" -d @workflow.json
curl -X POST http://localhost:8080/cases -H "Content-Type: application/json" -d '{"spec_id":"<id>","data":{}}'
curl http://localhost:8080/cases/<case-id>
```

## State Store

Default: `./workflow_db`. Override with `--state-store <path>`.

## Integration

- `knhk connect` - Register connectors for workflow tasks
- `knhk pipeline` - Execute workflows in ETL pipelines
- `knhk metrics` - Monitor workflow performance
