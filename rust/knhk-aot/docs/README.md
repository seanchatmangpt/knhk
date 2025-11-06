# knhk-aot Documentation

Ahead-of-Time (AOT) compilation guard and optimization framework for KNHK hot path operations.

## Overview

The `knhk-aot` crate provides AOT compilation optimizations and IR validation to ensure hot path operations meet the Chatman Constant constraint (≤8 ticks / ≤2ns). It includes template analysis, prebinding, MPHF generation, and specialization optimizations.

## Quick Start

```rust
use knhk_aot::{AotGuard, ValidationResult};

// Validate hook IR before execution
match AotGuard::validate_ir(op, run_len, k) {
    Ok(()) => {
        // IR is valid, safe to execute
        println!("IR validated successfully");
    }
    Err(result) => {
        eprintln!("Validation failed: {}", AotGuard::error_message(&result));
    }
}
```

## Core Components

### AotGuard

Main guard for validating hook IR before execution:

```rust
pub struct AotGuard;

impl AotGuard {
    /// Validate hook IR before execution
    pub fn validate_ir(op: u32, run_len: u64, k: u64) -> Result<(), ValidationResult>;
    
    /// Get validation error message
    pub fn error_message(result: &ValidationResult) -> String;
}
```

**Validation Rules:**
- Run length must be ≤ 8 (Chatman Constant)
- Operation must be in hot path set (H_hot)
- Operation-specific constraints (e.g., UNIQUE requires run_len ≤ 1)
- COUNT operations check k threshold

**Supported Operations:**
- ASK operations (1, 3, 7) - Always valid if run_len ≤ 8
- COUNT operations (2, 5, 6, 9, 10, 11) - Check k ≤ run_len
- UNIQUE (8) - Requires run_len ≤ 1
- COMPARE operations (12-16) - Always valid
- CONSTRUCT8 (32) - Requires run_len ≤ 8

### Template Analyzer (`template_analyzer`)

Analyzes CONSTRUCT templates to extract constant vs variable patterns:

```rust
use knhk_aot::template_analyzer::{TemplateAnalyzer, TemplateAnalysis};

let analyzer = TemplateAnalyzer::new();
let analysis = analyzer.analyze_template(template)?;

// Ground triples (emitted once)
for (s, p, o) in &analysis.ground_triples {
    // Emit constant triple
}

// Variable triples (emitted per solution)
for pattern in &analysis.variable_triples {
    // Emit variable triple
}
```

**Features:**
- Separates ground triples (constants) from variable triples
- Maps WHERE clause variables to template positions
- Optimizes CONSTRUCT8 template execution

### Prebinding (`prebinding`)

Prebinds constants to IR for AOT optimization:

```rust
use knhk_aot::prebinding::PreboundIr;

let ir = PreboundIr::from_template(&template, run_len);

// Check if constants are prebound
if let Some(s_const) = ir.s_const {
    // Subject is constant, use prebound value
} else {
    // Subject is variable, bind at runtime
}
```

**Features:**
- Precomputes constant values from templates
- Generates length mask hints for SIMD operations
- Zero-position bitmask hints for branchless execution

### MPHF (`mphf`)

Minimal Perfect Hash Function generation for predicate lookups:

```rust
use knhk_aot::mphf::{Mphf, MphfCache};

// Create MPHF from predicate list
let predicates = vec![1, 2, 3, 4, 5];
let mphf = Mphf::new(predicates);

// Lookup predicate index
if let Some(idx) = mphf.lookup(predicate) {
    // Use index for fast access
}

// Cache MPHF for reuse
let mut cache = MphfCache::new();
cache.insert("schema1", mphf);
```

**Features:**
- O(1) predicate lookup
- Perfect hash (no collisions)
- Cache support for schema-specific MPHFs

### Specialization (`specialization`)

Operation specialization for hot path optimization:

```rust
use knhk_aot::specialization::SpecializedOp;

// Create specialized operation
let specialized = SpecializedOp::specialize(op, template, run_len)?;

// Execute specialized operation (faster than generic)
let result = specialized.execute(context)?;
```

**Features:**
- Operation-specific optimizations
- Template-aware specialization
- Reduced branching in hot path

## Usage Examples

### IR Validation

```rust
use knhk_aot::{AotGuard, ValidationResult};

fn validate_hook(op: u32, run_len: u64, k: u64) -> Result<(), String> {
    match AotGuard::validate_ir(op, run_len, k) {
        Ok(()) => Ok(()),
        Err(result) => {
            let msg = AotGuard::error_message(&result);
            Err(format!("IR validation failed: {}", msg))
        }
    }
}

// Validate ASK_SP operation
validate_hook(1, 8, 0)?; // OK

// Validate COUNT_SP_GE operation
validate_hook(2, 8, 5)?; // OK (k ≤ run_len)

// Invalid: run_len > 8
validate_hook(1, 9, 0)?; // Error: InvalidRunLength

// Invalid: UNIQUE with run_len > 1
validate_hook(8, 2, 0)?; // Error: ExceedsTickBudget
```

### Template Analysis

```rust
use knhk_aot::template_analyzer::TemplateAnalyzer;
use knhk_aot::template::ConstructTemplate;

let analyzer = TemplateAnalyzer::new();
let template = ConstructTemplate::from_triples(triples)?;
let analysis = analyzer.analyze_template(&template)?;

// Emit ground triples once
for (s, p, o) in &analysis.ground_triples {
    emit_triple(*s, *p, *o);
}

// Emit variable triples per solution
for solution in solutions {
    for pattern in &analysis.variable_triples {
        let s = bind_subject(pattern, solution);
        let p = bind_predicate(pattern, solution);
        let o = bind_object(pattern, solution);
        emit_triple(s, p, o);
    }
}
```

### Prebinding Optimization

```rust
use knhk_aot::prebinding::PreboundIr;
use knhk_aot::template::ConstructTemplate;

let template = ConstructTemplate::from_triples(triples)?;
let ir = PreboundIr::from_template(&template, 8);

// Use prebound constants for branchless execution
let s = ir.s_const.unwrap_or(bind_s_from_solution());
let p = ir.p_const.unwrap_or(bind_p_from_solution());
let o = ir.o_const.unwrap_or(bind_o_from_solution());

// Use length mask hint for SIMD
let mask = ir.len_mask_hint; // 0xFF for len=8
```

### MPHF Lookup

```rust
use knhk_aot::mphf::{Mphf, MphfCache};

// Build MPHF for schema predicates
let predicates = vec![
    hash_iri("http://example.org/p1"),
    hash_iri("http://example.org/p2"),
    hash_iri("http://example.org/p3"),
];
let mphf = Mphf::new(predicates);

// Fast predicate lookup
let pred_hash = hash_iri("http://example.org/p2");
if let Some(idx) = mphf.lookup(pred_hash) {
    // Use idx for direct array access
    let run = &runs[idx];
}

// Cache MPHF per schema
let mut cache = MphfCache::new();
cache.insert("urn:knhk:schema:test", mphf);
```

## Key Features

- **IR Validation**: Enforces Chatman Constant (≤8 ticks) before execution
- **Template Analysis**: Separates constants from variables for optimization
- **Prebinding**: Precomputes constants for branchless hot path
- **MPHF**: O(1) predicate lookup with perfect hashing
- **Specialization**: Operation-specific optimizations
- **Guard Constraints**: Enforces max_run_len ≤ 8 at compile time

## Integration

### ETL Pipeline Integration

```rust
use knhk_aot::AotGuard;
use knhk_etl::Pipeline;

// Validate IR before pipeline execution
let op = 1; // ASK_SP
let run_len = 8;
let k = 0;

AotGuard::validate_ir(op, run_len, k)?;

// Execute pipeline
let pipeline = Pipeline::new();
pipeline.execute()?;
```

### Hot Path Integration

```rust
use knhk_aot::{AotGuard, prebinding::PreboundIr};
use knhk_hot::Engine;

// Validate and prebind before hot path execution
let ir = PreboundIr::from_template(&template, run_len)?;
AotGuard::validate_ir(op, run_len, k)?;

// Execute on hot path
let engine = Engine::new(s_ptr, p_ptr, o_ptr);
engine.eval_bool(&mut ir, &mut receipt)?;
```

## Dependencies

- No external dependencies (pure Rust, no_std compatible)
- Used by: `knhk-etl`, `knhk-hot`

## Performance

- **IR Validation**: O(1) constant-time validation
- **Template Analysis**: O(n) where n = template triple count
- **MPHF Lookup**: O(1) perfect hash lookup
- **Prebinding**: O(1) constant-time precomputation

## Related Documentation

- [Architecture](../../../docs/architecture.md) - System architecture
- [Performance](../../../docs/performance.md) - Performance guide (≤8 ticks constraint)
- [Hot Path](../../../rust/knhk-hot/docs/README.md) - Hot path operations
- [ETL Pipeline](../../../rust/knhk-etl/docs/README.md) - ETL pipeline integration
