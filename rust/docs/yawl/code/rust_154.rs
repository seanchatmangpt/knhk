let request = AllocationRequest {
    task_id: "task:specialized".to_string(),
    spec_id,
    required_roles: vec!["specialist".to_string()],
    required_capabilities: vec![],
    policy: AllocationPolicy::RoleBased,
    priority: 100,
};