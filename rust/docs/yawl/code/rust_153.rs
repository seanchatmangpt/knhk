let request = AllocationRequest {
    task_id: "task:assign".to_string(),
    spec_id,
    required_roles: vec!["worker".to_string()],
    required_capabilities: vec![],
    policy: AllocationPolicy::ShortestQueue,
    priority: 100,
};