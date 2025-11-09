# KNHK v1.0 Implementation Guide

**Version**: 1.0.0  
**Status**: Production Implementation Guide  
**Last Updated**: 2025-11-09

---

## Overview

This guide provides step-by-step instructions for implementing and deploying KNHK v1.0 for Fortune 5 enterprise production environments.

**Critical Principle**: "Never trust the text, only trust test results" - All implementations must be verifiable through tests and OTEL validation.

---

## Prerequisites

### System Requirements

- **Rust**: 1.75+ (stable)
- **Cargo**: Latest stable
- **Operating System**: Linux (x86_64, ARM64), macOS (x86_64, ARM64)
- **Memory**: Minimum 4GB RAM, 8GB+ recommended
- **Storage**: 10GB+ free space for build artifacts

### Dependencies

- **Build Tools**: `make`, `gcc`/`clang`
- **System Libraries**: OpenSSL, zlib
- **Optional**: Docker (for containerized deployment)

### Development Tools

- **Code Quality**: `cargo clippy`, `cargo fmt`
- **Testing**: `cargo test`
- **Documentation**: `cargo doc`
- **Security**: `cargo audit` (optional)

---

## Implementation Steps

### Phase 1: Environment Setup

1. **Clone Repository**
   ```bash
   git clone https://github.com/seanchatmangpt/knhk.git
   cd knhk
   ```

2. **Install Dependencies**
   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install system dependencies (Ubuntu/Debian)
   sudo apt-get update
   sudo apt-get install -y build-essential pkg-config libssl-dev
   ```

3. **Verify Installation**
   ```bash
   cargo --version
   rustc --version
   ```

### Phase 2: Build & Compilation

1. **Format Code**
   ```bash
   cd rust
   cargo fmt --all
   ```

2. **Run Clippy (Linting)**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Build Project**
   ```bash
   cargo build --release
   ```

4. **Run Tests**
   ```bash
   cargo test --all
   ```

### Phase 3: Configuration

1. **Environment Variables**
   ```bash
   # Set required environment variables
   export KNHK_LOG_LEVEL=info
   export KNHK_STATE_STORE_PATH=/var/lib/knhk
   export KNHK_WORKFLOW_DB_PATH=/var/lib/knhk/workflow_db
   ```

2. **Configuration Files**
   - Review `rust/knhk-config/src/lib.rs` for configuration options
   - Create `config.toml` with production settings
   - Set up logging configuration

3. **Fortune 5 Features** (if enabled)
   ```bash
   export KNHK_FORTUNE5_ENABLED=true
   export KNHK_SPIFFE_ENABLED=true
   export KNHK_KMS_ENABLED=true
   export KNHK_MULTI_REGION_ENABLED=true
   ```

### Phase 4: Deployment

1. **Production Build**
   ```bash
   cargo build --release --features fortune5
   ```

2. **Install Binary**
   ```bash
   sudo cp target/release/knhk /usr/local/bin/
   sudo chmod +x /usr/local/bin/knhk
   ```

3. **Create Systemd Service** (Linux)
   ```ini
   [Unit]
   Description=KNHK Workflow Engine
   After=network.target

   [Service]
   Type=simple
   User=knhk
   WorkingDirectory=/var/lib/knhk
   ExecStart=/usr/local/bin/knhk
   Restart=always
   RestartSec=10

   [Install]
   WantedBy=multi-user.target
   ```

4. **Start Service**
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable knhk
   sudo systemctl start knhk
   ```

### Phase 5: Validation

1. **Run Pre-Push Validation**
   ```bash
   # This runs all validation gates
   git push
   ```

2. **Verify DoD Compliance**
   ```bash
   # Check Definition of Done criteria
   ./scripts/dod_validation.sh
   ```

3. **Run Integration Tests**
   ```bash
   cargo test --test integration_tests
   ```

4. **Verify OTEL Instrumentation**
   ```bash
   # Check that spans are being generated
   # Review OTEL collector logs
   ```

---

## Fortune 5 Enterprise Features

### SPIFFE/SPIRE Integration

1. **Install SPIRE**
   ```bash
   # Follow SPIRE installation guide
   # https://spiffe.io/docs/latest/spire-about/getting-started/
   ```

2. **Configure SPIFFE**
   ```bash
   export KNHK_SPIFFE_SOCKET=/tmp/spire-agent.sock
   export KNHK_SPIFFE_TRUST_DOMAIN=example.org
   ```

3. **Verify SPIFFE**
   ```bash
   # Test SPIFFE identity
   knhk workflow list --spiffe-enabled
   ```

### KMS Integration

1. **Configure KMS Provider**
   ```bash
   export KNHK_KMS_PROVIDER=aws
   export KNHK_KMS_REGION=us-east-1
   export KNHK_KMS_KEY_ID=arn:aws:kms:us-east-1:123456789012:key/abc123
   ```

2. **Test KMS**
   ```bash
   # Verify key rotation
   knhk fortune5 kms rotate
   ```

### Multi-Region Support

1. **Configure Regions**
   ```bash
   export KNHK_REGIONS=us-east-1,us-west-2,eu-west-1
   export KNHK_PRIMARY_REGION=us-east-1
   ```

2. **Test Multi-Region**
   ```bash
   # Verify cross-region replication
   knhk fortune5 multi-region sync
   ```

---

## Testing Strategy

### Unit Tests
```bash
cargo test --lib
```

### Integration Tests
```bash
cargo test --test integration_tests
```

### Chicago TDD Tests
```bash
cargo test --test chicago_tdd
```

### Van der Aalst Validation
```bash
./scripts/van_der_aalst_validation.sh
```

### XES Conformance Validation
```bash
./scripts/xes_validation.sh
```

---

## Monitoring & Observability

### OTEL Instrumentation

1. **Configure OTEL Collector**
   ```yaml
   receivers:
     otlp:
       protocols:
         grpc:
           endpoint: 0.0.0.0:4317
   
   exporters:
     logging:
       loglevel: info
   
   service:
     pipelines:
       traces:
         receivers: [otlp]
         exporters: [logging]
   ```

2. **Verify Spans**
   ```bash
   # Check OTEL collector logs for spans
   tail -f /var/log/otel-collector.log
   ```

### Metrics

1. **Prometheus Metrics**
   ```bash
   # Metrics available at /metrics endpoint
   curl http://localhost:8080/metrics
   ```

2. **Key Metrics**
   - `knhk_workflow_cases_total`
   - `knhk_workflow_tasks_completed_total`
   - `knhk_workflow_execution_duration_seconds`
   - `knhk_hot_path_ticks`

---

## Troubleshooting

### Common Issues

1. **Compilation Errors**
   ```bash
   # Clean and rebuild
   cargo clean
   cargo build --release
   ```

2. **Clippy Warnings**
   ```bash
   # Fix clippy warnings
   cargo clippy --fix --allow-dirty
   ```

3. **Test Failures**
   ```bash
   # Run specific test with verbose output
   cargo test --test test_name -- --nocapture
   ```

4. **OTEL Issues**
   ```bash
   # Check OTEL collector status
   systemctl status otel-collector
   # Review collector logs
   journalctl -u otel-collector -f
   ```

---

## Performance Tuning

### Hot Path Optimization

1. **Verify Tick Budget**
   ```bash
   # Check hot path tick count
   cargo test --test hot_path_performance
   ```

2. **SoA Alignment**
   ```bash
   # Verify 64-byte alignment
   cargo test --test soa_alignment
   ```

### Memory Optimization

1. **Enable Zero-Copy**
   ```bash
   export KNHK_ZERO_COPY=true
   ```

2. **Monitor Memory Usage**
   ```bash
   # Use valgrind or similar tools
   valgrind --leak-check=full ./target/release/knhk
   ```

---

## Security Hardening

1. **Enable Security Features**
   ```bash
   export KNHK_SECURITY_AUDIT=true
   export KNHK_SECURITY_SCAN=true
   ```

2. **Run Security Audit**
   ```bash
   cargo audit
   ```

3. **Review Security Checklist**
   - [ ] No hardcoded secrets
   - [ ] Input validation enabled
   - [ ] Guard constraints enforced
   - [ ] Error handling comprehensive
   - [ ] Resource cleanup verified

---

## Rollback Procedures

1. **Stop Service**
   ```bash
   sudo systemctl stop knhk
   ```

2. **Restore Previous Version**
   ```bash
   git checkout <previous-version>
   cargo build --release
   sudo cp target/release/knhk /usr/local/bin/
   ```

3. **Start Service**
   ```bash
   sudo systemctl start knhk
   ```

---

## Related Documentation

- [Definition of Done](./definition-of-done/fortune5-production.md)
- [Release Checklist](./certification/release-checklist.md)
- [Validation Report](./validation/final-report.md)
- [Performance Baseline](./performance/baseline.md)
- [Gaps and Priorities](./status/gaps-and-priorities.md)

---

## Support

For issues or questions:
- **Documentation**: See [Main Documentation Index](../INDEX.md)
- **Evidence**: See [Evidence Directory](../evidence/)
- **Architecture**: See [Architecture Documentation](../ARCHITECTURE.md)

---

## Notes

- All implementations must pass pre-push validation gates
- DoD criteria must be met before production deployment
- OTEL validation is the source of truth for telemetry
- Test results are authoritative over documentation claims

