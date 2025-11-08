# JSON Parser Using knhk-patterns - Complete Guide

**Version:** 1.0.0  
**Purpose:** Guide to building a JSON parser using knhk-patterns workflow orchestration

## Overview

This guide demonstrates how to use `knhk-patterns` to orchestrate JSON parsing workflows. JSON parsing is an excellent use case for workflow patterns because it involves:

- **Sequential operations**: Tokenize → Parse → Validate
- **Parallel operations**: Parse multiple object fields concurrently
- **Conditional routing**: Handle different JSON types (string/number/boolean/null/object/array)
- **Recursive structures**: Nested objects and arrays

## Architecture: Two-Stage JSON Parsing

### Stage 1: Tokenization (knhk-hot)
- SIMD-accelerated structural character detection
- UTF-8 validation
- Token stream generation (SoA layout)

### Stage 2: Pattern Matching (knhk-patterns)
- Workflow orchestration for parsing logic
- Type-specific value extraction
- Object/array navigation

## Pattern Mapping to JSON Parsing

### Pattern 1: Sequence - Parse Pipeline

**Use Case**: Execute parsing stages sequentially

```rust
use knhk_patterns::*;
use std::sync::Arc;

// JSON parsing pipeline: tokenize → parse → validate
let parse_pipeline = SequencePattern::new(vec![
    // Stage 1: Tokenize
    Arc::new(|input: JsonInput| {
        let mut tokenizer = JsonTokenizer::new(input.bytes);
        let token_count = tokenizer.tokenize()?;
        Ok(JsonInput {
            bytes: input.bytes,
            tokens: tokenizer.tokens(),
            stage: ParseStage::Tokenized,
        })
    }),
    
    // Stage 2: Parse
    Arc::new(|input: JsonInput| {
        let parser = JsonParser::new(input.tokens);
        let value = parser.parse()?;
        Ok(JsonInput {
            bytes: input.bytes,
            tokens: input.tokens,
            value: Some(value),
            stage: ParseStage::Parsed,
        })
    }),
    
    // Stage 3: Validate
    Arc::new(|input: JsonInput| {
        input.value.as_ref()
            .ok_or_else(|| PatternError::ExecutionFailed("No value to validate".to_string()))?
            .validate()?;
        Ok(JsonInput {
            bytes: input.bytes,
            tokens: input.tokens,
            value: input.value,
            stage: ParseStage::Validated,
        })
    }),
])?;

let result = parse_pipeline.execute(json_input)?;
```

**Tick Budget**: 1 tick per stage = 3 ticks total (within ≤8 tick hot path)

### Pattern 2: Parallel Split - Parse Object Fields Concurrently

**Use Case**: Parse multiple object fields in parallel

```rust
// Parse object with multiple fields concurrently
let object_fields = vec![
    Arc::new(|input: JsonInput| {
        // Parse field "name"
        parse_field(&input, "name")
    }),
    Arc::new(|input: JsonInput| {
        // Parse field "version"
        parse_field(&input, "version")
    }),
    Arc::new(|input: JsonInput| {
        // Parse field "features"
        parse_field(&input, "features")
    }),
];

let parallel_parse = ParallelSplitPattern::new(object_fields)?;
let results = parallel_parse.execute(json_input)?;

// Results contains parsed values for all fields
assert_eq!(results.len(), 3);
```

**Tick Budget**: 2 ticks (SIMD-optimized parallel execution)

**Benefits**:
- Parse independent fields concurrently
- SIMD optimization for batch processing
- Reduces latency for large objects

### Pattern 4: Exclusive Choice - Route by JSON Type

**Use Case**: Handle different JSON value types (string/number/boolean/null)

```rust
let type_router = ExclusiveChoicePattern::new(vec![
    // Route to string parser
    (
        Arc::new(|input: &JsonInput| {
            matches!(input.next_token(), Some(Token::String(_)))
        }) as ConditionFn<JsonInput>,
        Arc::new(|input: JsonInput| {
            parse_string_value(input)
        }) as BranchFn<JsonInput>,
    ),
    
    // Route to number parser
    (
        Arc::new(|input: &JsonInput| {
            matches!(input.next_token(), Some(Token::Number(_)))
        }),
        Arc::new(|input: JsonInput| {
            parse_number_value(input)
        }),
    ),
    
    // Route to boolean parser
    (
        Arc::new(|input: &JsonInput| {
            matches!(input.next_token(), Some(Token::True | Token::False))
        }),
        Arc::new(|input: JsonInput| {
            parse_boolean_value(input)
        }),
    ),
    
    // Route to null parser
    (
        Arc::new(|input: &JsonInput| {
            matches!(input.next_token(), Some(Token::Null))
        }),
        Arc::new(|input: JsonInput| {
            parse_null_value(input)
        }),
    ),
])?;

let result = type_router.execute(json_input)?;
```

**Tick Budget**: 2 ticks (branchless pattern selection)

**Benefits**:
- Type-specific parsing logic
- Single branch execution (XOR-split)
- Efficient routing

### Pattern 6: Multi-Choice - Parse Array Elements

**Use Case**: Parse array elements that may have different types

```rust
let array_parser = MultiChoicePattern::new(vec![
    // Parse string elements
    (
        Arc::new(|input: &JsonInput| {
            matches!(input.next_token(), Some(Token::String(_)))
        }) as ConditionFn<JsonInput>,
        Arc::new(|input: JsonInput| {
            parse_string_value(input)
        }) as BranchFn<JsonInput>,
    ),
    
    // Parse number elements
    (
        Arc::new(|input: &JsonInput| {
            matches!(input.next_token(), Some(Token::Number(_)))
        }),
        Arc::new(|input: JsonInput| {
            parse_number_value(input)
        }),
    ),
    
    // Parse nested objects
    (
        Arc::new(|input: &JsonInput| {
            matches!(input.next_token(), Some(Token::ObjectStart))
        }),
        Arc::new(|input: JsonInput| {
            parse_object_value(input)
        }),
    ),
])?;

let results = array_parser.execute(json_input)?;
// Results contains all parsed array elements
```

**Tick Budget**: 3 ticks (SIMD-optimized for parallel element parsing)

**Benefits**:
- Handle heterogeneous arrays
- Parallel element parsing
- Type-specific extraction

### Pattern 10: Arbitrary Cycles - Parse Recursive Structures

**Use Case**: Parse nested objects/arrays with retry logic

```rust
let recursive_parser = ArbitraryCyclesPattern::new(
    // Parse one level of nesting
    Arc::new(|input: JsonInput| {
        match input.next_token() {
            Some(Token::ObjectStart) => parse_object(input),
            Some(Token::ArrayStart) => parse_array(input),
            _ => Ok(input), // Leaf value
        }
    }),
    
    // Continue if more nesting detected
    Arc::new(|input: &JsonInput| {
        matches!(
            input.next_token(),
            Some(Token::ObjectStart | Token::ArrayStart)
        )
    }),
    
    100, // max depth
)?;

let result = recursive_parser.execute(json_input)?;
```

**Tick Budget**: 2 ticks per iteration (max 100 iterations = 200 ticks warm path)

**Benefits**:
- Handle arbitrary nesting depth
- Retry logic for malformed structures
- Depth limit protection

## Complete JSON Parser Example

### Step 1: Define Data Types

```rust
#[derive(Clone, Debug)]
pub struct JsonInput {
    pub bytes: Vec<u8>,
    pub tokens: Vec<Token>,
    pub value: Option<JsonValue>,
    pub stage: ParseStage,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParseStage {
    Initial,
    Tokenized,
    Parsed,
    Validated,
}

#[derive(Clone, Debug)]
pub enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
}
```

### Step 2: Create Pattern-Based Parser

```rust
use knhk_patterns::*;
use std::sync::Arc;

pub struct PatternBasedJsonParser {
    // Main parsing pipeline
    parse_pipeline: SequencePattern<JsonInput>,
    
    // Type router
    type_router: ExclusiveChoicePattern<JsonInput>,
    
    // Array parser
    array_parser: MultiChoicePattern<JsonInput>,
}

impl PatternBasedJsonParser {
    pub fn new() -> PatternResult<Self> {
        // Create type router
        let type_router = ExclusiveChoicePattern::new(vec![
            (
                Arc::new(|input: &JsonInput| {
                    matches!(input.peek_token(), Some(Token::String(_)))
                }) as ConditionFn<JsonInput>,
                Arc::new(|input: JsonInput| parse_string(input)) as BranchFn<JsonInput>,
            ),
            (
                Arc::new(|input: &JsonInput| {
                    matches!(input.peek_token(), Some(Token::Number(_)))
                }),
                Arc::new(|input: JsonInput| parse_number(input)),
            ),
            (
                Arc::new(|input: &JsonInput| {
                    matches!(input.peek_token(), Some(Token::True | Token::False))
                }),
                Arc::new(|input: JsonInput| parse_boolean(input)),
            ),
            (
                Arc::new(|input: &JsonInput| {
                    matches!(input.peek_token(), Some(Token::Null))
                }),
                Arc::new(|input: JsonInput| parse_null(input)),
            ),
        ])?;
        
        // Create array parser
        let array_parser = MultiChoicePattern::new(vec![
            (
                Arc::new(|input: &JsonInput| {
                    matches!(input.peek_token(), Some(Token::String(_)))
                }) as ConditionFn<JsonInput>,
                Arc::new(|input: JsonInput| parse_string(input)) as BranchFn<JsonInput>,
            ),
            (
                Arc::new(|input: &JsonInput| {
                    matches!(input.peek_token(), Some(Token::Number(_)))
                }),
                Arc::new(|input: JsonInput| parse_number(input)),
            ),
        ])?;
        
        // Create main pipeline
        let parse_pipeline = SequencePattern::new(vec![
            Arc::new(|input: JsonInput| {
                // Tokenize
                let mut tokenizer = JsonTokenizer::new(input.bytes);
                tokenizer.tokenize()?;
                Ok(JsonInput {
                    bytes: input.bytes,
                    tokens: tokenizer.tokens(),
                    value: None,
                    stage: ParseStage::Tokenized,
                })
            }),
            Arc::new(|input: JsonInput| {
                // Parse using type router
                let results = type_router.execute(input.clone())?;
                Ok(results[0].clone())
            }),
        ])?;
        
        Ok(Self {
            parse_pipeline,
            type_router,
            array_parser,
        })
    }
    
    pub fn parse(&self, json_bytes: Vec<u8>) -> PatternResult<JsonValue> {
        let input = JsonInput {
            bytes: json_bytes,
            tokens: Vec::new(),
            value: None,
            stage: ParseStage::Initial,
        };
        
        let results = self.parse_pipeline.execute(input)?;
        let final_input = &results[0];
        
        final_input.value.clone()
            .ok_or_else(|| PatternError::ExecutionFailed("No value parsed".to_string()))
    }
}
```

### Step 3: Use Pattern Composition for Complex Parsing

```rust
use knhk_patterns::composition::PatternBuilder;

// Build complex JSON parsing workflow
let json_parser = PatternBuilder::new()
    // Stage 1: Tokenize
    .then(Arc::new(|input: JsonInput| {
        tokenize_json(input)
    }))
    
    // Stage 2: Parse based on root type
    .choice(vec![
        (
            Arc::new(|input: &JsonInput| {
                matches!(input.peek_token(), Some(Token::ObjectStart))
            }),
            Arc::new(|input: JsonInput| {
                // Parse object with parallel field parsing
                parse_object_parallel(input)
            }),
        ),
        (
            Arc::new(|input: &JsonInput| {
                matches!(input.peek_token(), Some(Token::ArrayStart))
            }),
            Arc::new(|input: JsonInput| {
                // Parse array with multi-choice
                parse_array_multi_choice(input)
            }),
        ),
    ])
    
    // Stage 3: Validate
    .then(Arc::new(|input: JsonInput| {
        validate_json(input)
    }))
    
    .build();

let result = json_parser.execute(json_input)?;
```

## Performance Characteristics

### Tick Budget Analysis

| Operation | Pattern Used | Tick Budget | Path |
|-----------|--------------|-------------|------|
| Tokenization | Sequence (stage 1) | 1 | Hot |
| Type routing | Exclusive Choice | 2 | Hot |
| Field parsing | Parallel Split | 2 | Hot |
| Array parsing | Multi-Choice | 3 | Hot |
| Recursive parsing | Arbitrary Cycles | 2/iteration | Warm |
| **Total (simple)** | **Sequence** | **≤8** | **Hot** |
| **Total (complex)** | **Composite** | **≤100** | **Warm** |

### Hot Path Optimization

For hot path (≤8 ticks), use:
- **Pattern 1 (Sequence)**: Linear parsing pipeline
- **Pattern 4 (Exclusive Choice)**: Type routing
- **Pattern 2 (Parallel Split)**: Field parsing (SIMD)

### Warm Path Optimization

For warm path (≤100 ticks), use:
- **Pattern 6 (Multi-Choice)**: Heterogeneous arrays
- **Pattern 10 (Arbitrary Cycles)**: Recursive structures
- **Pattern Composition**: Complex workflows

## Integration with knhk-etl Pipeline

Use `PipelinePatternExt` to integrate JSON parsing into ETL pipelines:

```rust
use knhk_patterns::PipelinePatternExt;
use knhk_etl::Pipeline;

let mut pipeline = Pipeline::new(
    vec![], // connectors
    "http://knhk.example.org/json".to_string(),
    false, // lockchain
    vec![], // downstream
);

// Parse JSON with parallel field extraction
let results = pipeline.execute_parallel(vec![
    |result: EmitResult| {
        // Extract field "name"
        extract_json_field(&result, "name")
    },
    |result: EmitResult| {
        // Extract field "version"
        extract_json_field(&result, "version")
    },
])?;

// Parse JSON with conditional routing
let results = pipeline.execute_conditional(vec![
    (
        |result: &EmitResult| {
            // Route large objects to parallel parser
            result.receipts_written > 100
        },
        |result: EmitResult| {
            parse_large_json_parallel(result)
        },
    ),
    (
        |_| true, // Default route
        |result: EmitResult| {
            parse_small_json_sequential(result)
        },
    ),
])?;
```

## Best Practices

### 1. Use Sequence for Linear Pipelines

```rust
// ✅ Good: Sequential stages
let pipeline = SequencePattern::new(vec![
    tokenize_stage,
    parse_stage,
    validate_stage,
])?;

// ❌ Bad: Nested patterns when sequence suffices
let nested = SequencePattern::new(vec![
    Arc::new(|input| {
        SequencePattern::new(vec![tokenize_stage])?.execute(input)
    }),
])?;
```

### 2. Use Parallel Split for Independent Operations

```rust
// ✅ Good: Parse independent fields in parallel
let parallel = ParallelSplitPattern::new(vec![
    parse_field_1,
    parse_field_2,
    parse_field_3,
])?;

// ❌ Bad: Sequential parsing when fields are independent
let sequential = SequencePattern::new(vec![
    parse_field_1,
    parse_field_2,
    parse_field_3,
])?;
```

### 3. Use Exclusive Choice for Type Routing

```rust
// ✅ Good: Route by type (XOR-split)
let router = ExclusiveChoicePattern::new(vec![
    (is_string, parse_string),
    (is_number, parse_number),
])?;

// ❌ Bad: Try all parsers sequentially
let try_all = SequencePattern::new(vec![
    try_parse_string,
    try_parse_number,
])?;
```

### 4. Validate at Ingress, Not Runtime

```rust
// ✅ Good: Validate pattern constraints at creation
let pattern = SequencePattern::new(branches)?; // Validates here

// ❌ Bad: Validate during execution
fn execute(&self, input: T) -> PatternResult<Vec<T>> {
    if self.branches.len() > 1024 { // ❌ Runtime validation
        return Err(PatternError::TooManyBranches);
    }
    // ...
}
```

## Testing JSON Parser with Patterns

### Chicago TDD: State-Based Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_object_parsing() {
        // Arrange
        let parser = PatternBasedJsonParser::new().unwrap();
        let json = br#"{"name": "KNHK", "version": 1}"#.to_vec();
        
        // Act
        let result = parser.parse(json).unwrap();
        
        // Assert: Verify final state
        match result {
            JsonValue::Object(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "name");
                assert_eq!(fields[0].1, JsonValue::String("KNHK".to_string()));
            }
            _ => panic!("Expected object"),
        }
    }
    
    #[test]
    fn test_array_parsing_with_multi_choice() {
        // Arrange
        let parser = PatternBasedJsonParser::new().unwrap();
        let json = br#"[1, "two", 3.0]"#.to_vec();
        
        // Act
        let result = parser.parse(json).unwrap();
        
        // Assert: Verify array elements parsed correctly
        match result {
            JsonValue::Array(elements) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0], JsonValue::Number(1.0));
                assert_eq!(elements[1], JsonValue::String("two".to_string()));
                assert_eq!(elements[2], JsonValue::Number(3.0));
            }
            _ => panic!("Expected array"),
        }
    }
}
```

## Summary

**Key Patterns for JSON Parsing:**

1. **Pattern 1 (Sequence)**: Linear parsing pipeline (tokenize → parse → validate)
2. **Pattern 2 (Parallel Split)**: Parse object fields concurrently
3. **Pattern 4 (Exclusive Choice)**: Route by JSON type (string/number/boolean/null)
4. **Pattern 6 (Multi-Choice)**: Parse heterogeneous arrays
5. **Pattern 10 (Arbitrary Cycles)**: Handle recursive structures

**Performance:**
- Hot path (≤8 ticks): Simple JSON parsing with Sequence + Exclusive Choice
- Warm path (≤100 ticks): Complex parsing with Multi-Choice + Arbitrary Cycles

**Integration:**
- Use `PipelinePatternExt` for ETL pipeline integration
- Compose patterns using `PatternBuilder` for complex workflows
- Validate at ingress (pattern creation), not runtime

---

**Document Version:** 1.0.0  
**Last Updated:** 2025-01-27  
**Maintainer:** KNHK Team

