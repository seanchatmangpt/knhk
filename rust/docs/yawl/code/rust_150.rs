use knhk_workflow_engine::resource::{AllocationPolicy, AllocationRequest};

let request = AllocationRequest {
    task_id: "task:approve".to_string(),
    spec_id,
    required_roles: vec!["approver".to_string()],
    required_capabilities: vec![],
    policy: AllocationPolicy::FourEyes,
    priority: 100,
};

let allocation = engine.resource_allocator().allocate(request).await?;