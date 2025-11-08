use knhk_workflow_engine::worklets::{Worklet, WorkletRule, WorkletId};

let worklet = Worklet {
    id: WorkletId::new(),
    name: "Approval Failed Handler".to_string(),
    description: "Handle approval failures".to_string(),
    workflow_spec: approval_workflow_spec,
    exception_types: vec!["resource_unavailable".to_string()],
    tags: vec!["approval".to_string(), "exception".to_string()],
    rules: vec![WorkletRule {
        condition: "resource_unavailable".to_string(),
        priority: 100,
    }],
    version: "1.0.0".to_string(),
};

engine.worklet_repository().register(worklet).await?;