# Data Flow

**Formal Foundation**: Data flow implements formal laws ensuring:
- **Law**: A = μ(O) - Actions are deterministic projections of observations
- **Epoch Containment**: μ ⊂ τ - All operations terminate within time bounds
- **Provenance**: hash(A) = hash(μ(O)) - Cryptographic verification enabled

See [Formal Mathematical Foundations](formal-foundations.md) for complete treatment.

## Overview

KNHK processes RDF data through a pipeline from file loading to query execution. The pipeline implements formal laws that guarantee deterministic behavior, safe parallelism, and cryptographic verification.

## Data Flow Stages

### 1. RDF Loading
- **Input**: RDF/Turtle file
- **Parser**: RDF/Turtle parsing (rio_turtle or custom parser)
- **Output**: Stream of triples (subject, predicate, object)

### 2. Term Hashing
- **Input**: RDF terms (URIs, literals, blanks)
- **Algorithm**: FNV-1a hash
- **Output**: uint64_t term IDs

### 3. SoA Storage
- **Input**: Hashed triples
- **Layout**: Separate S[], P[], O[] arrays
- **Alignment**: 64-byte aligned for cache optimization

### 4. Predicate Run Detection
- **Input**: SoA arrays
- **Process**: Group triples by predicate
- **Output**: Predicate run metadata (pred, offset, length)

### 5. Query Compilation
- **Input**: SPARQL query or direct IR construction
- **Process**: Validate operation type and parameters
- **Output**: Hook IR structure

### 6. Path Selection
- **Input**: Hook IR + predicate run metadata
- **Decision**: Hot path (≤8 ticks) vs cold path
- **Criteria**: 
  - Operation complexity
  - Predicate run size (≤8 for hot path)
  - Data availability
- **Formal Property**: Path selection enforces **Epoch Containment** (μ ⊂ τ) - operations that cannot meet τ ≤ 8 ticks route to warm/cold path

### 7. Evaluation
- **Hot Path**: Branchless SIMD execution
- **Cold Path**: Full SPARQL engine fallback
- **Output**: Query result (boolean or count)
- **Formal Law**: Implements **A = μ(O)** - Actions are deterministic projections of observations
- **Formal Property**: **Idempotence** (μ∘μ = μ) ensures re-evaluation produces identical results

## Data Flow Diagram

See `data-flow.mmd` for visual representation.

## Key Data Structures

### Triple Storage (SoA)
```
S[0..NROWS-1]  Subject IDs
P[0..NROWS-1]  Predicate IDs
O[0..NROWS-1]  Object IDs
```

### Predicate Run
```
pred: Predicate ID
off:  Offset in arrays (0-based)
len:  Length (must be ≤8 for hot path)
```

### Context
```
ctx.S            → S array pointer
ctx.P            → P array pointer
ctx.O            → O array pointer
ctx.triple_count → Total triples loaded
ctx.run          → Current predicate run metadata
```

## Memory Layout

### Cache Optimization
- Arrays are 64-byte aligned (single cacheline)
- Predicate runs are contiguous in memory
- Hot path data fits in L1 cache (≤8 elements)

### Access Patterns
- Sequential access within predicate runs
- SIMD vector loads (4 elements at a time)
- No random memory access in hot path

## Query Execution Flow

1. **Load RDF**: `knhk_load_rdf()` → Populates SoA arrays
2. **Initialize Context**: `knhk_init_context()` → Sets up context
3. **Create Query IR**: Construct `knhk_hook_ir_t` structure
4. **Evaluate**: `knhk_eval_bool()` → Returns result
5. **Result**: Boolean (0 or 1) or count value

## Performance Optimizations

### Cache Locality
- SoA layout enables SIMD access
- Single cacheline loads for 8 elements
- Predicate runs are contiguous

### SIMD Vectorization
- Processes 4 elements per SIMD instruction
- Branchless operations eliminate pipeline stalls
- Fully unrolled loops (NROWS=8)

### Memory Access
- Sequential access pattern
- Prefetch-friendly layout
- No pointer chasing

