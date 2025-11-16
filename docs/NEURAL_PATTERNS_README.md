# Neural Pattern Learning System for ggen v2.7.1

## Quick Start

```rust
use knhk_workflow_engine::ggen::neural_patterns::NeuralPatternLearner;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize learner with persistent storage
    let mut learner = NeuralPatternLearner::new(Path::new("./pattern_db"))?;

    // Learn from generated code
    let code = r#"
        fn process_data(input: Vec<u8>) -> Result<Output, Error> {
            validate_input(&input)?;
            let parsed = parse_data(input)?;
            transform(parsed)
        }
    "#;
    learner.learn_from_code(code, 0.95)?;

    // Get recommendations for new task
    let recommendations = learner.recommend_patterns("create error handling function")?;

    for rec in &recommendations {
        println!("Pattern: {} (confidence: {:.2})", rec.pattern.id, rec.confidence);
    }

    Ok(())
}
```

## What is This?

The Neural Pattern Learning System is an advanced ML component for ggen v2.7.1 that:

1. **Learns** from previously generated code patterns
2. **Recommends** optimal patterns for new generation tasks
3. **Adapts** based on quality feedback
4. **Retires** patterns that consistently fail

## Key Features

- ✨ **Pattern Recognition**: Extracts and matches code patterns using keywords and similarity
- 💾 **Persistent Memory**: LRU cache (1000 patterns) + sled database storage
- 📊 **Quality Metrics**: Tracks success rates, usage frequency, and quality scores
- 🔄 **Adaptive Learning**: Updates pattern weights based on feedback
- 🎯 **Smart Recommendations**: Hybrid scoring (60% quality + 40% similarity)
- 🗑️ **Auto-Retirement**: Removes patterns with poor performance
- 🌐 **Multi-Language**: Supports Rust, Python, JavaScript, Go, TypeScript
- 🔒 **Production-Ready**: Zero unwrap/expect, full error handling
- 🧵 **Thread-Safe**: Arc/RwLock for concurrent access
- 📈 **Observable**: Full OTEL instrumentation

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│            Neural Pattern Learner                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ LRU Cache    │  │ Pattern      │  │ Sled Storage │ │
│  │ (1000 items) │←→│ Library      │←→│ (Persistent) │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │ Pattern Recognition Engine                       │  │
│  │ - Extract keywords                               │  │
│  │ - Generate SHA256 IDs                           │  │
│  │ - Calculate similarity scores                   │  │
│  └──────────────────────────────────────────────────┘  │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │ Recommendation Engine                            │  │
│  │ - Quality scoring (success rate + usage)         │  │
│  │ - Similarity matching                           │  │
│  │ - Time-based decay                              │  │
│  └──────────────────────────────────────────────────┘  │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │ Learning Feedback Loop                           │  │
│  │ - Update pattern weights                         │  │
│  │ - Retire failing patterns                       │  │
│  │ - Persist to storage                            │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Core Concepts

### Pattern

A code pattern is a template with metadata:

```rust
Pattern {
    id: "pat_a7f3c2d8",              // SHA256-based ID
    code_template: "fn ...",          // Code template
    success_rate: 0.95,               // Quality score
    usage_count: 42,                  // Times used
    language: TargetLanguage::Rust,   // Target language
    keywords: ["fn", "Result<"],      // For matching
    last_used: 1699123456,            // Timestamp
}
```

### Recommendation

Recommendations include confidence and alternatives:

```rust
PatternRecommendation {
    pattern: Pattern { ... },
    confidence: 0.87,                 // 0.0 to 1.0
    rationale: "Pattern ... has ...", // Explanation
    alternative_patterns: [...]       // Top 3 alternatives
}
```

### Learning Algorithm

**Scoring Formula:**
```
combined_score = (quality_score × 0.6) + (similarity_score × 0.4)

where:
  quality_score = success_rate × e^(-decay_rate × age_days) × (1 + ln(usage_count))
  similarity_score = (matched_keywords / total_keywords) + language_bonus
```

## API Reference

### NeuralPatternLearner

```rust
// Create new learner
pub fn new(storage_path: &Path) -> WorkflowResult<Self>

// Learn from code with quality feedback (0.0 to 1.0)
pub fn learn_from_code(&mut self, code: &str, quality: f64) -> WorkflowResult<()>

// Get pattern recommendations for task
pub fn recommend_patterns(&self, task: &str) -> WorkflowResult<Vec<PatternRecommendation>>

// Apply pattern to context
pub fn apply_pattern(&self, pattern: &Pattern, context: &Context) -> WorkflowResult<String>

// Get statistics
pub fn get_pattern_stats(&self) -> WorkflowResult<PatternStatistics>

// Save to persistent storage
pub fn save_learning(&self) -> WorkflowResult<()>

// Retire failing patterns
pub fn retire_failing_patterns(&mut self) -> WorkflowResult<usize>
```

## Usage Examples

### 1. Basic Learning

```rust
let mut learner = NeuralPatternLearner::new(Path::new("./db"))?;

// Learn from successful code (quality = 0.95)
learner.learn_from_code("fn validate(x: i32) -> bool { x > 0 }", 0.95)?;

// Learn from failed code (quality = 0.3)
learner.learn_from_code("fn buggy() { panic!() }", 0.3)?;
```

### 2. Get Recommendations

```rust
let recommendations = learner.recommend_patterns("create validation function")?;

for (i, rec) in recommendations.iter().enumerate() {
    println!("#{} - {} (confidence: {:.2})", i+1, rec.pattern.id, rec.confidence);
    println!("   {}", rec.rationale);
}
```

### 3. Apply Pattern

```rust
let pattern = Pattern::from_code(
    "fn {{name}}({{param}}: {{type}}) -> bool { {{param}} > 0 }",
    TargetLanguage::Rust
);

let mut context = Context::new();
context.insert("name".to_string(), "validate_age".to_string());
context.insert("param".to_string(), "age".to_string());
context.insert("type".to_string(), "u32".to_string());

let code = learner.apply_pattern(&pattern, &context)?;
println!("{}", code);
// Output: fn validate_age(age: u32) -> bool { age > 0 }
```

### 4. Monitor Statistics

```rust
let stats = learner.get_pattern_stats()?;

println!("Total patterns: {}", stats.total_patterns);
println!("Active: {}, Retired: {}", stats.active_patterns, stats.retired_patterns);
println!("Average success: {:.1}%", stats.avg_success_rate * 100.0);

for (lang, count) in &stats.patterns_by_language {
    println!("  {}: {}", lang, count);
}
```

## Performance

| Operation | Complexity | Typical Time |
|-----------|-----------|--------------|
| Learn from code | O(1) | < 1ms |
| Get recommendations | O(n) | < 5ms for 1000 patterns |
| Apply pattern | O(1) | < 0.1ms |
| Save learning | O(n) | < 100ms for 1000 patterns |

**Memory Usage:**
- LRU Cache: ~1MB (1000 patterns)
- Pattern Library: ~10-100KB per pattern
- Storage: Compressed on disk

## Configuration

Default constants (can be customized):

```rust
PATTERN_CACHE_SIZE: 1000           // LRU cache size
PATTERN_DECAY_RATE: 0.01           // Decay rate per day
MIN_SUCCESS_RATE: 0.3              // Retirement threshold
MIN_USAGE_FOR_RETIREMENT: 10       // Min uses before retirement
```

## Integration with ggen

```rust
use knhk_workflow_engine::ggen::{CodeGenerator, NeuralPatternLearner};

// Initialize both systems
let generator = RustGenerator::new("templates/rust")?;
let mut learner = NeuralPatternLearner::new(Path::new("./patterns"))?;

// Get recommendation
let task = "create REST API endpoint with error handling";
let recommendations = learner.recommend_patterns(task)?;

if let Some(best) = recommendations.first() {
    // Use best pattern with code generator
    let mut context = Context::new();
    // ... populate context ...
    let code = learner.apply_pattern(&best.pattern, &context)?;

    // Learn from result
    learner.learn_from_code(&code, 0.92)?;
}
```

## Testing

Run tests:

```bash
cargo test --lib neural_patterns
```

Run specific test:

```bash
cargo test --lib test_pattern_recommendations -- --nocapture
```

## Files

- **Implementation**: `rust/knhk-workflow-engine/src/ggen/neural_patterns.rs` (731 lines)
- **Demo**: `docs/neural_patterns_demo.md` (Comprehensive examples)
- **Summary**: `docs/neural_patterns_implementation_summary.md` (Technical details)

## Error Handling

All operations return `WorkflowResult<T>`:

```rust
match learner.learn_from_code(code, 0.95) {
    Ok(()) => println!("Learned successfully"),
    Err(WorkflowError::Validation(msg)) => eprintln!("Invalid input: {}", msg),
    Err(WorkflowError::Internal(msg)) => eprintln!("Internal error: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Best Practices

1. **Initialize once** - Create learner at startup, reuse across requests
2. **Provide feedback** - Always call `learn_from_code()` with quality scores
3. **Persist regularly** - Call `save_learning()` periodically or on shutdown
4. **Monitor stats** - Track pattern effectiveness with `get_pattern_stats()`
5. **Retire failures** - Run `retire_failing_patterns()` weekly
6. **Use top recommendations** - Check top 3-5 recommendations, not just first
7. **Quality scores** - Be honest: 1.0 = perfect, 0.5 = neutral, 0.0 = broken

## OTEL Traces

All operations emit OpenTelemetry spans:

```
neural_patterns.new [duration: 5ms]
  └─ load_from_storage [duration: 2ms]

neural_patterns.learn_from_code [duration: 1ms]
  attributes: pattern_id="pat_...", quality=0.95

neural_patterns.recommend_patterns [duration: 3ms]
  attributes: task="...", recommendation_count=5
```

## Support

- **Issues**: See main KNHK repository
- **Documentation**: `cargo doc --open`
- **Examples**: See `docs/neural_patterns_demo.md`

## License

MIT License - Part of the KNHK Workflow Engine
