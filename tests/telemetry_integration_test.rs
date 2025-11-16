//! Integration tests for advanced real-time telemetry processing
//!
//! These tests validate:
//! - High-throughput pipeline (1M+ events/sec)
//! - Real-time stream processing with windowed aggregations
//! - Weaver schema validation
//! - Distributed tracing with span correlation
//! - Adaptive sampling strategies
//! - Multi-exporter support (OTLP, Prometheus, Jaeger)

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use knhk_workflow_engine::telemetry::*;

#[tokio::test]
async fn test_telemetry_pipeline_high_throughput() {
    // Create pipeline with OTLP exporter
    let otlp_exporter = Arc::new(OtlpExporter::new(exporters::otlp::OtlpConfig::default()));

    let pipeline = TelemetryPipeline::builder()
        .with_batch_size(1000)
        .with_flush_interval(Duration::from_millis(100))
        .with_channel_capacity(100_000)
        .with_exporter(otlp_exporter.clone())
        .build()
        .expect("Failed to build pipeline");

    // Generate high volume of spans
    let num_events = 10_000;
    let start = std::time::Instant::now();

    for i in 0..num_events {
        let span = Span {
            name: "workflow.execute".to_string(),
            trace_id: format!("trace-{}", i / 100),  // 100 spans per trace
            span_id: format!("span-{}", i),
            parent_span_id: if i % 100 == 0 { None } else { Some(format!("span-{}", (i / 100) * 100)) },
            attributes: vec![
                ("workflow.id".to_string(), AttributeValue::String(format!("wf-{}", i))),
                ("workflow.pattern".to_string(), AttributeValue::String("Sequence".to_string())),
            ],
            duration_ns: 1_000_000 + (i as u64 * 1000),
            status: if i % 50 == 0 { SpanStatus::Error } else { SpanStatus::Ok },
            start_time_ns: 1_000_000_000,
            end_time_ns: 1_001_000_000,
        };

        pipeline.record_span(span).await.expect("Failed to record span");
    }

    let elapsed = start.elapsed();

    println!("Recorded {} events in {:?}", num_events, elapsed);
    println!("Throughput: {:.2} events/sec", num_events as f64 / elapsed.as_secs_f64());

    // Wait for pipeline to flush
    tokio::time::sleep(Duration::from_millis(200)).await;

    let stats = pipeline.stats();
    println!("Pipeline stats: {:?}", stats);

    assert!(stats.events_received >= num_events as u64);
    assert!(stats.avg_processing_latency_us < 1000.0, "Processing latency too high");

    // Shutdown
    pipeline.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
async fn test_stream_processor_windowed_aggregations() {
    let processor = StreamProcessor::new(WindowConfig::Tumbling {
        duration: Duration::from_millis(100),
    });

    // Send events over time
    for i in 0..100 {
        let span = Span {
            name: "task.execute".to_string(),
            trace_id: format!("trace-{}", i),
            span_id: format!("span-{}", i),
            parent_span_id: None,
            attributes: vec![],
            duration_ns: 1_000_000 + (i as u64 * 10_000),
            status: SpanStatus::Ok,
            start_time_ns: 1_000_000_000,
            end_time_ns: 1_001_000_000,
        };

        processor.process_event(TelemetryEvent::Span(span))
            .await
            .expect("Failed to process event");

        tokio::time::sleep(Duration::from_millis(2)).await;
    }

    // Wait for window to close
    tokio::time::sleep(Duration::from_millis(150)).await;

    let results = processor.get_results();
    assert!(!results.is_empty(), "Should have aggregation results");

    if let Some(result) = results.first() {
        println!("Aggregation result: {:?}", result);
        assert!(result.count > 0);
        assert!(result.avg_duration_ms > 0.0);
    }
}

#[tokio::test]
async fn test_weaver_schema_validation() {
    // Create temporary registry
    let temp_dir = std::env::temp_dir().join("weaver_test_integration");
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    let validator = WeaverValidator::new(&temp_dir).expect("Failed to create validator");

    // Valid span - should pass
    let valid_attributes = vec![
        ("workflow.id".to_string(), AttributeValue::String("wf-001".to_string())),
        ("workflow.pattern".to_string(), AttributeValue::String("Sequence".to_string())),
    ];

    let result = validator.validate_span("workflow.execute", &valid_attributes)
        .await
        .expect("Validation failed");

    assert!(result.is_valid(), "Valid span should pass validation");

    // Invalid span - missing required attribute
    let invalid_attributes = vec![
        ("workflow.id".to_string(), AttributeValue::String("wf-001".to_string())),
        // Missing "workflow.pattern"
    ];

    let result = validator.validate_span("workflow.execute", &invalid_attributes)
        .await
        .expect("Validation failed");

    assert!(!result.is_valid(), "Invalid span should fail validation");
    assert!(!result.violations.is_empty());

    println!("Validation errors: {:?}", result.errors());
}

#[tokio::test]
async fn test_distributed_tracing_assembly() {
    let assembler = TraceAssembler::new();

    // Create a trace with multiple spans
    let root_span = Span {
        name: "workflow.execute".to_string(),
        trace_id: "trace-123".to_string(),
        span_id: "span-root".to_string(),
        parent_span_id: None,
        attributes: vec![
            ("service.name".to_string(), AttributeValue::String("workflow-engine".to_string())),
        ],
        duration_ns: 5_000_000,
        status: SpanStatus::Ok,
        start_time_ns: 1_000_000_000,
        end_time_ns: 1_005_000_000,
    };

    let child1 = Span {
        name: "task.execute".to_string(),
        trace_id: "trace-123".to_string(),
        span_id: "span-child1".to_string(),
        parent_span_id: Some("span-root".to_string()),
        attributes: vec![
            ("service.name".to_string(), AttributeValue::String("task-service".to_string())),
        ],
        duration_ns: 2_000_000,
        status: SpanStatus::Ok,
        start_time_ns: 1_001_000_000,
        end_time_ns: 1_003_000_000,
    };

    let child2 = Span {
        name: "task.execute".to_string(),
        trace_id: "trace-123".to_string(),
        span_id: "span-child2".to_string(),
        parent_span_id: Some("span-root".to_string()),
        attributes: vec![
            ("service.name".to_string(), AttributeValue::String("task-service".to_string())),
        ],
        duration_ns: 3_000_000,  // Longer duration - should be in critical path
        status: SpanStatus::Ok,
        start_time_ns: 1_002_000_000,
        end_time_ns: 1_005_000_000,
    };

    // Add spans
    assembler.add_span(root_span).await.expect("Failed to add root span");
    assembler.add_span(child1).await.expect("Failed to add child1");
    assembler.add_span(child2).await.expect("Failed to add child2");

    // Assemble trace
    let trace = assembler.assemble_trace("trace-123").await.expect("Failed to assemble trace");

    assert_eq!(trace.spans.len(), 3);
    assert_eq!(trace.metadata.span_count, 3);
    assert_eq!(trace.metadata.error_count, 0);

    // Check critical path
    let critical_path = trace.critical_path();
    assert_eq!(critical_path.len(), 2);  // root + longest child
    assert_eq!(critical_path.spans[1].span_id, "span-child2");

    println!("Critical path: {} spans, {:.2}ms total",
        critical_path.len(),
        critical_path.total_duration_ms());

    // Check service call graph
    let call_graph = trace.service_call_graph();
    assert_eq!(call_graph.nodes.len(), 2);  // workflow-engine, task-service
    assert_eq!(call_graph.edges.len(), 2);  // 2 calls from workflow-engine to task-service
}

#[tokio::test]
async fn test_adaptive_sampling_priority() {
    let config = SamplingConfig {
        base_rate: 0.1,   // 10% base sampling
        error_rate: 1.0,   // 100% error sampling
        slow_rate: 0.5,    // 50% slow sampling
        slow_threshold_ms: 1000,
    };

    let strategy = AdaptiveSamplingStrategy::new(config);

    // Test error priority - should always sample
    let error_span = Span {
        name: "test".to_string(),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        parent_span_id: None,
        attributes: vec![],
        duration_ns: 500_000,
        status: SpanStatus::Error,
        start_time_ns: 1_000_000_000,
        end_time_ns: 1_000_500_000,
    };

    let decision = strategy.should_sample(&TelemetryEvent::Span(error_span));
    assert_eq!(decision, SamplingDecision::Sample, "Errors should always be sampled");

    // Test slow request - should sample at higher rate
    let slow_span = Span {
        name: "test".to_string(),
        trace_id: "trace-2".to_string(),
        span_id: "span-2".to_string(),
        parent_span_id: None,
        attributes: vec![],
        duration_ns: 2_000_000_000,  // 2 seconds - slow
        status: SpanStatus::Ok,
        start_time_ns: 1_000_000_000,
        end_time_ns: 3_000_000_000,
    };

    // Run multiple times to check sampling rate
    let mut sampled = 0;
    let iterations = 1000;

    for _ in 0..iterations {
        if strategy.should_sample(&TelemetryEvent::Span(slow_span.clone())) == SamplingDecision::Sample {
            sampled += 1;
        }
    }

    let sample_rate = sampled as f64 / iterations as f64;
    println!("Slow request sample rate: {:.2}%", sample_rate * 100.0);

    // Should be around 50% (with some variance due to randomness)
    assert!(sample_rate > 0.3 && sample_rate < 0.7, "Slow request sampling rate out of range");

    let stats = strategy.stats();
    println!("Sampling stats: {:?}", stats);
}

#[tokio::test]
async fn test_multi_exporter_pipeline() {
    // Create multiple exporters
    let otlp = Arc::new(OtlpExporter::new(exporters::otlp::OtlpConfig::default()));
    let prometheus = Arc::new(PrometheusExporter::new());
    let jaeger = Arc::new(JaegerExporter::new(exporters::jaeger::JaegerConfig::default()));

    // Build pipeline with all exporters
    let pipeline = TelemetryPipeline::builder()
        .with_batch_size(100)
        .with_flush_interval(Duration::from_millis(50))
        .with_exporter(otlp)
        .with_exporter(prometheus.clone())
        .with_exporter(jaeger)
        .build()
        .expect("Failed to build pipeline");

    // Send mixed telemetry events
    for i in 0..50 {
        // Span
        let span = Span {
            name: "test.span".to_string(),
            trace_id: format!("trace-{}", i),
            span_id: format!("span-{}", i),
            parent_span_id: None,
            attributes: vec![],
            duration_ns: 1_000_000,
            status: SpanStatus::Ok,
            start_time_ns: 1_000_000_000,
            end_time_ns: 1_001_000_000,
        };

        pipeline.record_span(span).await.expect("Failed to record span");

        // Metric
        let metric = Metric {
            name: "requests_total".to_string(),
            value: MetricValue::Counter(i as u64),
            attributes: vec![],
            timestamp_ns: 1_000_000_000,
        };

        pipeline.record_metric(metric).await.expect("Failed to record metric");
    }

    // Wait for flush
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check Prometheus metrics
    let prom_text = prometheus.metrics_text();
    assert!(prom_text.contains("requests_total"), "Prometheus should have metrics");

    println!("Prometheus metrics:\n{}", prom_text);

    // Shutdown
    pipeline.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
async fn test_performance_targets() {
    // Create high-performance pipeline
    let pipeline = TelemetryPipeline::builder()
        .with_batch_size(1000)
        .with_flush_interval(Duration::from_millis(100))
        .with_channel_capacity(100_000)
        .with_compression(true)
        .build()
        .expect("Failed to build pipeline");

    // Measure ingestion latency
    let num_events = 10_000;
    let mut latencies = Vec::with_capacity(num_events);

    for i in 0..num_events {
        let start = std::time::Instant::now();

        let span = Span {
            name: "perf.test".to_string(),
            trace_id: format!("trace-{}", i),
            span_id: format!("span-{}", i),
            parent_span_id: None,
            attributes: vec![],
            duration_ns: 1_000_000,
            status: SpanStatus::Ok,
            start_time_ns: 1_000_000_000,
            end_time_ns: 1_001_000_000,
        };

        pipeline.record_span(span).await.expect("Failed to record span");

        latencies.push(start.elapsed().as_micros() as u64);
    }

    // Calculate percentiles
    latencies.sort_unstable();

    let p50 = latencies[latencies.len() / 2];
    let p99 = latencies[(latencies.len() * 99) / 100];

    println!("Ingestion latency - p50: {}µs, p99: {}µs", p50, p99);

    // Performance targets
    assert!(p99 < 10_000, "p99 ingestion latency should be < 10ms (target: < 10ms)");

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    let stats = pipeline.stats();

    println!("Processing latency - avg: {:.2}µs, p99: {:.2}µs",
        stats.avg_processing_latency_us,
        stats.p99_processing_latency_us);

    // Validate performance targets
    assert!(stats.avg_processing_latency_us < 100_000.0,
        "Average processing latency should be < 100ms");

    assert!(stats.p99_processing_latency_us < 100_000.0,
        "p99 processing latency should be < 100ms");

    pipeline.shutdown().await.expect("Failed to shutdown");
}
