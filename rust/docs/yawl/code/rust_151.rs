let request = AllocationRequest {
    task_id: "task:review".to_string(),
    spec_id,
    required_roles: vec!["reviewer".to_string()],
    required_capabilities: vec![],
    policy: AllocationPolicy::Chained,
    priority: 100,
};