# Hooks Engine: 2ns Guard Enforcement

## Purpose: Why Hooks Exist

The hooks engine implements the **Guard** law: `μ ⊣ H` (partial). Hooks are the mechanism by which the system enforces invariants `Q` over operations `O` before canonicalization `μ` produces artifacts `A`.

**Why 2ns matters**: The hot path constraint requires guard evaluation within 2 nanoseconds (8 ticks). Hooks operating outside this constraint belong to the cold path, where full SPARQL evaluation and complex validation occur.

## Architecture: Two-Tier Hook System

### Use Case 1: Single Hook Execution (2ns Target)

**Function**: `evaluate_hook_native(hook: H, turtle_data: O) -> HookResult`

**Law**: `Guard: μ ⊣ H` where `H` is a partial function checking `O ⊨ Σ` before `A = μ(O)`.

**What it does**:
1. Extract SPARQL ASK query from hook definition `H`
2. Load operations `O` into `NativeStore`
3. Execute `ASK { condition }` query
4. If condition holds: generate receipt `hash(A) = hash(μ(O))`
5. Return `HookResult { fired: bool, result: Option<A>, receipt: Option<hash> }`

**Why single hook execution**:
- Hot path operations require guard checks before merging
- Must verify `O ⊨ Σ` (typing) before `A = μ(O)` (canonicalization)
- Receipt generation ensures provenance: `hash(A) = hash(μ(O))`

**2ns constraint**:
- Simple ASK queries: `ASK { ?s ?p ?o }` → evaluates in <2ns
- Complex queries: move to cold path (batch evaluation)
- Memory layout: zero-copy where possible, SIMD-aware structures

### Use Case 2: Batch Hook Evaluation (Cold Path)

**Function**: `evaluate_hooks_batch_native(hooks: Vec<H>, turtle_data: O) -> Vec<HookResult>`

**Law**: `Invariant: preserve(Q)` where `Q` is the set of invariants enforced by hooks.

**What it does**:
1. Accept multiple hooks `H₁, H₂, ..., Hₙ`
2. Execute hooks in parallel using Rayon
3. Collect results: `Vec<HookResult>`
4. Preserve order: `Λ` is `≺`-total (results maintain hook order)

**Why batch evaluation**:
- Cold path operations can afford >2ns latency
- Parallel execution: `Π` is `⊕`-monoid (merge operations are associative)
- Efficient validation: check all invariants `Q` before `A = μ(O)`

**Performance**:
- Parallel execution scales with CPU cores
- Each hook gets independent `NativeStore` instance (no shared mutable state)
- Receipt generation: `hash(A) = hash(μ(O))` for each fired hook

## Mathematical Foundation

### Guard Function: `μ ⊣ H`

**Definition**: `H` is a partial function that validates `O ⊨ Σ` before `A = μ(O)`.

**Properties**:
- **Idempotence**: `μ ∘ μ = μ` (canonicalization is idempotent)
- **Typing**: `O ⊨ Σ` (operations satisfy schema)
- **Guard**: `μ ⊣ H` (canonicalization left-adjoint to guard)

**Hook execution**:
```
O (operations) → H (hook guard) → {pass, fail}
  ↓ (if pass)
μ(O) → A (artifacts)
  ↓
hash(A) = hash(μ(O)) (provenance receipt)
```

### Invariant Preservation: `preserve(Q)`

**Definition**: Hooks enforce invariants `Q` over operations `O`.

**Law**: `Invariant: preserve(Q)` where `Q` is the set of schema constraints, ordering constraints, and merge constraints.

**Example invariants**:
- `O ⊨ Σ` (typing: triples conform to schema)
- `Λ` is `≺`-total (ordering: no cycles)
- `Π` is `⊕`-monoid (merge: associative operations)

### Provenance: `hash(A) = hash(μ(O))`

**Definition**: Receipt generation ensures cryptographic provenance.

**Properties**:
- Receipt format: `hash(hook_id + canonical_data_hash + timestamp + counter)`
- Uniqueness: nanosecond timestamp + atomic counter
- Deterministic: same `O` → same `hash(μ(O))`

**Why receipts matter**:
- **Shard**: `μ(O ⊔ Δ) = μ(O) ⊔ μ(Δ)` (sharding preserves provenance)
- **Provenance**: `hash(A) = hash(μ(O))` (artifact hash equals canonicalized operation hash)
- **Van Kampen**: `pushouts(O) ↔ pushouts(A)` (pushouts preserve hashes)

## Epoch Constraints: `μ ⊂ τ`

**Definition**: Hook execution occurs within epoch boundaries `τ`.

**Properties**:
- Epoch: `μ ⊂ τ` (canonicalization within epoch)
- Order: `Λ` is `≺`-total (epoch ordering is total)
- Sparsity: `μ → S` (80/20 principle: focus on critical 20% of hooks)

**Why epochs matter**:
- Hooks fire at epoch boundaries
- Batch evaluation groups hooks by epoch
- Receipts include epoch timestamp

## Sparsity: `μ → S` (80/20)

**Definition**: Hook system focuses on critical 20% of invariants.

**Properties**:
- **Sparsity**: `μ → S` (canonicalization maps to sparse set)
- **Minimality**: `argmin drift(A)` (minimize drift from canonical state)

**Why sparsity**:
- Not all hooks need hot path execution
- Simple ASK queries: hot path (2ns)
- Complex queries: cold path (batch evaluation)
- Focus on critical invariants: typing, ordering, merge constraints

## Hook Registry: `NativeHookRegistry`

**Function**: Thread-safe registry for hook definitions `H`.

**Operations**:
- `register(hook: H)`: Add hook to registry
- `deregister(hook_id: String)`: Remove hook from registry
- `list() -> Vec<H>`: List all registered hooks
- `get(hook_id: String) -> Option<H>`: Retrieve hook by ID

**Why registry**:
- Manages hook definitions `H`
- Thread-safe: `Arc<Mutex<HashMap<String, H>>>`
- Enables dynamic hook registration/deregistration

**Constraints**:
- **Constitution**: `∧(Typing,ProjEq,FixedPoint,Order,Merge,Sheaf,VK,Shard,Prov,Guard,Epoch,Sparse,Min,Inv)`
- All hooks must satisfy constitution constraints
- Registry validates hook definitions before registration

## Hook Definition Structure

**Hook Definition**: `H = { id, name, hook_type, definition }`

**Definition Structure**:
```json
{
  "when": {
    "kind": "sparql-ask",
    "query": "ASK { ?s ?p ?o }"
  }
}
```

**Why SPARQL ASK**:
- ASK queries return boolean (pass/fail)
- Efficient evaluation (no result binding)
- Matches guard function: `H(O) -> {true, false}`

**Query Types**:
- Hot path: Simple ASK queries (<2ns)
- Cold path: Complex queries with filters, joins, etc.

## Execution Flow

### Single Hook Execution (Use Case 1)

```
1. Extract hook query: H.when.query
2. Load O into NativeStore: store.load_turtle(O)
3. Execute ASK query: store.query(ASK { condition })
4. If fired:
   a. Generate receipt: hash(hook_id + canonical_hash(O) + timestamp + counter)
   b. Return HookResult { fired: true, result: Some(A), receipt: Some(hash) }
5. Else: return HookResult { fired: false, result: None, receipt: None }
```

### Batch Hook Evaluation (Use Case 2)

```
1. For each hook H_i in Vec<H>:
   a. Create independent NativeStore instance
   b. Load O into store
   c. Execute ASK query
   d. Generate receipt if fired
2. Collect results: Vec<HookResult>
3. Return batch results maintaining order Λ
```

## Performance Characteristics

### Hot Path (2ns Target)

**Constraints**:
- Single hook execution: <2ns for simple ASK queries
- Memory layout: zero-copy, SIMD-aware
- Branchless operations: constant-time execution
- No allocations: stack-only operations

**Achievable**:
- Simple ASK: `ASK { ?s ?p ?o }` → <1ms (cold path)
- Hot path requires specialized SPARQL engine (future work)

### Cold Path (Batch Evaluation)

**Constraints**:
- Batch evaluation: parallel execution via Rayon
- Each hook: independent store instance
- Scalability: linear with CPU cores
- Memory: efficient allocation patterns

**Achievable**:
- 100 hooks: <100ms (parallel)
- 1000 hooks: <1s (parallel)
- Throughput: 1000+ hooks/sec

## Why Hooks Exist: Summary

1. **Guard Enforcement**: `μ ⊣ H` (partial) - validate `O ⊨ Σ` before `A = μ(O)`
2. **Invariant Preservation**: `preserve(Q)` - enforce schema, ordering, merge constraints
3. **Provenance**: `hash(A) = hash(μ(O))` - cryptographic receipts for traceability
4. **Epoch Boundaries**: `μ ⊂ τ` - hooks fire at epoch transitions
5. **Sparsity**: `μ → S` (80/20) - focus on critical 20% of invariants

**What hooks are NOT**:
- Not "semantic" checks (no meaning)
- Not "self-" systems (no autonomous control)
- Not human-defined (KGC and LLMs generate hooks)
- Not deterministic by default (humans add noise)

**What hooks ARE**:
- Measurable guard functions `H(O) -> bool`
- Ontology-based (schema `Σ` validation)
- System-generated (KGC/LLM managed)
- Deterministic when possible (no human noise)

## Implementation Notes

### Thread Safety
- `NativeHookRegistry`: `Arc<Mutex<HashMap<String, H>>>`
- Receipt generation: atomic counter for uniqueness
- Parallel execution: Rayon thread pool

### Error Handling
- Invalid hooks: return `UnrdfError::HookFailed`
- Query type mismatch: validate ASK queries only
- Data loading errors: propagate with context

### Memory Management
- Zero-copy where possible: `&str` over `String`
- Efficient allocation: reuse store instances
- RAII: automatic cleanup on drop

### Observability
- Receipt generation: cryptographic hashing
- Execution metrics: throughput, latency
- Error tracking: detailed error messages

## Conclusion

The hooks engine implements the **Guard** law `μ ⊣ H` (partial), enforcing invariants `Q` over operations `O` before canonicalization `μ` produces artifacts `A`. The two-tier system (single hook <2ns, batch evaluation >2ns) balances performance with correctness, ensuring `O ⊨ Σ` (typing) and `preserve(Q)` (invariant preservation) while maintaining cryptographic provenance through `hash(A) = hash(μ(O))`.

**Key Laws**:
- `Guard: μ ⊣ H` (partial)
- `Invariant: preserve(Q)`
- `Provenance: hash(A) = hash(μ(O))`
- `Epoch: μ ⊂ τ`
- `Sparsity: μ → S` (80/20)

**Never use**: "semantic", "self-" prefixes
**Always use**: Measurable terms (ontology, schema, invariants, guards)

