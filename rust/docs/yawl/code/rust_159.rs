let context = PatternExecutionContext {
    case_id,
    workflow_id: spec_id,
    variables: HashMap::new(),
};

let result = engine.worklet_executor()
    .handle_exception("resource_unavailable", context)
    .await?;