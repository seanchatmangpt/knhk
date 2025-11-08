use knhk_patterns::PipelinePatternExt;
use knhk_etl::Pipeline;

let mut pipeline = Pipeline::new(...);

// Execute with parallel validation
let results = pipeline.execute_parallel(vec![
    |result| { /* validator 1 */ Ok(result) },
    |result| { /* validator 2 */ Ok(result) },
])?;

// Execute with conditional routing
let results = pipeline.execute_conditional(vec![
    (|result| result.runs.len() > 100, |result| { /* process large */ Ok(result) }),
    (|_| true, |result| { /* process normal */ Ok(result) }),
])?;

// Execute with retry
let result = pipeline.execute_with_retry(
    |result| { /* processor */ Ok(result) },
    |result| result.runs.is_empty(),
    3,
)?;