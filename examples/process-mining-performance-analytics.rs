//! Process Mining Performance Analytics Example
//!
//! Demonstrates advanced performance analysis for workflows:
//! - Calculate cycle times and throughput metrics
//! - Identify slowest activities and bottlenecks
//! - Generate optimization recommendations
//! - Track performance trends over time
//!
//! Run: `cargo run --example process-mining-performance-analytics`

use chrono::{DateTime, Duration, Utc};
use hashbrown::HashMap;
use std::time::Instant;

// ============================================================================
// Performance Data Structures
// ============================================================================

#[derive(Debug, Clone)]
struct WorkflowInstance {
    id: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    activities: Vec<ActivityExecution>,
}

impl WorkflowInstance {
    fn cycle_time_ms(&self) -> i64 {
        self.end_time.signed_duration_since(self.start_time).num_milliseconds()
    }

    fn waiting_time_ms(&self) -> i64 {
        let mut total_waiting = Duration::zero();

        for window in self.activities.windows(2) {
            let gap = window[1].start_time.signed_duration_since(window[0].end_time);
            if gap > Duration::zero() {
                total_waiting = total_waiting + gap;
            }
        }

        total_waiting.num_milliseconds()
    }

    fn processing_time_ms(&self) -> i64 {
        self.activities.iter()
            .map(|a| a.duration_ms())
            .sum()
    }
}

#[derive(Debug, Clone)]
struct ActivityExecution {
    activity: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

impl ActivityExecution {
    fn duration_ms(&self) -> i64 {
        self.end_time.signed_duration_since(self.start_time).num_milliseconds()
    }
}

// ============================================================================
// Performance Metrics
// ============================================================================

#[derive(Debug, Clone)]
struct PerformanceMetrics {
    // Cycle time metrics
    avg_cycle_time_ms: f64,
    median_cycle_time_ms: f64,
    p95_cycle_time_ms: f64,
    p99_cycle_time_ms: f64,

    // Throughput metrics
    throughput_per_hour: f64,
    completed_instances: usize,

    // Time breakdown
    avg_processing_time_ms: f64,
    avg_waiting_time_ms: f64,
    processing_time_ratio: f64,

    // Activity metrics
    activity_stats: HashMap<String, ActivityStats>,

    // Bottlenecks
    bottlenecks: Vec<PerformanceBottleneck>,
}

#[derive(Debug, Clone)]
struct ActivityStats {
    activity: String,
    count: usize,
    avg_duration_ms: f64,
    std_deviation_ms: f64,
    min_duration_ms: i64,
    max_duration_ms: i64,
    p95_duration_ms: f64,
}

#[derive(Debug, Clone)]
struct PerformanceBottleneck {
    activity: String,
    impact_score: f64,
    avg_duration_ms: f64,
    frequency: usize,
    recommendation: String,
}

// ============================================================================
// Performance Analyzer
// ============================================================================

struct PerformanceAnalyzer {
    instances: Vec<WorkflowInstance>,
}

impl PerformanceAnalyzer {
    fn new(instances: Vec<WorkflowInstance>) -> Self {
        Self { instances }
    }

    fn analyze(&self) -> PerformanceMetrics {
        let cycle_times = self.calculate_cycle_time_metrics();
        let throughput = self.calculate_throughput();
        let time_breakdown = self.calculate_time_breakdown();
        let activity_stats = self.calculate_activity_stats();
        let bottlenecks = self.identify_bottlenecks(&activity_stats);

        PerformanceMetrics {
            avg_cycle_time_ms: cycle_times.0,
            median_cycle_time_ms: cycle_times.1,
            p95_cycle_time_ms: cycle_times.2,
            p99_cycle_time_ms: cycle_times.3,
            throughput_per_hour: throughput.0,
            completed_instances: throughput.1,
            avg_processing_time_ms: time_breakdown.0,
            avg_waiting_time_ms: time_breakdown.1,
            processing_time_ratio: time_breakdown.2,
            activity_stats,
            bottlenecks,
        }
    }

    fn calculate_cycle_time_metrics(&self) -> (f64, f64, f64, f64) {
        let mut cycle_times: Vec<i64> = self.instances
            .iter()
            .map(|i| i.cycle_time_ms())
            .collect();

        if cycle_times.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        cycle_times.sort();

        let avg = cycle_times.iter().sum::<i64>() as f64 / cycle_times.len() as f64;
        let median = cycle_times[cycle_times.len() / 2] as f64;
        let p95 = cycle_times[(cycle_times.len() as f64 * 0.95) as usize] as f64;
        let p99 = cycle_times[(cycle_times.len() as f64 * 0.99) as usize] as f64;

        (avg, median, p95, p99)
    }

    fn calculate_throughput(&self) -> (f64, usize) {
        if self.instances.is_empty() {
            return (0.0, 0);
        }

        let first_start = self.instances.iter()
            .map(|i| i.start_time)
            .min()
            .unwrap();

        let last_end = self.instances.iter()
            .map(|i| i.end_time)
            .max()
            .unwrap();

        let time_span_hours = last_end.signed_duration_since(first_start)
            .num_seconds() as f64 / 3600.0;

        let throughput = if time_span_hours > 0.0 {
            self.instances.len() as f64 / time_span_hours
        } else {
            0.0
        };

        (throughput, self.instances.len())
    }

    fn calculate_time_breakdown(&self) -> (f64, f64, f64) {
        if self.instances.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let total_processing: i64 = self.instances.iter()
            .map(|i| i.processing_time_ms())
            .sum();

        let total_waiting: i64 = self.instances.iter()
            .map(|i| i.waiting_time_ms())
            .sum();

        let count = self.instances.len() as f64;

        let avg_processing = total_processing as f64 / count;
        let avg_waiting = total_waiting as f64 / count;
        let ratio = if avg_processing + avg_waiting > 0.0 {
            avg_processing / (avg_processing + avg_waiting)
        } else {
            0.0
        };

        (avg_processing, avg_waiting, ratio)
    }

    fn calculate_activity_stats(&self) -> HashMap<String, ActivityStats> {
        let mut durations_by_activity: HashMap<String, Vec<i64>> = HashMap::new();

        for instance in &self.instances {
            for activity in &instance.activities {
                durations_by_activity
                    .entry(activity.activity.clone())
                    .or_default()
                    .push(activity.duration_ms());
            }
        }

        let mut stats = HashMap::new();

        for (activity, mut durations) in durations_by_activity {
            if durations.is_empty() {
                continue;
            }

            durations.sort();

            let count = durations.len();
            let sum: i64 = durations.iter().sum();
            let avg = sum as f64 / count as f64;

            let variance: f64 = durations.iter()
                .map(|&d| {
                    let diff = d as f64 - avg;
                    diff * diff
                })
                .sum::<f64>() / count as f64;

            let std_dev = variance.sqrt();
            let min = *durations.first().unwrap();
            let max = *durations.last().unwrap();
            let p95 = durations[(count as f64 * 0.95) as usize] as f64;

            stats.insert(
                activity.clone(),
                ActivityStats {
                    activity,
                    count,
                    avg_duration_ms: avg,
                    std_deviation_ms: std_dev,
                    min_duration_ms: min,
                    max_duration_ms: max,
                    p95_duration_ms: p95,
                },
            );
        }

        stats
    }

    fn identify_bottlenecks(&self, activity_stats: &HashMap<String, ActivityStats>) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        let total_time: f64 = activity_stats.values()
            .map(|s| s.avg_duration_ms * s.count as f64)
            .sum();

        for (activity, stats) in activity_stats {
            let time_contribution = stats.avg_duration_ms * stats.count as f64;
            let time_percentage = if total_time > 0.0 {
                time_contribution / total_time
            } else {
                0.0
            };

            // Bottleneck if:
            // 1. High time percentage (>20%)
            // 2. High standard deviation (inconsistent performance)
            // 3. Long average duration (>100ms)

            let mut impact_score = 0.0;
            let mut recommendations = Vec::new();

            if time_percentage > 0.2 {
                impact_score += time_percentage;
                recommendations.push(format!("Consumes {:.1}% of total time - optimize or parallelize", time_percentage * 100.0));
            }

            if stats.std_deviation_ms > stats.avg_duration_ms * 0.5 {
                impact_score += 0.3;
                recommendations.push("High variance indicates inconsistent performance - investigate causes".to_string());
            }

            if stats.avg_duration_ms > 100.0 {
                impact_score += 0.2;
                recommendations.push("Long average duration - consider caching or optimization".to_string());
            }

            if impact_score > 0.3 {
                bottlenecks.push(PerformanceBottleneck {
                    activity: activity.clone(),
                    impact_score,
                    avg_duration_ms: stats.avg_duration_ms,
                    frequency: stats.count,
                    recommendation: recommendations.join("; "),
                });
            }
        }

        bottlenecks.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());
        bottlenecks
    }
}

// ============================================================================
// Test Data Generation
// ============================================================================

fn generate_workflow_instances(count: usize) -> Vec<WorkflowInstance> {
    let mut instances = Vec::new();
    let base_time = Utc::now();

    for i in 0..count {
        let instance_start = base_time + Duration::seconds(i as i64 * 20);

        let mut activities = Vec::new();
        let mut current_time = instance_start;

        // Activity 1: validation (fast)
        let activity1_duration = Duration::milliseconds(10 + (i as i64 % 5));
        activities.push(ActivityExecution {
            activity: "validate_input".to_string(),
            start_time: current_time,
            end_time: current_time + activity1_duration,
        });
        current_time = current_time + activity1_duration + Duration::milliseconds(5); // 5ms wait

        // Activity 2: fetch data (medium, some variance)
        let activity2_duration = Duration::milliseconds(50 + (i as i64 % 30));
        activities.push(ActivityExecution {
            activity: "fetch_data".to_string(),
            start_time: current_time,
            end_time: current_time + activity2_duration,
        });
        current_time = current_time + activity2_duration + Duration::milliseconds(10); // 10ms wait

        // Activity 3: process (slow, high variance - bottleneck!)
        let activity3_duration = if i % 4 == 0 {
            Duration::milliseconds(200) // Sometimes very slow
        } else {
            Duration::milliseconds(80 + (i as i64 % 40))
        };
        activities.push(ActivityExecution {
            activity: "process_data".to_string(),
            start_time: current_time,
            end_time: current_time + activity3_duration,
        });
        current_time = current_time + activity3_duration + Duration::milliseconds(5);

        // Activity 4: save (fast)
        let activity4_duration = Duration::milliseconds(20);
        activities.push(ActivityExecution {
            activity: "save_result".to_string(),
            start_time: current_time,
            end_time: current_time + activity4_duration,
        });

        instances.push(WorkflowInstance {
            id: format!("wf_{:03}", i),
            start_time: instance_start,
            end_time: current_time + activity4_duration,
            activities,
        });
    }

    instances
}

// ============================================================================
// Main Example
// ============================================================================

fn main() {
    println!("=== Process Mining Performance Analytics Example ===\n");

    let start = Instant::now();

    // Generate test data
    println!("📊 Generating workflow instances...");
    let instances = generate_workflow_instances(50);
    println!("  ✅ Generated {} workflow instances\n", instances.len());

    // Analyze performance
    println!("🔍 Analyzing performance...");
    let analyzer = PerformanceAnalyzer::new(instances);
    let metrics = analyzer.analyze();
    println!("  ✅ Analysis complete\n");

    // Print comprehensive report
    println!("=== Performance Analysis Report ===\n");

    println!("⏱️  Cycle Time Metrics:");
    println!("  Average: {:.2}ms", metrics.avg_cycle_time_ms);
    println!("  Median: {:.2}ms", metrics.median_cycle_time_ms);
    println!("  P95: {:.2}ms", metrics.p95_cycle_time_ms);
    println!("  P99: {:.2}ms", metrics.p99_cycle_time_ms);

    println!("\n📈 Throughput Metrics:");
    println!("  Completed Instances: {}", metrics.completed_instances);
    println!("  Throughput: {:.2} workflows/hour", metrics.throughput_per_hour);

    println!("\n⚡ Time Breakdown:");
    println!("  Avg Processing Time: {:.2}ms", metrics.avg_processing_time_ms);
    println!("  Avg Waiting Time: {:.2}ms", metrics.avg_waiting_time_ms);
    println!("  Processing Ratio: {:.1}%", metrics.processing_time_ratio * 100.0);

    println!("\n📊 Activity Statistics:");
    let mut sorted_activities: Vec<_> = metrics.activity_stats.values().collect();
    sorted_activities.sort_by(|a, b| b.avg_duration_ms.partial_cmp(&a.avg_duration_ms).unwrap());

    for stats in sorted_activities {
        println!("\n  Activity: {}", stats.activity);
        println!("    Count: {}", stats.count);
        println!("    Avg: {:.2}ms (σ={:.2}ms)", stats.avg_duration_ms, stats.std_deviation_ms);
        println!("    Range: {}ms - {}ms", stats.min_duration_ms, stats.max_duration_ms);
        println!("    P95: {:.2}ms", stats.p95_duration_ms);
    }

    println!("\n🔍 Performance Bottlenecks:");
    if metrics.bottlenecks.is_empty() {
        println!("  ✅ No significant bottlenecks detected");
    } else {
        for (i, bottleneck) in metrics.bottlenecks.iter().enumerate() {
            println!("\n  Bottleneck #{}: {}", i + 1, bottleneck.activity);
            println!("    Impact Score: {:.2}", bottleneck.impact_score);
            println!("    Avg Duration: {:.2}ms", bottleneck.avg_duration_ms);
            println!("    Frequency: {}", bottleneck.frequency);
            println!("    Recommendation: {}", bottleneck.recommendation);
        }
    }

    println!("\n💡 Optimization Recommendations:\n");

    if metrics.processing_time_ratio < 0.7 {
        println!("  ⚠️  Low processing ratio ({:.1}%) - significant waiting time!", metrics.processing_time_ratio * 100.0);
        println!("     → Consider reducing gaps between activities");
        println!("     → Investigate resource allocation and scheduling\n");
    }

    if !metrics.bottlenecks.is_empty() {
        println!("  Focus on top {} bottleneck(s):", metrics.bottlenecks.len().min(3));
        for (i, b) in metrics.bottlenecks.iter().take(3).enumerate() {
            println!("  {}. {} - {}", i + 1, b.activity, b.recommendation.split(';').next().unwrap());
        }
        println!();
    }

    println!("  General optimization strategies:");
    println!("  - Enable caching for frequently called activities");
    println!("  - Parallelize independent activities");
    println!("  - Set up monitoring for P95/P99 latency");
    println!("  - Implement circuit breakers for unstable services");

    println!("\n⏱️  Analysis Time: {:?}\n", start.elapsed());

    println!("=== Key Insights ===\n");
    println!("1. Cycle Time Analysis:");
    println!("   - P95/P99 percentiles reveal tail latency");
    println!("   - Median better than average for skewed distributions\n");

    println!("2. Time Breakdown:");
    println!("   - Processing vs. waiting time ratio");
    println!("   - Low ratio indicates coordination overhead\n");

    println!("3. Activity-Level Metrics:");
    println!("   - Standard deviation reveals consistency");
    println!("   - High variance activities need investigation\n");

    println!("4. Bottleneck Detection:");
    println!("   - Impact score combines multiple factors");
    println!("   - Actionable recommendations for optimization\n");

    println!("=== Next Steps ===\n");
    println!("1. Set up continuous performance monitoring");
    println!("2. Implement alerting on P95/P99 thresholds");
    println!("3. Track performance trends over time");
    println!("4. Validate optimizations with A/B testing\n");
}
