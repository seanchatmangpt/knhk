// Connector Framework Module
//
// Provides a trait-based plugin system for integrating external systems
// with workflow tasks. Supports REST APIs, databases, message queues, and more.

pub mod core;
pub mod rest;
pub mod database;
pub mod message_queue;
pub mod registry;
pub mod resilience;
pub mod config;
pub mod pool;
pub mod error;

pub use core::{Connector, AsyncConnector, DynamicConnector};
pub use rest::{RestConnector, RestConfig, RestRequest, RestResponse};
pub use database::{DatabaseConnector, DatabaseConfig, DatabaseQuery, DatabaseResult};
pub use message_queue::{MessageQueueConnector, MQConfig, Message, MessageAcknowledgment};
pub use registry::{ConnectorRegistry, HealthStatus};
pub use resilience::{RetryPolicy, CircuitBreaker, BackoffStrategy, CircuitState};
pub use config::{ConnectorConfig, RetryPolicyConfig, CircuitBreakerConfig};
pub use pool::{ConnectorPool, PooledConnector};
pub use error::{ConnectorError, RegistryError, PoolError};
