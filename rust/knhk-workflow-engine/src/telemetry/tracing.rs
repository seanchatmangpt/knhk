//! Distributed tracing with span correlation and critical path analysis
//!
//! This module implements distributed tracing capabilities including:
//! - Span correlation across services
//! - Trace assembly from individual spans
//! - Critical path analysis
//! - Latency breakdown

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use super::{TelemetryResult, TelemetryError, Span, SpanStatus};

/// Trace assembler for building complete traces from spans
pub struct TraceAssembler {
    /// Traces by trace_id
    traces: Arc<RwLock<HashMap<String, Trace>>>,

    /// Orphaned spans (parent not found yet)
    orphaned_spans: Arc<RwLock<HashMap<String, Vec<Span>>>>,
}

impl TraceAssembler {
    /// Create a new trace assembler
    pub fn new() -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
            orphaned_spans: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a span to the trace assembler
    pub async fn add_span(&self, span: Span) -> TelemetryResult<()> {
        let trace_id = span.trace_id.clone();

        let mut traces = self.traces.write();

        // Get or create trace
        let trace = traces.entry(trace_id.clone())
            .or_insert_with(|| Trace::new(trace_id.clone()));

        // Add span to trace
        trace.add_span(span.clone());

        // Check if this span resolves any orphaned spans
        let mut orphaned = self.orphaned_spans.write();
        if let Some(orphans) = orphaned.remove(&span.span_id) {
            for orphan in orphans {
                trace.add_span(orphan);
            }
        }

        // If this span has a parent that's not in the trace yet, mark as orphaned
        if let Some(ref parent_id) = span.parent_span_id {
            if !trace.has_span(parent_id) {
                orphaned.entry(parent_id.clone())
                    .or_insert_with(Vec::new)
                    .push(span);
            }
        }

        Ok(())
    }

    /// Get a trace by ID
    pub async fn get_trace(&self, trace_id: &str) -> Option<Trace> {
        let traces = self.traces.read();
        traces.get(trace_id).cloned()
    }

    /// Assemble and return a complete trace
    pub async fn assemble_trace(&self, trace_id: &str) -> TelemetryResult<Trace> {
        let trace = self.get_trace(trace_id).await
            .ok_or_else(|| TelemetryError::TracingError(
                format!("Trace not found: {}", trace_id)
            ))?;

        Ok(trace)
    }

    /// Get all traces
    pub fn get_all_traces(&self) -> Vec<Trace> {
        let traces = self.traces.read();
        traces.values().cloned().collect()
    }

    /// Clear old traces (garbage collection)
    pub fn clear_old_traces(&self, max_age_ns: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let mut traces = self.traces.write();
        traces.retain(|_, trace| {
            if let Some(root) = trace.root_span() {
                now - root.end_time_ns < max_age_ns
            } else {
                false
            }
        });
    }
}

impl Default for TraceAssembler {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete distributed trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    /// Trace ID
    pub trace_id: String,

    /// All spans in this trace
    pub spans: Vec<Span>,

    /// Span relationships (span_id -> parent_span_id)
    pub relationships: HashMap<String, Option<String>>,

    /// Trace metadata
    pub metadata: TraceMetadata,
}

/// Trace metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceMetadata {
    /// Total span count
    pub span_count: usize,

    /// Total duration (nanoseconds)
    pub total_duration_ns: u64,

    /// Root span duration (nanoseconds)
    pub root_duration_ns: u64,

    /// Error count
    pub error_count: usize,

    /// Service names involved
    pub services: HashSet<String>,

    /// Operation names
    pub operations: HashSet<String>,
}

impl Trace {
    /// Create a new trace
    pub fn new(trace_id: String) -> Self {
        Self {
            trace_id,
            spans: Vec::new(),
            relationships: HashMap::new(),
            metadata: TraceMetadata::default(),
        }
    }

    /// Add a span to the trace
    pub fn add_span(&mut self, span: Span) {
        // Add relationship
        self.relationships.insert(
            span.span_id.clone(),
            span.parent_span_id.clone(),
        );

        // Update metadata
        self.metadata.span_count += 1;

        if span.status == SpanStatus::Error {
            self.metadata.error_count += 1;
        }

        // Extract service name from attributes if available
        for (key, value) in &span.attributes {
            if key == "service.name" {
                if let super::AttributeValue::String(service) = value {
                    self.metadata.services.insert(service.clone());
                }
            }
        }

        self.metadata.operations.insert(span.name.clone());

        // Add span
        self.spans.push(span);

        // Recalculate total duration
        self.recalculate_duration();
    }

    /// Check if trace has a span with given ID
    pub fn has_span(&self, span_id: &str) -> bool {
        self.spans.iter().any(|s| s.span_id == span_id)
    }

    /// Get root span (span with no parent)
    pub fn root_span(&self) -> Option<&Span> {
        self.spans.iter()
            .find(|s| s.parent_span_id.is_none())
    }

    /// Get children of a span
    pub fn children_of(&self, span_id: &str) -> Vec<&Span> {
        self.spans.iter()
            .filter(|s| s.parent_span_id.as_ref() == Some(&span_id.to_string()))
            .collect()
    }

    /// Get critical path (longest latency path through the trace)
    pub fn critical_path(&self) -> CriticalPath {
        let root = match self.root_span() {
            Some(r) => r,
            None => {
                return CriticalPath {
                    spans: Vec::new(),
                    total_duration_ns: 0,
                };
            }
        };

        let mut path = vec![root.clone()];
        let mut current_span = root;
        let mut total_duration = root.duration_ns;

        // Follow the longest child path recursively
        loop {
            let children = self.children_of(&current_span.span_id);

            if children.is_empty() {
                break;
            }

            // Find child with longest duration
            let longest_child = children.iter()
                .max_by_key(|s| s.duration_ns);

            if let Some(&child) = longest_child {
                path.push(child.clone());
                total_duration += child.duration_ns;
                current_span = child;
            } else {
                break;
            }
        }

        CriticalPath {
            spans: path,
            total_duration_ns: total_duration,
        }
    }

    /// Get latency breakdown by operation
    pub fn latency_breakdown(&self) -> HashMap<String, u64> {
        let mut breakdown = HashMap::new();

        for span in &self.spans {
            *breakdown.entry(span.name.clone()).or_insert(0) += span.duration_ns;
        }

        breakdown
    }

    /// Get service call graph
    pub fn service_call_graph(&self) -> ServiceCallGraph {
        let mut graph = ServiceCallGraph {
            nodes: HashSet::new(),
            edges: Vec::new(),
        };

        for span in &self.spans {
            // Extract service name
            let service = span.attributes.iter()
                .find(|(k, _)| k == "service.name")
                .and_then(|(_, v)| match v {
                    super::AttributeValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "unknown".to_string());

            graph.nodes.insert(service.clone());

            // If has parent, create edge
            if let Some(ref parent_id) = span.parent_span_id {
                if let Some(parent) = self.spans.iter().find(|s| s.span_id == *parent_id) {
                    let parent_service = parent.attributes.iter()
                        .find(|(k, _)| k == "service.name")
                        .and_then(|(_, v)| match v {
                            super::AttributeValue::String(s) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_else(|| "unknown".to_string());

                    graph.edges.push(ServiceCallEdge {
                        from: parent_service,
                        to: service,
                        latency_ns: span.duration_ns,
                    });
                }
            }
        }

        graph
    }

    /// Recalculate trace duration
    fn recalculate_duration(&mut self) {
        if let Some(root) = self.root_span() {
            self.metadata.root_duration_ns = root.duration_ns;
        }

        // Total duration is sum of all root-level spans
        self.metadata.total_duration_ns = self.spans.iter()
            .filter(|s| s.parent_span_id.is_none())
            .map(|s| s.duration_ns)
            .sum();
    }

    /// Get span by ID
    pub fn get_span(&self, span_id: &str) -> Option<&Span> {
        self.spans.iter().find(|s| s.span_id == span_id)
    }
}

/// Critical path through a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalPath {
    /// Spans in the critical path (ordered)
    pub spans: Vec<Span>,

    /// Total duration of critical path (nanoseconds)
    pub total_duration_ns: u64,
}

impl CriticalPath {
    /// Get total duration in milliseconds
    pub fn total_duration_ms(&self) -> f64 {
        self.total_duration_ns as f64 / 1_000_000.0
    }

    /// Get span count
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    /// Check if path is empty
    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }
}

/// Service call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCallGraph {
    /// Service nodes
    pub nodes: HashSet<String>,

    /// Call edges
    pub edges: Vec<ServiceCallEdge>,
}

/// Service call edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCallEdge {
    /// Source service
    pub from: String,

    /// Target service
    pub to: String,

    /// Call latency (nanoseconds)
    pub latency_ns: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trace_assembly() {
        let assembler = TraceAssembler::new();

        let root_span = Span {
            name: "root".to_string(),
            trace_id: "trace-123".to_string(),
            span_id: "span-1".to_string(),
            parent_span_id: None,
            attributes: vec![],
            duration_ns: 1_000_000,
            status: SpanStatus::Ok,
            start_time_ns: 1000,
            end_time_ns: 2000,
        };

        let child_span = Span {
            name: "child".to_string(),
            trace_id: "trace-123".to_string(),
            span_id: "span-2".to_string(),
            parent_span_id: Some("span-1".to_string()),
            attributes: vec![],
            duration_ns: 500_000,
            status: SpanStatus::Ok,
            start_time_ns: 1500,
            end_time_ns: 2000,
        };

        assembler.add_span(root_span).await.unwrap();
        assembler.add_span(child_span).await.unwrap();

        let trace = assembler.assemble_trace("trace-123").await.unwrap();

        assert_eq!(trace.spans.len(), 2);
        assert_eq!(trace.metadata.span_count, 2);
    }

    #[tokio::test]
    async fn test_critical_path() {
        let mut trace = Trace::new("trace-123".to_string());

        let root = Span {
            name: "root".to_string(),
            trace_id: "trace-123".to_string(),
            span_id: "span-1".to_string(),
            parent_span_id: None,
            attributes: vec![],
            duration_ns: 1_000_000,
            status: SpanStatus::Ok,
            start_time_ns: 1000,
            end_time_ns: 2000,
        };

        let child1 = Span {
            name: "child1".to_string(),
            trace_id: "trace-123".to_string(),
            span_id: "span-2".to_string(),
            parent_span_id: Some("span-1".to_string()),
            attributes: vec![],
            duration_ns: 800_000,  // Longest child
            status: SpanStatus::Ok,
            start_time_ns: 1200,
            end_time_ns: 2000,
        };

        let child2 = Span {
            name: "child2".to_string(),
            trace_id: "trace-123".to_string(),
            span_id: "span-3".to_string(),
            parent_span_id: Some("span-1".to_string()),
            attributes: vec![],
            duration_ns: 500_000,
            status: SpanStatus::Ok,
            start_time_ns: 1500,
            end_time_ns: 2000,
        };

        trace.add_span(root);
        trace.add_span(child1);
        trace.add_span(child2);

        let critical_path = trace.critical_path();

        assert_eq!(critical_path.spans.len(), 2);  // root + longest child
        assert_eq!(critical_path.spans[1].name, "child1");
    }
}
