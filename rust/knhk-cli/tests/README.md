# knhk-cli OpenTelemetry E2E Tests

## Test Coverage

### End-to-End Validation (`chicago_tdd_otel_e2e.rs`)
- **Purpose**: Verify CLI commands emit telemetry correctly
- **Status**: ⏳ Requires infrastructure
- **Requires**: 
  - knhk-cli binary
  - OTLP collector running (Docker or local)
  - Weaver binary (optional)

**Tests**:
- `test_cli_commands_emit_telemetry` - Verify CLI commands emit telemetry
- `test_pipeline_command_emits_telemetry` - Verify pipeline command emits telemetry
- `test_boot_command_emits_telemetry` - Verify boot command emits telemetry
- `test_connect_command_emits_telemetry` - Verify connect command emits telemetry
- `test_weaver_validates_cli_telemetry` - Verify Weaver validates CLI telemetry

**Run**: `cargo test --test chicago_tdd_otel_e2e --features otel -- --ignored`

## Running Tests

### Unit Tests (No Infrastructure)
```bash
cd rust/knhk-cli
cargo test --features otel
```

### E2E Tests (Requires Infrastructure)
```bash
# Start OTLP collector
docker compose -f tests/integration/docker-compose.yml up -d otel-collector

# Run E2E tests
cargo test --test chicago_tdd_otel_e2e --features otel -- --ignored
```


