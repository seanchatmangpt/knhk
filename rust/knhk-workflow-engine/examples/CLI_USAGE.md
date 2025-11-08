# KNHK Workflow Engine CLI Usage Guide

## Quick Start

### 1. Build the CLI

```bash
cd rust/knhk-workflow-engine
cargo build --release --bin knhk-workflow
```

### 2. List Available Commands

```bash
cargo run --release --bin knhk-workflow -- --help
```

### 3. List All Registered Patterns

```bash
cargo run --release --bin knhk-workflow -- list-patterns
```

This shows all 43 Van der Aalst workflow patterns registered in the engine.

### 4. Parse a Workflow from Turtle File

```bash
cargo run --release --bin knhk-workflow -- parse --file examples/simple-sequence.ttl
```

This parses the workflow and displays it as JSON. You can also save to a file:

```bash
cargo run --release --bin knhk-workflow -- parse --file examples/simple-sequence.ttl --output workflow.json
```

### 5. Register a Workflow

```bash
cargo run --release --bin knhk-workflow -- register --file examples/simple-sequence.ttl
```

This registers the workflow specification in the state store. The workflow will be validated for deadlocks before registration.

### 6. Get Workflow Specification

```bash
cargo run --release --bin knhk-workflow -- get-workflow <spec-id>
```

Replace `<spec-id>` with the workflow specification ID returned from registration.

### 7. Create a Workflow Case

```bash
cargo run --release --bin knhk-workflow -- create-case <spec-id> --data '{"customer_id":"12345","order_amount":1000.0}'
```

This creates a new case (workflow instance) with the provided data.

### 8. Start a Case

```bash
cargo run --release --bin knhk-workflow -- start-case <case-id>
```

This starts execution of the case.

### 9. Execute a Case

```bash
cargo run --release --bin knhk-workflow -- execute-case <case-id>
```

This executes the workflow case.

### 10. Get Case Status

```bash
cargo run --release --bin knhk-workflow -- get-case <case-id>
```

This shows the current state of the case.

### 11. Cancel a Case

```bash
cargo run --release --bin knhk-workflow -- cancel-case <case-id>
```

This cancels a running case.

### 12. Start REST API Server

```bash
cargo run --release --bin knhk-workflow -- serve --port 8080 --host 0.0.0.0
```

This starts the REST API server. You can then use HTTP requests to interact with the workflow engine:

```bash
# Register workflow
curl -X POST http://localhost:8080/workflows \
  -H "Content-Type: application/json" \
  -d @workflow.json

# Create case
curl -X POST http://localhost:8080/cases \
  -H "Content-Type: application/json" \
  -d '{"spec_id":"<spec-id>","data":{"key":"value"}}'

# Get case status
curl http://localhost:8080/cases/<case-id>
```

## Complete Workflow Example

```bash
# 1. Parse and register workflow
cargo run --release --bin knhk-workflow -- register --file examples/simple-sequence.ttl

# Output: Workflow registered: Simple Sequence Workflow (<spec-id>)

# 2. Create a case
cargo run --release --bin knhk-workflow -- create-case <spec-id> --data '{"input":"test"}'

# Output: Case created: <case-id>

# 3. Start the case
cargo run --release --bin knhk-workflow -- start-case <case-id>

# Output: Case started: <case-id>

# 4. Execute the case
cargo run --release --bin knhk-workflow -- execute-case <case-id>

# Output: Case executed: <case-id>

# 5. Check case status
cargo run --release --bin knhk-workflow -- get-case <case-id>

# Output: JSON with case details including state, timestamps, etc.
```

## State Store

By default, the state store is created at `./workflow_db`. You can specify a different path:

```bash
cargo run --release --bin knhk-workflow -- --state-store /path/to/store register --file workflow.ttl
```

## Example Workflow File

See `examples/simple-sequence.ttl` for a simple sequence workflow example.

## Integration with REST API

Once the server is running, you can use the REST API endpoints:

- `POST /workflows` - Register workflow
- `GET /workflows/:id` - Get workflow specification
- `POST /cases` - Create case
- `GET /cases/:id` - Get case status
- `POST /cases/:id/cancel` - Cancel case
- `GET /cases/:id/history` - Get case history

See `src/api/rest.rs` for the complete API specification.

