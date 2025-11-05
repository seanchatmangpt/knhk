# knhk-cli Documentation

Command-line interface for KNHK operations.

## File Structure

```
rust/knhk-cli/
├── src/
│   ├── main.rs             # Main entry point, CLI setup
│   └── commands/
│       ├── mod.rs          # Command module exports
│       ├── boot.rs         # boot init - System initialization
│       ├── connect.rs      # connect register - Connector management
│       ├── cover.rs         # cover define - Coverage definition
│       ├── reflex.rs       # reflex declare - Reflex declaration
│       ├── epoch.rs        # epoch create - Epoch creation
│       ├── pipeline.rs     # pipeline run - Pipeline execution
│       ├── receipt.rs       # Receipt operations
│       ├── metrics.rs      # Metrics viewing
│       ├── config.rs        # Configuration management
│       ├── route.rs         # Routing configuration
│       ├── hook.rs          # Hook management
│       ├── admit.rs         # Admission control
│       └── coverage.rs      # Coverage operations
└── Cargo.toml
```

## Commands

### System Commands
- `boot init <sigma> <q>` - Initialize schema (Σ) and invariants (Q)

### Connector Commands
- `connect register <name> <schema> <source>` - Register data connector

### Coverage Commands
- `cover define <select> <shard>` - Define coverage

### Reflex Commands
- `reflex declare <name> <op> <pred> <off> <len>` - Declare reflex

### Epoch Commands
- `epoch create <id> <tau> <lambda>` - Create epoch

### Pipeline Commands
- `pipeline run [--connectors] [--schema]` - Run ETL pipeline

## Architecture

- **Noun-Verb Interface**: Based on CONVO.txt API design
- **Command Modules**: Each command in separate module
- **Configuration**: Global config loaded at startup
- **OTEL Integration**: Metrics and tracing support

## Dependencies

- `clap-noun-verb` - CLI framework
- `knhk-config` - Configuration management
- `knhk-otel` - Observability (optional)

## Related Documentation

- [CLI Guide](../../../docs/cli.md) - Complete CLI reference
- [Architecture](../../../docs/architecture.md) - System architecture

