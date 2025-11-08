let request = AllocationRequest {
    task_id: "task:expert".to_string(),
    spec_id,
    required_roles: vec![],
    required_capabilities: vec!["expertise".to_string()],
    policy: AllocationPolicy::CapabilityBased,
    priority: 100,
};