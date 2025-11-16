// benches/warm_path_benches/telemetry_pipeline_bench.rs - Benchmark telemetry throughput
// Phase 3: Measure telemetry collection overhead and pipeline throughput

#![feature(test)]
extern crate test;

use knhk_warm::kernel::{
    TelemetryPipeline, TelemetryReceipt, CorrelatedEvent, TraceContext,
};
use knhk_warm::kernel::telemetry_pipeline::ReceiptStatus;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use test::Bencher;

fn create_receipt(id: usize) -> TelemetryReceipt {
    TelemetryReceipt {
        id: format!("bench-{}", id),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        execution_time_us: 100,
        trace_id: format!("trace-{}", id % 100),
        span_id: format!("span-{}", id),
        parent_span_id: if id > 0 { Some(format!("span-{}", id - 1)) } else { None },
        attributes: HashMap::new(),
        status: ReceiptStatus::Success,
    }
}

fn create_event(id: usize) -> CorrelatedEvent {
    CorrelatedEvent {
        id: format!("event-{}", id),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        event_type: "benchmark".to_string(),
        trace_id: format!("trace-{}", id % 100),
        correlation_id: format!("corr-{}", id % 50),
        data: HashMap::new(),
        related_events: vec![],
    }
}

#[bench]
fn bench_receipt_processing(b: &mut Bencher) {
    let pipeline = TelemetryPipeline::new(10000, 100000, Duration::from_secs(60));
    let mut counter = 0;

    b.iter(|| {
        let receipt = create_receipt(counter);
        pipeline.process_receipt(receipt).ok();
        counter += 1;
    });
}

#[bench]
fn bench_metric_aggregation(b: &mut Bencher) {
    let pipeline = TelemetryPipeline::new(10000, 100000, Duration::from_secs(60));
    let mut counter = 0;

    b.iter(|| {
        pipeline.process_metric(
            format!("metric.{}", counter % 100),
            counter as f64,
            HashMap::from([("tag".to_string(), "value".to_string())]),
        );
        counter += 1;
    });
}

#[bench]
fn bench_event_correlation(b: &mut Bencher) {
    let pipeline = TelemetryPipeline::new(10000, 100000, Duration::from_secs(60));
    let mut counter = 0;

    b.iter(|| {
        let event = create_event(counter);
        pipeline.process_event(event);
        counter += 1;
    });
}

#[bench]
fn bench_batch_creation(b: &mut Bencher) {
    let pipeline = TelemetryPipeline::new(1000, 100000, Duration::from_secs(60));

    // Pre-fill with data
    for i in 0..1000 {
        pipeline.process_receipt(create_receipt(i)).ok();
        pipeline.process_event(create_event(i));
        pipeline.process_metric(
            format!("metric.{}", i),
            i as f64,
            HashMap::new(),
        );
    }

    b.iter(|| {
        pipeline.flush_batch().ok();
    });
}

#[bench]
fn bench_trace_context_creation(b: &mut Bencher) {
    b.iter(|| {
        test::black_box(TraceContext::new());
    });
}

#[bench]
fn bench_trace_context_propagation(b: &mut Bencher) {
    let context = TraceContext::new();

    b.iter(|| {
        test::black_box(context.child_span());
    });
}

#[bench]
fn bench_pipeline_with_rate_limiting(b: &mut Bencher) {
    let pipeline = TelemetryPipeline::new(100, 1000, Duration::from_secs(60));
    let mut counter = 0;

    b.iter(|| {
        let receipt = create_receipt(counter);
        test::black_box(pipeline.process_receipt(receipt));
        counter += 1;
    });
}

#[bench]
fn bench_pipeline_statistics(b: &mut Bencher) {
    let pipeline = TelemetryPipeline::new(10000, 100000, Duration::from_secs(60));

    // Generate some activity
    for i in 0..100 {
        pipeline.process_receipt(create_receipt(i)).ok();
    }

    b.iter(|| {
        test::black_box(pipeline.get_stats());
    });
}