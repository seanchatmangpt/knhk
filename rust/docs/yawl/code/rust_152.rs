let request = AllocationRequest {
    task_id: "task:process".to_string(),
    spec_id,
    required_roles: vec!["processor".to_string()],
    required_capabilities: vec![],
    policy: AllocationPolicy::RoundRobin,
    priority: 100,
};