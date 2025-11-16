# ADR-001: Autonomous Loop Architecture

**Date**: 2025-11-16
**Status**: Accepted
**Deciders**: System Architecture Team

## Context

KNHK requires a system that continuously evolves the ontology based on observed telemetry data without human intervention. The system must be:

1. **Autonomous**: Run without human supervision
2. **Safe**: Prevent runaway evolution and breaking changes
3. **Observable**: Comprehensive monitoring and tracing
4. **Resilient**: Handle failures gracefully
5. **Production-ready**: No unsafe code, proper error handling

## Decision

We will implement the Autonomous Evolution Loop as a standalone crate (`knhk-autonomous-loop`) with the following architecture:

### Core Components

1. **LoopEngine**: Main orchestrator that runs continuous cycles
2. **EvolutionCycle**: Single cycle implementation (6 steps)
3. **LoopDependencies**: Dependency injection container
4. **LoopHealth**: Health monitoring and circuit breaker
5. **LoopTelemetry**: OpenTelemetry integration

### Six-Step Evolution Cycle

```
1. Observe    → Fetch recent telemetry receipts
2. Detect     → Mine patterns using PatternMiner
3. Propose    → Generate change proposals
4. Validate   → Check invariants Q preservation
5. Compile    → Create new snapshot Σ*
6. Promote    → Deploy to production if ready
```

### Safety Mechanisms

1. **Configuration Limits**:
   - `min_patterns_for_proposal`: Prevent premature evolution
   - `max_changes_per_cycle`: Prevent runaway changes
   - `pause_on_error_rate`: Circuit breaker threshold

2. **Validation Pipeline**:
   - All proposals must pass invariant checks
   - Production-ready flag required for auto-promotion
   - Weaver schema validation as source of truth

3. **Health Monitoring**:
   - Continuous health status tracking
   - Automatic pause on error threshold
   - Retry logic with exponential backoff

4. **Rollback Capability**:
   - Automatic rollback on SLO violations
   - Manual rollback support
   - Snapshot version history

### Dependency Injection

All dependencies are injected via `LoopDependencies`:
- Enables testing with mocks
- Loose coupling with KNHK components
- Runtime configuration flexibility

### Async-First Architecture

All I/O operations use Tokio async runtime:
- Non-blocking execution
- Efficient resource utilization
- Graceful concurrent processing

## Consequences

### Positive

1. **Autonomy**: Ontology evolves continuously without human intervention
2. **Safety**: Multiple safety mechanisms prevent runaway evolution
3. **Observability**: Comprehensive telemetry and health monitoring
4. **Testability**: Dependency injection enables thorough testing
5. **Modularity**: Standalone crate with clear interfaces
6. **Production-ready**: No unsafe code, proper error handling

### Negative

1. **Complexity**: Six-step cycle requires careful coordination
2. **Dependency Management**: Requires many KNHK components
3. **Testing Challenge**: Full integration tests require all dependencies
4. **Resource Usage**: Continuous loop consumes background resources

### Mitigations

1. **Complexity**: Clear separation of concerns, comprehensive documentation
2. **Dependencies**: Well-defined traits, mock implementations for testing
3. **Testing**: Comprehensive unit tests + integration tests with mocks
4. **Resources**: Configurable cycle interval, pause capability

## Alternatives Considered

### 1. Event-Driven Architecture

**Pros**:
- More reactive to changes
- Lower resource usage when idle

**Cons**:
- Harder to reason about cycle completion
- Complex event coordination
- Difficult to implement safety limits

**Decision**: Rejected. Periodic cycles are simpler and easier to monitor.

### 2. Cron-Based Execution

**Pros**:
- Familiar scheduling model
- External process control

**Cons**:
- No in-process health monitoring
- Harder to integrate with KNHK telemetry
- Less graceful shutdown

**Decision**: Rejected. In-process loop provides better integration.

### 3. Manual Trigger Only

**Pros**:
- Full human control
- No risk of runaway evolution

**Cons**:
- Defeats the purpose of autonomous evolution
- Requires constant human monitoring
- Slower ontology evolution

**Decision**: Rejected. Autonomy is a core requirement.

## Implementation Notes

### Phase 1: Core Loop (DONE)
- [x] LoopEngine orchestrator
- [x] EvolutionCycle implementation
- [x] Health monitoring
- [x] Telemetry integration
- [x] Comprehensive tests

### Phase 2: KNHK Integration (TODO)
- [ ] Wire up real SnapshotStore
- [ ] Integrate PatternMiner
- [ ] Connect DeltaSigmaValidator
- [ ] Implement PromotionPipeline
- [ ] End-to-end testing

### Phase 3: Production Hardening (TODO)
- [ ] Weaver schema validation
- [ ] Performance benchmarking
- [ ] Load testing
- [ ] Monitoring dashboards
- [ ] Alerting setup

## Success Metrics

1. **Uptime**: >99.9% loop availability
2. **Cycle Success Rate**: >95% successful cycles
3. **False Positive Rate**: <1% (validated by Weaver)
4. **Promotion Frequency**: 1-10 promotions per day
5. **Rollback Frequency**: <1 rollback per week

## Validation

This architecture will be validated by:

1. **Weaver Schema Validation**: All telemetry conforms to schema
2. **Integration Tests**: Full cycle execution with mocks
3. **Performance Benchmarks**: Cycle duration <10 seconds
4. **Production Testing**: Staged rollout with monitoring

## References

- KNHK Architecture Overview: `/docs/architecture.md`
- Pattern Mining Design: `../knhk-pattern-mining/docs/DESIGN.md`
- Delta-Sigma Validation: `../knhk-delta-sigma/docs/DESIGN.md`
- OpenTelemetry: https://opentelemetry.io/

## Approval

**Approved by**: System Architect
**Date**: 2025-11-16
**Review Date**: 2026-02-16 (90 days)
