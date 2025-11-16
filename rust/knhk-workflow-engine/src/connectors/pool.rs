// Connection Pool Management
//
// Provides connection pooling and reuse for connectors.

use crate::connectors::core::DynamicConnector;
use crate::connectors::error::PoolError;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn, instrument};

/// Pooled connector wrapper
pub struct PooledConnector {
    connector: Option<Box<dyn DynamicConnector>>,
    pool_name: String,
    pool: Arc<ConnectorPoolInner>,
    created_at: Instant,
}

impl PooledConnector {
    /// Execute the connector
    pub async fn execute(
        &self,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        self.connector
            .as_ref()
            .ok_or_else(|| Box::new(PoolError::InvalidConnector("Connector already released".to_string())) as Box<dyn std::error::Error + Send + Sync>)?
            .execute_dynamic(input)
            .await
    }

    /// Get age of this connection
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Check if connector is healthy
    pub fn is_healthy(&self) -> bool {
        self.connector
            .as_ref()
            .map(|c| c.is_healthy_dynamic())
            .unwrap_or(false)
    }
}

impl Drop for PooledConnector {
    fn drop(&mut self) {
        if let Some(connector) = self.connector.take() {
            let pool = self.pool.clone();
            let pool_name = self.pool_name.clone();

            // Return connector to pool asynchronously
            tokio::spawn(async move {
                pool.return_connector(pool_name, connector).await;
            });
        }
    }
}

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_size: usize,
    pub min_idle: usize,
    pub max_lifetime: Duration,
    pub idle_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_idle: 2,
            max_lifetime: Duration::from_secs(3600),    // 1 hour
            idle_timeout: Duration::from_secs(600),     // 10 minutes
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: usize,
    pub idle_connections: usize,
    pub active_connections: usize,
    pub reuse_count: u64,
    pub create_count: u64,
}

/// Inner pool structure
struct PoolInner {
    idle: Vec<(Box<dyn DynamicConnector>, Instant)>,
    config: PoolConfig,
    stats: PoolStats,
}

impl PoolInner {
    fn new(config: PoolConfig) -> Self {
        Self {
            idle: Vec::new(),
            config,
            stats: PoolStats {
                total_connections: 0,
                idle_connections: 0,
                active_connections: 0,
                reuse_count: 0,
                create_count: 0,
            },
        }
    }

    fn get_connector(&mut self) -> Option<Box<dyn DynamicConnector>> {
        // Remove expired connections
        let now = Instant::now();
        self.idle.retain(|(_, created)| {
            created.elapsed() < self.config.max_lifetime
        });

        // Get connector from idle pool
        if let Some((connector, _)) = self.idle.pop() {
            self.stats.idle_connections = self.idle.len();
            self.stats.active_connections += 1;
            self.stats.reuse_count += 1;
            debug!("Reusing pooled connector");
            Some(connector)
        } else {
            None
        }
    }

    fn return_connector(&mut self, connector: Box<dyn DynamicConnector>) {
        if self.idle.len() < self.config.max_size {
            self.idle.push((connector, Instant::now()));
            self.stats.idle_connections = self.idle.len();
            self.stats.active_connections = self.stats.active_connections.saturating_sub(1);
            debug!("Returned connector to pool");
        } else {
            self.stats.active_connections = self.stats.active_connections.saturating_sub(1);
            self.stats.total_connections = self.stats.total_connections.saturating_sub(1);
            debug!("Pool full, dropping connector");
        }
    }

    fn create_new(&mut self) -> bool {
        if self.stats.total_connections < self.config.max_size {
            self.stats.total_connections += 1;
            self.stats.active_connections += 1;
            self.stats.create_count += 1;
            true
        } else {
            false
        }
    }
}

/// Connector pool inner state
struct ConnectorPoolInner {
    pools: DashMap<String, Arc<tokio::sync::RwLock<PoolInner>>>,
    factories: DashMap<String, Arc<dyn Fn() -> Box<dyn DynamicConnector> + Send + Sync>>,
    semaphores: DashMap<String, Arc<Semaphore>>,
}

impl ConnectorPoolInner {
    async fn return_connector(&self, pool_name: String, connector: Box<dyn DynamicConnector>) {
        if let Some(pool_entry) = self.pools.get(&pool_name) {
            let mut pool = pool_entry.value().write().await;
            pool.return_connector(connector);

            // Release semaphore
            if let Some(sem) = self.semaphores.get(&pool_name) {
                sem.add_permits(1);
            }
        }
    }
}

/// Connector pool for managing connection lifecycle
pub struct ConnectorPool {
    inner: Arc<ConnectorPoolInner>,
}

impl ConnectorPool {
    /// Create a new connector pool
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ConnectorPoolInner {
                pools: DashMap::new(),
                factories: DashMap::new(),
                semaphores: DashMap::new(),
            }),
        }
    }

    /// Register a connector factory
    pub fn register<F>(&self, name: String, config: PoolConfig, factory: F)
    where
        F: Fn() -> Box<dyn DynamicConnector> + Send + Sync + 'static,
    {
        info!(name = %name, max_size = config.max_size, "Registering connector pool");

        let pool = Arc::new(tokio::sync::RwLock::new(PoolInner::new(config.clone())));
        self.inner.pools.insert(name.clone(), pool);
        self.inner.factories.insert(name.clone(), Arc::new(factory));
        self.inner.semaphores.insert(name.clone(), Arc::new(Semaphore::new(config.max_size)));
    }

    /// Get a connector from the pool
    #[instrument(skip(self), fields(name = %name))]
    pub async fn get(&self, name: &str) -> Result<PooledConnector, PoolError> {
        // Get semaphore
        let sem = self.inner.semaphores.get(name)
            .ok_or_else(|| PoolError::InvalidConnector(format!("Pool '{}' not found", name)))?;

        // Acquire permit (blocks if pool is exhausted)
        let _permit = sem.value().acquire().await
            .map_err(|_| PoolError::Exhausted)?;

        // Try to get from pool
        let pool_entry = self.inner.pools.get(name)
            .ok_or_else(|| PoolError::InvalidConnector(format!("Pool '{}' not found", name)))?;

        let mut pool = pool_entry.value().write().await;

        // Try to reuse existing connector
        if let Some(connector) = pool.get_connector() {
            drop(pool); // Release lock
            drop(_permit); // Permit is transferred to PooledConnector's Drop

            return Ok(PooledConnector {
                connector: Some(connector),
                pool_name: name.to_string(),
                pool: Arc::clone(&self.inner),
                created_at: Instant::now(),
            });
        }

        // Create new connector if allowed
        if !pool.create_new() {
            drop(pool);
            drop(_permit);
            warn!(name = %name, "Pool exhausted");
            return Err(PoolError::Exhausted);
        }

        drop(pool); // Release lock before creating connector

        // Get factory
        let factory = self.inner.factories.get(name)
            .ok_or_else(|| PoolError::InvalidConnector(format!("Factory for '{}' not found", name)))?;

        let connector = (factory.value())();
        drop(_permit); // Permit is transferred to PooledConnector's Drop

        debug!(name = %name, "Created new pooled connector");

        Ok(PooledConnector {
            connector: Some(connector),
            pool_name: name.to_string(),
            pool: Arc::clone(&self.inner),
            created_at: Instant::now(),
        })
    }

    /// Get pool statistics
    pub async fn stats(&self, name: &str) -> Result<PoolStats, PoolError> {
        let pool = self.inner.pools.get(name)
            .ok_or_else(|| PoolError::InvalidConnector(format!("Pool '{}' not found", name)))?;

        let pool_guard = pool.value().read().await;
        Ok(pool_guard.stats.clone())
    }

    /// Health check all pools
    pub async fn health_check_all(&self) -> Vec<(String, PoolStats)> {
        let mut results = Vec::new();

        for entry in self.inner.pools.iter() {
            let name = entry.key().clone();
            let pool = entry.value().read().await;
            results.push((name, pool.stats.clone()));
        }

        results
    }
}

impl Default for ConnectorPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::core::{Connector, AsyncConnector, ConnectorMetadata};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestInput;
    #[derive(Debug, Serialize, Deserialize)]
    struct TestOutput;
    #[derive(Debug, Serialize, Deserialize)]
    struct TestConfig;
    #[derive(Debug)]
    struct TestError;

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Test error")
        }
    }

    impl std::error::Error for TestError {}

    struct TestConnector;

    impl Connector for TestConnector {
        type Config = TestConfig;
        type Input = TestInput;
        type Output = TestOutput;
        type Error = TestError;

        fn execute(
            &self,
            _input: Self::Input,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Output, Self::Error>> + Send + '_>> {
            Box::pin(async { Ok(TestOutput) })
        }

        fn name(&self) -> &str {
            "test"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn capabilities(&self) -> Vec<&str> {
            vec!["test"]
        }
    }

    impl AsyncConnector for TestConnector {
        fn initialize(
            &mut self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }

        fn shutdown(
            &mut self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }

        fn is_healthy(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_pool_basic_usage() {
        let pool = ConnectorPool::new();

        pool.register(
            "test".to_string(),
            PoolConfig::default(),
            || Box::new(TestConnector),
        );

        let connector = pool.get("test").await.unwrap();
        assert!(connector.is_healthy());

        drop(connector); // Return to pool

        let stats = pool.stats("test").await.unwrap();
        assert_eq!(stats.create_count, 1);
        assert_eq!(stats.idle_connections, 1);
    }

    #[tokio::test]
    async fn test_pool_reuse() {
        let pool = ConnectorPool::new();

        pool.register(
            "test".to_string(),
            PoolConfig::default(),
            || Box::new(TestConnector),
        );

        // Get and release connector
        {
            let _connector = pool.get("test").await.unwrap();
        }

        // Get again - should reuse
        {
            let _connector = pool.get("test").await.unwrap();
        }

        let stats = pool.stats("test").await.unwrap();
        assert_eq!(stats.create_count, 1);
        assert_eq!(stats.reuse_count, 1);
    }

    #[tokio::test]
    async fn test_pool_exhaustion() {
        let pool = ConnectorPool::new();

        let config = PoolConfig {
            max_size: 2,
            ..Default::default()
        };

        pool.register("test".to_string(), config, || Box::new(TestConnector));

        let _conn1 = pool.get("test").await.unwrap();
        let _conn2 = pool.get("test").await.unwrap();

        // Should timeout waiting for available connection
        let timeout_result = tokio::time::timeout(
            Duration::from_millis(100),
            pool.get("test")
        ).await;

        assert!(timeout_result.is_err());
    }

    #[tokio::test]
    async fn test_pool_health_check() {
        let pool = ConnectorPool::new();

        pool.register("test1".to_string(), PoolConfig::default(), || Box::new(TestConnector));
        pool.register("test2".to_string(), PoolConfig::default(), || Box::new(TestConnector));

        let health = pool.health_check_all().await;
        assert_eq!(health.len(), 2);
    }
}
