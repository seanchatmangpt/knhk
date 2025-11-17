# Configuration Guide

**Complete Reference for KNHK Workflow Engine Configuration**

- **Config Format**: TOML, YAML, JSON, or environment variables
- **Location**: `./config.toml` or `/etc/knhk/config.toml`
- **Precedence**: CLI args > Env vars > Config file > Defaults
- **Last Updated**: 2025-11-17

---

## Table of Contents

1. [Server Configuration](#server-configuration)
2. [Execution Configuration](#execution-configuration)
3. [Storage Configuration](#storage-configuration)
4. [Validation Configuration](#validation-configuration)
5. [Observability Configuration](#observability-configuration)
6. [Security Configuration](#security-configuration)
7. [Performance Tuning](#performance-tuning)
8. [Example Configurations](#example-configurations)

---

## Server Configuration

### Port & Binding

```toml
[server]
# HTTP server listening port
port = 8080

# Bind address (0.0.0.0 = all interfaces)
bind_address = "0.0.0.0"

# Enable HTTPS
tls_enabled = false
tls_cert_path = "/etc/knhk/certs/server.crt"
tls_key_path = "/etc/knhk/certs/server.key"

# HTTP/2 support
http2_enabled = true

# Request timeout in seconds
request_timeout = 30

# Keep-alive timeout in seconds
keepalive_timeout = 60
```

**Environment Variables**:
```bash
KNHK_SERVER_PORT=8080
KNHK_SERVER_BIND_ADDRESS=0.0.0.0
KNHK_SERVER_TLS_ENABLED=false
KNHK_SERVER_REQUEST_TIMEOUT=30
```

---

### API Configuration

```toml
[api]
# API version
version = "v1"

# Prefix for all API routes
prefix = "/api"

# Enable OpenAPI/Swagger documentation
swagger_enabled = true
swagger_path = "/api/swagger"

# CORS settings
cors_enabled = true
cors_origins = ["http://localhost:3000", "https://app.example.com"]
cors_methods = ["GET", "POST", "PUT", "DELETE"]
cors_headers = ["Content-Type", "Authorization"]

# Max request body size in MB
max_request_size = 10

# Graceful shutdown timeout in seconds
shutdown_timeout = 30
```

---

## Execution Configuration

### Execution Engine

```toml
[execution]
# Number of worker threads for case execution
worker_threads = 8

# Queue depth for pending tasks
queue_depth = 1000

# Task execution timeout in seconds (0 = no timeout)
task_timeout = 0

# Maximum concurrent cases
max_concurrent_cases = 5000

# Case expiration in days (0 = no expiration)
case_expiration_days = 90

# Execution model: "eager", "lazy", or "hybrid"
execution_model = "eager"

# Enable deterministic execution (slower but reproducible)
deterministic = false

# Performance target: Chatman constant in ticks
chatman_constant_ticks = 8
```

**Performance Settings**:
- `worker_threads`: Match your CPU cores for optimal throughput
- `queue_depth`: Adjust based on available memory (1000 = ~500MB)
- `task_timeout`: Set to 0 for no timeout, or number of seconds
- `execution_model`:
  - `eager`: Start tasks immediately when enabled
  - `lazy`: Start tasks on-demand
  - `hybrid`: Eager for critical path, lazy otherwise

---

### Hook Configuration

```toml
[hooks]
# Enable the hook engine
enabled = true

# Maximum concurrent hook executions
max_concurrent = 20

# Hook execution timeout in seconds
timeout = 5

# Hook retry policy
retry_enabled = true
retry_max_attempts = 3
retry_backoff_ms = 100

# Pre-execution hooks
pre_task_enabled = true
pre_case_enabled = true

# Post-execution hooks
post_task_enabled = true
post_case_enabled = true
```

---

## Storage Configuration

### State Store

```toml
[storage]
# Storage backend: "sled", "sqlite", or "postgres"
backend = "sled"

# Storage directory (for file-based backends)
data_dir = "./data/knhk"

# Enable persistence
persistence_enabled = true

# Compression for stored data
compression = true

# Encryption at rest
encryption_enabled = false
encryption_key_path = "/etc/knhk/keys/encryption.key"
```

### Sled Configuration (default)

```toml
[storage.sled]
# Cache size in MB
cache_size_mb = 512

# Database directory
db_path = "./data/knhk/cases"

# Flush interval in seconds (0 = never auto-flush)
flush_interval = 30

# Enable checksums
checksums = true
```

### PostgreSQL Configuration

```toml
[storage.postgres]
# Connection string
connection_string = "postgresql://user:pass@localhost:5432/knhk"

# Connection pool size
pool_size = 20

# Connection timeout in seconds
connection_timeout = 10

# Query timeout in seconds
query_timeout = 30

# Enable prepared statements
prepared_statements = true

# SSL mode: disable, allow, prefer, require
ssl_mode = "prefer"
```

---

## Validation Configuration

### Deadlock Detection

```toml
[validation.deadlock]
# Enable deadlock detection
enabled = true

# Detection algorithm: "petri_net", "graph_analysis", or "hybrid"
algorithm = "hybrid"

# Timeout for detection in seconds
timeout_seconds = 30

# Run on workflow registration
check_on_register = true

# Run on case creation
check_on_case_create = false
```

### SHACL Validation

```toml
[validation.shacl]
# Enable SHACL constraint validation
enabled = true

# SHACL shapes file location
shapes_file = "./ontology/shapes.ttl"

# Validation timeout in seconds
timeout_seconds = 10

# Fail on constraint violation
fail_on_violation = true
```

### Soundness Checking

```toml
[validation.soundness]
# Enable soundness checking
enabled = true

# Soundness algorithm: "wft", "relaxed", or "full"
algorithm = "wft"

# Accept workflows with issues but mark as risky
allow_unsound = false
```

---

## Observability Configuration

### OTEL/Tracing

```toml
[observability.otel]
# Enable OpenTelemetry tracing
enabled = true

# OTEL collector endpoint
collector_endpoint = "http://localhost:4317"

# Sampling rate (0.0 to 1.0)
sampling_rate = 0.1

# Batch size for spans
batch_size = 512

# Export interval in seconds
export_interval = 5

# Service name for spans
service_name = "knhk-workflow-engine"

# Service version
service_version = "1.0.0"
```

### Metrics

```toml
[observability.metrics]
# Enable metrics collection
enabled = true

# Metrics export interval in seconds
export_interval = 60

# Include detailed metrics (higher cardinality)
detailed_metrics = false

# Metric categories to enable
enable_execution_metrics = true
enable_resource_metrics = true
enable_queue_metrics = true
enable_latency_metrics = true
```

### Logging

```toml
[observability.logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Log format: text, json
format = "json"

# Log to file
file_enabled = true
file_path = "./logs/knhk.log"

# Log file rotation
rotation_enabled = true
rotation_max_size_mb = 100
rotation_max_files = 10

# Log to stdout
stdout_enabled = true

# Structured logging with context
structured = true

# Performance logging (slow operations)
perf_logging_enabled = true
perf_threshold_ms = 100
```

---

## Security Configuration

### Authentication

```toml
[security.auth]
# Enable authentication
enabled = false

# Auth type: "none", "bearer", "api_key", "oauth2"
type = "none"

# Bearer token validation
bearer_enabled = false
bearer_secret = "${KNHK_BEARER_SECRET}"

# API key validation
api_key_enabled = false
api_key_header = "X-API-Key"

# API keys (comma-separated)
api_keys = ["key1", "key2"]
```

### Authorization

```toml
[security.authz]
# Enable authorization checks
enabled = false

# Authz type: "none", "role", "policy"
type = "none"

# Role-based access control
rbac_enabled = false
roles_file = "./config/roles.toml"

# Policy-based access control
policy_enabled = false
policy_engine = "rego"
policies_dir = "./policies"
```

### Audit Logging

```toml
[security.audit]
# Enable audit logging
enabled = true

# Audit log location
log_path = "./logs/audit.log"

# Log sensitive data
log_sensitive_data = false

# Audit events to log
log_case_creation = true
log_task_completion = true
log_case_deletion = true
log_api_access = false
```

---

## Performance Tuning

### Memory Optimization

```toml
[performance.memory]
# Enable memory pooling for allocations
pooling_enabled = true

# Pool size in MB
pool_size_mb = 512

# Buffer reuse
buffer_reuse = true

# GC behavior: "aggressive", "balanced", "lazy"
gc_behavior = "balanced"
```

### Caching

```toml
[performance.cache]
# Enable caching of workflow specs
spec_cache_enabled = true
spec_cache_size = 1000
spec_cache_ttl_seconds = 3600

# Enable pattern matching cache
pattern_cache_enabled = true
pattern_cache_size = 5000
```

### Concurrency

```toml
[performance.concurrency]
# Enable work-stealing scheduler
work_stealing = true

# NUMA awareness
numa_aware = false

# Lock-free data structures
lock_free = true

# Task migration between threads
task_migration = true
```

---

## Example Configurations

### Development Configuration

```toml
# config.dev.toml

[server]
port = 8080
bind_address = "127.0.0.1"
request_timeout = 300

[execution]
worker_threads = 2
max_concurrent_cases = 100
deterministic = true

[storage]
backend = "sled"
data_dir = "./data/dev"
persistence_enabled = true

[observability.otel]
enabled = false

[observability.logging]
level = "debug"
format = "text"

[security.auth]
enabled = false
```

**Run**: `knhk --config config.dev.toml`

---

### Production Configuration

```toml
# config.prod.toml

[server]
port = 8080
bind_address = "0.0.0.0"
tls_enabled = true
tls_cert_path = "/etc/knhk/certs/server.crt"
tls_key_path = "/etc/knhk/certs/server.key"
request_timeout = 30

[execution]
worker_threads = 16
max_concurrent_cases = 10000
task_timeout = 600
execution_model = "hybrid"

[storage]
backend = "postgres"
persistence_enabled = true
compression = true
encryption_enabled = true
encryption_key_path = "/etc/knhk/keys/encryption.key"

[storage.postgres]
connection_string = "postgresql://knhk_user:${DB_PASSWORD}@db.internal:5432/knhk_prod"
pool_size = 50
ssl_mode = "require"

[validation]
deadlock.check_on_register = true
shacl.fail_on_violation = true

[observability.otel]
enabled = true
collector_endpoint = "http://otel-collector:4317"
sampling_rate = 0.5

[observability.logging]
level = "warn"
format = "json"
file_enabled = true

[security.auth]
enabled = true
type = "api_key"
api_keys = "${KNHK_API_KEYS}"

[security.audit]
enabled = true
log_case_creation = true
```

---

### High-Throughput Configuration

```toml
# config.high-throughput.toml

[execution]
worker_threads = 32
queue_depth = 5000
max_concurrent_cases = 50000
execution_model = "eager"

[storage]
backend = "postgres"
compression = true

[storage.postgres]
pool_size = 100
prepared_statements = true

[performance.cache]
spec_cache_enabled = true
spec_cache_size = 10000
pattern_cache_enabled = true
pattern_cache_size = 50000

[performance.concurrency]
work_stealing = true
lock_free = true
task_migration = true

[observability.logging]
level = "warn"
perf_logging_enabled = true
perf_threshold_ms = 50
```

---

## Environment Variable Override Examples

```bash
# Server
export KNHK_SERVER_PORT=9000
export KNHK_SERVER_BIND_ADDRESS=0.0.0.0

# Execution
export KNHK_EXECUTION_WORKER_THREADS=16
export KNHK_EXECUTION_MAX_CONCURRENT_CASES=5000

# Storage (PostgreSQL)
export KNHK_STORAGE_BACKEND=postgres
export KNHK_STORAGE_POSTGRES_CONNECTION_STRING="postgresql://user:pass@localhost/knhk"

# OTEL
export KNHK_OBSERVABILITY_OTEL_ENABLED=true
export KNHK_OBSERVABILITY_OTEL_COLLECTOR_ENDPOINT=http://localhost:4317

# Auth
export KNHK_SECURITY_AUTH_ENABLED=true
export KNHK_SECURITY_AUTH_TYPE=api_key
export KNHK_SECURITY_AUTH_API_KEYS="key1,key2,key3"

# Logging
export KNHK_OBSERVABILITY_LOGGING_LEVEL=debug
export KNHK_OBSERVABILITY_LOGGING_FORMAT=json
```

---

## Performance Tuning Guide

### For High Throughput (1000+ cases/sec)

1. Set `worker_threads` = number of CPU cores
2. Increase `queue_depth` to 5000-10000
3. Use PostgreSQL with pool size 50+
4. Enable caching (`spec_cache_size = 10000`)
5. Set `execution_model = "eager"`
6. Enable work-stealing scheduler
7. Reduce logging level to `warn`
8. Set OTEL sampling to 0.1 or lower

### For Low Latency (P99 < 100ms)

1. Keep `queue_depth` reasonable (1000-2000)
2. Enable `deterministic = true`
3. Set `task_timeout` appropriately
4. Use SSD storage (Sled) or local PostgreSQL
5. Enable connection pooling
6. Reduce logging overhead
7. Monitor and tune OTEL sampling
8. Profile with Chicago TDD benchmarks

### For Development/Testing

1. Use Sled backend with small `cache_size_mb = 64`
2. Set `worker_threads = 2`
3. Enable debug logging (`level = "debug"`)
4. Disable auth for simpler testing
5. Small cache sizes (`spec_cache_size = 100`)

---

## Related Documentation

- [API Endpoints Reference](./api-endpoints.md) - Complete API guide
- [Error Codes Reference](./error-codes.md) - Error handling guide
- [How-To: Kubernetes Deployment](../how-to/kubernetes-deployment.md) - Production setup
- [How-To: Performance Tuning](../how-to/troubleshooting.md#performance-issues) - Tuning guide
