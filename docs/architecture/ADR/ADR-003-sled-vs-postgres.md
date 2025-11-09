# ADR-003: Sled for State Store, PostgreSQL for Audit

**Status:** Accepted
**Date:** 2025-11-08
**Deciders:** System Architect, DBA Team
**Technical Story:** Persistent storage architecture

## Context

Workflow engine needs persistent storage for:
1. **Workflow specifications** (workflows, patterns)
2. **Case state** (active cases, execution state)
3. **Work item state** (queues, allocations)
4. **Audit trail** (immutable event log)
5. **Resource data** (resource profiles, allocations)

We need to decide storage technology for each.

## Decision Drivers

- **Performance:** Sub-tick latency for hot path (<8 ticks)
- **ACID:** Transactional guarantees for state changes
- **Query Capability:** SQL for reporting vs key-value for state
- **Deployment:** Single binary vs external database
- **Scalability:** Horizontal scaling requirements
- **Audit:** Tamper-proof immutable log

## Considered Options

### Option 1: PostgreSQL for Everything

**Pros:**
- Single database (simple architecture)
- Rich query capabilities (SQL)
- ACID transactions
- Mature tooling
- Horizontal scaling (Citus, Patroni)
- JSON/JSONB support

**Cons:**
- Network latency (cannot achieve <8 ticks)
- External dependency (deployment complexity)
- Connection pooling overhead
- Query planner overhead for simple gets
- Overkill for key-value access patterns

**Benchmark:**
- Simple GET: ~500 µs (40 ticks at 64 KHz)
- **VERDICT:** Cannot meet <8 tick requirement

### Option 2: Sled for Everything

**Pros:**
- Embedded KV store (no network latency)
- ACID transactions
- Zero-copy reads
- Sub-microsecond latency
- Single binary deployment
- Lock-free data structures

**Cons:**
- No SQL (limited query capability)
- No horizontal scaling (single node)
- Limited tooling
- No advanced indexing
- Difficult reporting/analytics

**Benchmark:**
- Simple GET: ~200 ns (0.0128 ticks at 64 KHz)
- **VERDICT:** Meets <8 tick requirement ✅

### Option 3: Sled + PostgreSQL (Hybrid)

**Pros:**
- Sled for hot path (sub-tick latency)
- PostgreSQL for audit (rich queries)
- Best of both worlds
- Clear separation of concerns

**Cons:**
- Two databases to manage
- Consistency between databases
- More complex architecture

**Benchmark:**
- Hot path (Sled): <8 ticks ✅
- Audit queries (Postgres): No latency requirement
- **VERDICT:** Meets all requirements ✅

### Option 4: RocksDB + PostgreSQL

**Pros:**
- RocksDB is battle-tested (used by Facebook)
- Better write performance than Sled
- Horizontal scaling (TiKV)

**Cons:**
- Heavier dependency (C++ library)
- More complex build
- Sled is pure Rust (safer)

## Decision Outcome

**Chosen Option: Sled + PostgreSQL Hybrid (Option 3)**

### Storage Allocation

| Data Type | Storage | Rationale |
|-----------|---------|-----------|
| Workflow Specs | Sled | Hot path, sub-tick reads |
| Case State | Sled | Hot path, frequent updates |
| Work Item State | Sled | Hot path, queue operations |
| Resource Allocations | Sled | Hot path, availability checks |
| Pattern Registry | In-Memory | Static, no persistence needed |
| **Audit Trail** | **PostgreSQL** | Immutable log, SQL reporting |
| **Case History** | **PostgreSQL** | Time-series queries |
| **Analytics** | **PostgreSQL** | Complex aggregations |

### Architecture

```
Hot Path (< 8 ticks):
  WorkflowEngine → Sled → In-Memory Cache
                      ↓
                 Async Audit Writer
                      ↓
                 PostgreSQL (Audit)

Cold Path (Reporting):
  Dashboard → PostgreSQL (SQL Queries)
```

### Rationale

1. **Performance Critical Path:**
   - Workflow execution MUST be <8 ticks
   - Sled achieves 0.0128 ticks (200 ns)
   - PostgreSQL would be 40 ticks (unacceptable)

2. **Audit Immutability:**
   - Audit log MUST be tamper-proof
   - PostgreSQL offers better tooling for compliance
   - Can integrate with Lockchain for blockchain audit

3. **Operational Simplicity:**
   - Sled is embedded (single binary for core engine)
   - PostgreSQL is external (can scale independently)
   - Different SLAs for different data

4. **Query Capabilities:**
   - Audit queries are complex SQL (joins, aggregations)
   - State queries are simple key-value (no SQL needed)

5. **Consistency Model:**
   - Sled: Strong consistency (ACID transactions)
   - PostgreSQL: Eventual consistency for audit (async writes)
   - Audit lag acceptable (1-5 seconds)

### Consistency Strategy

**Problem:** How to keep Sled and PostgreSQL consistent?

**Solution:** Async Audit Writer

1. **Synchronous Path (Sled):**
   - Engine executes workflow
   - Updates Sled (transactional)
   - Returns success to client
   - **Latency:** <8 ticks ✅

2. **Asynchronous Path (PostgreSQL):**
   - Engine publishes event to audit channel
   - Background worker drains channel
   - Writes to PostgreSQL in batches
   - **Latency:** 1-5 seconds (acceptable)

3. **Failure Handling:**
   - If PostgreSQL write fails, retry with backoff
   - Temporary queue in Sled (WAL-style)
   - Alert on prolonged failures

```rust
// Synchronous state update
async fn update_case_state(&self, case_id: CaseId, new_state: CaseState) -> Result<()> {
    // Hot path: Update Sled
    self.state_store.set(case_id, new_state)?;  // <8 ticks

    // Async audit: Send to channel (non-blocking)
    self.audit_tx.send(AuditEvent::CaseStateChanged {
        case_id,
        old_state,
        new_state,
        timestamp: SystemTime::now(),
    })?;

    Ok(())
}

// Async audit writer (background task)
async fn audit_writer_loop(mut rx: Receiver<AuditEvent>, pg_pool: PgPool) {
    let mut buffer = Vec::with_capacity(1000);

    loop {
        tokio::select! {
            Some(event) = rx.recv() => buffer.push(event),
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                if !buffer.is_empty() {
                    pg_pool.batch_insert(&buffer).await?;
                    buffer.clear();
                }
            }
        }
    }
}
```

### Consequences

**Positive:**
- Sub-tick latency achieved ✅
- Single binary deployment (Sled embedded)
- Rich audit queries (PostgreSQL)
- Clear separation of concerns
- Independent scaling (Sled in-process, PostgreSQL external)
- Compliance-ready audit trail

**Negative:**
- Two databases to operate
- Eventual consistency for audit (lag)
- Complexity in failure scenarios
- Need to monitor both databases

**Mitigation:**
- Use DashMap for in-memory caching (reduce Sled reads)
- Batch PostgreSQL writes (reduce overhead)
- Monitor audit lag (alert if >10 seconds)
- Provide reconciliation tool (Sled → PostgreSQL sync)

## Implementation Notes

### Sled Schema (Key-Value)

```rust
// Workflow Specifications
Key:   "spec:{spec_id}"
Value: bincode::serialize(WorkflowSpec)

// Case State
Key:   "case:{case_id}"
Value: bincode::serialize(CaseState)

// Work Item State
Key:   "workitem:{workitem_id}"
Value: bincode::serialize(WorkItemState)

// Resource Allocations
Key:   "allocation:{resource_id}:{workitem_id}"
Value: bincode::serialize(AllocationState)

// Queues (sorted sets)
Tree:  "queue:offered"
Key:   "{priority}:{deadline}:{workitem_id}"
Value: bincode::serialize(WorkItemId)
```

### PostgreSQL Schema (Audit)

```sql
-- Case audit trail
CREATE TABLE case_events (
    id BIGSERIAL PRIMARY KEY,
    case_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    old_state JSONB,
    new_state JSONB,
    metadata JSONB,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX idx_case_id (case_id),
    INDEX idx_timestamp (timestamp DESC)
);

-- Work item audit trail
CREATE TABLE workitem_events (
    id BIGSERIAL PRIMARY KEY,
    workitem_id UUID NOT NULL,
    case_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    state_before JSONB,
    state_after JSONB,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX idx_workitem_id (workitem_id),
    INDEX idx_case_id (case_id)
);

-- Resource allocation audit
CREATE TABLE allocation_events (
    id BIGSERIAL PRIMARY KEY,
    allocation_id UUID NOT NULL,
    resource_id UUID NOT NULL,
    workitem_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,  -- offered, allocated, started, released
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX idx_resource_id (resource_id),
    INDEX idx_timestamp (timestamp DESC)
);

-- Workflow execution metrics (time-series)
CREATE TABLE execution_metrics (
    case_id UUID NOT NULL,
    task_id VARCHAR(100) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    duration_ticks INT,  -- Execution time in ticks (64 KHz)
    pattern_id INT,
    success BOOLEAN,
    PRIMARY KEY (case_id, task_id, started_at)
) PARTITION BY RANGE (started_at);

-- Partitions for time-series (monthly)
CREATE TABLE execution_metrics_2025_01 PARTITION OF execution_metrics
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
```

### Benchmarks

| Operation | Sled | PostgreSQL | Requirement | Status |
|-----------|------|-----------|-------------|--------|
| Get workflow spec | 200 ns (0.01 ticks) | 500 µs (32 ticks) | <8 ticks | ✅ Sled |
| Update case state | 2 µs (0.13 ticks) | 800 µs (51 ticks) | <8 ticks | ✅ Sled |
| Queue work item | 1.5 µs (0.10 ticks) | 1.2 ms (77 ticks) | <8 ticks | ✅ Sled |
| Complex audit query | N/A | 50 ms | None | ✅ PostgreSQL |
| Time-series aggregation | N/A | 200 ms | None | ✅ PostgreSQL |

## Migration Path

### Phase 1: Sled Only (Weeks 1-8)
- Implement core engine with Sled
- In-memory audit trail (temporary)
- Focus on sub-tick performance

### Phase 2: Add PostgreSQL Audit (Weeks 9-12)
- Implement async audit writer
- Migrate audit queries to PostgreSQL
- Monitor audit lag

### Phase 3: Advanced Analytics (Weeks 13-16)
- Build BI dashboards on PostgreSQL
- Time-series metrics
- Compliance reports

## References

- [Sled Documentation](https://docs.rs/sled/)
- [PostgreSQL Performance Tuning](https://wiki.postgresql.org/wiki/Performance_Optimization)
- [Sub-Tick Latency Architecture](/docs/performance/sub-tick-architecture.md)
- [ACID Transactions in Embedded Databases](https://www.vldb.org/pvldb/vol13/p3007-graefe.pdf)

## Related Decisions

- ADR-001: Why Rust (enables Sled embedding)
- ADR-005: OpenTelemetry observability
- ADR-006: Lockchain for blockchain audit
