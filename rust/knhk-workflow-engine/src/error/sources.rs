//! Custom error sources for different layers
//!
//! Provides error types for:
//! - State store operations
//! - RDF validation
//! - Connector operations

use thiserror::Error;

/// State store errors
#[derive(Error, Debug)]
pub enum StateStoreError {
    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// RDF store error
    #[cfg(feature = "rdf")]
    #[error("RDF store error")]
    RdfError(#[from] oxigraph::store::StorageError),

    /// Serialization error
    #[error("Serialization error")]
    SerializationError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error")]
    IoError(#[from] std::io::Error),

    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Invalid data format
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),
}

#[cfg(feature = "storage")]
impl From<sled::Error> for StateStoreError {
    fn from(err: sled::Error) -> Self {
        StateStoreError::DatabaseError(err.to_string())
    }
}

/// RDF validation errors
#[derive(Error, Debug)]
pub enum RdfValidationError {
    /// Invalid triple
    #[error("Invalid triple: {0}")]
    InvalidTriple(String),

    /// Schema violation
    #[error("Schema violation: {0}")]
    SchemaViolation(String),

    /// SPARQL query error
    #[cfg(feature = "rdf")]
    #[error("SPARQL query error")]
    QueryError(#[from] oxigraph::sparql::QueryEvaluationError),

    /// Parse error
    #[error("RDF parse error: {0}")]
    ParseError(String),

    /// Missing required property
    #[error("Missing required property: {0}")]
    MissingProperty(String),

    /// Type mismatch
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        /// Expected type
        expected: String,
        /// Actual type
        actual: String,
    },
}

/// Connector errors
#[derive(Error, Debug)]
pub enum ConnectorError {
    /// Execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Circuit breaker open
    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Health check failed
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: retry after {retry_after_ms}ms")]
    RateLimitExceeded {
        /// Milliseconds to wait before retry
        retry_after_ms: u64,
    },
}

/// Registry errors
#[derive(Error, Debug)]
pub enum RegistryError {
    /// Already registered
    #[error("Already registered: {0}")]
    AlreadyRegistered(String),

    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Initialization failed
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
}

/// Pool errors
#[derive(Error, Debug)]
pub enum PoolError {
    /// Pool exhausted
    #[error("Connection pool exhausted")]
    Exhausted,

    /// Invalid connector
    #[error("Invalid connector: {0}")]
    InvalidConnector(String),

    /// Timeout
    #[error("Pool timeout")]
    Timeout,
}

/// Retry errors
#[derive(Error, Debug)]
pub enum RetryError {
    /// Max retries exceeded
    #[error("Max retries exceeded: {0} attempts")]
    MaxRetriesExceeded(u32),

    /// Permanent failure
    #[error("Permanent failure: {0}")]
    PermanentFailure(String),
}

/// Circuit breaker errors
#[derive(Error, Debug)]
pub enum CircuitBreakerError {
    /// Circuit breaker open
    #[error("Circuit breaker is open")]
    Open,

    /// Failure
    #[error("Circuit breaker failure: {0}")]
    Failure(String),

    /// State transition failed
    #[error("State transition failed")]
    StateTransitionFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_store_error_display() {
        let error = StateStoreError::KeyNotFound("test-key".to_string());
        assert_eq!(error.to_string(), "Key not found: test-key");
    }

    #[test]
    fn test_rdf_validation_error_display() {
        let error = RdfValidationError::TypeMismatch {
            expected: "String".to_string(),
            actual: "Integer".to_string(),
        };
        assert!(error.to_string().contains("expected String"));
        assert!(error.to_string().contains("got Integer"));
    }

    #[test]
    fn test_connector_error_display() {
        let error = ConnectorError::RateLimitExceeded {
            retry_after_ms: 5000,
        };
        assert!(error.to_string().contains("5000"));
    }

    #[test]
    fn test_registry_error_display() {
        let error = RegistryError::AlreadyRegistered("my-connector".to_string());
        assert!(error.to_string().contains("my-connector"));
    }

    #[test]
    fn test_pool_error_display() {
        let error = PoolError::Exhausted;
        assert_eq!(error.to_string(), "Connection pool exhausted");
    }

    #[test]
    fn test_retry_error_display() {
        let error = RetryError::MaxRetriesExceeded(5);
        assert!(error.to_string().contains("5 attempts"));
    }

    #[test]
    fn test_circuit_breaker_error_display() {
        let error = CircuitBreakerError::Open;
        assert_eq!(error.to_string(), "Circuit breaker is open");
    }
}
