# Neural Pattern Learning System for ggen v2.7.1

## Overview

The Neural Pattern Learning System is a hyper-advanced machine learning component that learns from previously generated code patterns and provides intelligent recommendations for new code generation tasks.

## Architecture

### Components

1. **Pattern Recognition Engine** - Extracts and matches code patterns
2. **Pattern Memory System** - LRU cache (1000 patterns) + persistent sled storage
3. **Code Quality Metrics** - Tracks pattern success rates and usage
4. **Learning Feedback Loop** - Updates pattern weights based on outcomes
5. **Pattern Recommendation** - Suggests best patterns with confidence scores

### Key Features

- Zero unwrap/expect (production-ready error handling)
- Thread-safe with Arc/RwLock
- Full OTEL instrumentation with tracing
- Persistent pattern storage with sled
- Time-based decay (recent patterns prioritized)
- Automatic pattern retirement (poor performers)
- Multi-language support (Rust, Python, JavaScript, Go, TypeScript)

## Usage Examples

### 1. Basic Usage

```rust
use knhk_workflow_engine::ggen::neural_patterns::{NeuralPatternLearner, Context};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the learner with persistent storage
    let mut learner = NeuralPatternLearner::new(Path::new("./pattern_db"))?;

    // Learn from generated code with quality feedback (0.0 to 1.0)
    let code = r#"
        fn process_data(input: Vec<u8>) -> Result<Output, Error> {
            validate_input(&input)?;
            let parsed = parse_data(input)?;
            transform(parsed)
        }
    "#;
    learner.learn_from_code(code, 0.95)?;

    // Get pattern recommendations for a new task
    let recommendations = learner.recommend_patterns("create error handling function")?;

    for rec in &recommendations {
        println!("Pattern: {} (confidence: {:.2})", rec.pattern.id, rec.confidence);
        println!("Rationale: {}", rec.rationale);
    }

    Ok(())
}
```

### 2. Learning from Multiple Code Samples

```rust
use knhk_workflow_engine::ggen::neural_patterns::NeuralPatternLearner;
use std::path::Path;

fn train_patterns() -> Result<(), Box<dyn std::error::Error>> {
    let mut learner = NeuralPatternLearner::new(Path::new("./pattern_db"))?;

    // Learn from successful patterns
    let patterns = vec![
        (r#"fn validate_input(data: &[u8]) -> Result<(), Error> { ... }"#, 0.95),
        (r#"async fn fetch_data(url: &str) -> Result<Data, Error> { ... }"#, 0.90),
        (r#"impl From<String> for CustomType { ... }"#, 0.88),
    ];

    for (code, quality) in patterns {
        learner.learn_from_code(code, quality)?;
    }

    // Save all patterns to persistent storage
    learner.save_learning()?;

    Ok(())
}
```

### 3. Pattern Application

```rust
use knhk_workflow_engine::ggen::neural_patterns::{
    NeuralPatternLearner, Pattern, Context, TargetLanguage
};
use std::path::Path;

fn apply_pattern_example() -> Result<(), Box<dyn std::error::Error>> {
    let learner = NeuralPatternLearner::new(Path::new("./pattern_db"))?;

    // Create a pattern template
    let template = r#"
        fn {{function_name}}(input: {{input_type}}) -> {{output_type}} {
            // Validate input
            if input.is_empty() {
                return Err(Error::InvalidInput);
            }

            // Process
            let result = process_{{function_name}}(input)?;
            Ok(result)
        }
    "#;

    let pattern = Pattern::from_code(template, TargetLanguage::Rust);

    // Create context with variables
    let mut context = Context::new();
    context.insert("function_name".to_string(), "parse_json".to_string());
    context.insert("input_type".to_string(), "String".to_string());
    context.insert("output_type".to_string(), "Result<JsonValue, Error>".to_string());

    // Apply pattern to generate code
    let generated_code = learner.apply_pattern(&pattern, &context)?;
    println!("Generated code:\n{}", generated_code);

    Ok(())
}
```

### 4. Pattern Statistics and Monitoring

```rust
use knhk_workflow_engine::ggen::neural_patterns::NeuralPatternLearner;
use std::path::Path;

fn monitor_patterns() -> Result<(), Box<dyn std::error::Error>> {
    let learner = NeuralPatternLearner::new(Path::new("./pattern_db"))?;

    // Get comprehensive statistics
    let stats = learner.get_pattern_stats()?;

    println!("Pattern Statistics:");
    println!("  Total patterns: {}", stats.total_patterns);
    println!("  Active patterns: {}", stats.active_patterns);
    println!("  Retired patterns: {}", stats.retired_patterns);
    println!("  Average success rate: {:.2}%", stats.avg_success_rate * 100.0);
    println!("  Total usages: {}", stats.total_usages);

    println!("\nPatterns by language:");
    for (lang, count) in &stats.patterns_by_language {
        println!("  {}: {}", lang, count);
    }

    Ok(())
}
```

### 5. Automatic Pattern Retirement

```rust
use knhk_workflow_engine::ggen::neural_patterns::NeuralPatternLearner;
use std::path::Path;

fn retire_failing_patterns() -> Result<(), Box<dyn std::error::Error>> {
    let mut learner = NeuralPatternLearner::new(Path::new("./pattern_db"))?;

    // Retire patterns with poor performance
    // (usage_count >= 10 AND success_rate < 0.3)
    let retired_count = learner.retire_failing_patterns()?;

    println!("Retired {} failing patterns", retired_count);

    Ok(())
}
```

## Pattern Recommendation Algorithm

The recommendation system uses a hybrid scoring approach:

1. **Quality Score** (60% weight):
   - Success rate (0.0 to 1.0)
   - Usage count (logarithmic boost)
   - Time-based decay (recent patterns prioritized)

2. **Similarity Score** (40% weight):
   - Keyword matching between pattern and task
   - Language matching bonus
   - Context requirements alignment

**Formula:**
```
combined_score = (quality_score * 0.6) + (similarity_score * 0.4)

where:
  quality_score = success_rate * exp(-decay_rate * age_days) * (1 + ln(usage_count))
  similarity_score = (matched_keywords / total_keywords) + language_bonus
```

## Data Structures

### Pattern

```rust
pub struct Pattern {
    pub id: String,                      // SHA256-based unique ID
    pub code_template: String,           // Code pattern template
    pub context_requirements: Vec<String>, // Required context variables
    pub success_rate: f64,               // 0.0 to 1.0
    pub usage_count: u64,                // Times pattern was used
    pub language: TargetLanguage,        // Target language
    keywords: Vec<String>,               // For pattern matching
    last_used: u64,                      // Timestamp (seconds)
    total_quality: f64,                  // Accumulated quality
}
```

### PatternRecommendation

```rust
pub struct PatternRecommendation {
    pub pattern: Pattern,                    // Recommended pattern
    pub confidence: f64,                     // Confidence score (0.0 to 1.0)
    pub rationale: String,                   // Human-readable explanation
    pub alternative_patterns: Vec<Pattern>,  // Other options
}
```

### PatternStatistics

```rust
pub struct PatternStatistics {
    pub total_patterns: usize,
    pub active_patterns: usize,
    pub retired_patterns: usize,
    pub avg_success_rate: f64,
    pub total_usages: u64,
    pub patterns_by_language: HashMap<String, usize>,
}
```

## Configuration

### Constants

- `PATTERN_CACHE_SIZE`: 1000 (LRU cache size)
- `PATTERN_DECAY_RATE`: 0.01 (per day)
- `MIN_SUCCESS_RATE`: 0.3 (retirement threshold)
- `MIN_USAGE_FOR_RETIREMENT`: 10 (minimum uses before retirement)

## Performance Characteristics

- **Pattern Learning**: O(1) average (LRU cache + hash map)
- **Pattern Recommendation**: O(n) where n = number of patterns
- **Pattern Application**: O(1) for simple templates
- **Storage**: Persistent sled database (MVCC, lock-free)
- **Thread Safety**: Arc/RwLock (concurrent reads, exclusive writes)

## Integration with ggen v2.7.1

The neural pattern learner integrates seamlessly with the existing ggen components:

```rust
use knhk_workflow_engine::ggen::{
    CodeGenerator, GenerationContext, NeuralPatternLearner
};
use std::path::Path;

fn integrated_generation() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize pattern learner
    let learner = NeuralPatternLearner::new(Path::new("./pattern_db"))?;

    // Get recommendations for task
    let task = "create REST API endpoint with error handling";
    let recommendations = learner.recommend_patterns(task)?;

    if let Some(best) = recommendations.first() {
        println!("Using pattern: {} (confidence: {:.2})",
                 best.pattern.id, best.confidence);

        // Use with code generator...
    }

    Ok(())
}
```

## OTEL Instrumentation

All major operations are instrumented with OpenTelemetry tracing:

- `NeuralPatternLearner::new()` - Initialization span
- `learn_from_code()` - Pattern learning span
- `recommend_patterns()` - Recommendation span
- `apply_pattern()` - Pattern application span
- `save_learning()` - Persistence span

Example trace:
```
neural_patterns.new [5ms]
  └─ load_from_storage [2ms]
     └─ sled.iter [1ms]

neural_patterns.learn_from_code [1ms]
  pattern_id: "pat_a7f3c2..."
  quality: 0.95

neural_patterns.recommend_patterns [3ms]
  task: "create error handler"
  recommendation_count: 5
```

## Error Handling

All operations return `WorkflowResult<T>` with comprehensive error messages:

```rust
pub enum WorkflowError {
    Validation(String),      // Invalid input
    Internal(String),        // Internal error
    StatePersistence(String),// Storage error
    // ... other variants
}
```

No unwrap() or expect() in production code paths.

## Best Practices

1. **Initialize once**: Create learner at startup, reuse across requests
2. **Provide feedback**: Always call `learn_from_code()` with quality scores
3. **Persist regularly**: Call `save_learning()` periodically
4. **Monitor statistics**: Track `get_pattern_stats()` for insights
5. **Retire failures**: Run `retire_failing_patterns()` periodically
6. **Use recommendations**: Always check top 3-5 recommendations
7. **Update quality**: Provide feedback after code generation succeeds/fails

## Testing

The implementation includes comprehensive tests:

```bash
# Run all neural pattern tests
cargo test --lib neural_patterns

# Run specific test
cargo test --lib test_pattern_recommendations

# Run with output
cargo test --lib neural_patterns -- --nocapture
```

## Future Enhancements

Potential improvements for future versions:

1. **Vector Embeddings**: Use actual ML embeddings instead of hash-based similarity
2. **Pattern Clustering**: Group similar patterns automatically
3. **Active Learning**: Request user feedback on uncertain recommendations
4. **Transfer Learning**: Share patterns across projects
5. **Pattern Evolution**: Automatically refine patterns based on usage
6. **Context-Aware Templates**: Dynamic template generation
7. **Multi-Pattern Fusion**: Combine multiple patterns intelligently

## License

MIT License - Part of the KNHK Workflow Engine

## Support

For issues, questions, or contributions:
- GitHub: https://github.com/yourusername/knhk
- Documentation: See main README.md
