# Workflow Engine CLI Integration

**Status**: ✅ Complete | **Commands**: 10 | **Patterns**: 43

## Quick Reference

```bash
# Parse and register
knhk workflow parse workflow.ttl
knhk workflow register workflow.ttl

# Create and execute
knhk workflow create <spec-id> --data '{"input":"test"}'
knhk workflow start <case-id>
knhk workflow execute <case-id>

# List patterns
knhk workflow patterns
```

## Commands

| Command | Description |
|---------|-------------|
| `parse` | Parse workflow from Turtle file |
| `register` | Register workflow specification |
| `create` | Create workflow case |
| `start` | Start case execution |
| `execute` | Execute workflow case |
| `get` | Get case status |
| `cancel` | Cancel case |
| `list` | List cases |
| `patterns` | List all 43 patterns |
| `serve` | Start REST API server |

## Implementation

- **File**: `rust/knhk-cli/src/workflow.rs` (341 lines)
- **Dependencies**: tokio, axum, knhk-workflow-engine
- **Pattern**: Noun-verb (`knhk workflow <verb>`)
- **Runtime**: Tokio (shared, thread-safe)
- **State Store**: Sled (default: `./workflow_db`)

## Documentation

- **[Complete Command Reference](WORKFLOW_COMMANDS.md)** - All commands with examples
- **[Workflow Engine README](../knhk-workflow-engine/README.md)** - Engine documentation
