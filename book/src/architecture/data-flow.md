# Data Flow

## Basic Query Flow

1. **RDF Loading**: Parse RDF/Turtle files → SoA arrays
2. **Predicate Run Detection**: Group triples by predicate (len ≤ 8)
3. **Query Compilation**: SPARQL → Hook IR
4. **Path Selection**: Hot path vs cold path routing
5. **Evaluation**: Branchless SIMD execution
6. **Result Return**: Boolean or count result

## Enterprise Pipeline Flow

1. **Connect**: Register connectors (Kafka, Salesforce, etc.)
2. **Ingest**: Poll connectors → Raw triples
3. **Transform**: Validate against Σ schema → Typed triples (IRI → u64)
4. **Load**: Group by predicate → SoA arrays (64-byte aligned)
5. **Reflex**: Execute hooks (μ) → Actions (A) + Receipts
6. **Emit**: Write receipts to lockchain → Send actions to downstream APIs
7. **Provenance**: hash(A) = hash(μ(O)) verified via receipts

## Hot Path Execution

### Requirements
- Predicate run size ≤8 elements (guard constraint enforced)
- Simple operations (ASK, COUNT, triple match)
- Data hot in L1 cache
- Single predicate queries
- Branchless operations (constant-time execution)
- ≤8 ticks (Chatman Constant: 2ns = 8 ticks)

### Execution Steps
1. Pin run (validate len ≤ 8)
2. Create IR (operation, S, P, O, k)
3. Execute via SIMD (branchless)
4. Generate receipt (ticks, lanes, span_id, a_hash)
5. Return result

## Cold Path Fallback

Queries that exceed hot path constraints automatically fall back to full SPARQL engine execution.

## Receipt Flow

1. Hook execution generates receipt
2. Receipt merged (⊕) with previous receipts
3. Receipt written to lockchain (Merkle-linked)
4. Receipt verified: hash(A) = hash(μ(O))

## See Also

- [Three-Tier Architecture](three-tier.md) - Architecture overview
- [Key Components](components.md) - Component descriptions

