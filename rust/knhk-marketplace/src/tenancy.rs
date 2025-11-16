// Multi-Tenant Isolation and Management
// Logical and physical isolation between customers

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum TenancyError {
    #[error("Tenant not found")]
    NotFound,

    #[error("Invalid tenant configuration")]
    InvalidConfig,

    #[error("Isolation violation")]
    IsolationViolation,

    #[error("Resource quota exceeded")]
    QuotaExceeded,

    #[error("Access denied")]
    AccessDenied,
}

/// Tenant configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub customer_id: String,
    pub name: String,
    pub tier: String,
    /// CPU quota (millicores)
    pub cpu_quota: u32,
    /// Memory quota (MB)
    pub memory_quota: u32,
    /// Storage quota (GB)
    pub storage_quota: u32,
    /// API rate limit (requests/second)
    pub api_rate_limit: u32,
    /// Data isolation type
    pub isolation_type: IsolationType,
    pub enabled: bool,
}

/// Data isolation type
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum IsolationType {
    /// Logical isolation (same database, row-level security)
    Logical,
    /// Physical isolation (separate schema)
    Schema,
    /// Database isolation (separate database)
    Database,
    /// Hardware isolation (separate instance)
    Hardware,
}

impl Tenant {
    /// Create new tenant
    pub fn new(id: String, customer_id: String, name: String, tier: String) -> Self {
        Self {
            id,
            customer_id,
            name,
            tier,
            cpu_quota: 500,        // 0.5 CPU
            memory_quota: 512,     // 512 MB
            storage_quota: 10,     // 10 GB
            api_rate_limit: 100,   // 100 RPS
            isolation_type: IsolationType::Logical,
            enabled: true,
        }
    }

    /// With custom CPU quota
    pub fn with_cpu_quota(mut self, quota: u32) -> Self {
        self.cpu_quota = quota;
        self
    }

    /// With custom isolation type
    pub fn with_isolation(mut self, isolation: IsolationType) -> Self {
        self.isolation_type = isolation;
        self
    }

    /// Get resource summary
    pub fn get_resources(&self) -> TenantResources {
        TenantResources {
            cpu_millis: self.cpu_quota,
            memory_mb: self.memory_quota,
            storage_gb: self.storage_quota,
            api_rps: self.api_rate_limit,
        }
    }
}

/// Tenant resource allocation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantResources {
    pub cpu_millis: u32,
    pub memory_mb: u32,
    pub storage_gb: u32,
    pub api_rps: u32,
}

/// Tenant manager for multi-tenant isolation
#[derive(Debug)]
pub struct TenantManager {
    tenants: HashMap<String, Tenant>,
    /// Map from customer_id to tenant_ids
    customer_tenants: HashMap<String, Vec<String>>,
    /// Resource usage tracking
    resource_usage: HashMap<String, TenantResources>,
}

impl TenantManager {
    /// Create new tenant manager
    pub fn new() -> Self {
        Self {
            tenants: HashMap::new(),
            customer_tenants: HashMap::new(),
            resource_usage: HashMap::new(),
        }
    }

    /// Create tenant
    pub fn create_tenant(&mut self, tenant: Tenant) -> Result<Tenant, TenancyError> {
        // Phase 10 implementation stub
        // TODO: Implement tenant creation
        // Step 1: Validate tenant config
        // Step 2: Create isolated resources (schema/database/instance)
        // Step 3: Set up access controls
        // Step 4: Initialize monitoring

        let tenant_id = tenant.id.clone();
        let customer_id = tenant.customer_id.clone();

        self.tenants.insert(tenant_id.clone(), tenant.clone());
        self.customer_tenants
            .entry(customer_id)
            .or_insert_with(Vec::new)
            .push(tenant_id);

        self.resource_usage.insert(
            tenant.id.clone(),
            TenantResources {
                cpu_millis: 0,
                memory_mb: 0,
                storage_gb: 0,
                api_rps: 0,
            },
        );

        tracing::info!("Tenant created: {}", tenant.id);

        Ok(tenant)
    }

    /// Get tenant
    pub fn get_tenant(&self, id: &str) -> Result<Tenant, TenancyError> {
        self.tenants.get(id).cloned().ok_or(TenancyError::NotFound)
    }

    /// Delete tenant
    pub fn delete_tenant(&mut self, id: &str) -> Result<(), TenancyError> {
        // Phase 10 implementation stub
        // TODO: Implement tenant deletion
        // Step 1: Verify no active workloads
        // Step 2: Clean up resources
        // Step 3: Remove from all indexes

        self.tenants.remove(id);
        self.resource_usage.remove(id);

        tracing::info!("Tenant deleted: {}", id);

        Ok(())
    }

    /// Get customer tenants
    pub fn get_customer_tenants(&self, customer_id: &str) -> Result<Vec<Tenant>, TenancyError> {
        let tenant_ids = self.customer_tenants
            .get(customer_id)
            .ok_or(TenancyError::NotFound)?;

        Ok(tenant_ids.iter()
            .filter_map(|id| self.tenants.get(id).cloned())
            .collect())
    }

    /// Verify tenant access
    pub fn verify_access(&self, tenant_id: &str, customer_id: &str) -> Result<(), TenancyError> {
        // Phase 10 implementation stub
        // TODO: Implement access verification
        // Step 1: Get tenant
        // Step 2: Verify customer ownership
        // Step 3: Check if enabled

        let tenant = self.get_tenant(tenant_id)?;

        if tenant.customer_id != customer_id {
            return Err(TenancyError::AccessDenied);
        }

        if !tenant.enabled {
            return Err(TenancyError::AccessDenied);
        }

        Ok(())
    }

    /// Check resource quota
    pub fn check_quota(&self, tenant_id: &str, required_cpu: u32) -> Result<(), TenancyError> {
        // Phase 10 implementation stub
        // TODO: Implement quota checking
        // Step 1: Get tenant
        // Step 2: Get current usage
        // Step 3: Calculate remaining
        // Step 4: Verify sufficient resources

        let tenant = self.get_tenant(tenant_id)?;
        let usage = self.resource_usage.get(tenant_id);

        if let Some(usage) = usage {
            let available = tenant.cpu_quota.saturating_sub(usage.cpu_millis);
            if available < required_cpu {
                return Err(TenancyError::QuotaExceeded);
            }
        }

        Ok(())
    }

    /// Record resource usage
    pub fn record_usage(&mut self, tenant_id: &str, cpu_ms: u32, mem_mb: u32, storage_gb: u32) -> Result<(), TenancyError> {
        let resource = self.resource_usage.get_mut(tenant_id)
            .ok_or(TenancyError::NotFound)?;

        resource.cpu_millis += cpu_ms;
        resource.memory_mb += mem_mb;
        resource.storage_gb += storage_gb;

        tracing::trace!(
            "Resource usage recorded: {} cpu, {} mem, {} storage",
            cpu_ms,
            mem_mb,
            storage_gb
        );

        Ok(())
    }

    /// Get isolation guarantee
    pub fn get_isolation_level(&self, tenant_id: &str) -> Result<IsolationType, TenancyError> {
        let tenant = self.get_tenant(tenant_id)?;
        Ok(tenant.isolation_type)
    }
}

impl Default for TenantManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_creation() {
        let tenant = Tenant::new(
            "tenant1".to_string(),
            "cust1".to_string(),
            "Tenant 1".to_string(),
            "professional".to_string(),
        );

        assert_eq!(tenant.id, "tenant1");
        assert_eq!(tenant.customer_id, "cust1");
    }

    #[test]
    fn test_tenant_with_isolation() {
        let tenant = Tenant::new(
            "tenant1".to_string(),
            "cust1".to_string(),
            "Tenant 1".to_string(),
            "professional".to_string(),
        ).with_isolation(IsolationType::Database);

        assert_eq!(tenant.isolation_type, IsolationType::Database);
    }

    #[test]
    fn test_tenant_resources() {
        let tenant = Tenant::new(
            "tenant1".to_string(),
            "cust1".to_string(),
            "Tenant 1".to_string(),
            "professional".to_string(),
        );

        let resources = tenant.get_resources();
        assert_eq!(resources.cpu_millis, 500);
    }

    #[test]
    fn test_tenant_manager_creation() {
        let manager = TenantManager::new();
        assert_eq!(manager.tenants.len(), 0);
    }

    #[test]
    fn test_tenant_manager_create() {
        let mut manager = TenantManager::new();
        let tenant = Tenant::new(
            "tenant1".to_string(),
            "cust1".to_string(),
            "Tenant 1".to_string(),
            "professional".to_string(),
        );

        let result = manager.create_tenant(tenant);
        assert!(result.is_ok());
        assert_eq!(manager.tenants.len(), 1);
    }

    #[test]
    fn test_tenant_manager_get() {
        let mut manager = TenantManager::new();
        let tenant = Tenant::new(
            "tenant1".to_string(),
            "cust1".to_string(),
            "Tenant 1".to_string(),
            "professional".to_string(),
        );

        manager.create_tenant(tenant).unwrap();
        let result = manager.get_tenant("tenant1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tenant_manager_verify_access() {
        let mut manager = TenantManager::new();
        let tenant = Tenant::new(
            "tenant1".to_string(),
            "cust1".to_string(),
            "Tenant 1".to_string(),
            "professional".to_string(),
        );

        manager.create_tenant(tenant).unwrap();
        let result = manager.verify_access("tenant1", "cust1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tenant_manager_verify_access_denied() {
        let mut manager = TenantManager::new();
        let tenant = Tenant::new(
            "tenant1".to_string(),
            "cust1".to_string(),
            "Tenant 1".to_string(),
            "professional".to_string(),
        );

        manager.create_tenant(tenant).unwrap();
        let result = manager.verify_access("tenant1", "cust2");
        assert!(result.is_err());
    }

    #[test]
    fn test_tenant_manager_check_quota() {
        let mut manager = TenantManager::new();
        let tenant = Tenant::new(
            "tenant1".to_string(),
            "cust1".to_string(),
            "Tenant 1".to_string(),
            "professional".to_string(),
        );

        manager.create_tenant(tenant).unwrap();
        let result = manager.check_quota("tenant1", 100);
        assert!(result.is_ok());
    }
}
