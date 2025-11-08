# Framework Generalization Principles - Key Learnings

## Overview

This document captures key learnings about using the KNHK framework GENERALLY, not through domain-specific wrappers. These principles emerged from refactoring `knhk-json-bench` to properly demonstrate framework capabilities.

## Core Principle: Generalization Over Specialization

**The framework provides general capabilities - use them directly, don't wrap them.**

### ❌ Anti-Pattern: Domain-Specific Wrappers

```rust
// BAD: JSON-specific tokenizer wrapper hiding the framework
pub struct JsonTokenizer {
    input: Vec<u8>,
    tokens: Vec<Token>,
    // ... JSON-specific state
}

impl JsonTokenizer {
    pub fn tokenize(&mut self) -> Result<usize, TokenizeError> {
        // Framework hidden behind JSON-specific API
        let dispatcher = CpuDispatcher::get();
        // ... JSON-specific logic
    }
}
```

**Problems:**
- Framework capabilities hidden behind domain-specific API
- Harder to reuse framework components
- Violates framework's general-purpose design
- Creates unnecessary abstraction layers

### ✅ Correct Pattern: Direct Framework Usage

```rust
// GOOD: Use framework components directly
pub fn parse_json(json_bytes: Vec<u8>) -> Result<Vec<u8>, PatternError> {
    // Stage 1: Use knhk-hot directly
    let dispatcher = CpuDispatcher::get(); // Framework: CPU feature detection
    let mut structural_index = StructuralIndex::new();
    unsafe {
        stage1_structural_index(&json_bytes, &mut structural_index);
    }

    // Stage 2: Use knhk-patterns directly
    let workflow = PatternBuilder::new()
        .choice(vec![/* ... */])
        .then(/* ... */)
        .build();
    
    workflow.execute(state)?
}
```

**Benefits:**
- Framework capabilities visible and reusable
- Easy to understand framework integration
- Demonstrates general-purpose design
- No unnecessary abstraction layers

## Pattern Composition Best Practices

### 1. Use Patterns for Orchestration, Not Implementation

**Patterns orchestrate workflows - they don't implement domain logic.**

```rust
// GOOD: Patterns orchestrate, domain logic in closures
let workflow = PatternBuilder::new()
    .choice(vec![
        (
            Arc::new(|state: &ParseState| state.data.len() < 100), // Condition
            Arc::new(|mut state: ParseState| {                    // Domain logic
                state.processed_count = state.structural_positions.len();
                Ok(state)
            }),
        ),
    ])
    .build();
```

**Key Insight:** Patterns handle orchestration (routing, parallelization, retry). Domain logic lives in `BranchFn` closures.

### 2. Compose Patterns, Don't Nest Domain Logic

**Patterns compose naturally - use them together.**

```rust
// GOOD: Patterns compose (MultiChoice → ArbitraryCycles)
let multi_choice = MultiChoicePattern::new(vec![/* ... */])?;
let results = multi_choice.execute(state)?;

let retry_pattern = ArbitraryCyclesPattern::new(
    branch_fn,
    condition_fn,
    3, // max retries
)?;
let retry_results = retry_pattern.execute(final_state)?;
```

**Key Insight:** Patterns are composable building blocks. Use them together to create complex workflows.

### 3. Use Generic State Types

**State types should be generic, not domain-specific.**

```rust
// GOOD: Generic state type
#[derive(Clone, Debug)]
pub struct ParseState {
    pub data: Vec<u8>,                    // Generic data
    pub structural_positions: Vec<u32>,   // Framework output
    pub processed_count: usize,           // Generic counter
    pub result: Option<Vec<u8>>,         // Generic result
}
```

**Key Insight:** Generic state types work with any domain. Domain-specific types lock you into one use case.

## Framework Component Integration

### knhk-hot: SIMD Structural Detection

**Use directly for structural character detection:**

```rust
// Framework: CPU feature detection
let dispatcher = CpuDispatcher::get();

// Framework: SIMD structural detection
let mut structural_index = StructuralIndex::new();
unsafe {
    stage1_structural_index(&json_bytes, &mut structural_index);
}
```

**Key Insight:** `knhk-hot` provides general SIMD capabilities. Use them directly, don't wrap them.

### knhk-patterns: Workflow Orchestration

**Use patterns to orchestrate workflows:**

```rust
// Framework: Pattern composition
let workflow = PatternBuilder::new()
    .choice(vec![/* ExclusiveChoicePattern */])
    .then(/* SequencePattern */)
    .build();

// Framework: Pattern execution
let results = workflow.execute(state)?;
```

**Key Insight:** Patterns orchestrate workflows generically. They work with any domain.

### knhk-etl: Pipeline Integration

**Use PipelinePatternExt to integrate patterns with ETL:**

```rust
use knhk_patterns::PipelinePatternExt;

// Framework: ETL pipeline with patterns
let mut pipeline = Pipeline::new(/* ... */);

// Framework: Parallel processing
let results = pipeline.execute_parallel(vec![
    |result| { /* processor 1 */ Ok(result) },
    |result| { /* processor 2 */ Ok(result) },
])?;
```

**Key Insight:** `PipelinePatternExt` extends ETL pipelines with pattern capabilities. Use it to orchestrate ETL workflows.

## Pattern Selection Guide

### When to Use Each Pattern

| Pattern | Use Case | Example |
|---------|----------|---------|
| **SequencePattern** | Sequential steps | Tokenize → Parse → Validate |
| **ParallelSplitPattern** | Independent operations | Parse field1 || Parse field2 |
| **ExclusiveChoicePattern** | Route by condition (XOR) | If size < 100: simple, else: parallel |
| **MultiChoicePattern** | Multiple matching routes (OR) | Process all matching conditions |
| **ArbitraryCyclesPattern** | Retry logic | Retry until condition met or max attempts |

### Pattern Composition Examples

**Example 1: Conditional → Parallel**

```rust
.choice(vec![
    (condition, |state| {
        // Use ParallelSplitPattern inside branch
        let parallel = ParallelSplitPattern::new(branches)?;
        parallel.execute(state)
    }),
])
```

**Example 2: MultiChoice → Retry**

```rust
let multi_choice = MultiChoicePattern::new(choices)?;
let results = multi_choice.execute(state)?;

let retry = ArbitraryCyclesPattern::new(
    branch_fn,
    condition_fn,
    max_attempts,
)?;
let final_results = retry.execute(combined_state)?;
```

## Common Mistakes to Avoid

### ❌ Mistake 1: Wrapping Framework Components

```rust
// BAD: Wrapping framework in domain-specific API
pub struct JsonParser {
    tokenizer: JsonTokenizer, // Hides framework
}

// GOOD: Use framework directly
pub fn parse_json(bytes: Vec<u8>) -> Result<Vec<u8>, PatternError> {
    // Use framework components directly
}
```

### ❌ Mistake 2: Domain-Specific State Types

```rust
// BAD: Domain-specific state
pub struct JsonParseState {
    tokens: Vec<JsonToken>,  // JSON-specific
    current_token: JsonToken, // JSON-specific
}

// GOOD: Generic state
pub struct ParseState {
    data: Vec<u8>,              // Generic
    structural_positions: Vec<u32>, // Framework output
    result: Option<Vec<u8>>,   // Generic
}
```

### ❌ Mistake 3: Implementing Domain Logic in Patterns

```rust
// BAD: Domain logic in pattern implementation
impl Pattern<JsonState> for JsonPattern {
    fn execute(&self, input: JsonState) -> Result<Vec<JsonState>> {
        // JSON-specific parsing logic here ❌
    }
}

// GOOD: Domain logic in closures, patterns orchestrate
let workflow = PatternBuilder::new()
    .then(Arc::new(|state: ParseState| {
        // Domain logic here ✅
        Ok(state)
    }))
    .build();
```

## Summary

### Key Principles

1. **Use framework components directly** - Don't wrap them in domain-specific APIs
2. **Patterns orchestrate, closures implement** - Patterns handle workflow, closures handle domain logic
3. **Generic state types** - Work with any domain, not locked to one
4. **Compose patterns** - Use patterns together to create complex workflows
5. **Framework provides capabilities** - Use them generically, not domain-specifically

### The Goal

**Demonstrate framework capabilities, not build a complete domain solution.**

The framework provides general capabilities:
- `knhk-hot`: SIMD structural detection
- `knhk-patterns`: Workflow orchestration
- `knhk-etl`: Pipeline integration

Use them directly to demonstrate how the framework works, not to build a production-ready domain solution.

---

**Document Version:** 1.0.0  
**Last Updated:** 2025-01-XX  
**Related:** `rust/knhk-json-bench/src/lib.rs` - Reference implementation

