//! YAWL Resource Filters
//!
//! Implements YAWL resource filters with TRIZ Principle 40: Composite Materials
//! - Multiple filter types combined into composite filter
//!
//! Based on: org.yawlfoundation.yawl.resourcing.filters

use crate::error::{WorkflowError, WorkflowResult};
use crate::resource::allocation::types::{Capability, Resource, ResourceId, Role};
use crate::resourcing::three_phase::AllocationContext;
use std::sync::Arc;

/// Filter result
#[derive(Debug, Clone)]
pub struct FilterResult {
    /// Whether filter passed
    pub passed: bool,
    /// Filter reason
    pub reason: Option<String>,
}

/// Resource filter trait
pub trait ResourceFilter: Send + Sync {
    /// Filter a resource against context
    fn filter(
        &self,
        resource: &Resource,
        context: &AllocationContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorkflowResult<FilterResult>> + Send>>;
}

/// Filter type (TRIZ Principle 32: Color Changes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    /// Capability filter - Match skills
    Capability,
    /// Role filter - Job role
    Role,
    /// Organizational group filter - Team membership
    OrgGroup,
    /// Position filter - Hierarchy level
    Position,
    /// Experience filter - Min experience level
    WithExperience,
    /// Least queued filter - Workload-based
    LeastQueued,
    /// Familiarity filter - Previous case familiarity
    Familiarity,
    /// Availability filter - Online/offline status
    Availability,
    /// Pile filter - Shared queue eligibility
    Pile,
    /// Custom filter - User-defined logic
    Custom,
}

/// Capability filter
pub struct CapabilityFilter {
    required_capabilities: Vec<String>,
}

impl CapabilityFilter {
    pub fn new(required_capabilities: Vec<String>) -> Self {
        Self {
            required_capabilities,
        }
    }
}

impl ResourceFilter for CapabilityFilter {
    fn filter(
        &self,
        resource: &Resource,
        _context: &AllocationContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorkflowResult<FilterResult>> + Send>>
    {
        let required = self.required_capabilities.clone();
        let resource_capabilities: Vec<String> = resource
            .capabilities
            .iter()
            .map(|c| c.name.clone())
            .collect();

        Box::pin(async move {
            let passed = required
                .iter()
                .all(|req| resource_capabilities.contains(req));

            Ok(FilterResult {
                passed,
                reason: if passed {
                    None
                } else {
                    Some("Missing required capabilities".to_string())
                },
            })
        })
    }
}

/// Role filter
pub struct RoleFilter {
    required_roles: Vec<String>,
}

impl RoleFilter {
    pub fn new(required_roles: Vec<String>) -> Self {
        Self { required_roles }
    }
}

impl ResourceFilter for RoleFilter {
    fn filter(
        &self,
        resource: &Resource,
        _context: &AllocationContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorkflowResult<FilterResult>> + Send>>
    {
        let required = self.required_roles.clone();
        let resource_roles: Vec<String> = resource.roles.iter().map(|r| r.name.clone()).collect();

        Box::pin(async move {
            let passed = required.iter().any(|req| resource_roles.contains(req));

            Ok(FilterResult {
                passed,
                reason: if passed {
                    None
                } else {
                    Some("Missing required role".to_string())
                },
            })
        })
    }
}

/// Availability filter
pub struct AvailabilityFilter;

impl ResourceFilter for AvailabilityFilter {
    fn filter(
        &self,
        resource: &Resource,
        _context: &AllocationContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorkflowResult<FilterResult>> + Send>>
    {
        let available = resource.available;

        Box::pin(async move {
            Ok(FilterResult {
                passed: available,
                reason: if available {
                    None
                } else {
                    Some("Resource not available".to_string())
                },
            })
        })
    }
}

/// Composite filter (TRIZ Principle 40: Composite Materials)
///
/// Combines multiple filters with AND/OR logic
pub struct CompositeFilter {
    filters: Vec<Arc<dyn ResourceFilter>>,
    logic: FilterLogic,
}

/// Filter logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterLogic {
    /// All filters must pass (AND)
    And,
    /// Any filter must pass (OR)
    Or,
}

impl CompositeFilter {
    pub fn new(filters: Vec<Arc<dyn ResourceFilter>>, logic: FilterLogic) -> Self {
        Self { filters, logic }
    }
}

impl ResourceFilter for CompositeFilter {
    fn filter(
        &self,
        resource: &Resource,
        context: &AllocationContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorkflowResult<FilterResult>> + Send>>
    {
        let filters = self.filters.clone();
        let logic = self.logic;

        let resource_clone = resource.clone();
        let context_clone = context.clone();
        Box::pin(async move {
            let mut results = Vec::new();

            for filter in filters {
                let result = filter.filter(&resource_clone, &context_clone).await?;
                results.push(result.passed);
            }

            let passed = match logic {
                FilterLogic::And => results.iter().all(|&r| r),
                FilterLogic::Or => results.iter().any(|&r| r),
            };

            Ok(FilterResult {
                passed,
                reason: if passed {
                    None
                } else {
                    Some(format!("Composite filter failed ({:?})", logic))
                },
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::allocation::types::{Capability, Resource, ResourceId, Role};

    #[tokio::test]
    async fn test_capability_filter() {
        let resource = Resource {
            id: ResourceId::new(),
            name: "Resource 1".to_string(),
            capabilities: vec![Capability {
                id: "skill1".to_string(),
                name: "skill1".to_string(),
                level: 100,
            }],
            roles: vec![],
            workload: 0,
            queue_length: 0,
            available: true,
        };

        let filter = CapabilityFilter::new(vec!["skill1".to_string()]);
        let context = AllocationContext {
            task_id: "task1".to_string(),
            case_id: "case1".to_string(),
            required_capabilities: vec![],
            required_roles: vec![],
            workload_constraints: std::collections::HashMap::new(),
        };

        let result = filter.filter(&resource, &context).await.unwrap();
        assert!(result.passed);
    }
}
