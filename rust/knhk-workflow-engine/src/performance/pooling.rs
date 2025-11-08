//! Connection pooling for workflow engine

use crate::error::{WorkflowError, WorkflowResult};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tracing::{debug, warn};

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum pool size
    pub min_size: usize,
    /// Maximum pool size
    pub max_size: usize,
    /// Connection timeout (seconds)
    pub timeout_secs: u64,
    /// Idle timeout (seconds)
    pub idle_timeout_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_size: 5,
            max_size: 100,
            timeout_secs: 30,
            idle_timeout_secs: 300,
        }
    }
}

/// Idle connection entry with timestamp
#[derive(Debug, Clone)]
struct IdleConnection {
    /// When this connection was returned to the pool
    returned_at: Instant,
}

/// Connection pool
pub struct ConnectionPool {
    config: PoolConfig,
    /// Active connections count
    active: Arc<Mutex<usize>>,
    /// Idle connections with timestamps
    idle: Arc<Mutex<Vec<IdleConnection>>>,
    /// Notifier for waiting connections
    notify: Arc<Notify>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: PoolConfig) -> Self {
        let min_size = config.min_size;
        let mut idle_connections = Vec::with_capacity(min_size);
        // Pre-populate idle pool with min_size connections
        for _ in 0..min_size {
            idle_connections.push(IdleConnection {
                returned_at: Instant::now(),
            });
        }

        Self {
            config,
            active: Arc::new(Mutex::new(0)),
            idle: Arc::new(Mutex::new(idle_connections)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Get a connection from the pool
    ///
    /// This method implements connection lifecycle management with:
    /// - Idle connection reuse (with validation)
    /// - New connection creation up to max_size
    /// - Timeout handling for pool exhaustion
    /// - Connection validation before returning
    pub async fn get_connection(&self) -> WorkflowResult<PoolConnection> {
        let timeout = Duration::from_secs(self.config.timeout_secs);
        let start = Instant::now();

        loop {
            // Clean up expired idle connections
            self.cleanup_idle_connections().await?;

            // Try to get a connection
            let connection_result = {
                let mut active = self.active.lock().map_err(|e| {
                    WorkflowError::Internal(format!("Failed to acquire active lock: {}", e))
                })?;
                let mut idle = self.idle.lock().map_err(|e| {
                    WorkflowError::Internal(format!("Failed to acquire idle lock: {}", e))
                })?;

                // Try to get from idle pool first (with validation)
                if let Some(idle_conn) = idle.pop() {
                    // Validate connection hasn't expired
                    let idle_timeout = Duration::from_secs(self.config.idle_timeout_secs);
                    if idle_conn.returned_at.elapsed() < idle_timeout {
                        // Connection is still valid
                        *active += 1;
                        debug!("Reusing idle connection (active: {}, idle: {})", *active, idle.len());
                        drop(active);
                        drop(idle);
                        return Ok(PoolConnection {
                            pool: Arc::new(self.clone()),
                            created_at: Instant::now(),
                        });
                    }
                    // Connection expired, will create new one below
                }

                // Check if we can create a new connection
                let total_connections = *active + idle.len();
                if total_connections < self.config.max_size {
                    *active += 1;
                    debug!("Creating new connection (active: {}, idle: {})", *active, idle.len());
                    drop(active);
                    drop(idle);
                    return Ok(PoolConnection {
                        pool: Arc::new(self.clone()),
                        created_at: Instant::now(),
                    });
                }

                // Pool is full
                drop(active);
                drop(idle);
                None
            };

            if let Some(conn) = connection_result {
                return Ok(conn);
            }

            // Check timeout
            if start.elapsed() >= timeout {
                return Err(WorkflowError::ResourceUnavailable(format!(
                    "Connection pool timeout: no connections available within {} seconds (max_size: {})",
                    self.config.timeout_secs,
                    self.config.max_size
                )));
            }

            // Wait for a connection to become available
            let notify = Arc::clone(&self.notify);
            let wait_timeout = timeout.saturating_sub(start.elapsed());
            tokio::select! {
                _ = notify.notified() => {
                    // Connection returned, try again
                    continue;
                }
                _ = tokio::time::sleep(wait_timeout.min(Duration::from_millis(100))) => {
                    // Timeout or check interval, continue loop
                    continue;
                }
            }
        }
    }

    /// Clean up expired idle connections
    async fn cleanup_idle_connections(&self) -> WorkflowResult<()> {
        let idle_timeout = Duration::from_secs(self.config.idle_timeout_secs);
        let mut idle = self.idle.lock().map_err(|e| {
            WorkflowError::Internal(format!("Failed to acquire idle lock: {}", e))
        })?;

        let initial_len = idle.len();
        idle.retain(|conn| conn.returned_at.elapsed() < idle_timeout);
        let removed = initial_len - idle.len();

        if removed > 0 {
            debug!("Cleaned up {} expired idle connections", removed);
        }

        Ok(())
    }

    /// Return a connection to the pool
    ///
    /// This method is called automatically when PoolConnection is dropped.
    /// It validates the connection before returning it to the idle pool.
    fn return_connection_internal(&self, connection_age: Duration) {
        // Validate connection before returning
        let max_age = Duration::from_secs(self.config.idle_timeout_secs * 2); // Allow connections up to 2x idle timeout

        if connection_age > max_age {
            warn!("Connection too old ({}s), not returning to pool", connection_age.as_secs());
            // Just decrement active count, don't return to pool
            if let Ok(mut active) = self.active.lock() {
                *active = active.saturating_sub(1);
            }
            return;
        }

        // Return to idle pool
        if let Ok(mut active) = self.active.lock() {
            *active = active.saturating_sub(1);
        } else {
            warn!("Failed to acquire pool lock when returning connection");
            return;
        }

        if let Ok(mut idle) = self.idle.lock() {
            idle.push(IdleConnection {
                returned_at: Instant::now(),
            });
            let active_count = self.active.lock().map(|a| *a).unwrap_or(0);
            debug!("Connection returned to pool (active: {}, idle: {})", active_count, idle.len());
        } else {
            warn!("Failed to acquire idle lock when returning connection");
            return;
        }

        // Notify waiting connections
        self.notify.notify_one();
    }

    /// Get pool stats
    pub fn stats(&self) -> WorkflowResult<PoolStats> {
        let active = self.active.lock().map_err(|e| {
            WorkflowError::Internal(format!("Failed to acquire active lock: {}", e))
        })?;
        let idle = self.idle.lock().map_err(|e| {
            WorkflowError::Internal(format!("Failed to acquire idle lock: {}", e))
        })?;
        Ok(PoolStats {
            active: *active,
            idle: idle.len(),
            max_size: self.config.max_size,
        })
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            active: Arc::clone(&self.active),
            idle: Arc::clone(&self.idle),
            notify: Arc::clone(&self.notify),
        }
    }
}

/// Pool connection handle
///
/// This handle manages a connection from the pool. When dropped,
/// the connection is automatically returned to the pool.
pub struct PoolConnection {
    pool: Arc<ConnectionPool>,
    /// When this connection was created
    created_at: Instant,
}

impl Drop for PoolConnection {
    fn drop(&mut self) {
        let connection_age = self.created_at.elapsed();
        let pool = Arc::clone(&self.pool);
        pool.return_connection_internal(connection_age);
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Active connections
    pub active: usize,
    /// Idle connections
    pub idle: usize,
    /// Maximum pool size
    pub max_size: usize,
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new(PoolConfig::default())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool_basic() {
        let pool = ConnectionPool::default();
        let stats = pool.stats().expect("stats should succeed");
        assert_eq!(stats.active, 0);
        assert_eq!(stats.idle, 5); // min_size from default config
    }

    #[tokio::test]
    async fn test_get_connection() {
        let pool = ConnectionPool::default();
        let conn = pool.get_connection().await.expect("should get connection");
        let stats = pool.stats().expect("stats should succeed");
        assert_eq!(stats.active, 1);
        assert_eq!(stats.idle, 4); // One connection taken from idle pool
        drop(conn);
        // Connection should be returned
        tokio::time::sleep(Duration::from_millis(10)).await;
        let stats = pool.stats().expect("stats should succeed");
        assert_eq!(stats.active, 0);
        assert_eq!(stats.idle, 5); // Connection returned
    }

    #[tokio::test]
    async fn test_pool_exhaustion() {
        let config = PoolConfig {
            min_size: 2,
            max_size: 3,
            timeout_secs: 1,
            idle_timeout_secs: 300,
        };
        let pool = ConnectionPool::new(config);
        
        // Exhaust the pool
        let conn1 = pool.get_connection().await.expect("should get connection");
        let conn2 = pool.get_connection().await.expect("should get connection");
        let conn3 = pool.get_connection().await.expect("should get connection");
        
        // Next connection should timeout
        let result = tokio::time::timeout(
            Duration::from_millis(1100),
            pool.get_connection()
        ).await;
        
        assert!(result.is_ok(), "timeout should occur");
        assert!(result.unwrap().is_err(), "should get timeout error");
        
        // Return connections
        drop(conn1);
        drop(conn2);
        drop(conn3);
    }

    #[tokio::test]
    async fn test_idle_timeout() {
        let config = PoolConfig {
            min_size: 1,
            max_size: 10,
            timeout_secs: 30,
            idle_timeout_secs: 1, // 1 second idle timeout
        };
        let pool = ConnectionPool::new(config);
        
        // Get and return a connection
        let conn = pool.get_connection().await.expect("should get connection");
        drop(conn);
        
        // Wait for idle timeout
        tokio::time::sleep(Duration::from_millis(1100)).await;
        
        // Cleanup should remove expired connection
        pool.cleanup_idle_connections().await.expect("cleanup should succeed");
        
        let stats = pool.stats().expect("stats should succeed");
        assert_eq!(stats.idle, 0, "expired connection should be removed");
    }

    #[tokio::test]
    async fn test_connection_validation() {
        let pool = ConnectionPool::default();
        let conn = pool.get_connection().await.expect("should get connection");
        
        // Connection should have valid timestamp
        assert!(conn.created_at.elapsed() < Duration::from_secs(1));
        
        drop(conn);
    }
}
