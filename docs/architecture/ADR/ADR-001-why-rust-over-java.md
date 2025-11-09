# ADR-001: Why Rust Over Java for Workflow Engine

**Status:** Accepted
**Date:** 2025-11-08
**Deciders:** System Architect, Performance Team
**Technical Story:** Enterprise workflow engine reimplementation

## Context

YAWL is implemented in Java with Hibernate ORM, Spring Framework, and Tomcat. We need to decide whether to:
1. Continue with Java (maintain compatibility)
2. Migrate to Rust (performance and safety)
3. Hybrid approach (Rust engine + Java services)

## Decision Drivers

- **Performance:** Sub-tick latency requirement (<8 ticks = 0.125ms on 64KHz clock)
- **Memory Safety:** Zero-copy data structures, no garbage collection pauses
- **Concurrency:** Lock-free data structures for high throughput
- **Reliability:** Type safety, no null pointer exceptions, exhaustive error handling
- **Deployment:** Single binary, minimal runtime dependencies
- **Enterprise Adoption:** Rust gaining traction in finance, healthcare

## Considered Options

### Option 1: Continue with Java

**Pros:**
- Existing YAWL codebase can be reused
- Large Java developer pool
- Mature ecosystem (Spring, Hibernate, etc.)
- Well-known deployment patterns

**Cons:**
- GC pauses make sub-tick latency impossible
- No compile-time memory safety
- Heap allocation overhead
- Runtime dependencies (JVM, application server)
- Cannot achieve <8 tick requirement

### Option 2: Full Rust Migration

**Pros:**
- Zero-cost abstractions (no runtime overhead)
- Memory safety without GC (no pauses)
- Lock-free concurrency primitives
- Sub-tick latency achievable via knhk-hot
- Single binary deployment
- Modern async/await with tokio
- Type safety catches bugs at compile time

**Cons:**
- Steeper learning curve
- Smaller developer pool
- Some enterprise libraries less mature
- Migration effort required

### Option 3: Hybrid Approach

**Pros:**
- Reuse YAWL services (resource service, etc.)
- Rust engine for performance-critical paths
- Gradual migration possible

**Cons:**
- JNI overhead negates Rust benefits
- Complex deployment (JVM + binary)
- FFI boundary error handling
- Two codebases to maintain

## Decision Outcome

**Chosen Option: Full Rust Migration (Option 2)**

### Rationale

1. **Performance is Non-Negotiable:**
   - Sub-tick latency (<8 ticks) is absolute requirement
   - Only achievable without GC
   - Rust zero-cost abstractions deliver this

2. **Memory Safety Critical for Enterprise:**
   - Fortune 500 companies require 99.99% uptime
   - Memory bugs are #1 cause of production failures
   - Rust's borrow checker prevents entire classes of bugs

3. **Concurrency Model Superior:**
   - Lock-free data structures (DashMap, crossbeam)
   - No data races (enforced by compiler)
   - Async/await scales to millions of concurrent cases

4. **Modern Observability:**
   - OpenTelemetry native integration
   - No instrumentation overhead
   - Weaver schema validation prevents false positives

5. **Deployment Simplicity:**
   - Single static binary
   - No JVM tuning required
   - Minimal attack surface

### Consequences

**Positive:**
- Achieve <8 tick latency requirement
- Zero-downtime deployments
- Predictable performance (no GC pauses)
- Memory safety guarantees
- Modern async ecosystem
- Competitive advantage (few Rust workflow engines exist)

**Negative:**
- YAWL code cannot be directly reused (must reimplement)
- Smaller Rust developer pool
- Team training required
- Some libraries less mature (XQuery, SOAP)

**Mitigation:**
- Implement YAWL compatibility layer for gradual migration
- Use FFI bridges where Rust libraries immature (Saxon for XQuery)
- Invest in team training (2-week Rust bootcamp)
- Prioritize 80/20 features to reduce implementation scope

## Implementation Notes

### Migration Strategy

1. **Phase 1 (Weeks 1-4):** Core engine (patterns, cases, workflows)
2. **Phase 2 (Weeks 5-8):** Interface B (work items, resources)
3. **Phase 3 (Weeks 9-12):** Integration (connectors, services)
4. **Phase 4 (Weeks 13-16):** Advanced features (worklets, data gateway)

### Technology Choices

| Component | Rust Crate | Java Equivalent |
|-----------|-----------|-----------------|
| Web Framework | Axum | Spring Boot |
| ORM | (Not used - Sled KV) | Hibernate |
| HTTP Client | reqwest | Apache HttpClient |
| XML Parser | quick-xml | DOM/SAX |
| JSON | serde_json | Jackson |
| Async Runtime | tokio | CompletableFuture |
| Telemetry | tracing + OTLP | OpenTelemetry Java |

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Team learning curve | High | Medium | 2-week training, pair programming |
| Library immaturity | Medium | Medium | Use FFI bridges (JNI to Saxon) |
| Migration bugs | High | High | YAWL compatibility layer, gradual cutover |
| Performance issues | Low | High | Early benchmarking, hot-path optimization |

## References

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [knhk-hot Sub-Tick Architecture](/docs/architecture/sub-tick-latency.md)
- [YAWL Architecture](http://www.yawlfoundation.org/architecture.html)
- [Chatman Constant (8 Ticks)](/docs/performance/chatman-constant.md)

## Related Decisions

- ADR-003: Sled vs PostgreSQL for state store
- ADR-004: Why add gRPC to REST API
- ADR-005: OpenTelemetry over OpenXES
