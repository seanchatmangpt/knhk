# Quick CLI Reference

## Installation

```bash
cd rust/knhk-workflow-engine
cargo install --path . --bin knhk-workflow
```

Or use directly:

```bash
cargo run --release --bin knhk-workflow -- <command>
```

## Commands

### Parse Workflow
```bash
knhk-workflow parse --file examples/simple-sequence.ttl
knhk-workflow parse --file workflow.ttl --output workflow.json
```

### Register Workflow
```bash
knhk-workflow register --file examples/simple-sequence.ttl
```

### List Patterns
```bash
knhk-workflow list-patterns
```

### Create Case
```bash
knhk-workflow create-case <spec-id> --data '{"key":"value"}'
```

### Manage Cases
```bash
knhk-workflow start-case <case-id>
knhk-workflow execute-case <case-id>
knhk-workflow get-case <case-id>
knhk-workflow cancel-case <case-id>
```

### Start Server
```bash
knhk-workflow serve --port 8080
```

## Example Session

```bash
# 1. List all available patterns (43 patterns)
knhk-workflow list-patterns

# 2. Parse a workflow
knhk-workflow parse --file examples/simple-sequence.ttl

# 3. Register the workflow
knhk-workflow register --file examples/simple-sequence.ttl
# Output: Workflow registered: Simple Sequence Workflow (<uuid>)

# 4. Create a case with data
knhk-workflow create-case <uuid-from-step-3> --data '{"input":"test"}'
# Output: Case created: <case-uuid>

# 5. Start the case
knhk-workflow start-case <case-uuid>

# 6. Execute the case
knhk-workflow execute-case <case-uuid>

# 7. Check status
knhk-workflow get-case <case-uuid>
```

## REST API (after starting server)

```bash
# Start server
knhk-workflow serve --port 8080

# In another terminal:
# Register workflow
curl -X POST http://localhost:8080/workflows \
  -H "Content-Type: application/json" \
  -d @workflow.json

# Create case
curl -X POST http://localhost:8080/cases \
  -H "Content-Type: application/json" \
  -d '{"spec_id":"<spec-id>","data":{"key":"value"}}'

# Get case
curl http://localhost:8080/cases/<case-id>
```

