//! GAT-based Query Engine for Resource Selection - Hyper-Advanced Rust
//!
//! Implements a Generic Associated Type (GAT) query engine for efficient resource selection.
//! This enables compile-time query optimization and zero-cost abstractions.
//!
//! # Hyper-Advanced Patterns
//!
//! - Generic Associated Types (GATs) for type-safe query builders
//! - Const generics for compile-time query optimization
//! - Zero-cost abstractions (queries compile to efficient code)
//!
//! # TRIZ Principle 10: Prior Action
//!
//! Query plans are optimized at compile-time, reducing runtime overhead.

use crate::error::{WorkflowError, WorkflowResult};
use crate::resource::allocation::{Resource, ResourceId};
use crate::resource::yawl_resource::FilterContext;
use std::collections::HashMap;

/// Query trait with Generic Associated Type (GAT)
///
/// GATs allow associated types to be generic, enabling type-safe query builders.
pub trait ResourceQuery {
    /// Query result type (GAT)
    type Result<'a>
    where
        Self: 'a;

    /// Execute query against resources
    fn execute<'a>(&self, resources: &'a [Resource], context: &'a FilterContext) -> Self::Result<'a>;
}

/// Simple filter query - returns matching resource IDs
pub struct FilterQuery<F>
where
    F: Fn(&Resource, &FilterContext) -> bool,
{
    filter: F,
}

impl<F> FilterQuery<F>
where
    F: Fn(&Resource, &FilterContext) -> bool,
{
    /// Create new filter query
    pub fn new(filter: F) -> Self {
        Self { filter }
    }
}

impl<F> ResourceQuery for FilterQuery<F>
where
    F: Fn(&Resource, &FilterContext) -> bool,
{
    type Result<'a> = Vec<ResourceId>;

    fn execute<'a>(&self, resources: &'a [Resource], context: &'a FilterContext) -> Self::Result<'a> {
        resources
            .iter()
            .filter(|resource| (self.filter)(resource, context))
            .map(|r| r.id.clone())
            .collect()
    }
}

/// Composite query - combines multiple queries
///
/// # Hyper-Advanced Pattern: GAT Composition
///
/// GATs enable type-safe composition of queries.
pub struct CompositeQuery<Q1, Q2>
where
    Q1: ResourceQuery,
    Q2: ResourceQuery,
{
    query1: Q1,
    query2: Q2,
    operator: QueryCompositeOperator,
}

/// Query composite operator for combining queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryCompositeOperator {
    /// Intersection (AND) - resources matching both queries
    And,
    /// Union (OR) - resources matching either query
    Or,
    /// Difference (NOT) - resources in query1 but not query2
    Difference,
}

impl<Q1, Q2> CompositeQuery<Q1, Q2>
where
    Q1: ResourceQuery<Result<'a> = Vec<ResourceId>>,
    Q2: ResourceQuery<Result<'a> = Vec<ResourceId>>,
{
    /// Create new composite query
    pub fn new(query1: Q1, query2: Q2, operator: QueryCompositeOperator) -> Self {
        Self {
            query1,
            query2,
            operator,
        }
    }
}

impl<Q1, Q2> ResourceQuery for CompositeQuery<Q1, Q2>
where
    Q1: ResourceQuery<Result<'a> = Vec<ResourceId>>,
    Q2: ResourceQuery<Result<'a> = Vec<ResourceId>>,
{
    type Result<'a> = Vec<ResourceId>;

    fn execute<'a>(&self, resources: &'a [Resource], context: &'a FilterContext) -> Self::Result<'a> {
        let result1 = self.query1.execute(resources, context);
        let result2 = self.query2.execute(resources, context);

        match self.operator {
            QueryCompositeOperator::And => {
                // Intersection: resources in both result sets
                result1
                    .into_iter()
                    .filter(|id| result2.contains(id))
                    .collect()
            }
            QueryCompositeOperator::Or => {
                // Union: resources in either result set
                let mut union = result1;
                for id in result2 {
                    if !union.contains(&id) {
                        union.push(id);
                    }
                }
                union
            }
            QueryCompositeOperator::Difference => {
                // Difference: resources in result1 but not result2
                result1
                    .into_iter()
                    .filter(|id| !result2.contains(id))
                    .collect()
            }
        }
    }
}

/// Const-generic query optimizer
///
/// # Hyper-Advanced Pattern: Const Generics
///
/// Query optimization happens at compile-time using const generics.
pub struct OptimizedQuery<const OPTIMIZE: bool, Q>
where
    Q: ResourceQuery,
{
    query: Q,
}

impl<const OPTIMIZE: bool, Q> OptimizedQuery<OPTIMIZE, Q>
where
    Q: ResourceQuery,
{
    /// Create new optimized query
    pub fn new(query: Q) -> Self {
        Self { query }
    }
}

impl<const OPTIMIZE: bool, Q> ResourceQuery for OptimizedQuery<OPTIMIZE, Q>
where
    Q: ResourceQuery<Result<'a> = Vec<ResourceId>>,
{
    type Result<'a> = Vec<ResourceId>;

    fn execute<'a>(&self, resources: &'a [Resource], context: &'a FilterContext) -> Self::Result<'a> {
        let result = self.query.execute(resources, context);

        // Compile-time optimization: if OPTIMIZE is true, sort results for cache efficiency
        if OPTIMIZE {
            // Sort by resource ID for better cache locality
            let mut sorted = result;
            sorted.sort();
            sorted
        } else {
            result
        }
    }
}

/// Query builder for fluent API
///
/// # Hyper-Advanced Pattern: Builder Pattern with GATs
///
/// Enables type-safe query construction with compile-time guarantees.
pub struct QueryBuilder {
    resources: Vec<Resource>,
}

impl QueryBuilder {
    /// Create new query builder
    pub fn new(resources: Vec<Resource>) -> Self {
        Self { resources }
    }

    /// Filter by capability
    pub fn with_capability(self, capability: String) -> FilterQuery<impl Fn(&Resource, &FilterContext) -> bool> {
        FilterQuery::new(move |resource, _context| {
            resource
                .capabilities
                .iter()
                .any(|c| c.name == capability)
        })
    }

    /// Filter by role
    pub fn with_role(self, role: String) -> FilterQuery<impl Fn(&Resource, &FilterContext) -> bool> {
        FilterQuery::new(move |resource, _context| {
            resource.roles.iter().any(|r| r.id.to_string() == role)
        })
    }

    /// Filter by workload threshold
    pub fn with_max_workload(self, max_workload: usize) -> FilterQuery<impl Fn(&Resource, &FilterContext) -> bool> {
        FilterQuery::new(move |resource, _context| resource.queue_length <= max_workload)
    }

    /// Filter by availability
    pub fn available(self) -> FilterQuery<impl Fn(&Resource, &FilterContext) -> bool> {
        FilterQuery::new(|resource, _context| resource.available)
    }

    /// Execute query and return matching resources
    pub fn execute<Q>(&self, query: Q, context: &FilterContext) -> WorkflowResult<Vec<Resource>>
    where
        Q: ResourceQuery<Result<'a> = Vec<ResourceId>>,
    {
        let resource_ids = query.execute(&self.resources, context);
        let mut results = Vec::new();

        for id in resource_ids {
            if let Some(resource) = self.resources.iter().find(|r| r.id == id) {
                results.push(resource.clone());
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::allocation::{Capability, Role};

    fn create_test_resource(id: &str, capabilities: Vec<String>, available: bool) -> Resource {
        Resource {
            id: ResourceId::new(),
            name: id.to_string(),
            roles: vec![],
            capabilities: capabilities
                .into_iter()
                .map(|c| Capability {
                    id: c.clone(),
                    name: c,
                    level: 100,
                })
                .collect(),
            workload: 0,
            queue_length: if available { 0 } else { 10 },
            available,
        }
    }

    #[test]
    fn test_filter_query() {
        let resources = vec![
            create_test_resource("r1", vec!["skill1".to_string()], true),
            create_test_resource("r2", vec!["skill2".to_string()], true),
            create_test_resource("r3", vec!["skill1".to_string(), "skill2".to_string()], true),
        ];

        let context = FilterContext {
            required_roles: vec![],
            required_capabilities: vec![],
            task_id: "task1".to_string(),
            case_data: serde_json::json!({}),
        };

        let query = FilterQuery::new(|resource, _context| {
            resource
                .capabilities
                .iter()
                .any(|c| c.name == "skill1")
        });

        let results = query.execute(&resources, &context);
        assert_eq!(results.len(), 2); // r1 and r3
    }

    #[test]
    fn test_composite_query_and() {
        let resources = vec![
            create_test_resource("r1", vec!["skill1".to_string()], true),
            create_test_resource("r2", vec!["skill2".to_string()], true),
            create_test_resource("r3", vec!["skill1".to_string(), "skill2".to_string()], true),
        ];

        let context = FilterContext {
            required_roles: vec![],
            required_capabilities: vec![],
            task_id: "task1".to_string(),
            case_data: serde_json::json!({}),
        };

        let query1 = FilterQuery::new(|resource, _context| {
            resource
                .capabilities
                .iter()
                .any(|c| c.name == "skill1")
        });
        let query2 = FilterQuery::new(|resource, _context| {
            resource
                .capabilities
                .iter()
                .any(|c| c.name == "skill2")
        });

        let composite = CompositeQuery::new(query1, query2, QueryCompositeOperator::And);
        let results = composite.execute(&resources, &context);
        assert_eq!(results.len(), 1); // Only r3 has both skills
    }

    #[test]
    fn test_optimized_query() {
        let resources = vec![
            create_test_resource("r1", vec!["skill1".to_string()], true),
            create_test_resource("r2", vec!["skill1".to_string()], true),
        ];

        let context = FilterContext {
            required_roles: vec![],
            required_capabilities: vec![],
            task_id: "task1".to_string(),
            case_data: serde_json::json!({}),
        };

        let query = FilterQuery::new(|resource, _context| {
            resource
                .capabilities
                .iter()
                .any(|c| c.name == "skill1")
        });

        let optimized = OptimizedQuery::<true, _>::new(query);
        let results = optimized.execute(&resources, &context);
        assert_eq!(results.len(), 2);
        // Results should be sorted when OPTIMIZE is true
        assert!(results[0] <= results[1]);
    }
}

