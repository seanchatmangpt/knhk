use std::time::{Duration, SystemTime, Instant};
use async_trait::async_trait;

#[async_trait]
pub trait Timebase: Send + Sync {
    fn now_wall(&self) -> SystemTime;   // civil time (calendars, SLAs)
    fn now_mono(&self) -> Instant;      // monotonic (timeouts)
    fn scale(&self) -> f64;             // time dilation (1.0 = real time)
    async fn sleep(&self, d: Duration); // completes when virtual time reaches now + d
    async fn sleep_until_wall(&self, t: SystemTime);
    async fn sleep_until_mono(&self, t: Instant);
}
Two concrete clocks:

// Real clock (production)
pub struct SysClock;
#[async_trait]
impl Timebase for SysClock {
    fn now_wall(&self) -> SystemTime { SystemTime::now() }
    fn now_mono(&self) -> Instant { Instant::now() }
    fn scale(&self) -> f64 { 1.0 }
    async fn sleep(&self, d: Duration) { tokio::time::sleep(d).await }
    async fn sleep_until_wall(&self, t: SystemTime) {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let tgt = t.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        tokio::time::sleep(tgt.saturating_sub(now)).await
    }
    async fn sleep_until_mono(&self, t: Instant) { tokio::time::sleep_until(t.into()).await }
}
// Simulated clock (tests + model runs)
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use parking_lot::Mutex;
use futures::future::BoxFuture;

#[derive(Clone)]
pub struct SimClock {
    // monotonic epoch
    mono: Arc<Mutex<Instant>>,
    // wall epoch
    wall: Arc<Mutex<SystemTime>>,
    // heap of (due_mono, id, waker)
    q: Arc<Mutex<BinaryHeap<(Reverse<Instant>, u64, BoxFuture<'static, ()>)>>>,
    scale: f64,
    next_id: Arc<Mutex<u64>>,
}
impl SimClock {
    pub fn new(start_wall: SystemTime, start_mono: Instant, scale: f64) -> Self { /* init */ }
    pub fn freeze(&self) { self.set_scale(0.0) }
    pub fn set_scale(&self, s: f64) { /* update */ }
    pub fn warp_mono(&self, delta: Duration) { /* advance mono+wall and run due tasks */ }
    pub fn set_wall(&self, t: SystemTime) { /* set wall and realign */ }
    pub fn jump_to_business_day(&self, day_yyyymmdd: &str) { /* helper in tests */ }
    fn run_due(&self) { /* pop all due futures and poll them */ }
}
#[async_trait]
impl Timebase for SimClock {
    fn now_wall(&self) -> SystemTime { *self.wall.lock() }
    fn now_mono(&self) -> Instant { *self.mono.lock() }
    fn scale(&self) -> f64 { self.scale }
    async fn sleep(&self, d: Duration) { self.sleep_until_mono(self.now_mono() + d).await }
    async fn sleep_until_wall(&self, t: SystemTime) {
        let now = self.now_wall().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let tgt = t.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        self.sleep(tgt.saturating_sub(now)).await
    }
    async fn sleep_until_mono(&self, t: Instant) {
        use tokio::sync::oneshot;
        let (tx, rx) = oneshot::channel::<()>();
        let mut fut: BoxFuture<'static, ()> = Box::pin(async move { let _ = tx.send(()); });
        let id = { let mut n = self.next_id.lock(); *n += 1; *n };
        self.q.lock().push((Reverse(t), id, fut));
        // block until SimClock.warp_mono() runs_due and sends the signal
        let _ = rx.await;
    }
}
2) Engine wiring
Make WorkflowEngine accept a Timebase:

pub struct WorkflowEngine<T: Timebase> {
    time: Arc<T>,
    // ...
}
impl<T: Timebase> WorkflowEngine<T> {
    pub fn new(state: StateStore, time: Arc<T>) -> Self { /* as before */ }
}
Timers in patterns map to time.sleep*. Examples:

P30 Transient Trigger: one-shot sleep_until_*.

P31 Persistent Trigger: loop with a persisted next due time.

P16 Deferred Choice: wait on event or timeout, whichever fires first.

Milestones, SLAs: compare now_wall() to thresholds.

3) Ten ways to manipulate time in tests
Freeze
Set scale=0. Time does not move unless you say so. Deterministic.

Warp (jump forward)
Advance by hours/days/months instantly. All due timers fire in order via the event heap.

Dilation (timescale)
Run month-long flows with scale=86_400.0 (1 test second = 1 day). Good for soak tests.

Tokio virtual time
Alternative: tokio::time::pause(); advance(Duration::from_days(30));. Keep it for unit tests that don’t need the custom heap.

Calendar rewrites
Replace wall time with business-calendar functions (skip weekends/holidays). Verify SLAs measured in business days vs calendar days.

Skew and drift injection
Provide two clocks to two subsystems with ±Δ offsets. Validate that monotonic time drives safety, wall time only for SLAs.

DST/Leap anomalies
Jump across DST forward/back, 23-hour and 25-hour days, and insert leap seconds. Validate persistence and interval arithmetic.

Crash + resume
Persist due_at for every timer. Kill engine mid-wait, warp past due_at, restart, ensure catch-up semantics.

Catch-up vs coalescing
For P31 intervals, choose policy: run missed n times (catch-up) or run once (coalesce). Test both.

Temporal fuzzing
Randomize jitter, reorder near-simultaneous expirations, and vary scale during execution. Property tests assert invariants still hold.

4) Month-long scenarios to cover all temporal patterns
Scenario	Patterns	What to assert
SLA escalation at 30 calendar days	P30, P40	Escalation fires exactly once at T+30d even across DST
Business-day approval timeout (10 business days)	P16, P18	Timer excludes weekends/holidays; milestone gates early firing
Rolling monthly billing (RRULE FREQ=MONTHLY)	P31	Next execution correct for month lengths; Feb edge cases
Partial join with tardy branch	P33/P34	Firing at ≥k arrivals after warps; no double fire
Cancelling discriminator on first completion	P27	First arrival wins; later arrivals are cancelled after warp
MI with runtime count and deadline	P15 + P24	Force complete at deadline after big jump
Region cancel by timer	P21	Cancel all nodes in scope upon timeout, even after restart
Recurring watchdog	P31 + crash/resume	Persistent trigger continues schedule after cold restart
5) Test harness utilities
5.1 Builders
pub struct TemporalTest {
    pub clock: Arc<SimClock>,
    pub engine: WorkflowEngine<SimClock>,
}
impl TemporalTest {
    pub fn new_start(date: &str) -> Self {
        let wall = humantime::parse_rfc3339(format!("{date}T00:00:00Z")).unwrap();
        let mono = Instant::now();
        let clock = Arc::new(SimClock::new(wall, mono, 0.0));
        let engine = WorkflowEngine::new(StateStore::new(tempdir().path()).unwrap(), clock.clone());
        Self { clock, engine }
    }
    pub fn warp_days(&self, d: i64) { self.clock.warp_mono(Duration::from_secs((d as u64)*86_400)) }
    pub fn set_scale(&self, s: f64) { self.clock.set_scale(s) }
}
5.2 Business calendar helper
pub trait BusinessCalendar {
    fn is_holiday(&self, day: chrono::NaiveDate) -> bool;
    fn add_business_days(&self, start: chrono::NaiveDate, n: u32) -> chrono::NaiveDate;
}
Use in patterns that measure “business days” so tests can swap calendars.

6) Concrete test examples
6.1 30-day SLA escalation (transient trigger)
#[tokio::test]
async fn sla_escalates_after_30_days() {
    let tt = TemporalTest::new_start("2025-01-01");
    // schedule a one-shot
    let due = tt.clock.now_wall() + std::time::Duration::from_secs(30*86_400);
    // engine pattern P30 uses time.sleep_until_wall(due)
    let fired = Arc::new(AtomicBool::new(false));
    // simulate: trigger would schedule next node; we flip a flag here
    let f = fired.clone();
    tokio::spawn(async move {
        // pretend pattern executed and awaited
        // ... time.sleep_until_wall(due).await
        f.store(true, Ordering::Relaxed);
    });

    // advance 29 days
    tt.warp_days(29);
    assert_eq!(fired.load(Ordering::Relaxed), false);

    // cross DST? warp one more day
    tt.warp_days(1);
    assert_eq!(fired.load(Ordering::Relaxed), true);
}
6.2 Persistent trigger with catch-up
#[tokio::test]
async fn persistent_trigger_catch_up() {
    let tt = TemporalTest::new_start("2025-03-01");
    // Every day at 00:00Z for 31 days
    let mut count = 0u32;
    let c = Arc::new(Mutex::new(0u32));
    let cc = c.clone();
    tokio::spawn(async move {
        for _ in 0..31 {
            // P31: sleep until next day
            // time.sleep(Duration::from_secs(86_400)).await;
            *cc.lock() += 1;
        }
    });

    // jump 15 days at once
    tt.warp_days(15);
    // catch-up policy: run all 15 immediately in order
    assert!(*c.lock() >= 15);

    // jump remaining 16
    tt.warp_days(16);
    assert_eq!(*c.lock(), 31);
}
6.3 Deferred choice with timeout
#[tokio::test]
async fn deferred_choice_timeout_vs_event() {
    let tt = TemporalTest::new_start("2025-05-01");
    // event not delivered, timeout = 3 business days
    tt.warp_days(1); // Fri
    tt.warp_days(2); // skip weekend via BusinessCalendar in pattern
    tt.warp_days(1); // Mon -> fire timeout branch
    // assert: engine scheduled timeout path
}
6.4 Crash/resume past due
#[tokio::test]
async fn resume_runs_overdue_timer() {
    let tt1 = TemporalTest::new_start("2025-06-01");
    // persist timer with due_at=2025-06-15T00:00Z
    // simulate crash before due
    let persisted = export_sled(tt1.engine); // helper

    // restart
    let tt2 = TemporalTest::new_start("2025-06-01");
    import_sled(&tt2.engine, persisted);

    // warp to 2025-06-20
    tt2.warp_days(19);
    // engine should detect overdue and fire immediately (coalesced once)
}
7) Temporal fault injection matrix
Cover these in CI:

Skew: ±5 min between subsystems; verify joins use monotonic time only.

Drift: scale drifting from 1.0→0.99 over “month”; ensure long sleeps don’t accumulate error.

DST forward/back: jump over spring forward and fall back; ensure interval timers use monotonic.

Leap second: duplicate 23:59:60 or skip; ensure wall math tolerates.

Backfill: add missed triggers after long warp; test both catch-up and coalesce modes.

Cancellation race: cancel a region at T while branches complete at T+ε; assert exactly-once downstream.

Heavy timer load: 1e6 scheduled items; heap and wheel stay O(log n)/O(1).

8) Coverage of YAWL temporal behaviors
P16 Deferred Choice: event vs timeout race via select! on event channel and sleep_until_*.

P18 Milestone: gate predicates reference now_wall() with calendar helper.

P30/P31 Triggers: one-shot and recurring; persistent schedule across restarts from due_at persisted.

P21 Region Cancel by Time: schedule cancel message to scope at due.

P28 Structured Loop: loop test uses wall or monotonic depending on business rule.

P33/P34 Partial Joins: time only matters for ordering; warps must not cause double-fire.

9) Model checking + property tests
Use proptest to generate calendars, intervals, DST boundaries:

proptest! {
  #[test]
  fn rrule_monthly_never_skips_end_of_month(start in any::<chrono::NaiveDate>()) {
      // generate 12 monthly occurrences; ensure strictly increasing and valid dates
  }
}
Invariants to assert:

No timer fires twice for same timer_id.

Recurring timers produce a non-decreasing sequence of due_at.

Cancelling discriminator cancels all siblings even after warp.

After crash/resume, every overdue due_at either fires (catch-up) or coalesces exactly once.

10) Practical recipes
Month in a second: SimClock::new(t0, mono0, 2_592_000.0) then run your e2e.

Exact civil schedule: drive with RRULE (iCal) and test across variable month lengths.

SLA vs timeout: compute SLA in wall time, enforce timeout in monotonic time.

Business calendar: inject calendar to convert “N business days” to absolute due_at before scheduling.

Outcome: with a virtual clock, event heap, and the ten manipulations above, you can simulate and verify month-long (and year-long) workflows in seconds, while exercising every temporal edge case across all 43 patterns without relying on real time.

You said:
OK, so I know we’re using led for data but a critical element is RDF so for instance, FIBO as workflow source
ChatGPT said:

Thought for 35s
Yes. Treat RDF as the source of truth (O) and compile it to hot executable artifacts (A). FIBO supplies Σ (typing) and YAWL supplies control-flow patterns. The compiler μ maps RDF → workflow IR once, offline, so A = μ(O). Execution never reads RDF at runtime.

Target picture
O: RDF graphs (FIBO + YAWL-in-RDF + policy).

Σ: FIBO classes, roles, instruments, events; YAWL task/condition/flow vocabulary; OWL-Time/ICAL for timers.

μ: RDF→IR compiler with SHACL gates and SPARQL extractors.

A: compact IR: pattern IDs, bitsets, timers, role/capability IDs, receipts; 64-byte crystals for hot path.

Invariants: μ∘μ = μ, O ⊨ Σ, hash(A) = hash(μ(O)), shard law μ(O ⊔ Δ) = μ(O) ⊔ μ(Δ).

Data contracts
1) RDF vocabularies (authoritative O)
Domain: FIBO (e.g., fibo-fbc, fibo-banking), PROV-O for receipts, ORG/SKOS for roles.

Control flow: YAWL-in-RDF (Task, Condition, Flow, SplitType, JoinType, PatternId).

Time: OWL-Time (time:Interval, time:hasDuration) and/or iCalendar (ical:RRULE) for P30/P31.

Allocation: Roles/capabilities via SKOS/ORG; policy IRIs for Four-Eyes, Round-Robin, etc.

2) IR (A) for hot/warm execution
Graph IR: adjacency bitsets per node; pattern table of 43 YAWL patterns; per-node flags; partial-join thresholds.

Timer IR: normalized schedule: {kind: OneShot|Recurring, due_at|rrule_norm, catch_up: bool}.

Resource IR: {role_ids[], capability_ids[], policy_id, priority, four_eyes: bool}.

Bindings: FIBO URIs → small integers via perfect hashing.

Receipts: PROV-O mapped to compact receipt records.

All IR payloads fit into cache-aligned slabs. Only IR is touched at runtime.

Compiler μ (RDF → IR)
Pipeline (warm path, once per spec or delta):

Ingress

Read Turtle/Trig via a streaming parser (RIO/Sophia/Oxigraph).

Snapshot named graphs into content-addressed blobs.

Compute graph hash for provenance.

Typing (Σ)

Enforce FIBO + YAWL typing with SHACL (compiled to bitset checks).

Fast-fail: no SHACL in hot path; gates run here only.

Extraction (SPARQL)

SPARQL SELECT/CONSTRUCT to extract Tasks, Conditions, Flows, Splits/Joins, MI parameters, timers, resources.

Materialize a normalized DAG.

Lowering

Map split/join pairs to PatternId 1..43.

Compute adjacency arrays and bitsets per node.

Resolve partial-join thresholds (P33/34), discriminator (P27), loops (P28), MI (P12–15).

Normalize timers to monotonic schedules with civil anchors.

Allocation plan

Map role/capability IRIs to u32 ids; encode policies.

Precompute four-eyes bindings and queues.

Sealing

Serialize IR; record hash(A) = hash(μ(O)).

Store IR in sled as spec: and index: keys; store raw RDF under rdf: keys.

Properties: μ∘μ = μ (idempotent), Shard holds for named-graph deltas, Q (invariants) preserved by SHACL before lowering.

Minimal RDF → IR examples
A) Turtle (O): FIBO-backed ATM mini-workflow
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix fibo: <https://spec.edmcouncil.org/fibo/> .
@prefix org:  <http://www.w3.org/ns/org#> .
@prefix role: <http://example.com/roles#> .
@prefix time: <http://www.w3.org/2006/time#> .
@prefix ical: <http://www.w3.org/2002/12/cal/ical#> .

<workflow:atm> a yawl:WorkflowSpec ;
  yawl:name "ATM Withdrawal" ;
  yawl:startCondition <c:start> ;
  yawl:endCondition   <c:end> ;
  yawl:task <t:auth>, <t:ledger>, <t:dispense> .

<c:start> a yawl:Condition .
<c:authd> a yawl:Condition .
<c:posted> a yawl:Condition .
<c:end>   a yawl:Condition .

<t:auth> a yawl:Task ;
  yawl:name "Authorize" ;
  yawl:splitType yawl:XOR ; yawl:joinType yawl:AND ;
  yawl:inputCondition  <c:start> ;
  yawl:outputCondition <c:authd> ;
  org:role role:IssuerACS .

<t:ledger> a yawl:Task ;
  yawl:name "Post Ledger" ;
  yawl:splitType yawl:AND ; yawl:joinType yawl:AND ;       # P2/P3
  yawl:inputCondition  <c:authd> ;
  yawl:outputCondition <c:posted> ;
  fibo:refersTo fibo:BankingAccount ;
  ical:RRULE "FREQ=DAILY;BYHOUR=0;BYMINUTE=0;BYSECOND=0" . # P31

<t:dispense> a yawl:Task ;
  yawl:name "Dispense Cash" ;
  yawl:splitType yawl:AND ; yawl:joinType yawl:XOR ;
  yawl:inputCondition  <c:posted> ;
  yawl:outputCondition <c:end> ;
  role:requires role:DeviceCustody .

# Cancel region if not posted within 10 minutes (P21 via OWL-Time)
<t:ledger-timeout> a yawl:Trigger ;
  time:hasDuration [ time:numericDuration "600"^^xsd:decimal ; time:unitType time:unitSecond ] ;
  yawl:cancelsRegion ( <t:ledger> <t:dispense> ) .
B) SHACL (Σ): sketch enforcing YAWL typing
@prefix sh: <http://www.w3.org/ns/shacl#> .
@yawlTaskShape a sh:NodeShape ;
  sh:targetClass yawl:Task ;
  sh:property [ sh:path yawl:splitType ; sh:in (yawl:AND yawl:XOR yawl:OR) ] ;
  sh:property [ sh:path yawl:joinType  ; sh:in (yawl:AND yawl:XOR yawl:OR) ] .
C) SPARQL extraction (μ: extract nodes/edges)
# Tasks
SELECT ?t ?split ?join WHERE {
  ?t a yawl:Task ; yawl:splitType ?split ; yawl:joinType ?join .
}

# Flows
SELECT ?from ?to WHERE {
  ?from yawl:outputCondition ?c .
  ?to   yawl:inputCondition  ?c .
}

# Timers (RRULE and OWL-Time)
CONSTRUCT {
  ?t  <urn:knhk:ir/timer> ?norm .
} WHERE {
  { ?t ical:RRULE ?r . BIND(STR(?r) AS ?norm) }
  UNION
  { ?t time:hasDuration ?dur . BIND(CONCAT("DUR:", STR(?dur)) AS ?norm) }
}
The compiler lowers split/join to PatternIds; converts flows to dense indices and bitsets; normalizes timers.

Storage (sled)
rdf:<graph_hash> → raw Turtle/Trig (compressed).

spec:<graph_hash> → IR blob (A).

index:workflow:<spec_id> → graph_hash.

prov:receipt:<case_id>:<step> → compact PROV record.

This keeps cold storage simple. No RDF queries at runtime.

Engine wiring
The WorkflowEngine uses only IR.

Admission gate Θ runs SHACL + PB-congruence once on ingest.

Pattern registry fully covers 43 patterns.

Timers run from normalized IR schedules (see virtual time facility).

Resource allocator consumes role/capability ids created by μ.

Hot path remains pure: no RDF, no JSON, no parsing. Only pattern ids, bitsets, counters, and timers.

Pattern coverage map (YAWL → IR)
External-facing (legacy surface):

P1 Sequence, P2 Parallel Split, P3 Synchronization, P4 Exclusive Choice, P5 Simple Merge.

P6 Multi-Choice, P7/37 Sync merges, P8 Multi-Merge, P9/P26/P27 discriminators.

P10/P28 loops, P12–P15 MI variants, P16 Deferred Choice, P18 Milestone.

P19–P25 cancellations, P30 Transient Trigger, P31 Persistent Trigger, P33/P34 partial joins, P35/36/38/39 advanced joins/splits, P40–P43 terminations.

Internal lowering rules select the exact IR opcode and parameters per pattern. Warm path computes these once.

Rust scaffolding (compiler μ)