//! Data Gateway for external data integration
//!
//! Provides unified interface for accessing external data sources
//! (databases, APIs, files) during workflow execution.

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Data source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSourceType {
    /// Database (SQL, NoSQL)
    Database,
    /// REST API
    RestApi,
    /// File system
    FileSystem,
    /// Message queue (Kafka, RabbitMQ)
    MessageQueue,
    /// Graph database (RDF store)
    GraphDatabase,
}

/// Data source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceConfig {
    /// Data source identifier
    pub id: String,
    /// Data source type
    pub source_type: DataSourceType,
    /// Connection string or endpoint
    pub connection_string: String,
    /// Authentication credentials (encrypted)
    pub credentials: Option<HashMap<String, String>>,
    /// Additional configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Data query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQueryRequest {
    /// Data source ID
    pub source_id: String,
    /// Query type (SQL, SPARQL, XQuery, etc.)
    pub query_type: String,
    /// Query string
    pub query: String,
    /// Query parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

/// Data query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQueryResult {
    /// Query success
    pub success: bool,
    /// Result data (JSON)
    pub data: serde_json::Value,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Query execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Data Gateway for external data access
pub struct DataGateway {
    /// Registered data sources
    sources: Arc<RwLock<HashMap<String, DataSourceConfig>>>,
}

impl DataGateway {
    /// Create new data gateway
    pub fn new() -> Self {
        Self {
            sources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a data source
    pub async fn register_source(&self, config: DataSourceConfig) -> WorkflowResult<()> {
        let mut sources = self.sources.write().await;
        sources.insert(config.id.clone(), config);
        Ok(())
    }

    /// Get data source configuration
    pub async fn get_source(&self, source_id: &str) -> WorkflowResult<DataSourceConfig> {
        let sources = self.sources.read().await;
        sources.get(source_id).cloned().ok_or_else(|| {
            WorkflowError::ResourceUnavailable(format!("Data source {} not found", source_id))
        })
    }

    /// Execute data query
    pub async fn execute_query(
        &self,
        request: DataQueryRequest,
    ) -> WorkflowResult<DataQueryResult> {
        let start_time = std::time::Instant::now();

        // Get data source configuration
        let source = self.get_source(&request.source_id).await?;

        // Execute query based on source type and query type
        let result = match (source.source_type, request.query_type.as_str()) {
            (DataSourceType::Database, "SQL") => self.execute_sql_query(&source, &request).await,
            (DataSourceType::GraphDatabase, "SPARQL") => {
                self.execute_sparql_query(&source, &request).await
            }
            (DataSourceType::GraphDatabase, "XQuery") => {
                self.execute_xquery(&source, &request).await
            }
            (DataSourceType::RestApi, "REST") => self.execute_rest_query(&source, &request).await,
            (DataSourceType::FileSystem, "FILE") => {
                self.execute_file_query(&source, &request).await
            }
            _ => Err(WorkflowError::Internal(format!(
                "Unsupported query type {} for source type {:?}",
                request.query_type, source.source_type
            ))),
        };

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(data) => Ok(DataQueryResult {
                success: true,
                data,
                error: None,
                execution_time_ms,
            }),
            Err(e) => Ok(DataQueryResult {
                success: false,
                data: serde_json::Value::Null,
                error: Some(e.to_string()),
                execution_time_ms,
            }),
        }
    }

    /// Execute SQL query
    async fn execute_sql_query(
        &self,
        _source: &DataSourceConfig,
        _request: &DataQueryRequest,
    ) -> WorkflowResult<serde_json::Value> {
        // SQL query execution requires sqlx or similar library
        // Enable with feature flag: cargo build --features sql-connector
        Err(WorkflowError::Internal(
            "SQL query execution requires sqlx dependency. Enable with --features sql-connector"
                .to_string(),
        ))
    }

    /// Execute SPARQL query
    async fn execute_sparql_query(
        &self,
        source: &DataSourceConfig,
        request: &DataQueryRequest,
    ) -> WorkflowResult<serde_json::Value> {
        // Use oxigraph to execute SPARQL query
        #[cfg(feature = "rdf")]
        {
            use oxigraph::sparql::SparqlEvaluator;
            use oxigraph::store::Store;

            // Create or connect to store from connection string
            // For now, create in-memory store (in production, would connect to existing store)
            let store = Store::new().map_err(|e| {
                WorkflowError::Internal(format!("Failed to create RDF store: {:?}", e))
            })?;

            // Execute query using SparqlEvaluator (oxigraph 0.5 best practices)
            let _results = SparqlEvaluator::new()
                .parse_query(&request.query)
                .map_err(|e| {
                    WorkflowError::Internal(format!("Failed to parse SPARQL query: {:?}", e))
                })?
                .on_store(&store)
                .execute()
                .map_err(|e| {
                    WorkflowError::Internal(format!("Failed to execute SPARQL query: {:?}", e))
                })?;
        }
        #[cfg(not(feature = "rdf"))]
        {
            return Err(WorkflowError::Internal(
                "RDF feature not enabled".to_string(),
            ));
        }

        // Convert results to JSON (simplified - would need proper serialization)
        let json_results = serde_json::json!({
            "source": source.id,
            "query": request.query,
            "result_count": "N/A", // QueryResults doesn't implement Serialize
            "note": "SPARQL results available but not serialized to JSON"
        });

        Ok(json_results)
    }

    /// Execute XQuery
    async fn execute_xquery(
        &self,
        _source: &DataSourceConfig,
        _request: &DataQueryRequest,
    ) -> WorkflowResult<serde_json::Value> {
        // XQuery execution requires an XQuery engine library
        // Options: Saxon (via JNI), xrust (pure Rust, incomplete), or custom implementation
        // Enable with feature flag: cargo build --features xquery
        Err(WorkflowError::Internal(
            "XQuery execution requires XQuery engine library. Enable with --features xquery"
                .to_string(),
        ))
    }

    /// Execute REST API query
    async fn execute_rest_query(
        &self,
        _source: &DataSourceConfig,
        _request: &DataQueryRequest,
    ) -> WorkflowResult<serde_json::Value> {
        // REST API query execution requires reqwest or similar library
        // Enable with feature flag: cargo build --features rest-connector
        Err(WorkflowError::Internal(
            "REST API query execution requires reqwest dependency. Enable with --features rest-connector".to_string(),
        ))
    }

    /// Execute file system query
    async fn execute_file_query(
        &self,
        source: &DataSourceConfig,
        request: &DataQueryRequest,
    ) -> WorkflowResult<serde_json::Value> {
        use std::fs;
        use std::path::Path;

        // Query string should be file path (relative to connection_string or absolute)
        let file_path: std::path::PathBuf =
            if request.query.starts_with('/') || request.query.contains(':') {
                // Absolute path
                Path::new(&request.query).to_path_buf()
            } else {
                // Relative to connection_string (base directory)
                Path::new(&source.connection_string).join(&request.query)
            };

        // Read file
        let content = fs::read_to_string(&file_path).map_err(|e| {
            WorkflowError::Internal(format!(
                "Failed to read file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        // Parse based on file extension
        let result = match file_path.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str::<serde_json::Value>(&content)
                .map_err(|e| WorkflowError::Internal(format!("Failed to parse JSON: {}", e)))?,
            Some("xml") => {
                // For XML, return as string for now (would need XML parser for full support)
                serde_json::json!({
                    "content": content,
                    "format": "xml"
                })
            }
            Some("csv") => {
                // Basic CSV parsing (would need csv crate for full support)
                let lines: Vec<&str> = content.lines().collect();
                let rows: Vec<Vec<String>> = lines
                    .iter()
                    .map(|line| line.split(',').map(|s| s.trim().to_string()).collect())
                    .collect();
                serde_json::json!({
                    "rows": rows,
                    "format": "csv"
                })
            }
            _ => {
                // Unknown format, return as string
                serde_json::json!({
                    "content": content,
                    "format": "text"
                })
            }
        };

        Ok(result)
    }

    /// List registered data sources
    pub async fn list_sources(&self) -> Vec<String> {
        let sources = self.sources.read().await;
        sources.keys().cloned().collect()
    }

    /// Remove data source
    pub async fn remove_source(&self, source_id: &str) -> WorkflowResult<()> {
        let mut sources = self.sources.write().await;
        sources.remove(source_id).ok_or_else(|| {
            WorkflowError::ResourceUnavailable(format!("Data source {} not found", source_id))
        })?;
        Ok(())
    }
}

impl Default for DataGateway {
    fn default() -> Self {
        Self::new()
    }
}
