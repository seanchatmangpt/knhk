// rust/knhk-etl/tests/slo_monitor_test.rs
// Tests for SLO monitoring and p99 latency tracking

use knhk_etl::runtime_class::RuntimeClass;
use knhk_etl::slo_monitor::{SloMonitor, SloViolation};

#[test]
fn test_slo_monitor_r1_within_slo() {
    let mut monitor = SloMonitor::new(RuntimeClass::R1, 1000);
    
    // Record samples within SLO (≤2ns)
    for _ in 0..100 {
        monitor.record_latency(1); // 1ns < 2ns SLO
    }
    
    assert!(monitor.check_slo_violation().is_ok());
    let p99 = monitor.get_p99_latency();
    assert!(p99 <= 2);
}

#[test]
fn test_slo_monitor_r1_violation() {
    let mut monitor = SloMonitor::new(RuntimeClass::R1, 1000);
    
    // Record samples exceeding SLO
    for _ in 0..100 {
        monitor.record_latency(5); // 5ns > 2ns SLO
    }
    
    let violation = monitor.check_slo_violation();
    assert!(violation.is_err());
    
    if let Err(v) = violation {
        assert_eq!(v.class, RuntimeClass::R1);
        assert!(v.p99_latency_ns > v.slo_threshold_ns);
        assert!(v.violation_percent > 0.0);
    }
}

#[test]
fn test_slo_monitor_w1_within_slo() {
    let mut monitor = SloMonitor::new(RuntimeClass::W1, 1000);
    
    // Record samples within SLO (≤1ms)
    for _ in 0..100 {
        monitor.record_latency(500_000); // 500µs < 1ms SLO
    }
    
    assert!(monitor.check_slo_violation().is_ok());
}

#[test]
fn test_slo_monitor_w1_violation() {
    let mut monitor = SloMonitor::new(RuntimeClass::W1, 1000);
    
    // Record samples exceeding SLO
    for _ in 0..100 {
        monitor.record_latency(2_000_000); // 2ms > 1ms SLO
    }
    
    assert!(monitor.check_slo_violation().is_err());
}

#[test]
fn test_slo_monitor_c1_within_slo() {
    let mut monitor = SloMonitor::new(RuntimeClass::C1, 1000);
    
    // Record samples within SLO (≤500ms)
    for _ in 0..100 {
        monitor.record_latency(200_000_000); // 200ms < 500ms SLO
    }
    
    assert!(monitor.check_slo_violation().is_ok());
}

#[test]
fn test_p99_calculation() {
    let mut monitor = SloMonitor::new(RuntimeClass::R1, 1000);
    
    // Record 100 samples with known distribution
    for i in 0..100 {
        monitor.record_latency(i as u64);
    }
    
    let p99 = monitor.get_p99_latency();
    // p99 should be around index 99 (99th percentile)
    assert!(p99 >= 90);
}

#[test]
fn test_window_size_limit() {
    let mut monitor = SloMonitor::new(RuntimeClass::R1, 100);
    
    // Record more samples than window size
    for i in 0..200 {
        monitor.record_latency(i as u64);
    }
    
    // Should only keep window_size samples
    assert_eq!(monitor.sample_count(), 100);
}

#[test]
fn test_insufficient_samples() {
    let mut monitor = SloMonitor::new(RuntimeClass::R1, 1000);
    
    // Record fewer than 100 samples
    for _ in 0..50 {
        monitor.record_latency(100);
    }
    
    // p99 should return 0 (insufficient samples)
    assert_eq!(monitor.get_p99_latency(), 0);
}

#[test]
fn test_slo_violation_message() {
    let violation = SloViolation::new(RuntimeClass::R1, 5, 2);
    let message = violation.message();
    assert!(message.contains("R1"));
    assert!(message.contains("5"));
    assert!(message.contains("2"));
}

#[test]
fn test_clear_samples() {
    let mut monitor = SloMonitor::new(RuntimeClass::R1, 1000);
    
    for _ in 0..50 {
        monitor.record_latency(100);
    }
    
    assert_eq!(monitor.sample_count(), 50);
    monitor.clear();
    assert_eq!(monitor.sample_count(), 0);
}

