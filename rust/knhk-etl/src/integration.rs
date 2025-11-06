// rust/knhk-etl/src/integration.rs
// Integration layer connecting ETL pipeline with connectors, lockchain, and OTEL

// Includes warm path query execution integration

extern crate alloc;
extern crate std;

use super::*;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Integrated pipeline with all components wired together
pub struct IntegratedPipeline {
    connectors: Vec<String>,
    schema_iri: String,
    lockchain_enabled: bool,
    downstream_endpoints: Vec<String>,
        warm_path_executor: Option<alloc::boxed::Box<dyn WarmPathQueryExecutor>>,
}

/// Trait for warm path query execution (abstracted for no_std compatibility)
pub trait WarmPathQueryExecutor: Send + Sync {
    fn execute_query(&self, sparql: &str) -> Result<WarmPathQueryResult, String>;
}

pub enum WarmPathQueryResult {
    Boolean(bool),
    Solutions(Vec<BTreeMap<String, String>>),
    Graph(Vec<String>),
}

impl IntegratedPipeline {
    pub fn new(
        connectors: Vec<String>,
        schema_iri: String,
        lockchain_enabled: bool,
        downstream_endpoints: Vec<String>,
    ) -> Self {
        Self {
            connectors,
            schema_iri,
            lockchain_enabled,
            downstream_endpoints,
                        warm_path_executor: None,
        }
    }

    /// Set warm path executor for SPARQL query execution
        pub fn set_warm_path_executor(&mut self, executor: alloc::boxed::Box<dyn WarmPathQueryExecutor>) {
        self.warm_path_executor = Some(executor);
    }

    /// Execute pipeline with full integration
    pub fn execute(&mut self) -> Result<IntegratedResult, PipelineError> {
        // Use the base Pipeline for execution
        let mut pipeline = Pipeline::new(
            self.connectors.clone(),
            self.schema_iri.clone(),
            self.lockchain_enabled,
            self.downstream_endpoints.clone(),
        );
        
        let result = pipeline.execute()?;
        
        // Record OTEL metrics using proper API
        let metrics_recorded = {
            {
                use knhk_otel::{Tracer, Metric, MetricValue, MetricsHelper};
                use std::time::{SystemTime, UNIX_EPOCH};
                
                let mut tracer = Tracer::new();
                
                // Record pipeline execution metrics using MetricsHelper
                MetricsHelper::record_connector_throughput(&mut tracer, "pipeline", result.actions_sent);
                
                // Record receipt generation
                if result.receipts_written > 0 {
                    MetricsHelper::record_receipt(&mut tracer, &format!("pipeline_batch_{}", result.receipts_written));
                }
                
                // Record lockchain writes
                let timestamp_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                
                for hash in &result.lockchain_hashes {
                    let metric = Metric {
                        name: "knhk.lockchain.entry".to_string(),
                        value: MetricValue::Counter(1),
                        timestamp_ms,
                        attributes: {
                            let mut attrs = alloc::collections::BTreeMap::new();
                            attrs.insert("hash".to_string(), hash.clone());
                            attrs
                        },
                    };
                    tracer.record_metric(metric);
                }
                
                tracer.metrics().len()
            }
            {
                0
            }
        };
        
        Ok(IntegratedResult {
            receipts_written: result.receipts_written,
            actions_sent: result.actions_sent,
            lockchain_hashes: result.lockchain_hashes,
            metrics_recorded,
            warm_path_queries_executed: 0,
        })
    }

    /// Execute warm path query if executor is available
        pub fn execute_warm_path_query(&self, sparql: &str) -> Result<WarmPathQueryResult, PipelineError> {
        if let Some(ref executor) = self.warm_path_executor {
            executor.execute_query(sparql)
                .map_err(|e| PipelineError::ReflexError(format!("Warm path query failed: {}", e)))
        } else {
            Err(PipelineError::ReflexError("Warm path executor not configured".to_string()))
        }
    }
}

pub struct IntegratedResult {
    pub receipts_written: usize,
    pub actions_sent: usize,
    pub lockchain_hashes: Vec<String>,
    pub metrics_recorded: usize,
    pub warm_path_queries_executed: usize,
}
