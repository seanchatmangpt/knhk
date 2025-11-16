//! Prometheus metrics exporter
//!
//! Provides Prometheus-compatible metrics export for workflow engine.

use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// Metric sample
#[derive(Debug, Clone)]
pub struct MetricSample {
    pub name: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp_ms: u64,
}

/// Prometheus exporter for workflow metrics
pub struct PrometheusExporter {
    metrics: Arc<Mutex<Vec<MetricSample>>>,
}

impl PrometheusExporter {
    /// Create new Prometheus exporter
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record a counter metric
    pub fn record_counter(&self, name: String, value: f64, labels: HashMap<String, String>) {
        let sample = MetricSample {
            name,
            metric_type: MetricType::Counter,
            value,
            labels,
            timestamp_ms: Self::current_timestamp_ms(),
        };

        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push(sample);
        }
    }

    /// Record a gauge metric
    pub fn record_gauge(&self, name: String, value: f64, labels: HashMap<String, String>) {
        let sample = MetricSample {
            name,
            metric_type: MetricType::Gauge,
            value,
            labels,
            timestamp_ms: Self::current_timestamp_ms(),
        };

        if let Ok(mut metrics) = self.metrics.lock() {
            // For gauge, replace existing value with same name and labels
            metrics.retain(|m| {
                !(m.name == sample.name
                    && m.labels == sample.labels
                    && m.metric_type == MetricType::Gauge)
            });
            metrics.push(sample);
        }
    }

    /// Record a histogram metric
    pub fn record_histogram(&self, name: String, value: f64, labels: HashMap<String, String>) {
        let sample = MetricSample {
            name,
            metric_type: MetricType::Histogram,
            value,
            labels,
            timestamp_ms: Self::current_timestamp_ms(),
        };

        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push(sample);
        }
    }

    /// Export metrics in Prometheus text format
    pub fn export(&self) -> String {
        let metrics = self.metrics.lock().unwrap();
        let mut output = String::new();

        // Group metrics by name
        let mut grouped: HashMap<String, Vec<&MetricSample>> = HashMap::new();
        for metric in metrics.iter() {
            grouped
                .entry(metric.name.clone())
                .or_insert_with(Vec::new)
                .push(metric);
        }

        for (name, samples) in grouped.iter() {
            if samples.is_empty() {
                continue;
            }

            let metric_type = samples[0].metric_type;

            // Write HELP and TYPE
            output.push_str(&format!("# HELP {} Workflow engine metric\n", name));
            output.push_str(&format!(
                "# TYPE {} {}\n",
                name,
                match metric_type {
                    MetricType::Counter => "counter",
                    MetricType::Gauge => "gauge",
                    MetricType::Histogram => "histogram",
                }
            ));

            // Write samples
            match metric_type {
                MetricType::Counter | MetricType::Gauge => {
                    for sample in samples {
                        let labels_str = Self::format_labels(&sample.labels);
                        output.push_str(&format!("{}{} {}\n", name, labels_str, sample.value));
                    }
                }
                MetricType::Histogram => {
                    // For histogram, we need to aggregate into buckets
                    // For simplicity, just output the raw values for now
                    for sample in samples {
                        let labels_str = Self::format_labels(&sample.labels);
                        output.push_str(&format!(
                            "{}_bucket{} {}\n",
                            name, labels_str, sample.value
                        ));
                    }
                }
            }

            output.push('\n');
        }

        output
    }

    /// Format labels for Prometheus text format
    fn format_labels(labels: &HashMap<String, String>) -> String {
        if labels.is_empty() {
            return String::new();
        }

        let mut parts: Vec<String> = labels
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect();
        parts.sort();

        format!("{{{}}}", parts.join(","))
    }

    /// Get current timestamp in milliseconds
    fn current_timestamp_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Clear all metrics (useful for testing)
    pub fn clear(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
    }

    /// HTTP handler for /metrics endpoint
    #[cfg(feature = "http")]
    pub async fn metrics_handler(
        self: Arc<Self>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.export())
    }
}

impl Default for PrometheusExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_exporter_counter() {
        let exporter = PrometheusExporter::new();

        let mut labels = HashMap::new();
        labels.insert("status".to_string(), "success".to_string());

        exporter.record_counter("test_counter_total".to_string(), 42.0, labels);

        let output = exporter.export();
        assert!(output.contains("# HELP test_counter_total"));
        assert!(output.contains("# TYPE test_counter_total counter"));
        assert!(output.contains("test_counter_total{status=\"success\"} 42"));
    }

    #[test]
    fn test_prometheus_exporter_gauge() {
        let exporter = PrometheusExporter::new();

        let labels = HashMap::new();
        exporter.record_gauge("test_gauge".to_string(), 10.0, labels.clone());
        exporter.record_gauge("test_gauge".to_string(), 20.0, labels);

        let output = exporter.export();
        assert!(output.contains("# TYPE test_gauge gauge"));
        // Should only have the latest value (20.0)
        assert!(output.contains("test_gauge 20"));
        assert!(!output.contains("test_gauge 10"));
    }

    #[test]
    fn test_format_labels() {
        let mut labels = HashMap::new();
        labels.insert("method".to_string(), "GET".to_string());
        labels.insert("status".to_string(), "200".to_string());

        let formatted = PrometheusExporter::format_labels(&labels);
        // Labels should be sorted alphabetically
        assert_eq!(formatted, "{method=\"GET\",status=\"200\"}");
    }

    #[test]
    fn test_clear_metrics() {
        let exporter = PrometheusExporter::new();

        exporter.record_counter("test".to_string(), 1.0, HashMap::new());
        assert!(!exporter.export().is_empty());

        exporter.clear();
        assert_eq!(exporter.export(), "");
    }
}
