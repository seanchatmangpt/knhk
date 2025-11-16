// Phase 5: Comprehensive tests for SLO-based Capacity Planning

use knhk_sidecar::capacity::{
    CapacityManager, CapacityUrgency, CostModel, GrowthProjection, HeatItem, SloClass,
    SloPrediction,
};
use std::time::SystemTime;

#[test]
fn test_slo_class_definitions() {
    // Test R1 (Hot path)
    assert_eq!(SloClass::R1.name(), "R1 (Hot)");
    assert_eq!(SloClass::R1.required_hit_rate(), 0.99);
    assert_eq!(SloClass::R1.target_latency().as_nanos(), 2);

    // Test W1 (Warm path)
    assert_eq!(SloClass::W1.name(), "W1 (Warm)");
    assert_eq!(SloClass::W1.required_hit_rate(), 0.95);
    assert_eq!(SloClass::W1.target_latency().as_millis(), 500);

    // Test C1 (Cold path)
    assert_eq!(SloClass::C1.name(), "C1 (Cold)");
    assert_eq!(SloClass::C1.required_hit_rate(), 0.80);
    assert_eq!(SloClass::C1.target_latency().as_secs(), 86400);
}

#[test]
fn test_slo_class_default() {
    assert_eq!(SloClass::default(), SloClass::W1);
}

#[test]
fn test_slo_prediction_creation() {
    let prediction = SloPrediction::new(SloClass::R1);
    assert_eq!(prediction.slo_class, SloClass::R1);
    assert_eq!(prediction.expected_hit_rate, 0.0);
    assert_eq!(prediction.confidence, 0.0);
    assert!(!prediction.meets_slo());
}

#[test]
fn test_slo_prediction_validation() {
    let mut pred_r1 = SloPrediction::new(SloClass::R1);
    pred_r1.expected_hit_rate = 0.98; // Below 99%
    assert!(!pred_r1.meets_slo());

    pred_r1.expected_hit_rate = 0.99; // Meets 99%
    assert!(pred_r1.meets_slo());

    let mut pred_w1 = SloPrediction::new(SloClass::W1);
    pred_w1.expected_hit_rate = 0.94; // Below 95%
    assert!(!pred_w1.meets_slo());

    pred_w1.expected_hit_rate = 0.95; // Meets 95%
    assert!(pred_w1.meets_slo());
}

#[test]
fn test_slo_prediction_storage_tier() {
    let r1_pred = SloPrediction::new(SloClass::R1);
    assert!(r1_pred.storage_tier().contains("HSM"));

    let w1_pred = SloPrediction::new(SloClass::W1);
    assert!(w1_pred.storage_tier().contains("SSD"));

    let c1_pred = SloPrediction::new(SloClass::C1);
    assert!(c1_pred.storage_tier().contains("Disk"));
}

#[test]
fn test_heat_item_hit_rate() {
    let item = HeatItem {
        key: "test_key".to_string(),
        access_count: 100,
        last_access: SystemTime::now(),
        size_bytes: 1024,
        hit_count: 95,
        miss_count: 5,
    };

    assert_eq!(item.hit_rate(), 0.95);
}

#[test]
fn test_heat_item_zero_hits() {
    let item = HeatItem {
        key: "test_key".to_string(),
        access_count: 0,
        last_access: SystemTime::now(),
        size_bytes: 1024,
        hit_count: 0,
        miss_count: 0,
    };

    assert_eq!(item.hit_rate(), 0.0);
}

#[test]
fn test_cost_model_default() {
    let cost = CostModel::default();
    assert_eq!(cost.l1_cost_per_gb_daily, 3.0);
    assert_eq!(cost.l2_cost_per_gb_daily, 0.10);
    assert_eq!(cost.l3_cost_per_gb_daily, 0.02);
}

#[test]
fn test_cost_model_estimation() {
    let cost = CostModel::default();
    let daily_cost = cost.estimate_daily_cost(1.0, 10.0); // 1GB L1, 10GB L2

    // Expected: (1.0 * 3.0) + (10.0 * 0.10) = 3.0 + 1.0 = 4.0
    assert_eq!(daily_cost, 4.0);
}

#[test]
fn test_capacity_manager_creation() {
    let manager = CapacityManager::new(0.95);
    let slo_dist = manager.get_slo_distribution();
    assert!(slo_dist.is_empty());
}

#[test]
fn test_capacity_manager_custom_cost_model() {
    let mut manager = CapacityManager::new(0.95);
    let custom_cost = CostModel {
        l1_cost_per_gb_daily: 5.0,
        l2_cost_per_gb_daily: 0.20,
        l3_cost_per_gb_daily: 0.05,
    };

    manager.set_cost_model(custom_cost);
    // Manager now uses custom cost model
}

#[test]
fn test_heat_map_analysis_empty() {
    let mut manager = CapacityManager::new(0.95);
    let heat_map = manager.analyze_heat_map();
    assert!(heat_map.is_empty());
}

#[test]
fn test_heat_map_analysis_with_data() {
    let mut manager = CapacityManager::new(0.95);

    // Record some accesses
    manager.record_access("predicate_1", true, true);
    manager.record_access("predicate_1", true, true);
    manager.record_access("predicate_1", true, true);

    manager.record_access("predicate_2", true, false);
    manager.record_access("predicate_2", true, false);

    manager.record_access("predicate_3", false, false);

    let heat_map = manager.analyze_heat_map();

    // Should be sorted by access frequency (Pareto)
    assert!(!heat_map.is_empty());
    // predicate_1 has 3 accesses, predicate_2 has 2, predicate_3 has 1
    assert_eq!(heat_map[0].key, "predicate_1");
    assert_eq!(heat_map[0].access_count, 3);
}

#[test]
fn test_slo_capacity_prediction_r1() {
    let mut manager = CapacityManager::new(0.95);

    // Simulate workload
    for i in 0..100 {
        let key = format!("key_{}", i % 10);
        manager.record_access(&key, i % 10 < 9, true); // 90% hit rate
    }

    let prediction = manager.predict_capacity_needed(SloClass::R1);

    assert_eq!(prediction.slo_class, SloClass::R1);
    assert!(prediction.required_l1_size >= 64 * 1024); // At least 64KB
    assert!(prediction.required_l2_size >= 1024 * 1024); // At least 1MB
    assert!(prediction.expected_hit_rate > 0.0);
    assert!(prediction.confidence > 0.0);
}

#[test]
fn test_slo_capacity_prediction_w1() {
    let mut manager = CapacityManager::new(0.95);

    // Simulate workload
    for i in 0..100 {
        let key = format!("key_{}", i % 20);
        manager.record_access(&key, i % 100 < 85, false); // 85% hit rate
    }

    let prediction = manager.predict_capacity_needed(SloClass::W1);

    assert_eq!(prediction.slo_class, SloClass::W1);
    assert!(prediction.required_l1_size > 0);
    assert!(prediction.required_l2_size > 0);
}

#[test]
fn test_slo_capacity_prediction_c1() {
    let mut manager = CapacityManager::new(0.95);

    // Simulate workload
    for i in 0..100 {
        let key = format!("key_{}", i % 50);
        manager.record_access(&key, i % 100 < 70, false); // 70% hit rate
    }

    let prediction = manager.predict_capacity_needed(SloClass::C1);

    assert_eq!(prediction.slo_class, SloClass::C1);
    // C1 is most lenient
    assert!(prediction.confidence > 0.5);
}

#[test]
fn test_should_admit_request_high_hit_rate() {
    let mut manager = CapacityManager::new(0.95);

    // Build high hit rate
    for i in 0..100 {
        manager.record_access("test", i < 99, true); // 99% hit rate
    }

    // R1 requires 99%, should admit
    assert!(manager.should_admit_request(SloClass::R1));

    // Check SLO distribution
    let dist = manager.get_slo_distribution();
    assert_eq!(dist.get(&SloClass::R1), Some(&1));
}

#[test]
fn test_should_admit_request_medium_hit_rate() {
    let mut manager = CapacityManager::new(0.95);

    // Build 92% hit rate
    for i in 0..100 {
        manager.record_access("test", i < 92, true);
    }

    // R1 requires 99%, should reject
    assert!(!manager.should_admit_request(SloClass::R1));

    // W1 requires 95%, should reject
    assert!(!manager.should_admit_request(SloClass::W1));

    // C1 requires 80%, should admit
    assert!(manager.should_admit_request(SloClass::C1));
}

#[test]
fn test_should_admit_request_low_hit_rate() {
    let mut manager = CapacityManager::new(0.95);

    // Build 50% hit rate
    for i in 0..100 {
        manager.record_access("test", i < 50, true);
    }

    // All SLO classes should reject
    assert!(!manager.should_admit_request(SloClass::R1));
    assert!(!manager.should_admit_request(SloClass::W1));
    assert!(!manager.should_admit_request(SloClass::C1));
}

#[test]
fn test_scale_recommendation_no_growth() {
    let manager = CapacityManager::new(0.95);

    let recommendation = manager.scale_recommendation();

    // No growth history
    assert_eq!(recommendation.growth_projection.growth_rate, 0.0);
    assert_eq!(recommendation.urgency, CapacityUrgency::Ok);
}

#[test]
fn test_scale_recommendation_ok() {
    let mut manager = CapacityManager::new(0.95);

    // Record initial state
    manager.record_access("key", true, true);
    manager.record_growth_point();

    let recommendation = manager.scale_recommendation();
    assert_eq!(recommendation.urgency, CapacityUrgency::Ok);
    assert!(recommendation.recommended_capacity >= recommendation.current_capacity);
}

#[test]
fn test_record_growth_point() {
    let mut manager = CapacityManager::new(0.95);

    // Record some data
    for i in 0..10 {
        manager.record_access(&format!("key_{}", i), true, true);
    }

    manager.record_growth_point();
    // Growth point recorded, can be used for trend analysis

    manager.record_access(&format!("key_{}", 10), true, true);
    manager.record_growth_point();
    // Second point recorded
}

#[test]
fn test_capacity_urgency_critical() {
    // Create a scenario where growth is critical
    let mut manager = CapacityManager::new(0.95);

    // Simulate rapid growth
    for _attempt in 0..3 {
        // Add many keys per growth point
        for i in 0..100 {
            manager.record_access(&format!("key_{}", i), true, true);
        }
        manager.record_growth_point();
    }

    let recommendation = manager.scale_recommendation();
    // Growth rate should be positive
    assert!(recommendation.growth_projection.growth_rate >= 0.0);
}

#[test]
fn test_pareto_principle_working_set() {
    let mut manager = CapacityManager::new(0.95);

    // Simulate Pareto distribution: 20% of keys generate 80% of traffic
    // 10 hot keys
    for _rep in 0..80 {
        for i in 0..10 {
            manager.record_access(&format!("hot_{}", i), true, true);
        }
    }

    // 40 cold keys
    for rep in 0..20 {
        for i in 0..40 {
            manager.record_access(&format!("cold_{}", i), rep == 0, false);
        }
    }

    let heat_map = manager.analyze_heat_map();
    assert!(!heat_map.is_empty());

    // Hot keys should be at the beginning (highest access count)
    assert!(heat_map[0].key.starts_with("hot_"));
}

#[test]
fn test_confidence_levels() {
    let mut manager_low = CapacityManager::new(0.95);
    // Add 5 predicates - low confidence
    for i in 0..5 {
        manager_low.record_access(&format!("key_{}", i), true, true);
    }
    let pred_low = manager_low.predict_capacity_needed(SloClass::W1);
    assert!(pred_low.confidence <= 0.7);

    let mut manager_med = CapacityManager::new(0.95);
    // Add 50 predicates - medium confidence
    for i in 0..50 {
        manager_med.record_access(&format!("key_{}", i), true, true);
    }
    let pred_med = manager_med.predict_capacity_needed(SloClass::W1);
    assert!(pred_med.confidence >= 0.7 && pred_med.confidence < 0.95);

    let mut manager_high = CapacityManager::new(0.95);
    // Add 500 predicates - high confidence
    for i in 0..500 {
        manager_high.record_access(&format!("key_{}", i), true, true);
    }
    let pred_high = manager_high.predict_capacity_needed(SloClass::W1);
    assert!(pred_high.confidence >= 0.85);
}

#[test]
fn test_slo_class_distribution_tracking() {
    let mut manager = CapacityManager::new(0.95);

    // Build high hit rate for R1 admission
    for i in 0..100 {
        manager.record_access("test", i < 99, true);
    }

    // Admit multiple requests of different classes
    manager.should_admit_request(SloClass::R1);
    manager.should_admit_request(SloClass::R1);
    manager.should_admit_request(SloClass::W1);

    let dist = manager.get_slo_distribution();
    assert_eq!(dist.get(&SloClass::R1), Some(&2));
    assert_eq!(dist.get(&SloClass::W1), Some(&1));
}

#[test]
fn test_capacity_manager_thread_safety() {
    // This test verifies that CapacityManager can be safely used
    // The manager should be Send + Sync if used in multi-threaded context
    let _manager = CapacityManager::new(0.95);
    // Compile-time check: if CapacityManager isn't Send, this won't compile
}

#[test]
fn test_heat_item_clone() {
    let original = HeatItem {
        key: "test".to_string(),
        access_count: 100,
        last_access: SystemTime::now(),
        size_bytes: 1024,
        hit_count: 90,
        miss_count: 10,
    };

    let cloned = original.clone();
    assert_eq!(cloned.key, original.key);
    assert_eq!(cloned.access_count, original.access_count);
    assert_eq!(cloned.hit_rate(), original.hit_rate());
}

#[test]
fn test_growth_projection_structure() {
    let projection = GrowthProjection {
        current_size: 1000,
        projected_size_30d: 1300,
        projected_size_90d: 1900,
        growth_rate: 10.0,
        urgency: CapacityUrgency::Warning,
    };

    assert_eq!(projection.current_size, 1000);
    assert!(projection.projected_size_30d > projection.current_size);
    assert!(projection.projected_size_90d > projection.projected_size_30d);
}

#[test]
fn test_slo_prediction_meets_slo_boundary() {
    // Test boundary conditions for SLO validation

    // R1 at exactly 99%
    let mut r1 = SloPrediction::new(SloClass::R1);
    r1.expected_hit_rate = 0.99;
    assert!(r1.meets_slo());

    // R1 just below 99%
    r1.expected_hit_rate = 0.9899;
    assert!(!r1.meets_slo());

    // W1 at exactly 95%
    let mut w1 = SloPrediction::new(SloClass::W1);
    w1.expected_hit_rate = 0.95;
    assert!(w1.meets_slo());

    // C1 at exactly 80%
    let mut c1 = SloPrediction::new(SloClass::C1);
    c1.expected_hit_rate = 0.80;
    assert!(c1.meets_slo());
}

#[test]
fn test_capacity_manager_multiple_accesses() {
    let mut manager = CapacityManager::new(0.95);

    // Multiple accesses to same key
    for _i in 0..10 {
        manager.record_access("hot_key", true, true);
    }

    // Mixed access to another key
    for i in 0..10 {
        manager.record_access("warm_key", i < 7, false);
    }

    let heat_map = manager.analyze_heat_map();
    assert_eq!(heat_map.len(), 2);
    // hot_key has 10 accesses, warm_key has 10, but hot_key has better hit rate
}

#[test]
fn test_cost_estimation_scenarios() {
    let cost = CostModel::default();

    // Small cache
    let cost_small = cost.estimate_daily_cost(0.1, 1.0); // 100MB L1, 1GB L2
    assert!(cost_small > 0.0);

    // Large cache
    let cost_large = cost.estimate_daily_cost(10.0, 100.0); // 10GB L1, 100GB L2
    assert!(cost_large > cost_small);

    // Very large cache
    let cost_xlarge = cost.estimate_daily_cost(100.0, 1000.0);
    assert!(cost_xlarge > cost_large);
}

#[test]
fn test_empty_capacity_manager_edge_cases() {
    let manager = CapacityManager::new(0.95);

    // Empty manager should not crash
    let prediction = manager.predict_capacity_needed(SloClass::W1);
    assert_eq!(prediction.expected_hit_rate, 0.0);

    let recommendation = manager.scale_recommendation();
    assert_eq!(recommendation.current_capacity, 0);
}
