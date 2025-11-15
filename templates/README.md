# Integration Templates

Ready-to-use code templates for common KNHK integrations. Copy and customize for your use case.

---

## 📋 Available Templates

### [Kafka Integration Template](kafka-integration-template.rs)
**What it provides**: Complete Kafka producer/consumer setup
- Producer: Send telemetry and events
- Consumer: Receive and process messages
- Error handling and retry logic
- Telemetry integration for monitoring
- Configuration best practices
- **Use this**: When integrating with Kafka

**Includes**:
- Producer setup and message sending
- Consumer setup and message handling
- Batch configuration
- Error recovery
- Telemetry for all operations

**Lines**: ~200 | **Integration time**: 30-45 min

---

### [OpenTelemetry Collector Integration](otlp-collector-integration.rs)
**What it provides**: OTLP exporter configuration
- Export spans, metrics, logs to collector
- Batching and buffering
- Proper initialization and shutdown
- Error handling
- Configuration options
- **Use this**: When setting up telemetry collection

**Includes**:
- OTLP exporter setup
- Batch configuration
- Resource attributes
- Sampler setup
- Exporter initialization

**Lines**: ~150 | **Integration time**: 15-30 min

---

### [PostgreSQL Integration Template](postgres-integration-template.rs)
**What it provides**: Database connection pool and queries
- Connection pool setup (sqlx)
- Query execution with proper error handling
- Transaction support
- Telemetry for database operations
- Migration management
- **Use this**: When using PostgreSQL

**Includes**:
- Pool initialization
- Query patterns
- Transaction handling
- Error management
- Telemetry integration

**Lines**: ~180 | **Integration time**: 30-45 min

---

### [Axum HTTP Server Template](http-server-template.rs)
**What it provides**: Complete web server setup
- Route definitions
- Middleware for logging, telemetry, errors
- Request/response handling
- JSON serialization
- Error responses
- **Use this**: When building HTTP APIs

**Includes**:
- Server initialization
- Router setup
- Middleware configuration
- Route handlers
- Error handling

**Lines**: ~200 | **Integration time**: 30-45 min

---

### [Workflow Engine Template](workflow-engine-template.rs)
**What it provides**: Workflow execution setup
- Initialize workflow engine
- Register workflows
- Execute workflows with monitoring
- Handle state transitions
- Error recovery
- **Use this**: When using the workflow engine

**Includes**:
- Engine initialization
- Workflow registration
- Execution with logging
- State management
- Error handling

**Lines**: ~180 | **Integration time**: 30-45 min

---

### [CLI Command Template](cli-command-template.rs)
**What it provides**: Complete CLI command implementation
- Argument parsing with clap
- Subcommand handling
- Telemetry integration
- Error handling and reporting
- Help text and documentation
- **Use this**: When building CLI tools

**Includes**:
- clap command definition
- Argument validation
- Subcommand routing
- Error reporting
- Telemetry

**Lines**: ~150 | **Integration time**: 20-30 min

---

## 🎯 Quick Selection Guide

| Task | Template |
|------|----------|
| Send events to Kafka | [Kafka Integration](kafka-integration-template.rs) |
| Send telemetry to collector | [OTLP Integration](otlp-collector-integration.rs) |
| Use PostgreSQL database | [PostgreSQL Integration](postgres-integration-template.rs) |
| Build web API | [HTTP Server](http-server-template.rs) |
| Run workflows | [Workflow Engine](workflow-engine-template.rs) |
| Build CLI tool | [CLI Command](cli-command-template.rs) |

---

## 💡 How to Use These Templates

1. **Copy the entire file**: Use as starting point for your implementation
2. **Read the comments**: Each section is explained
3. **Replace placeholders**: Customize for your use case
4. **Add your logic**: Implement your business logic around the template
5. **Test thoroughly**: Use the example patterns for testing

---

## 🔧 Integration Steps

For any template:

1. **Copy** the template file
2. **Review** the comments and structure
3. **Customize** configuration (URLs, credentials, etc.)
4. **Implement** your specific logic
5. **Test** with real/mock services
6. **Monitor** with telemetry integration

---

## 📊 Templates Statistics

| Template | Lines | Setup Time | Service |
|----------|-------|------------|---------|
| Kafka | ~200 | 30-45 min | Message Queue |
| OTLP | ~150 | 15-30 min | Telemetry |
| PostgreSQL | ~180 | 30-45 min | Database |
| Axum | ~200 | 30-45 min | Web Server |
| Workflow | ~180 | 30-45 min | Workflow Execution |
| CLI | ~150 | 20-30 min | Command Line |

**Total lines**: ~1060
**Average template**: ~177 lines
**Format**: Production Rust code

---

## 🔗 Related Documentation

- **Code Examples**: [Working examples](../examples/)
- **How-to Guides**: [Integration guides](../papers/how-to-guides/)
- **Troubleshooting**: [Common issues](../docs/troubleshooting/)
- **Quick Reference**: [Checklists and cards](../docs/reference/cards/)

---

## 🚀 Integration Workflow

**Typical integration process**:

```
1. Identify service → Choose template
2. Copy template → Customize configuration
3. Implement logic → Add business code
4. Test integration → Verify connectivity
5. Add telemetry → Monitor in production
6. Deploy → Use in production
```

---

## ⚙️ Configuration Best Practices

Each template includes:
- ✅ Configurable settings (no hardcoding)
- ✅ Error handling and retry logic
- ✅ Telemetry for observability
- ✅ Resource cleanup (shutdown)
- ✅ Production-ready patterns

---

## 🔐 Security Notes

All templates follow security best practices:
- No credentials in code
- Configuration from environment variables
- Proper error reporting (no sensitive data)
- TLS support where applicable
- Input validation

---

**Last Updated**: 2025-11-15
**Version**: v1.1.0
**Framework**: Integration Templates
