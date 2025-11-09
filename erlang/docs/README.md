# Erlang Cold Path Documentation

Erlang implementation for cold path operations.

## File Structure

```
erlang/knhk_rc/src/
├── knhk_rc_app.erl         # Application callback
├── knhk_rc_sup.erl         # Supervisor tree
├── knhk_sigma.erl          # Schema registry (Σ management)
├── knhk_q.erl              # Invariant registry (Q constraints, preserve(Q))
├── knhk_ingest.erl         # Delta ingestion (O ⊔ Δ)
├── knhk_lockchain.erl      # Lockchain (Merkle-linked receipts)
├── knhk_hooks.erl          # Hook management
├── knhk_epoch.erl          # Epoch scheduling (Λ ≺-total, τ ≤ 8)
├── knhk_route.erl          # Action routing to downstream systems
├── knhk_connect.erl        # Connector management
├── knhk_cover.erl          # Coverage management
├── knhk_otel.erl           # OTEL integration
├── knhk_darkmatter.erl     # Dark matter connector framework
├── knhk_rc.erl             # Main API module
└── knhk_stubs.erl          # Stub implementations
```

## Core Components

### Main API (`knhk_rc.erl`)

Primary API module for reflexive control operations:

```erlang
%% Initialize Σ and Q
boot(#{sigma := Sigma, q := Q}) -> ok.

%% Register connector
connect(#{name := Name, schema := SigmaIri, source := Src, 
          map := Map, guard := Guard}) -> ConnId.

%% Define cover over O
cover(#{select := SelectSpec, shard := ShardSpec}) -> CoverId.

%% Admit delta into O (O ⊔ Δ)
admit(Delta) -> ok.

%% Declare reflex (hot path operation)
reflex(#{name := Name, op := Op, run := #{pred := P, off := Off, len := Len},
         args := Args, epoch := EpochTag}) -> HookId.

%% Plan epoch (Λ ≺-total, τ ≤ 8)
epoch(#{tau := Tau, lambda := Plan, cover := CoverId}) -> EpochId.

%% Execute epoch (A = μ(O))
run(EpochId) -> {Actions, Receipt}.

%% Route actions to outputs
route(#{name := Name, kind := Kind, target := Target, encode := Codec}) -> RouteId.

%% Get receipt
receipt(Id) -> Receipt.

%% Merge receipts (Π ⊕)
merge(Receipts) -> MergedReceipt.

%% Get metrics
metrics() -> MetricsMap.

%% Get coverage
coverage() -> CoverageMap.
```

### Schema Registry (`knhk_sigma.erl`)

Manages schema (Σ) registry and validation:

**API**:
```erlang
%% Start schema registry
start_link() -> {ok, Pid}.

%% Load schema from RDF file or binary
load(Schema) -> ok | {error, Reason}.

%% Query schema by IRI
query(SchemaIri) -> {ok, Schema} | {error, not_found}.

%% Validate data against schema (O ⊨ Σ)
validate(SchemaIri, Data) -> {ok, valid} | {error, invalid}.

%% List all loaded schemas
list() -> [SchemaIri].

%% Get schema version
get_version(SchemaIri) -> {ok, Version} | {error, not_found}.
```

**Features**:
- Schema loading and versioning
- Schema validation (O ⊨ Σ)
- Validation result caching
- Schema querying by IRI

### Invariant Registry (`knhk_q.erl`)

Manages invariants (Q) and preservation checking:

**API**:
```erlang
%% Start invariant registry
start_link() -> {ok, Pid}.

%% Load invariant from SPARQL query
load(Invariant) -> ok | {error, Reason}.

%% Check invariants (preserve(Q))
check(Data) -> {ok, preserved} | {error, {violations, Violations}}.

%% List all loaded invariants
list() -> {ok, [Invariant]}.

%% Get recent violations
get_violations() -> {ok, [Violation]}.
```

**Features**:
- Invariant loading from SPARQL queries
- Invariant preservation checking (preserve(Q))
- Violation tracking (last 100 violations)
- Check count statistics

### Delta Ingestion (`knhk_ingest.erl`)

Handles delta (Δ) admission and observation merging:

**API**:
```erlang
%% Submit delta for ingestion (O ⊔ Δ)
submit(Delta) -> ok | {error, Reason}.

%% Get current observation state
get_observation() -> Observation.

%% Merge deltas
merge_deltas(Deltas) -> MergedDelta.
```

**Features**:
- Delta operations (O ⊔ Δ)
- Observation merging
- State updates
- Type validation

### Lockchain (`knhk_lockchain.erl`)

Merkle-linked receipt storage:

**API**:
```erlang
%% Read receipt by ID
read(ReceiptId) -> {ok, Receipt} | {error, not_found}.

%% Merge receipts (Π ⊕)
merge(Receipts) -> MergedReceipt.

%% Verify receipt integrity
verify(ReceiptId) -> {ok, valid} | {error, invalid}.

%% Get merkle root
merkle_root() -> {ok, RootHash}.
```

**Features**:
- Receipt storage with Merkle linking
- Provenance tracking (hash(A) = hash(μ(O)))
- Receipt merging (associative, commutative)
- Integrity verification

### Hook Management (`knhk_hooks.erl`)

Manages knowledge hooks:

**API**:
```erlang
%% Install hook
install(Name, Op, Pred, Off, Len, Args, EpochTag) -> HookId.

%% Execute hook
execute(HookId, Data) -> HookResult.

%% List hooks
list() -> [HookId].

%% Remove hook
remove(HookId) -> ok.
```

**Features**:
- Hook installation and management
- Hook execution
- Hook registry
- Epoch tagging

### Epoch Scheduling (`knhk_epoch.erl`)

Manages epoch execution with ordering and tick budget:

**API**:
```erlang
%% Schedule epoch (Λ ≺-total, τ ≤ 8)
schedule(Tau, Plan, CoverId) -> EpochId.

%% Run epoch (A = μ(O))
run(EpochId) -> {Actions, Receipt}.

%% Get epoch status
status(EpochId) -> {ok, Status} | {error, not_found}.
```

**Features**:
- Epoch creation and management
- ≺-total ordering (Λ)
- Tick budget enforcement (τ ≤ 8)
- Deterministic execution

## Key Features

- **Schema Registry**: Σ management with validation (O ⊨ Σ)
- **Invariant Registry**: Q constraints with preservation checking (preserve(Q))
- **Delta Ingestion**: O ⊔ Δ operations with type validation
- **Lockchain**: Receipt storage with Merkle linking and provenance
- **Epoch Scheduling**: ≺-total ordering (Λ) with tick budget (τ ≤ 8)
- **Hook Management**: Knowledge hook installation and execution
- **Action Routing**: Route actions (A) to downstream systems
- **OTEL Integration**: Metrics and tracing support

## Supervision Tree

```
knhk_rc_sup (supervisor)
├── knhk_sigma (gen_server) - Schema registry
├── knhk_q (gen_server) - Invariant registry
├── knhk_ingest (gen_server) - Delta ingestion
├── knhk_lockchain (gen_server) - Receipt storage
├── knhk_hooks (gen_server) - Hook management
├── knhk_epoch (gen_server) - Epoch scheduling
├── knhk_route (gen_server) - Action routing
├── knhk_connect (gen_server) - Connector management
├── knhk_cover (gen_server) - Coverage management
└── knhk_otel (gen_server) - OTEL integration
```

## Usage Examples

### Initialize System

```erlang
%% Load schema and invariants
knhk_rc:boot(#{
    sigma => <<"urn:knhk:schema:enterprise">>,
    q => <<"ASK WHERE { ?s a <http://example.org/Employee> }">>
}).
```

### Register Connector

```erlang
%% Register Kafka connector
ConnId = knhk_rc:connect(#{
    name => <<"kafka-prod">>,
    schema => <<"urn:knhk:schema:kafka">>,
    source => <<"kafka://localhost:9092">>,
    map => #{subject => "$.id", predicate => "$.type", object => "$.data"},
    guard => #{max_run_len => 8, max_batch_size => 1000}
}).
```

### Admit Delta

```erlang
%% Admit delta into observation
Delta = #{additions => [...], removals => [...]},
knhk_rc:admit(Delta).
```

### Declare Reflex

```erlang
%% Install hot path hook
HookId = knhk_rc:reflex(#{
    name => <<"check-permission">>,
    op => <<"ASK_SP">>,
    run => #{pred => 16#C0FFEE, off => 0, len => 8},
    args => #{},
    epoch => <<"epoch-1">>
}).
```

### Execute Epoch

```erlang
%% Schedule and run epoch
EpochId = knhk_rc:epoch(#{
    tau => 8,
    lambda => [<<"hook1">>, <<"hook2">>],
    cover => <<"cover-1">>
}),

{Actions, Receipt} = knhk_rc:run(EpochId).
```

## Related Documentation

- [Architecture Guide](../../docs/ARCHITECTURE.md) - 🆕 Consolidated 80/20 guide (System architecture)
- [Architecture Reference](../../docs/architecture.md) - Detailed architecture reference
- [Integration](../../docs/integration.md) - Integration guide

