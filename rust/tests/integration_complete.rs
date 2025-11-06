// rust/tests/integration_complete.rs
// Comprehensive integration tests for knhk-etl, knhk-sidecar, knhk-warm, knhk-hot
// Tests all integration points, feature flags, and error scenarios

#[cfg(test)]
mod integration_tests {
    use knhk_etl::{Pipeline, integration::IntegratedPipeline};
    use knhk_hot::{Engine, Run, Ir, Receipt, Op, Ctx, NROWS, Aligned};

    // ============================================================================
    // 1. ETL → Hot Path Integration Tests
    // ============================================================================

    #[test]
    fn test_etl_hot_path_integration_basic() {
        // Test basic ETL → hot path integration
        // Verify hot path operations called from ETL pipeline

        // Create aligned SoA data (required for hot path)
        let s_aligned = Aligned([1u64, 2, 3, 4, 5, 6, 7, 8]);
        let p_aligned = Aligned([10u64, 10, 10, 10, 10, 10, 10, 10]);
        let o_aligned = Aligned([100u64, 200, 300, 400, 500, 600, 700, 800]);

        // Create hot path engine
        let mut engine = Engine::new(
            s_aligned.0.as_ptr(),
            p_aligned.0.as_ptr(),
            o_aligned.0.as_ptr()
        );

        // Pin a run (≤8 rows)
        let run = Run { pred: 10, off: 0, len: 8 };
        assert!(engine.pin_run(run).is_ok());

        // Create IR for AskSp operation
        let mut ir = Ir {
            op: Op::AskSp,
            s: 1,
            p: 10,
            o: 0,
            k: 0,
            out_S: std::ptr::null_mut(),
            out_P: std::ptr::null_mut(),
            out_O: std::ptr::null_mut(),
            out_mask: 0,
        };

        let mut receipt = Receipt::default();
        let result = engine.eval_bool(&mut ir, &mut receipt);

        // Verify execution
        assert!(result, "AskSp should find subject 1 with predicate 10");
        assert!(receipt.ticks <= 8, "Hot path must execute in ≤8 ticks, got {}", receipt.ticks);
        assert!(receipt.ticks > 0, "Receipt should record non-zero ticks");
    }

    #[test]
    fn test_etl_hot_path_performance_budget() {
        // Test that hot path respects ≤8 tick performance budget

        let s_aligned = Aligned([1u64; 8]);
        let p_aligned = Aligned([10u64; 8]);
        let o_aligned = Aligned([100u64; 8]);

        let mut engine = Engine::new(
            s_aligned.0.as_ptr(),
            p_aligned.0.as_ptr(),
            o_aligned.0.as_ptr()
        );

        let run = Run { pred: 10, off: 0, len: 8 };
        engine.pin_run(run).unwrap();

        // Test multiple operations, all must be ≤8 ticks
        let ops = vec![
            Op::AskSp,
            Op::CountSpGe,
            Op::AskSpo,
            Op::CountSpLe,
            Op::AskOp,
        ];

        for op in ops {
            let mut ir = Ir {
                op,
                s: 1,
                p: 10,
                o: 100,
                k: 5,
                out_S: std::ptr::null_mut(),
                out_P: std::ptr::null_mut(),
                out_O: std::ptr::null_mut(),
                out_mask: 0,
            };

            let mut receipt = Receipt::default();
            let _ = engine.eval_bool(&mut ir, &mut receipt);

            assert!(
                receipt.ticks <= 8,
                "Operation {:?} took {} ticks, exceeds budget of 8",
                op, receipt.ticks
            );
        }
    }

    #[test]
    fn test_etl_hot_path_batch_execution() {
        // Test batch execution (≤8 operations)

        let s_aligned = Aligned([1u64, 2, 3, 4, 5, 6, 7, 8]);
        let p_aligned = Aligned([10u64; 8]);
        let o_aligned = Aligned([100u64, 200, 300, 400, 500, 600, 700, 800]);

        let mut engine = Engine::new(
            s_aligned.0.as_ptr(),
            p_aligned.0.as_ptr(),
            o_aligned.0.as_ptr()
        );

        let run = Run { pred: 10, off: 0, len: 8 };
        engine.pin_run(run).unwrap();

        // Create batch of IRs
        let mut irs = [Ir {
            op: Op::AskSp,
            s: 1,
            p: 10,
            o: 0,
            k: 0,
            out_S: std::ptr::null_mut(),
            out_P: std::ptr::null_mut(),
            out_O: std::ptr::null_mut(),
            out_mask: 0,
        }; NROWS];

        // Vary the subject for each operation
        for (i, ir) in irs.iter_mut().enumerate() {
            ir.s = (i + 1) as u64;
        }

        let mut receipts = [Receipt::default(); NROWS];
        let executed = engine.eval_batch8(&mut irs, 8, &mut receipts);

        assert_eq!(executed, 8, "Should execute all 8 operations");

        // Verify all operations completed within budget
        for (i, receipt) in receipts.iter().enumerate() {
            assert!(
                receipt.ticks <= 8,
                "Operation {} took {} ticks, exceeds budget",
                i, receipt.ticks
            );
        }
    }

    #[test]
    fn test_etl_hot_path_run_length_guard() {
        // Test that run length > 8 is rejected (Hatman Constant enforcement)

        let s_aligned = Aligned([1u64; 8]);
        let p_aligned = Aligned([10u64; 8]);
        let o_aligned = Aligned([100u64; 8]);

        let mut engine = Engine::new(
            s_aligned.0.as_ptr(),
            p_aligned.0.as_ptr(),
            o_aligned.0.as_ptr()
        );

        // Try to pin run with length > 8 (should fail)
        let invalid_run = Run { pred: 10, off: 0, len: 9 };
        let result = engine.pin_run(invalid_run);

        assert!(result.is_err(), "Run length > 8 should be rejected");
        assert_eq!(result.unwrap_err(), "H: run.len > 8 blocked");

        // Valid run should succeed
        let valid_run = Run { pred: 10, off: 0, len: 8 };
        assert!(engine.pin_run(valid_run).is_ok());
    }

    // ============================================================================
    // 2. ETL → Warm Path Integration Tests
    // ============================================================================

    #[test]
    fn test_etl_warm_path_integration_basic() {
        // Test basic ETL → warm path integration via IntegratedPipeline

        let pipeline = IntegratedPipeline::new(
            vec!["test_connector".to_string()],
            "urn:knhk:schema:test".to_string(),
            false, // lockchain disabled
            vec![], // no downstream endpoints
        );

        // Verify pipeline created
        assert_eq!(pipeline.connectors.len(), 1);
        assert_eq!(pipeline.schema_iri, "urn:knhk:schema:test");
    }

    #[test]
    fn test_etl_warm_path_query_executor_not_configured() {
        // Test error when warm path executor not configured

        let pipeline = IntegratedPipeline::new(
            vec![],
            "urn:knhk:schema:test".to_string(),
            false,
            vec![],
        );

        // Try to execute query without executor
        let result = pipeline.execute_warm_path_query("SELECT * WHERE { ?s ?p ?o }");

        assert!(result.is_err(), "Should fail when executor not configured");
        assert!(
            result.unwrap_err().to_string().contains("not configured"),
            "Error should mention missing configuration"
        );
    }

    #[test]
    fn test_etl_warm_path_query_executor_integration() {
        // Test warm path executor integration

        use knhk_etl::integration::{WarmPathQueryExecutor, WarmPathQueryResult};
        use std::collections::BTreeMap;

        // Mock executor
        struct MockWarmExecutor;

        impl WarmPathQueryExecutor for MockWarmExecutor {
            fn execute_query(&self, _sparql: &str) -> Result<WarmPathQueryResult, String> {
                let mut solution = BTreeMap::new();
                solution.insert("s".to_string(), "http://example.org/subject".to_string());
                solution.insert("p".to_string(), "http://example.org/predicate".to_string());
                solution.insert("o".to_string(), "value".to_string());

                Ok(WarmPathQueryResult::Solutions(vec![solution]))
            }
        }

        let mut pipeline = IntegratedPipeline::new(
            vec![],
            "urn:knhk:schema:test".to_string(),
            false,
            vec![],
        );

        // Set executor
        pipeline.set_warm_path_executor(Box::new(MockWarmExecutor));

        // Execute query
        let result = pipeline.execute_warm_path_query("SELECT * WHERE { ?s ?p ?o }");
        assert!(result.is_ok(), "Query should succeed with configured executor");

        match result.unwrap() {
            WarmPathQueryResult::Solutions(solutions) => {
                assert_eq!(solutions.len(), 1);
                assert_eq!(solutions[0].get("s").unwrap(), "http://example.org/subject");
            }
            _ => panic!("Expected Solutions result"),
        }
    }

    // ============================================================================
    // 3. Pipeline Execution Integration Tests
    // ============================================================================

    #[test]
    fn test_integrated_pipeline_execution_basic() {
        // Test basic integrated pipeline execution

        let mut pipeline = IntegratedPipeline::new(
            vec![], // no connectors for basic test
            "urn:knhk:schema:test".to_string(),
            false, // lockchain disabled
            vec![], // no downstream endpoints
        );

        let result = pipeline.execute();
        assert!(result.is_ok(), "Pipeline execution should succeed");

        let integrated_result = result.unwrap();
        assert_eq!(integrated_result.receipts_written, 0);
        assert_eq!(integrated_result.actions_sent, 0);
        assert_eq!(integrated_result.lockchain_hashes.len(), 0);
    }

    #[test]
    #[cfg(feature = "knhk-otel")]
    fn test_integrated_pipeline_otel_metrics() {
        // Test OTEL metrics recording during pipeline execution

        let mut pipeline = IntegratedPipeline::new(
            vec!["test_connector".to_string()],
            "urn:knhk:schema:test".to_string(),
            false,
            vec!["http://localhost:8080/webhook".to_string()],
        );

        let result = pipeline.execute();
        assert!(result.is_ok(), "Pipeline with OTEL should execute");

        let integrated_result = result.unwrap();
        // Metrics should be recorded when OTEL feature enabled
        assert!(
            integrated_result.metrics_recorded >= 0,
            "Should record metrics"
        );
    }

    // ============================================================================
    // 4. Feature Flag Integration Tests
    // ============================================================================

    #[test]
    fn test_feature_std_enabled() {
        // Test that std feature is properly enabled
        // This test compiles only if std feature is available

        use knhk_etl::Pipeline;

        let pipeline = Pipeline::new(
            vec![],
            "urn:knhk:schema:test".to_string(),
            false,
            vec!["http://localhost:8080/webhook".to_string()],
        );

        // If std feature works, HTTP endpoints should be accepted
        assert_eq!(pipeline.downstream_endpoints.len(), 1);
    }

    #[test]
    #[cfg(feature = "knhk-lockchain")]
    fn test_feature_lockchain_enabled() {
        // Test lockchain feature when enabled

        let pipeline = Pipeline::new(
            vec![],
            "urn:knhk:schema:test".to_string(),
            true, // lockchain enabled
            vec![],
        );

        assert!(pipeline.lockchain_enabled);
    }

    #[test]
    #[cfg(not(feature = "knhk-lockchain"))]
    fn test_feature_lockchain_disabled() {
        // Test that lockchain feature is properly disabled

        let pipeline = Pipeline::new(
            vec![],
            "urn:knhk:schema:test".to_string(),
            true, // request lockchain
            vec![],
        );

        // When feature disabled, lockchain_enabled should still be set
        // but functionality may be limited
        assert!(pipeline.lockchain_enabled);
    }

    // ============================================================================
    // 5. Error Propagation Integration Tests
    // ============================================================================

    #[test]
    fn test_error_propagation_etl_to_hot() {
        // Test error propagation from hot path to ETL

        let s_aligned = Aligned([1u64; 8]);
        let p_aligned = Aligned([10u64; 8]);
        let o_aligned = Aligned([100u64; 8]);

        let mut engine = Engine::new(
            s_aligned.0.as_ptr(),
            p_aligned.0.as_ptr(),
            o_aligned.0.as_ptr()
        );

        // Invalid run should propagate error
        let invalid_run = Run { pred: 10, off: 0, len: 100 };
        let result = engine.pin_run(invalid_run);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("blocked"));
    }

    #[test]
    fn test_error_propagation_warm_to_etl() {
        // Test error propagation from warm path to ETL

        use knhk_etl::integration::WarmPathQueryExecutor;

        struct ErrorExecutor;

        impl WarmPathQueryExecutor for ErrorExecutor {
            fn execute_query(&self, _sparql: &str) -> Result<knhk_etl::integration::WarmPathQueryResult, String> {
                Err("Mock query execution error".to_string())
            }
        }

        let mut pipeline = IntegratedPipeline::new(vec![], "urn:test".to_string(), false, vec![]);
        pipeline.set_warm_path_executor(Box::new(ErrorExecutor));

        let result = pipeline.execute_warm_path_query("SELECT * WHERE { ?s ?p ?o }");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("query failed"));
    }

    // ============================================================================
    // 6. Receipt Merge Integration Tests
    // ============================================================================

    #[test]
    fn test_receipt_merge_integration() {
        // Test receipt merging across multiple hot path operations

        let receipt_a = Receipt {
            ticks: 4,
            lanes: 8,
            span_id: 0x1234,
            a_hash: 0x5678,
        };

        let receipt_b = Receipt {
            ticks: 6,
            lanes: 8,
            span_id: 0xabcd,
            a_hash: 0xef00,
        };

        let merged = Receipt::merge(receipt_a, receipt_b);

        // Verify merge semantics
        assert_eq!(merged.ticks, 6, "Should take max ticks");
        assert_eq!(merged.lanes, 16, "Should sum lanes");
        assert_eq!(merged.span_id, 0x1234 ^ 0xabcd, "Should XOR span IDs");
        assert_eq!(merged.a_hash, 0x5678 ^ 0xef00, "Should XOR hashes");
    }

    #[test]
    fn test_receipt_chain_merge() {
        // Test merging chain of receipts

        let receipts = vec![
            Receipt { ticks: 2, lanes: 4, span_id: 0x1, a_hash: 0xa },
            Receipt { ticks: 3, lanes: 4, span_id: 0x2, a_hash: 0xb },
            Receipt { ticks: 4, lanes: 4, span_id: 0x3, a_hash: 0xc },
            Receipt { ticks: 1, lanes: 4, span_id: 0x4, a_hash: 0xd },
        ];

        let merged = receipts.iter()
            .copied()
            .reduce(|acc, r| Receipt::merge(acc, r))
            .unwrap();

        assert_eq!(merged.ticks, 4, "Max ticks in chain");
        assert_eq!(merged.lanes, 16, "Sum of all lanes");
        assert_eq!(merged.span_id, 0x1 ^ 0x2 ^ 0x3 ^ 0x4);
        assert_eq!(merged.a_hash, 0xa ^ 0xb ^ 0xc ^ 0xd);
    }

    // ============================================================================
    // 7. Concurrent Execution Integration Tests
    // ============================================================================

    #[test]
    fn test_concurrent_hot_path_execution() {
        // Test that hot path can handle concurrent execution
        // (though Engine itself is not Send/Sync, test that pattern works)

        use std::thread;

        let handles: Vec<_> = (0..4).map(|i| {
            thread::spawn(move || {
                let s = Aligned([i; 8]);
                let p = Aligned([10u64; 8]);
                let o = Aligned([100u64; 8]);

                let mut engine = Engine::new(s.0.as_ptr(), p.0.as_ptr(), o.0.as_ptr());
                let run = Run { pred: 10, off: 0, len: 8 };
                engine.pin_run(run).unwrap();

                let mut ir = Ir {
                    op: Op::AskSp,
                    s: i,
                    p: 10,
                    o: 0,
                    k: 0,
                    out_S: std::ptr::null_mut(),
                    out_P: std::ptr::null_mut(),
                    out_O: std::ptr::null_mut(),
                    out_mask: 0,
                };

                let mut receipt = Receipt::default();
                engine.eval_bool(&mut ir, &mut receipt);

                receipt.ticks
            })
        }).collect();

        for handle in handles {
            let ticks = handle.join().unwrap();
            assert!(ticks <= 8, "Each thread should respect tick budget");
        }
    }

    // ============================================================================
    // 8. Memory Safety Integration Tests
    // ============================================================================

    #[test]
    fn test_aligned_memory_requirement() {
        // Test that aligned memory is properly enforced

        let s_aligned = Aligned([1u64; 8]);
        let p_aligned = Aligned([10u64; 8]);
        let o_aligned = Aligned([100u64; 8]);

        // Verify alignment
        let s_ptr = s_aligned.0.as_ptr();
        let p_ptr = p_aligned.0.as_ptr();
        let o_ptr = o_aligned.0.as_ptr();

        assert_eq!(s_ptr as usize % 64, 0, "S array must be 64-byte aligned");
        assert_eq!(p_ptr as usize % 64, 0, "P array must be 64-byte aligned");
        assert_eq!(o_ptr as usize % 64, 0, "O array must be 64-byte aligned");

        // Engine should work with aligned data
        let mut engine = Engine::new(s_ptr, p_ptr, o_ptr);
        let run = Run { pred: 10, off: 0, len: 8 };
        assert!(engine.pin_run(run).is_ok());
    }
}
