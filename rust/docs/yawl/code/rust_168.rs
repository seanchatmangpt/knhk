// Task with resource allocation policy
let task = Task {
    id: "task:approve".to_string(),
    name: "Approve Order".to_string(),
    allocation_policy: Some(AllocationPolicy::FourEyes),
    required_roles: vec!["approver".to_string()],
    required_capabilities: vec![],
    exception_worklet: Some(worklet_id),
    // ... other fields ...
};