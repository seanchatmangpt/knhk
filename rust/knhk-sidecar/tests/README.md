# knhk-sidecar OpenTelemetry E2E Tests

## Test Coverage

### End-to-End Validation (`chicago_tdd_otel_e2e.rs`)
- **Purpose**: Verify sidecar service emits telemetry correctly
- **Status**: ⏳ Requires infrastructure
- **Requires**: 
  - knhk-sidecar service running
  - OTLP collector running (Docker or local)
  - Weaver binary (optional)

**Tests**:
- `test_sidecar_service_emits_telemetry` - Verify sidecar service emits telemetry
- `test_sidecar_operations_record_metrics` - Verify sidecar operations record metrics
- `test_weaver_validates_sidecar_telemetry` - Verify Weaver validates sidecar telemetry

**Run**: `cargo test --test chicago_tdd_otel_e2e --features fortune5 -- --ignored`

## Running Tests

### Unit Tests (No Infrastructure)
```bash
cd rust/knhk-sidecar
cargo test --features fortune5
```

### E2E Tests (Requires Infrastructure)
```bash
# Start OTLP collector
docker compose -f tests/integration/docker-compose.yml up -d otel-collector

# Start sidecar service
cargo run --bin knhk-sidecar --features fortune5 &

# Run E2E tests
cargo test --test chicago_tdd_otel_e2e --features fortune5 -- --ignored
```



