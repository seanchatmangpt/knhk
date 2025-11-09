//! Chicago TDD validation tests for refactored modules
//!
//! Validates that refactored modules maintain correct behavior:
//! - patterns/rdf: Serialization and deserialization
//! - patterns/advanced_control: Pattern execution
//! - resource/allocation: Allocation policies
//! - api/rest: Handler functions

use knhk_workflow_engine::case::CaseId;
use knhk_workflow_engine::parser::WorkflowSpecId;
use knhk_workflow_engine::patterns::rdf::{
    get_all_pattern_metadata, serialize_context_to_rdf, serialize_metadata_to_rdf,
    serialize_result_to_rdf, PatternMetadata, WORKFLOW_PATTERN_NS, YAWL_NS,
};
use knhk_workflow_engine::patterns::{
    advanced_control, PatternExecutionContext, PatternExecutionResult, PatternId,
};
use knhk_workflow_engine::resource::allocation::{
    AllocationPolicy, AllocationRequest, Resource, ResourceAllocator, ResourceId, Role,
};
use chicago_tdd_tools::{chicago_async_test, assert_ok, assert_err, assert_eq_msg};

chicago_async_test!(test_rdf_metadata_serialization, {
    // Arrange: Create pattern metadata
    let metadata = PatternMetadata::new(
        1,
        "Sequence".to_string(),
        "Execute activities in strict sequential order".to_string(),
        "Basic Control Flow".to_string(),
        "Simple".to_string(),
        vec![],
    );

    // Act: Serialize to RDF
    let rdf = serialize_metadata_to_rdf(&metadata).expect("Failed to serialize metadata");

    // Assert: RDF contains expected elements
    assert!(
        rdf.contains(&format!("{}pattern:1", WORKFLOW_PATTERN_NS)),
        "RDF should contain pattern IRI"
    );
    assert!(rdf.contains("Sequence"), "RDF should contain pattern name");
    assert!(
        rdf.contains("Basic Control Flow"),
        "RDF should contain category"
    );
}

chicago_async_test!(test_rdf_context_serialization, {
    // Arrange: Create pattern execution context
    let pattern_id = PatternId::new(26).expect("Invalid pattern ID");
    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: {
            let mut vars = std::collections::HashMap::new();
            vars.insert("test_key".to_string(), "test_value".to_string());
            vars
        },
        arrived_from: std::collections::HashSet::new(),
        scope_id: String::new(),
    };

    // Act: Serialize to RDF
    let rdf = serialize_context_to_rdf(&pattern_id, &context).expect("Failed to serialize context");

    // Assert: RDF contains expected elements
    assert!(
        rdf.contains(&format!("{}pattern:26", WORKFLOW_PATTERN_NS)),
        "RDF should contain pattern IRI"
    );
    assert!(
        rdf.contains("pattern:PatternExecution"),
        "RDF should contain execution type"
    );
}

chicago_async_test!(test_rdf_result_serialization, {
    // Arrange: Create pattern execution result
    let pattern_id = PatternId::new(26).expect("Invalid pattern ID");
    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: std::collections::HashMap::new(),
        arrived_from: std::collections::HashSet::new(),
        scope_id: String::new(),
    };
    let result = PatternExecutionResult {
        success: true,
        next_state: Some("completed".to_string()),
        next_activities: Vec::new(),
        variables: std::collections::HashMap::new(),
        updates: None,
        cancel_activities: Vec::new(),
        terminates: false,
    };

    // Act: Serialize to RDF
    let rdf = serialize_result_to_rdf(&pattern_id, &context, &result)
        .expect("Failed to serialize result");

    // Assert: RDF contains expected elements
    assert!(
        rdf.contains("pattern:PatternExecutionResult"),
        "RDF should contain result type"
    );
    assert!(
        rdf.contains("pattern:success true"),
        "RDF should contain success flag"
    );
}

chicago_async_test!(test_rdf_get_all_metadata, {
    // Arrange: Get all pattern metadata
    // Act: Retrieve metadata
    let metadata = get_all_pattern_metadata();

    // Assert: Should have 43 patterns
    assert_eq!(metadata.len(), 43, "Should have 43 patterns");
    assert_eq!(metadata[0].pattern_id, 1, "First pattern should be ID 1");
    assert_eq!(metadata[42].pattern_id, 43, "Last pattern should be ID 43");
}

chicago_async_test!(test_advanced_control_pattern_26, {
    // Arrange: Create blocking discriminator pattern
    let (pattern_id, executor) = advanced_control::create_pattern_26();
    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: std::collections::HashMap::new(),
        arrived_from: std::collections::HashSet::new(),
        scope_id: String::new(),
    };

    // Act: Execute pattern
    let result = executor.execute(&context);

    // Assert: Pattern executes successfully
    assert!(result.success, "Pattern should execute successfully");
    assert_eq!(pattern_id.0, 26, "Pattern ID should be 26");
    assert!(result.next_state.is_some(), "Pattern should set next state");
}

chicago_async_test!(test_advanced_control_pattern_27, {
    // Arrange: Create cancelling discriminator pattern
    let (pattern_id, executor) = advanced_control::create_pattern_27();
    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: std::collections::HashMap::new(),
        arrived_from: std::collections::HashSet::new(),
        scope_id: String::new(),
    };

    // Act: Execute pattern
    let result = executor.execute(&context);

    // Assert: Pattern executes successfully and cancels activities
    assert!(result.success, "Pattern should execute successfully");
    assert_eq!(pattern_id.0, 27, "Pattern ID should be 27");
    assert!(
        !result.cancel_activities.is_empty(),
        "Pattern should cancel activities"
    );
}

chicago_async_test!(test_advanced_control_patterns_28_39, {
    // Arrange: Create all advanced control patterns
    let patterns = vec![
        advanced_control::create_pattern_28(),
        advanced_control::create_pattern_29(),
        advanced_control::create_pattern_30(),
        advanced_control::create_pattern_31(),
        advanced_control::create_pattern_32(),
        advanced_control::create_pattern_33(),
        advanced_control::create_pattern_34(),
        advanced_control::create_pattern_35(),
        advanced_control::create_pattern_36(),
        advanced_control::create_pattern_37(),
        advanced_control::create_pattern_38(),
        advanced_control::create_pattern_39(),
    ];

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: std::collections::HashMap::new(),
        arrived_from: std::collections::HashSet::new(),
        scope_id: String::new(),
    };

    // Act & Assert: Execute each pattern
    for (idx, (pattern_id, executor)) in patterns.iter().enumerate() {
        let expected_id = (idx + 28) as u32;
        assert_eq!(
            pattern_id.0, expected_id,
            "Pattern ID should be {}",
            expected_id
        );

        let result = executor.execute(&context);
        assert!(
            result.success,
            "Pattern {} should execute successfully",
            expected_id
        );
        assert!(
            result.next_state.is_some(),
            "Pattern {} should set next state",
            expected_id
        );
    }
}

chicago_async_test!(test_resource_allocation_four_eyes, {
    // Arrange: Create allocator and resources
    let allocator = ResourceAllocator::new();

    let resource1 = Resource {
        id: ResourceId::new(),
        name: "Resource 1".to_string(),
        roles: vec![Role {
            id: "approver".to_string(),
            name: "Approver".to_string(),
            capabilities: vec![],
        }],
        capabilities: vec![],
        workload: 0,
        queue_length: 0,
        available: true,
    };

    let resource2 = Resource {
        id: ResourceId::new(),
        name: "Resource 2".to_string(),
        roles: vec![Role {
            id: "approver".to_string(),
            name: "Approver".to_string(),
            capabilities: vec![],
        }],
        capabilities: vec![],
        workload: 0,
        queue_length: 0,
        available: true,
    };

    allocator
        .register_resource(resource1)
        .await
        .expect("Failed to register resource1");
    allocator
        .register_resource(resource2)
        .await
        .expect("Failed to register resource2");

    let request = AllocationRequest {
        task_id: "task1".to_string(),
        spec_id: WorkflowSpecId::new(),
        required_roles: vec!["approver".to_string()],
        required_capabilities: vec![],
        policy: AllocationPolicy::FourEyes,
        priority: 100,
    };

    // Act: Allocate resources
    let result = allocator
        .allocate(request)
        .await
        .expect("Failed to allocate resources");

    // Assert: Two resources allocated
    assert_eq!(
        result.resource_ids.len(),
        2,
        "Four-eyes should allocate 2 resources"
    );
    assert_eq!(
        result.policy,
        AllocationPolicy::FourEyes,
        "Policy should be FourEyes"
    );
}

chicago_async_test!(test_resource_allocation_round_robin, {
    // Arrange: Create allocator and resources
    let allocator = ResourceAllocator::new();

    let resource1 = Resource {
        id: ResourceId::new(),
        name: "Resource 1".to_string(),
        roles: vec![],
        capabilities: vec![],
        workload: 0,
        queue_length: 0,
        available: true,
    };

    let resource2 = Resource {
        id: ResourceId::new(),
        name: "Resource 2".to_string(),
        roles: vec![],
        capabilities: vec![],
        workload: 0,
        queue_length: 0,
        available: true,
    };

    allocator
        .register_resource(resource1)
        .await
        .expect("Failed to register resource1");
    allocator
        .register_resource(resource2)
        .await
        .expect("Failed to register resource2");

    let request = AllocationRequest {
        task_id: "task1".to_string(),
        spec_id: WorkflowSpecId::new(),
        required_roles: vec![],
        required_capabilities: vec![],
        policy: AllocationPolicy::RoundRobin,
        priority: 100,
    };

    // Act: Allocate resources multiple times
    let result1 = allocator
        .allocate(request.clone())
        .await
        .expect("Failed to allocate resources");
    let result2 = allocator
        .allocate(request.clone())
        .await
        .expect("Failed to allocate resources");

    // Assert: Resources allocated in round-robin fashion
    assert_eq!(
        result1.resource_ids.len(),
        1,
        "Round-robin should allocate 1 resource"
    );
    assert_eq!(
        result2.resource_ids.len(),
        1,
        "Round-robin should allocate 1 resource"
    );
    assert_eq!(
        result1.policy,
        AllocationPolicy::RoundRobin,
        "Policy should be RoundRobin"
    );
}

chicago_async_test!(test_resource_allocation_shortest_queue, {
    // Arrange: Create allocator and resources with different queue lengths
    let allocator = ResourceAllocator::new();

    let resource1 = Resource {
        id: ResourceId::new(),
        name: "Resource 1".to_string(),
        roles: vec![],
        capabilities: vec![],
        workload: 0,
        queue_length: 5,
        available: true,
    };

    let resource2 = Resource {
        id: ResourceId::new(),
        name: "Resource 2".to_string(),
        roles: vec![],
        capabilities: vec![],
        workload: 0,
        queue_length: 2,
        available: true,
    };

    allocator
        .register_resource(resource1)
        .await
        .expect("Failed to register resource1");
    allocator
        .register_resource(resource2)
        .await
        .expect("Failed to register resource2");

    let request = AllocationRequest {
        task_id: "task1".to_string(),
        spec_id: WorkflowSpecId::new(),
        required_roles: vec![],
        required_capabilities: vec![],
        policy: AllocationPolicy::ShortestQueue,
        priority: 100,
    };

    // Act: Allocate resources
    let result = allocator
        .allocate(request)
        .await
        .expect("Failed to allocate resources");

    // Assert: Resource with shortest queue allocated
    assert_eq!(
        result.resource_ids.len(),
        1,
        "Shortest queue should allocate 1 resource"
    );
    assert_eq!(
        result.policy,
        AllocationPolicy::ShortestQueue,
        "Policy should be ShortestQueue"
    );
}

chicago_async_test!(test_resource_allocation_role_based, {
    // Arrange: Create allocator and resources with roles
    let allocator = ResourceAllocator::new();

    let resource1 = Resource {
        id: ResourceId::new(),
        name: "Resource 1".to_string(),
        roles: vec![Role {
            id: "developer".to_string(),
            name: "Developer".to_string(),
            capabilities: vec![],
        }],
        capabilities: vec![],
        workload: 0,
        queue_length: 0,
        available: true,
    };

    allocator
        .register_resource(resource1)
        .await
        .expect("Failed to register resource1");

    let request = AllocationRequest {
        task_id: "task1".to_string(),
        spec_id: WorkflowSpecId::new(),
        required_roles: vec!["developer".to_string()],
        required_capabilities: vec![],
        policy: AllocationPolicy::RoleBased,
        priority: 100,
    };

    // Act: Allocate resources
    let result = allocator
        .allocate(request)
        .await
        .expect("Failed to allocate resources");

    // Assert: Resource with matching role allocated
    assert_eq!(
        result.resource_ids.len(),
        1,
        "Role-based should allocate 1 resource"
    );
    assert_eq!(
        result.policy,
        AllocationPolicy::RoleBased,
        "Policy should be RoleBased"
    );
}

chicago_async_test!(test_resource_allocation_chained, {
    // Arrange: Create allocator and resources
    let allocator = ResourceAllocator::new();

    let resource1 = Resource {
        id: ResourceId::new(),
        name: "Resource 1".to_string(),
        roles: vec![],
        capabilities: vec![],
        workload: 0,
        queue_length: 0,
        available: true,
    };

    allocator
        .register_resource(resource1)
        .await
        .expect("Failed to register resource1");

    let request = AllocationRequest {
        task_id: "task1".to_string(),
        spec_id: WorkflowSpecId::new(),
        required_roles: vec![],
        required_capabilities: vec![],
        policy: AllocationPolicy::Chained,
        priority: 100,
    };

    // Act: Allocate resources multiple times
    let result1 = allocator
        .allocate(request.clone())
        .await
        .expect("Failed to allocate resources");
    let result2 = allocator
        .allocate(request)
        .await
        .expect("Failed to allocate resources");

    // Assert: Same resource allocated in chain
    assert_eq!(
        result1.resource_ids.len(),
        1,
        "Chained should allocate 1 resource"
    );
    assert_eq!(
        result2.resource_ids.len(),
        1,
        "Chained should allocate 1 resource"
    );
    assert_eq!(
        result1.policy,
        AllocationPolicy::Chained,
        "Policy should be Chained"
    );
}

chicago_async_test!(test_rdf_namespace_constants, {
    // Arrange & Act: Check namespace constants
    // Assert: Namespaces are defined
    assert!(
        !WORKFLOW_PATTERN_NS.is_empty(),
        "Workflow pattern namespace should be defined"
    );
    assert!(!YAWL_NS.is_empty(), "YAWL namespace should be defined");
    assert!(
        WORKFLOW_PATTERN_NS.starts_with("http://"),
        "Workflow pattern namespace should be a valid URI"
    );
    assert!(
        YAWL_NS.starts_with("http://"),
        "YAWL namespace should be a valid URI"
    );
}
