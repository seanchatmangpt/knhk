//! Prometheus metrics exporter
//!
//! Exports telemetry metrics in Prometheus format for scraping.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::debug;

use crate::telemetry::{TelemetryEvent, TelemetryResult, Metric, MetricValue};
use super::Exporter;

/// Prometheus exporter
pub struct PrometheusExporter {
    /// Metric registry (metric_name -> current value)
    registry: Arc<RwLock<HashMap<String, PrometheusMetric>>>,
}

/// Prometheus metric representation
#[derive(Debug, Clone)]
struct PrometheusMetric {
    /// Metric name
    name: String,

    /// Metric type (counter, gauge, histogram)
    metric_type: String,

    /// Current value(s)
    value: PrometheusValue,

    /// Help text
    help: String,

    /// Labels
    labels: HashMap<String, String>,
}

/// Prometheus metric value
#[derive(Debug, Clone)]
enum PrometheusValue {
    /// Counter value
    Counter(u64),

    /// Gauge value
    Gauge(f64),

    /// Histogram buckets
    Histogram {
        buckets: Vec<(f64, u64)>,  // (upper_bound, count)
        sum: f64,
        count: u64,
    },
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add or update a metric in the registry
    fn update_metric(&self, metric: &Metric) {
        let mut registry = self.registry.write();

        // Extract labels from attributes
        let labels: HashMap<String, String> = metric.attributes.iter()
            .filter_map(|(k, v)| {
                if let crate::telemetry::AttributeValue::String(s) = v {
                    Some((k.clone(), s.clone()))
                } else {
                    None
                }
            })
            .collect();

        let prom_value = match &metric.value {
            MetricValue::Counter(val) => PrometheusValue::Counter(*val),
            MetricValue::Gauge(val) => PrometheusValue::Gauge(*val),
            MetricValue::Histogram { buckets, counts, sum, count } => {
                let histogram_buckets: Vec<(f64, u64)> = buckets.iter()
                    .zip(counts.iter())
                    .map(|(bucket, count)| (*bucket, *count))
                    .collect();

                PrometheusValue::Histogram {
                    buckets: histogram_buckets,
                    sum: *sum,
                    count: *count,
                }
            }
        };

        let metric_type = match &metric.value {
            MetricValue::Counter(_) => "counter",
            MetricValue::Gauge(_) => "gauge",
            MetricValue::Histogram { .. } => "histogram",
        };

        registry.insert(
            metric.name.clone(),
            PrometheusMetric {
                name: metric.name.clone(),
                metric_type: metric_type.to_string(),
                value: prom_value,
                help: format!("Metric: {}", metric.name),
                labels,
            },
        );
    }

    /// Get metrics in Prometheus text format
    pub fn metrics_text(&self) -> String {
        let registry = self.registry.read();
        let mut output = String::new();

        for (_, metric) in registry.iter() {
            // Help text
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));

            // Type
            output.push_str(&format!("# TYPE {} {}\n", metric.name, metric.metric_type));

            // Labels string
            let labels_str = if metric.labels.is_empty() {
                String::new()
            } else {
                let labels: Vec<String> = metric.labels.iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                format!("{{{}}}", labels.join(","))
            };

            // Value(s)
            match &metric.value {
                PrometheusValue::Counter(val) => {
                    output.push_str(&format!("{}{} {}\n", metric.name, labels_str, val));
                }
                PrometheusValue::Gauge(val) => {
                    output.push_str(&format!("{}{} {}\n", metric.name, labels_str, val));
                }
                PrometheusValue::Histogram { buckets, sum, count } => {
                    // Bucket counts
                    for (upper_bound, bucket_count) in buckets {
                        output.push_str(&format!(
                            "{}_bucket{{le=\"{}\"{}}} {}\n",
                            metric.name,
                            upper_bound,
                            if labels_str.is_empty() { "" } else { &format!(",{}", &labels_str[1..labels_str.len()-1]) },
                            bucket_count
                        ));
                    }

                    // +Inf bucket
                    output.push_str(&format!(
                        "{}_bucket{{le=\"+Inf\"{}}} {}\n",
                        metric.name,
                        if labels_str.is_empty() { "" } else { &format!(",{}", &labels_str[1..labels_str.len()-1]) },
                        count
                    ));

                    // Sum
                    output.push_str(&format!("{}_sum{} {}\n", metric.name, labels_str, sum));

                    // Count
                    output.push_str(&format!("{}_count{} {}\n", metric.name, labels_str, count));
                }
            }

            output.push('\n');
        }

        output
    }

    /// Clear all metrics
    pub fn clear(&self) {
        let mut registry = self.registry.write();
        registry.clear();
    }

    /// Get metric count
    pub fn metric_count(&self) -> usize {
        self.registry.read().len()
    }
}

impl Default for PrometheusExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Exporter for PrometheusExporter {
    async fn export(&self, events: &[TelemetryEvent]) -> TelemetryResult<()> {
        for event in events {
            if let TelemetryEvent::Metric(metric) = event {
                self.update_metric(metric);
            }
        }

        debug!("Updated Prometheus registry with {} events", events.len());

        Ok(())
    }

    async fn flush(&self) -> TelemetryResult<()> {
        debug!("Prometheus exporter flush (no-op)");
        Ok(())
    }

    async fn shutdown(&self) -> TelemetryResult<()> {
        debug!("Shutting down Prometheus exporter");
        self.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::{Metric, MetricValue, AttributeValue};

    #[tokio::test]
    async fn test_prometheus_counter_export() {
        let exporter = PrometheusExporter::new();

        let metric = Metric {
            name: "requests_total".to_string(),
            value: MetricValue::Counter(42),
            attributes: vec![
                ("method".to_string(), AttributeValue::String("GET".to_string())),
                ("status".to_string(), AttributeValue::String("200".to_string())),
            ],
            timestamp_ns: 1000,
        };

        let events = vec![TelemetryEvent::Metric(metric)];

        exporter.export(&events).await.unwrap();

        let text = exporter.metrics_text();
        assert!(text.contains("requests_total"));
        assert!(text.contains("42"));
        assert!(text.contains("method=\"GET\""));
    }

    #[tokio::test]
    async fn test_prometheus_histogram_export() {
        let exporter = PrometheusExporter::new();

        let metric = Metric {
            name: "request_duration_seconds".to_string(),
            value: MetricValue::Histogram {
                buckets: vec![0.1, 0.5, 1.0, 5.0],
                counts: vec![10, 50, 100, 150],
                sum: 250.5,
                count: 200,
            },
            attributes: vec![],
            timestamp_ns: 1000,
        };

        let events = vec![TelemetryEvent::Metric(metric)];

        exporter.export(&events).await.unwrap();

        let text = exporter.metrics_text();
        assert!(text.contains("request_duration_seconds_bucket"));
        assert!(text.contains("request_duration_seconds_sum"));
        assert!(text.contains("request_duration_seconds_count"));
        assert!(text.contains("le=\"+Inf\""));
    }

    #[test]
    fn test_prometheus_text_format() {
        let exporter = PrometheusExporter::new();

        let metric = Metric {
            name: "test_gauge".to_string(),
            value: MetricValue::Gauge(3.14),
            attributes: vec![],
            timestamp_ns: 1000,
        };

        let registry = exporter.registry.write();
        drop(registry);

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            exporter.export(&[TelemetryEvent::Metric(metric)]).await.unwrap();
        });

        let text = exporter.metrics_text();

        assert!(text.contains("# HELP test_gauge"));
        assert!(text.contains("# TYPE test_gauge gauge"));
        assert!(text.contains("test_gauge 3.14"));
    }
}
