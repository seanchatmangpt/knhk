use knhk_workflow_engine::patterns::{PatternId, PatternExecutionContext};

let context = PatternExecutionContext {
    case_id,
    workflow_id: spec_id,
    variables: HashMap::new(),
};

let result = engine.execute_pattern(PatternId(1), context).await?;