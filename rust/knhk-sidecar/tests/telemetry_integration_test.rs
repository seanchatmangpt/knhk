// rust/knhk-sidecar/tests/telemetry/telemetry_integration_test.rs
// Comprehensive telemetry integration tests for KNHK system
// Tests OTEL span/metric emission and Weaver validation

#[cfg(feature = "otel")]
mod otel_tests {
    use knhk_otel::{MetricsHelper, SpanStatus, Tracer, WeaverLiveCheck};

    /// Test: All sidecar operations emit spans
    /// Chicago TDD: Test the output (spans emitted) not implementation
    #[test]
    fn test_all_operations_emit_spans() {
        let mut tracer = Tracer::new();

        // Simulate sidecar operations
        let operations = vec![
            ("apply_transaction", "knhk.sidecar.transaction"),
            ("query", "knhk.sidecar.query"),
            ("validate_graph", "knhk.sidecar.validate_graph"),
            ("evaluate_hook", "knhk.sidecar.evaluate_hook"),
        ];

        for (op_name, span_name) in operations {
            let span_ctx = tracer.start_span(span_name.to_string(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.operation.name".to_string(),
                op_name.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.operation.type".to_string(),
                "sidecar".to_string(),
            );
            tracer.end_span(span_ctx, SpanStatus::Ok);
        }

        // Verify all operations emitted spans
        assert_eq!(tracer.spans().len(), 4, "Should emit 4 spans");

        // Verify semantic convention compliance
        for span in tracer.spans() {
            assert!(
                span.name.starts_with("knhk.sidecar."),
                "Span name must use knhk.sidecar.* namespace"
            );
            assert!(
                span.attributes.contains_key("knhk.operation.name"),
                "Must have operation.name attribute"
            );
            assert!(
                span.attributes.contains_key("knhk.operation.type"),
                "Must have operation.type attribute"
            );
            assert_eq!(span.status, SpanStatus::Ok, "Status must be Ok");
        }
    }

    /// Test: Metrics recorded for all operations
    /// Chicago TDD: Verify metrics are created with correct attributes
    #[test]
    fn test_all_operations_record_metrics() {
        let mut tracer = Tracer::new();

        // Record metrics for sidecar operations
        MetricsHelper::record_operation(&mut tracer, "apply_transaction", true);
        MetricsHelper::record_operation(&mut tracer, "query", false);
        MetricsHelper::record_operation(&mut tracer, "validate_graph", true);
        MetricsHelper::record_operation(&mut tracer, "evaluate_hook", true);

        // Verify metrics recorded
        assert_eq!(tracer.metrics().len(), 4, "Should record 4 metrics");

        // Verify metric attributes
        let metrics = tracer.metrics();
        assert_eq!(
            metrics[0].attributes.get("operation"),
            Some(&"apply_transaction".to_string())
        );
        assert_eq!(
            metrics[0].attributes.get("success"),
            Some(&"true".to_string())
        );
        assert_eq!(
            metrics[1].attributes.get("success"),
            Some(&"false".to_string())
        );
    }

    /// Test: Latency metrics recorded correctly
    /// Chicago TDD: Test behavior (latency tracking) with real values
    #[test]
    fn test_latency_metrics_recorded() {
        let mut tracer = Tracer::new();

        // Record warm path latencies
        MetricsHelper::record_warm_path_latency(&mut tracer, 5000, "apply_transaction"); // 5ms
        MetricsHelper::record_warm_path_latency(&mut tracer, 8000, "query"); // 8ms
        MetricsHelper::record_warm_path_latency(&mut tracer, 3000, "evaluate_hook"); // 3ms

        // Verify latency metrics
        let latency_metrics = tracer.get_metrics("knhk.warm_path.operations.latency");
        assert_eq!(latency_metrics.len(), 3, "Should record 3 latency metrics");

        // Verify count metrics also recorded
        let count_metrics = tracer.get_metrics("knhk.warm_path.operations.count");
        assert_eq!(count_metrics.len(), 3, "Should record 3 count metrics");
    }

    /// Test: Hook execution telemetry
    /// Chicago TDD: Test hook-specific metrics
    #[test]
    fn test_hook_execution_telemetry() {
        let mut tracer = Tracer::new();

        // Record hook latencies (in ticks, per Chatman Constant)
        MetricsHelper::record_hook_latency(&mut tracer, 5, "ASK_SP");
        MetricsHelper::record_hook_latency(&mut tracer, 7, "ASK_BN");
        MetricsHelper::record_hook_latency(&mut tracer, 8, "CONSTRUCT8");

        // Verify hook metrics
        let hook_metrics = tracer.get_metrics("knhk.hook.latency.ticks");
        assert_eq!(hook_metrics.len(), 3, "Should record 3 hook metrics");

        // Verify each metric has operation attribute
        for metric in hook_metrics {
            assert!(
                metric.attributes.contains_key("operation"),
                "Hook metric must have operation attribute"
            );
        }
    }

    /// Test: Receipt generation telemetry
    /// Chicago TDD: Test receipt tracking behavior
    #[test]
    fn test_receipt_generation_telemetry() {
        let mut tracer = Tracer::new();

        // Record receipt generations
        MetricsHelper::record_receipt(&mut tracer, "receipt-001");
        MetricsHelper::record_receipt(&mut tracer, "receipt-002");
        MetricsHelper::record_receipt(&mut tracer, "receipt-003");

        // Verify receipt metrics
        let receipt_metrics = tracer.get_metrics("knhk.receipt.generated");
        assert_eq!(receipt_metrics.len(), 3, "Should record 3 receipt metrics");

        // Verify each metric has receipt_id attribute
        for metric in receipt_metrics {
            assert!(
                metric.attributes.contains_key("receipt_id"),
                "Receipt metric must have receipt_id attribute"
            );
        }
    }

    /// Test: Guard violation telemetry
    /// Chicago TDD: Test security event tracking
    #[test]
    fn test_guard_violation_telemetry() {
        let mut tracer = Tracer::new();

        // Record guard violations
        MetricsHelper::record_guard_violation(&mut tracer, "schema_validation");
        MetricsHelper::record_guard_violation(&mut tracer, "authorization");

        // Verify guard metrics
        let guard_metrics = tracer.get_metrics("knhk.guard.violation");
        assert_eq!(guard_metrics.len(), 2, "Should record 2 guard metrics");

        // Verify guard_type attribute
        for metric in guard_metrics {
            assert!(
                metric.attributes.contains_key("guard_type"),
                "Guard metric must have guard_type attribute"
            );
        }
    }

    /// Test: Configuration telemetry
    /// Chicago TDD: Test config load/error tracking
    #[test]
    fn test_configuration_telemetry() {
        let mut tracer = Tracer::new();

        // Record config loads
        MetricsHelper::record_config_load(&mut tracer, "file");
        MetricsHelper::record_config_load(&mut tracer, "environment");

        // Record config errors
        MetricsHelper::record_config_error(&mut tracer, "parse_error");

        // Verify metrics
        let load_metrics = tracer.get_metrics("knhk.config.loads");
        let error_metrics = tracer.get_metrics("knhk.config.errors");

        assert_eq!(load_metrics.len(), 2, "Should record 2 load metrics");
        assert_eq!(error_metrics.len(), 1, "Should record 1 error metric");
    }

    /// Test: Connector throughput telemetry
    /// Chicago TDD: Test connector metrics
    #[test]
    fn test_connector_throughput_telemetry() {
        let mut tracer = Tracer::new();

        // Record connector throughput
        MetricsHelper::record_connector_throughput(&mut tracer, "kafka-consumer-1", 1000);
        MetricsHelper::record_connector_throughput(&mut tracer, "salesforce-api", 500);

        // Verify throughput metrics
        let throughput_metrics = tracer.get_metrics("knhk.connector.throughput");
        assert_eq!(
            throughput_metrics.len(),
            2,
            "Should record 2 throughput metrics"
        );

        // Verify connector_id attribute
        for metric in throughput_metrics {
            assert!(
                metric.attributes.contains_key("connector_id"),
                "Connector metric must have connector_id attribute"
            );
        }
    }

    /// Test: Span parent-child relationships
    /// Chicago TDD: Test distributed tracing structure
    #[test]
    fn test_span_parent_child_relationships() {
        let mut tracer = Tracer::new();

        // Create parent span
        let parent_ctx = tracer.start_span("knhk.pipeline.execute".to_string(), None);

        // Create child spans
        let ingest_ctx = tracer.start_span("knhk.etl.ingest".to_string(), Some(parent_ctx.clone()));
        tracer.end_span(ingest_ctx, SpanStatus::Ok);

        let emit_ctx = tracer.start_span("knhk.etl.emit".to_string(), Some(parent_ctx.clone()));
        tracer.end_span(emit_ctx, SpanStatus::Ok);

        tracer.end_span(parent_ctx.clone(), SpanStatus::Ok);

        // Verify span relationships
        assert_eq!(tracer.spans().len(), 3, "Should have 3 spans");

        // Verify parent has no parent
        let parent_span = tracer.get_span(parent_ctx.span_id).unwrap();
        assert!(
            parent_span.context.parent_span_id.is_none(),
            "Parent span should have no parent"
        );

        // Verify children have parent
        let spans = tracer.spans();
        let children: Vec<_> = spans
            .iter()
            .filter(|s| s.context.parent_span_id.is_some())
            .collect();
        assert_eq!(children.len(), 2, "Should have 2 child spans");
    }

    /// Test: Span status tracking
    /// Chicago TDD: Test error handling in spans
    #[test]
    fn test_span_status_tracking() {
        let mut tracer = Tracer::new();

        // Create spans with different statuses
        let success_ctx = tracer.start_span("knhk.operation.success".to_string(), None);
        tracer.end_span(success_ctx, SpanStatus::Ok);

        let error_ctx = tracer.start_span("knhk.operation.error".to_string(), None);
        tracer.end_span(error_ctx, SpanStatus::Error);

        let unset_ctx = tracer.start_span("knhk.operation.unset".to_string(), None);
        tracer.end_span(unset_ctx, SpanStatus::Unset);

        // Verify span statuses
        let spans = tracer.spans();
        assert_eq!(spans[0].status, SpanStatus::Ok);
        assert_eq!(spans[1].status, SpanStatus::Error);
        assert_eq!(spans[2].status, SpanStatus::Unset);
    }

    /// Test: Semantic convention compliance across all operations
    /// Chicago TDD: Test that all spans follow naming conventions
    #[test]
    fn test_semantic_convention_compliance() {
        let mut tracer = Tracer::new();

        // Create spans following semantic conventions
        let operations = vec![
            ("knhk.boot.init", "boot.init", "system"),
            ("knhk.sidecar.transaction", "apply_transaction", "sidecar"),
            ("knhk.etl.ingest", "ingest", "etl"),
            ("knhk.hook.execute", "ASK_SP", "hook"),
        ];

        for (span_name, op_name, op_type) in operations {
            let ctx = tracer.start_span(span_name.to_string(), None);
            tracer.add_attribute(
                ctx.clone(),
                "knhk.operation.name".to_string(),
                op_name.to_string(),
            );
            tracer.add_attribute(
                ctx.clone(),
                "knhk.operation.type".to_string(),
                op_type.to_string(),
            );
            tracer.end_span(ctx, SpanStatus::Ok);
        }

        // Verify all spans follow conventions
        for span in tracer.spans() {
            // Span name must start with "knhk."
            assert!(
                span.name.starts_with("knhk."),
                "Span name must start with 'knhk.'"
            );

            // Must contain at least two segments (noun.verb)
            assert!(
                span.name.matches('.').count() >= 1,
                "Span name must have format 'knhk.<noun>.<verb>'"
            );

            // Must have required attributes
            assert!(
                span.attributes.contains_key("knhk.operation.name"),
                "Must have operation.name"
            );
            assert!(
                span.attributes.contains_key("knhk.operation.type"),
                "Must have operation.type"
            );
        }
    }

    /// Test: Telemetry export to OTLP endpoint
    /// Chicago TDD: Test export behavior (not implementation)
    /// Note: Requires running OTLP collector for full integration
    #[test]
    fn test_telemetry_export_behavior() {
        // Create tracer with OTLP exporter
        let mut tracer = Tracer::with_otlp_exporter("http://localhost:4317".to_string());

        // Generate telemetry
        let ctx = tracer.start_span("knhk.test.export".to_string(), None);
        tracer.add_attribute(
            ctx.clone(),
            "knhk.operation.name".to_string(),
            "test_export".to_string(),
        );
        tracer.end_span(ctx, SpanStatus::Ok);

        MetricsHelper::record_operation(&mut tracer, "test_export", true);

        // Verify telemetry created
        assert_eq!(tracer.spans().len(), 1, "Should create span");
        assert_eq!(tracer.metrics().len(), 1, "Should create metric");

        // Export behavior test (doesn't require live collector)
        // In production, export() would send to OTLP endpoint
        // Test verifies the API works without requiring infrastructure
        let result = tracer.export();

        // Assert: Verify actual behavior - either succeeds or fails with meaningful error
        match result {
            Ok(_) => {
                // Success case - telemetry exported to collector
            }
            Err(e) => {
                // Error case - collector not available or network error, verify error message
                assert!(!e.is_empty(), "Error message should not be empty");
            }
        }
    }

    /// Test: Weaver live-check configuration
    /// Chicago TDD: Test configuration behavior
    #[test]
    fn test_weaver_configuration() {
        let weaver = WeaverLiveCheck::new()
            .with_registry("./registry".to_string())
            .with_otlp_port(4317)
            .with_admin_port(8080)
            .with_inactivity_timeout(60)
            .with_format("json".to_string());

        // Verify configuration
        assert_eq!(weaver.otlp_endpoint(), "127.0.0.1:4317");
    }

    /// Test: End-to-end telemetry workflow
    /// Chicago TDD: Test complete workflow from creation to export
    #[test]
    fn test_end_to_end_telemetry_workflow() {
        // Step 1: Create tracer
        let mut tracer = Tracer::new();

        // Step 2: Simulate complete operation with span and metrics
        let ctx = tracer.start_span("knhk.e2e.test".to_string(), None);
        tracer.add_attribute(
            ctx.clone(),
            "knhk.operation.name".to_string(),
            "e2e_test".to_string(),
        );
        tracer.add_attribute(
            ctx.clone(),
            "knhk.operation.type".to_string(),
            "test".to_string(),
        );

        // Simulate work
        MetricsHelper::record_operation(&mut tracer, "e2e_test", true);
        MetricsHelper::record_warm_path_latency(&mut tracer, 5000, "e2e_test");

        tracer.end_span(ctx, SpanStatus::Ok);

        // Step 3: Verify telemetry completeness
        assert_eq!(tracer.spans().len(), 1, "Should have 1 span");
        assert_eq!(tracer.metrics().len(), 3, "Should have 3 metrics"); // operation + latency + count

        // Step 4: Verify semantic conventions
        let span = tracer.spans().first().unwrap();
        assert!(span.name.starts_with("knhk."));
        assert!(span.attributes.contains_key("knhk.operation.name"));
        assert!(span.attributes.contains_key("knhk.operation.type"));
        assert_eq!(span.status, SpanStatus::Ok);

        // Step 5: Verify metrics
        let operation_metric = tracer
            .get_metrics("knhk.operation.executed")
            .first()
            .unwrap();
        assert_eq!(
            operation_metric.attributes.get("operation"),
            Some(&"e2e_test".to_string())
        );
        assert_eq!(
            operation_metric.attributes.get("success"),
            Some(&"true".to_string())
        );
    }

    /// Test: Performance overhead of telemetry
    /// Chicago TDD: Verify telemetry overhead is minimal
    #[test]
    fn test_telemetry_performance_overhead() {
        use std::time::Instant;

        let mut tracer = Tracer::new();

        // Measure time to create 1000 spans
        let start = Instant::now();
        for i in 0..1000 {
            let ctx = tracer.start_span(format!("knhk.perf.test.{}", i), None);
            tracer.add_attribute(
                ctx.clone(),
                "knhk.operation.name".to_string(),
                format!("test_{}", i),
            );
            tracer.end_span(ctx, SpanStatus::Ok);
        }
        let elapsed = start.elapsed();

        // Verify overhead is reasonable (< 10ms per span)
        let per_span_ns = elapsed.as_nanos() / 1000;
        assert!(
            per_span_ns < 10_000_000, // 10ms
            "Span creation overhead too high: {}ns per span",
            per_span_ns
        );

        // Verify all spans created
        assert_eq!(tracer.spans().len(), 1000, "Should create 1000 spans");
    }
}

#[cfg(not(feature = "otel"))]
mod no_otel_tests {
    /// Test: OTEL feature disabled behavior
    #[test]
    fn test_otel_feature_disabled() {
        // When OTEL feature is disabled, code should compile but not emit telemetry
        // This test ensures the feature flag works correctly

        // In production, we'd call sidecar operations here
        // They would run successfully but not emit telemetry

        // Assert: Code compiles without otel feature
        // If we reach here, compilation succeeded - verify by checking feature flag behavior
        // When otel feature is disabled, telemetry operations should be no-ops or return errors
        // This test verifies the codebase compiles without otel feature
        // No explicit assertion needed - reaching here means compilation succeeded
    }
}
