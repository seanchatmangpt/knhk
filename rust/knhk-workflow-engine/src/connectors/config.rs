// Connector Configuration System
//
// YAML-based connector configuration with validation and instantiation.

use crate::connectors::core::DynamicConnector;
use crate::connectors::resilience::{BackoffStrategy, RetryPolicy};
use crate::connectors::{DatabaseConfig, DatabaseConnector, MQConfig, MessageQueueConnector, RestConfig, RestConnector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tracing::{debug, info};

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicyConfig {
    pub max_retries: u32,
    pub backoff: BackoffStrategyConfig,
    #[serde(default = "default_jitter")]
    pub jitter: bool,
}

fn default_jitter() -> bool {
    true
}

impl From<RetryPolicyConfig> for RetryPolicy {
    fn from(config: RetryPolicyConfig) -> Self {
        Self {
            max_retries: config.max_retries,
            backoff: config.backoff.into(),
            jitter: config.jitter,
        }
    }
}

/// Backoff strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BackoffStrategyConfig {
    Fixed {
        delay_ms: u64,
    },
    Exponential {
        base_ms: u64,
        multiplier: f64,
        max_delay_ms: u64,
    },
    Linear {
        base_ms: u64,
        increment_ms: u64,
    },
}

impl From<BackoffStrategyConfig> for BackoffStrategy {
    fn from(config: BackoffStrategyConfig) -> Self {
        match config {
            BackoffStrategyConfig::Fixed { delay_ms } => Self::Fixed { delay_ms },
            BackoffStrategyConfig::Exponential { base_ms, multiplier, max_delay_ms } => {
                Self::Exponential { base_ms, multiplier, max_delay_ms }
            }
            BackoffStrategyConfig::Linear { base_ms, increment_ms } => {
                Self::Linear { base_ms, increment_ms }
            }
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub threshold: u32,
    pub timeout_ms: u64,
}

/// Connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub name: String,
    pub connector_type: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub retry_policy: Option<RetryPolicyConfig>,
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    pub timeout_ms: u64,
    pub config_data: serde_json::Value,
}

fn default_enabled() -> bool {
    true
}

/// Configuration error
#[derive(Debug)]
pub enum ConfigError {
    InvalidType(String),
    Validation(String),
    Instantiation(String),
    Serialization(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidType(msg) => write!(f, "Invalid connector type: {}", msg),
            Self::Validation(msg) => write!(f, "Validation error: {}", msg),
            Self::Instantiation(msg) => write!(f, "Instantiation error: {}", msg),
            Self::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl ConnectorConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.name.is_empty() {
            return Err(ConfigError::Validation("Connector name cannot be empty".to_string()));
        }

        if self.connector_type.is_empty() {
            return Err(ConfigError::Validation("Connector type cannot be empty".to_string()));
        }

        if self.timeout_ms == 0 {
            return Err(ConfigError::Validation("Timeout must be greater than 0".to_string()));
        }

        Ok(())
    }

    /// Instantiate a connector from this configuration
    pub async fn instantiate(&self) -> Result<Box<dyn DynamicConnector>, ConfigError> {
        self.validate()?;

        info!(
            name = %self.name,
            connector_type = %self.connector_type,
            "Instantiating connector"
        );

        match self.connector_type.as_str() {
            "rest" => self.instantiate_rest(),
            "database" => self.instantiate_database(),
            "message_queue" => self.instantiate_message_queue(),
            unknown => Err(ConfigError::InvalidType(unknown.to_string())),
        }
    }

    fn instantiate_rest(&self) -> Result<Box<dyn DynamicConnector>, ConfigError> {
        // Deserialize config_data into RestConfig
        let mut rest_config: RestConfig = serde_json::from_value(self.config_data.clone())
            .map_err(|e| ConfigError::Serialization(e.to_string()))?;

        // Override timeout if specified
        rest_config.timeout_ms = self.timeout_ms;

        // Apply retry policy
        if let Some(retry_policy) = &self.retry_policy {
            rest_config.retry_policy = Some(retry_policy.clone().into());
        }

        // Apply circuit breaker
        if let Some(cb_config) = &self.circuit_breaker {
            rest_config.circuit_breaker_threshold = Some(cb_config.threshold);
            rest_config.circuit_breaker_timeout_ms = Some(cb_config.timeout_ms);
        }

        debug!("Creating REST connector");
        let connector = RestConnector::new(rest_config)
            .map_err(|e| ConfigError::Instantiation(e.to_string()))?;

        Ok(Box::new(connector))
    }

    fn instantiate_database(&self) -> Result<Box<dyn DynamicConnector>, ConfigError> {
        let db_config: DatabaseConfig = serde_json::from_value(self.config_data.clone())
            .map_err(|e| ConfigError::Serialization(e.to_string()))?;

        debug!("Creating database connector");
        let connector = DatabaseConnector::new(db_config)
            .map_err(|e| ConfigError::Instantiation(e.to_string()))?;

        Ok(Box::new(connector))
    }

    fn instantiate_message_queue(&self) -> Result<Box<dyn DynamicConnector>, ConfigError> {
        let mq_config: MQConfig = serde_json::from_value(self.config_data.clone())
            .map_err(|e| ConfigError::Serialization(e.to_string()))?;

        debug!("Creating message queue connector");
        let connector = MessageQueueConnector::new(mq_config)
            .map_err(|e| ConfigError::Instantiation(e.to_string()))?;

        Ok(Box::new(connector))
    }
}

/// Connector configuration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfigFile {
    pub connectors: Vec<ConnectorConfig>,
}

impl ConnectorConfigFile {
    /// Load from YAML file
    pub fn from_yaml(yaml: &str) -> Result<Self, ConfigError> {
        serde_yaml::from_str(yaml)
            .map_err(|e| ConfigError::Serialization(e.to_string()))
    }

    /// Load from JSON
    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(json)
            .map_err(|e| ConfigError::Serialization(e.to_string()))
    }

    /// Validate all connectors
    pub fn validate_all(&self) -> Result<(), ConfigError> {
        for connector in &self.connectors {
            connector.validate()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = ConnectorConfig {
            name: "test".to_string(),
            connector_type: "rest".to_string(),
            enabled: true,
            retry_policy: None,
            circuit_breaker: None,
            timeout_ms: 5000,
            config_data: serde_json::json!({
                "base_url": "http://localhost",
                "default_headers": {}
            }),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_name() {
        let config = ConnectorConfig {
            name: "".to_string(),
            connector_type: "rest".to_string(),
            enabled: true,
            retry_policy: None,
            circuit_breaker: None,
            timeout_ms: 5000,
            config_data: serde_json::json!({}),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_zero_timeout() {
        let config = ConnectorConfig {
            name: "test".to_string(),
            connector_type: "rest".to_string(),
            enabled: true,
            retry_policy: None,
            circuit_breaker: None,
            timeout_ms: 0,
            config_data: serde_json::json!({}),
        };

        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_instantiate_rest_connector() {
        let config = ConnectorConfig {
            name: "test-rest".to_string(),
            connector_type: "rest".to_string(),
            enabled: true,
            retry_policy: Some(RetryPolicyConfig {
                max_retries: 3,
                backoff: BackoffStrategyConfig::Fixed { delay_ms: 100 },
                jitter: false,
            }),
            circuit_breaker: Some(CircuitBreakerConfig {
                threshold: 5,
                timeout_ms: 10000,
            }),
            timeout_ms: 5000,
            config_data: serde_json::json!({
                "base_url": "http://localhost:8080",
                "timeout_ms": 5000,
                "default_headers": {}
            }),
        };

        let connector = config.instantiate().await;
        assert!(connector.is_ok());
    }

    #[tokio::test]
    async fn test_instantiate_database_connector() {
        let config = ConnectorConfig {
            name: "test-db".to_string(),
            connector_type: "database".to_string(),
            enabled: true,
            retry_policy: None,
            circuit_breaker: None,
            timeout_ms: 5000,
            config_data: serde_json::json!({
                "connection_string": "postgres://localhost/test",
                "max_connections": 10,
                "min_connections": 2,
                "connection_timeout_ms": 5000,
                "idle_timeout_ms": 60000
            }),
        };

        let connector = config.instantiate().await;
        assert!(connector.is_ok());
    }

    #[test]
    fn test_config_file_from_yaml() {
        let yaml = r#"
connectors:
  - name: api
    connector_type: rest
    enabled: true
    timeout_ms: 5000
    config_data:
      base_url: "http://api.example.com"
      default_headers: {}
  - name: db
    connector_type: database
    enabled: true
    timeout_ms: 10000
    config_data:
      connection_string: "postgres://localhost/db"
      max_connections: 10
      min_connections: 2
      connection_timeout_ms: 5000
      idle_timeout_ms: 60000
"#;

        let config_file = ConnectorConfigFile::from_yaml(yaml).unwrap();
        assert_eq!(config_file.connectors.len(), 2);
        assert!(config_file.validate_all().is_ok());
    }

    #[test]
    fn test_retry_policy_config_conversion() {
        let config = RetryPolicyConfig {
            max_retries: 5,
            backoff: BackoffStrategyConfig::Exponential {
                base_ms: 100,
                multiplier: 2.0,
                max_delay_ms: 10000,
            },
            jitter: true,
        };

        let policy: RetryPolicy = config.into();
        assert_eq!(policy.max_retries, 5);
        assert!(policy.jitter);
    }
}
