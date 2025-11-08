# Workflow Engine CLI Integration

**Status**: ✅ Complete | **Commands**: 10 | **Patterns**: 43

## Implementation

- **File**: `rust/knhk-cli/src/workflow.rs` (341 lines)
- **Dependencies**: tokio, axum, knhk-workflow-engine
- **Pattern**: Noun-verb (`knhk workflow <verb>`)

## Commands

| Verb | Description |
|------|-------------|
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

## Architecture

- **Runtime**: Tokio runtime (shared, thread-safe)
- **State Store**: Sled database (default: `./workflow_db`)
- **Error Handling**: User-friendly messages with proper propagation
- **Integration**: Auto-discovered by clap-noun-verb framework

## See Also

- **[Command Reference](WORKFLOW_COMMANDS.md)** - Complete command documentation
- **[Workflow Engine README](../knhk-workflow-engine/README.md)** - Engine documentation
