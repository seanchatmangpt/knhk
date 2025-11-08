//! Permutation and stress tests for refactored modules
//!
//! Tests various combinations and edge cases:
//! - Permutation tests: Different orderings and combinations of operations
//! - Stress tests: High load, boundary conditions, concurrent operations

use knhk_workflow_engine::case::CaseId;
use knhk_workflow_engine::parser::WorkflowSpecId;
use knhk_workflow_engine::patterns::rdf::{
    get_all_pattern_metadata, serialize_context_to_rdf, serialize_metadata_to_rdf, PatternMetadata,
    WORKFLOW_PATTERN_NS,
};
use knhk_workflow_engine::patterns::{advanced_control, PatternExecutionContext, PatternId};
use knhk_workflow_engine::resource::allocation::{
    AllocationPolicy, AllocationRequest, Resource, ResourceAllocator, ResourceId, Role,
};
use std::collections::HashMap;

// ============================================================================
// PERMUTATION TESTS
// ============================================================================

#[tokio::test]
async fn test_rdf_serialization_permutations() {
    // Test all combinations of metadata serialization
    let patterns = vec![1, 5, 10, 20, 30, 43];

    for pattern_id in patterns {
        let metadata = PatternMetadata::new(
            pattern_id,
            format!("Pattern {}", pattern_id),
            format!("Description for pattern {}", pattern_id),
            "Test Category".to_string(),
            "Medium".to_string(),
            vec![],
        );

        let rdf = serialize_metadata_to_rdf(&metadata)
            .expect(&format!("Failed to serialize pattern {}", pattern_id));

        assert!(
            rdf.contains(&format!("{}pattern:{}", WORKFLOW_PATTERN_NS, pattern_id)),
            "RDF should contain pattern {} IRI",
            pattern_id
        );
    }
}

#[tokio::test]
async fn test_pattern_execution_permutations() {
    // Test all advanced control patterns in different orders
    let pattern_creators = vec![
        advanced_control::create_pattern_26,
        advanced_control::create_pattern_27,
        advanced_control::create_pattern_28,
        advanced_control::create_pattern_29,
        advanced_control::create_pattern_30,
        advanced_control::create_pattern_31,
        advanced_control::create_pattern_32,
        advanced_control::create_pattern_33,
        advanced_control::create_pattern_34,
        advanced_control::create_pattern_35,
        advanced_control::create_pattern_36,
        advanced_control::create_pattern_37,
        advanced_control::create_pattern_38,
        advanced_control::create_pattern_39,
    ];

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: std::collections::HashSet::new(),
        scope_id: String::new(),
    };

    // Execute patterns in forward order
    for create_pattern in &pattern_creators {
        let (_pattern_id, executor) = create_pattern();
        let result = executor.execute(&context);
        assert!(result.success, "Pattern should execute successfully");
    }

    // Execute patterns in reverse order
    for create_pattern in pattern_creators.iter().rev() {
        let (_pattern_id, executor) = create_pattern();
        let result = executor.execute(&context);
        assert!(result.success, "Pattern should execute successfully");
    }
}

#[tokio::test]
async fn test_resource_allocation_policy_permutations() {
    let allocator = ResourceAllocator::new();

    // Create multiple resources
    let resources: Vec<Resource> = (0..5)
        .map(|i| Resource {
            id: ResourceId::new(),
            name: format!("Resource {}", i),
            roles: vec![Role {
                id: format!("role_{}", i % 2),
                name: format!("Role {}", i % 2),
                capabilities: vec![],
            }],
            capabilities: vec![],
            workload: i,
            queue_length: i * 2,
            available: true,
        })
        .collect();

    for resource in &resources {
        allocator
            .register_resource(resource.clone())
            .await
            .expect("Failed to register resource");
    }

    // Test all allocation policies
    let policies = vec![
        AllocationPolicy::FourEyes,
        AllocationPolicy::RoundRobin,
        AllocationPolicy::ShortestQueue,
        AllocationPolicy::RoleBased,
        AllocationPolicy::Chained,
    ];

    for policy in policies {
        let request = AllocationRequest {
            task_id: format!("task_{:?}", policy),
            spec_id: WorkflowSpecId::new(),
            required_roles: vec!["role_0".to_string()],
            required_capabilities: vec![],
            policy,
            priority: 100,
        };

        let result = allocator.allocate(request).await;

        // Four-eyes needs at least 2 resources, others should succeed
        if policy == AllocationPolicy::FourEyes {
            assert!(
                result.is_ok(),
                "Four-eyes allocation should succeed with 5 resources"
            );
            let allocation = result.unwrap();
            assert_eq!(allocation.resource_ids.len(), 2);
        } else {
            assert!(result.is_ok(), "Allocation should succeed");
        }
    }
}

// ============================================================================
// STRESS TESTS
// ============================================================================

#[tokio::test]
async fn test_rdf_serialization_stress() {
    // Stress test: Serialize many patterns rapidly
    let metadata_list: Vec<PatternMetadata> = (1..=43)
        .map(|id| {
            PatternMetadata::new(
                id,
                format!("Pattern {}", id),
                format!("Description {}", id),
                "Category".to_string(),
                "Complexity".to_string(),
                vec![],
            )
        })
        .collect();

    for metadata in &metadata_list {
        let rdf = serialize_metadata_to_rdf(metadata).expect("Failed to serialize metadata");
        assert!(!rdf.is_empty(), "RDF should not be empty");
        assert!(
            rdf.contains("rdf:type"),
            "RDF should contain RDF type declaration"
        );
    }
}

#[tokio::test]
async fn test_pattern_execution_stress() {
    // Stress test: Execute patterns many times
    let (pattern_id, executor) = advanced_control::create_pattern_26();
    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: std::collections::HashSet::new(),
        scope_id: String::new(),
    };

    // Execute pattern 1000 times
    for _ in 0..1000 {
        let result = executor.execute(&context);
        assert!(result.success, "Pattern should execute successfully");
        assert_eq!(pattern_id.0, 26, "Pattern ID should remain 26");
    }
}

#[tokio::test]
async fn test_resource_allocation_stress() {
    // Stress test: Many concurrent allocations
    let allocator = ResourceAllocator::new();

    // Register 100 resources
    for i in 0..100 {
        let resource = Resource {
            id: ResourceId::new(),
            name: format!("Resource {}", i),
            roles: vec![Role {
                id: "worker".to_string(),
                name: "Worker".to_string(),
                capabilities: vec![],
            }],
            capabilities: vec![],
            workload: 0,
            queue_length: i % 10,
            available: true,
        };

        allocator
            .register_resource(resource)
            .await
            .expect("Failed to register resource");
    }

    // Perform 1000 allocations
    for i in 0..1000 {
        let request = AllocationRequest {
            task_id: format!("task_{}", i),
            spec_id: WorkflowSpecId::new(),
            required_roles: vec!["worker".to_string()],
            required_capabilities: vec![],
            policy: AllocationPolicy::RoundRobin,
            priority: (i % 256) as u8,
        };

        let result = allocator.allocate(request).await;
        assert!(result.is_ok(), "Allocation {} should succeed", i);
    }
}

#[tokio::test]
async fn test_resource_allocation_boundary_conditions() {
    // Stress test: Boundary conditions
    let allocator = ResourceAllocator::new();

    // Test with zero resources
    let request = AllocationRequest {
        task_id: "task1".to_string(),
        spec_id: WorkflowSpecId::new(),
        required_roles: vec![],
        required_capabilities: vec![],
        policy: AllocationPolicy::RoundRobin,
        priority: 100,
    };

    let result = allocator.allocate(request).await;
    assert!(
        result.is_err(),
        "Allocation should fail with zero resources"
    );

    // Test with exactly one resource
    let resource = Resource {
        id: ResourceId::new(),
        name: "Single Resource".to_string(),
        roles: vec![],
        capabilities: vec![],
        workload: 0,
        queue_length: 0,
        available: true,
    };

    allocator
        .register_resource(resource)
        .await
        .expect("Failed to register resource");

    let request = AllocationRequest {
        task_id: "task2".to_string(),
        spec_id: WorkflowSpecId::new(),
        required_roles: vec![],
        required_capabilities: vec![],
        policy: AllocationPolicy::RoundRobin,
        priority: 100,
    };

    let result = allocator.allocate(request).await;
    assert!(
        result.is_ok(),
        "Allocation should succeed with one resource"
    );

    // Test four-eyes with only one resource (should fail)
    let request = AllocationRequest {
        task_id: "task3".to_string(),
        spec_id: WorkflowSpecId::new(),
        required_roles: vec![],
        required_capabilities: vec![],
        policy: AllocationPolicy::FourEyes,
        priority: 100,
    };

    let result = allocator.allocate(request).await;
    assert!(
        result.is_err(),
        "Four-eyes should fail with only one resource"
    );
}

#[tokio::test]
async fn test_pattern_context_variable_stress() {
    // Stress test: Many variables in context
    let mut variables = HashMap::new();

    // Add 1000 variables
    for i in 0..1000 {
        variables.insert(format!("var_{}", i), format!("value_{}", i));
    }

    let context = PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables,
        arrived_from: std::collections::HashSet::new(),
        scope_id: String::new(),
    };

    let pattern_id = PatternId::new(26).expect("Invalid pattern ID");
    let rdf = serialize_context_to_rdf(&pattern_id, &context).expect("Failed to serialize context");

    assert!(!rdf.is_empty(), "RDF should not be empty");
    assert!(
        rdf.contains("pattern:hasVariables"),
        "RDF should contain variables"
    );
}

#[tokio::test]
async fn test_resource_workload_updates_stress() {
    // Stress test: Many workload updates
    let allocator = ResourceAllocator::new();

    let resource = Resource {
        id: ResourceId::new(),
        name: "Stress Resource".to_string(),
        roles: vec![],
        capabilities: vec![],
        workload: 0,
        queue_length: 0,
        available: true,
    };

    let resource_id = resource.id;
    allocator
        .register_resource(resource)
        .await
        .expect("Failed to register resource");

    // Update workload 1000 times
    for i in 0..1000 {
        let delta = if i % 2 == 0 { 1 } else { -1 };
        allocator
            .update_workload(resource_id, delta)
            .await
            .expect("Failed to update workload");
    }

    // Verify resource still exists and is accessible
    let retrieved = allocator.get_resource(resource_id).await;
    assert!(retrieved.is_ok(), "Resource should still be accessible");
}

#[tokio::test]
async fn test_all_patterns_metadata_stress() {
    // Stress test: Get and serialize all pattern metadata
    let metadata = get_all_pattern_metadata();
    assert_eq!(metadata.len(), 43, "Should have 43 patterns");

    // Serialize all metadata
    for meta in &metadata {
        let rdf = serialize_metadata_to_rdf(meta).expect("Failed to serialize metadata");
        assert!(!rdf.is_empty(), "RDF should not be empty");
        assert!(
            rdf.contains(&format!(
                "{}pattern:{}",
                WORKFLOW_PATTERN_NS, meta.pattern_id
            )),
            "RDF should contain pattern IRI"
        );
    }
}

#[tokio::test]
async fn test_concurrent_pattern_execution() {
    // Stress test: Simulate concurrent pattern execution
    let (pattern_id, executor) = advanced_control::create_pattern_26();

    // Create multiple contexts
    let contexts: Vec<PatternExecutionContext> = (0..100)
        .map(|i| {
            let mut vars = HashMap::new();
            vars.insert("index".to_string(), i.to_string());
            PatternExecutionContext {
                case_id: CaseId::new(),
                workflow_id: WorkflowSpecId::new(),
                variables: vars,
                arrived_from: std::collections::HashSet::new(),
                scope_id: String::new(),
            }
        })
        .collect();

    // Execute patterns sequentially (simulating concurrent execution)
    for context in &contexts {
        let result = executor.execute(context);
        assert!(result.success, "Pattern should execute successfully");
        assert_eq!(pattern_id.0, 26, "Pattern ID should remain 26");
    }
}

#[tokio::test]
async fn test_resource_availability_toggle_stress() {
    // Stress test: Toggle resource availability many times
    let allocator = ResourceAllocator::new();

    let resource = Resource {
        id: ResourceId::new(),
        name: "Toggle Resource".to_string(),
        roles: vec![],
        capabilities: vec![],
        workload: 0,
        queue_length: 0,
        available: true,
    };

    let resource_id = resource.id;
    allocator
        .register_resource(resource)
        .await
        .expect("Failed to register resource");

    // Toggle availability 1000 times
    for i in 0..1000 {
        let available = i % 2 == 0;
        allocator
            .set_availability(resource_id, available)
            .await
            .expect("Failed to set availability");

        // Verify state
        let retrieved = allocator.get_resource(resource_id).await.unwrap();
        assert_eq!(
            retrieved.available, available,
            "Availability should match toggle state"
        );
    }
}

#[tokio::test]
async fn test_mixed_allocation_policies_stress() {
    // Stress test: Mix different allocation policies
    let allocator = ResourceAllocator::new();

    // Register resources
    for i in 0..20 {
        let resource = Resource {
            id: ResourceId::new(),
            name: format!("Resource {}", i),
            roles: vec![Role {
                id: format!("role_{}", i % 3),
                name: format!("Role {}", i % 3),
                capabilities: vec![],
            }],
            capabilities: vec![],
            workload: 0,
            queue_length: i % 5,
            available: true,
        };

        allocator
            .register_resource(resource)
            .await
            .expect("Failed to register resource");
    }

    // Mix policies in sequence
    let policies = vec![
        AllocationPolicy::RoundRobin,
        AllocationPolicy::ShortestQueue,
        AllocationPolicy::RoleBased,
        AllocationPolicy::Chained,
        AllocationPolicy::FourEyes,
    ];

    for (i, policy) in policies.iter().cycle().take(100).enumerate() {
        let request = AllocationRequest {
            task_id: format!("task_{}", i),
            spec_id: WorkflowSpecId::new(),
            required_roles: vec!["role_0".to_string()],
            required_capabilities: vec![],
            policy: *policy,
            priority: (i % 256) as u8,
        };

        let result = allocator.allocate(request).await;

        if *policy == AllocationPolicy::FourEyes {
            assert!(result.is_ok(), "Four-eyes should succeed with 20 resources");
        } else {
            assert!(result.is_ok(), "Allocation should succeed");
        }
    }
}
