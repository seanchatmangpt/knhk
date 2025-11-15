//! Process Discovery Example
//!
//! Demonstrates how to discover process structure from execution traces,
//! identify workflow patterns, compare discovered vs. declared process,
//! and find deviations from expected behavior.
//!
//! This example shows:
//! - Discovering process graph from event logs
//! - Identifying workflow patterns (sequences, loops, parallel)
//! - Validating against expected patterns
//! - Finding process deviations
//!
//! Run: `cargo run --example process-mining-discovery`

use chrono::{DateTime, Duration, Utc};
use hashbrown::{HashMap, HashSet};
use std::time::Instant;

// ============================================================================
// Event Log Structures
// ============================================================================

#[derive(Debug, Clone)]
struct ProcessEvent {
    case_id: String,
    activity: String,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct EventLog {
    events: Vec<ProcessEvent>,
}

impl EventLog {
    fn new(events: Vec<ProcessEvent>) -> Self {
        let mut sorted_events = events;
        sorted_events.sort_by_key(|e| e.timestamp);
        Self { events: sorted_events }
    }

    fn get_cases(&self) -> Vec<String> {
        let mut cases: Vec<_> = self
            .events
            .iter()
            .map(|e| e.case_id.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        cases.sort();
        cases
    }

    fn events_for_case(&self, case_id: &str) -> Vec<&ProcessEvent> {
        self.events.iter().filter(|e| e.case_id == case_id).collect()
    }
}

// ============================================================================
// Process Graph Discovery
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct ProcessEdge {
    from: String,
    to: String,
    frequency: usize,
    probability: f64,
}

#[derive(Debug, Clone)]
struct ProcessGraph {
    nodes: Vec<String>,
    edges: Vec<ProcessEdge>,
    start_activities: Vec<String>,
    end_activities: Vec<String>,
}

impl ProcessGraph {
    fn discover(event_log: &EventLog) -> Self {
        let mut nodes = HashSet::new();
        let mut transition_counts: HashMap<(String, String), usize> = HashMap::new();
        let mut outgoing_counts: HashMap<String, usize> = HashMap::new();
        let mut start_activities = HashSet::new();
        let mut end_activities = HashSet::new();

        // Discover nodes, edges, start/end activities
        for case_id in event_log.get_cases() {
            let case_events = event_log.events_for_case(&case_id);

            if let Some(first) = case_events.first() {
                start_activities.insert(first.activity.clone());
            }

            if let Some(last) = case_events.last() {
                end_activities.insert(last.activity.clone());
            }

            for event in &case_events {
                nodes.insert(event.activity.clone());
            }

            for window in case_events.windows(2) {
                let from = window[0].activity.clone();
                let to = window[1].activity.clone();

                *transition_counts.entry((from.clone(), to)).or_insert(0) += 1;
                *outgoing_counts.entry(from).or_insert(0) += 1;
            }
        }

        // Build edges with probabilities
        let mut edges = Vec::new();

        for ((from, to), frequency) in transition_counts {
            let total_outgoing = *outgoing_counts.get(&from).unwrap_or(&1);
            let probability = frequency as f64 / total_outgoing as f64;

            edges.push(ProcessEdge {
                from,
                to,
                frequency,
                probability,
            });
        }

        edges.sort_by(|a, b| b.frequency.cmp(&a.frequency));

        let mut nodes: Vec<_> = nodes.into_iter().collect();
        nodes.sort();

        let mut start: Vec<_> = start_activities.into_iter().collect();
        start.sort();

        let mut end: Vec<_> = end_activities.into_iter().collect();
        end.sort();

        Self {
            nodes,
            edges,
            start_activities: start,
            end_activities: end,
        }
    }

    fn print(&self) {
        println!("\n=== Discovered Process Graph ===\n");

        println!("📍 Activities ({}):", self.nodes.len());
        for node in &self.nodes {
            println!("  - {}", node);
        }

        println!("\n🚀 Start Activities:");
        for start in &self.start_activities {
            println!("  - {}", start);
        }

        println!("\n🏁 End Activities:");
        for end in &self.end_activities {
            println!("  - {}", end);
        }

        println!("\n🔀 Transitions ({}):", self.edges.len());
        for edge in &self.edges {
            println!(
                "  {} → {} (freq: {}, prob: {:.2})",
                edge.from, edge.to, edge.frequency, edge.probability
            );
        }
    }
}

// ============================================================================
// Pattern Discovery
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum PatternType {
    Sequence,
    ParallelSplit,
    Loop,
    Choice,
}

#[derive(Debug, Clone)]
struct DiscoveredPattern {
    pattern_type: PatternType,
    activities: Vec<String>,
    confidence: f64,
    description: String,
}

struct PatternDiscovery;

impl PatternDiscovery {
    fn discover_patterns(graph: &ProcessGraph) -> Vec<DiscoveredPattern> {
        let mut patterns = Vec::new();

        // Discover sequences (high probability transitions)
        for edge in &graph.edges {
            if edge.probability > 0.8 {
                patterns.push(DiscoveredPattern {
                    pattern_type: PatternType::Sequence,
                    activities: vec![edge.from.clone(), edge.to.clone()],
                    confidence: edge.probability,
                    description: format!("{} always followed by {}", edge.from, edge.to),
                });
            }
        }

        // Discover parallel splits (one activity → multiple next activities)
        let mut outgoing: HashMap<String, Vec<&ProcessEdge>> = HashMap::new();
        for edge in &graph.edges {
            outgoing.entry(edge.from.clone()).or_default().push(edge);
        }

        for (source, edges) in outgoing {
            if edges.len() > 1 {
                let activities: Vec<_> = edges.iter().map(|e| e.to.clone()).collect();
                let avg_prob: f64 = edges.iter().map(|e| e.probability).sum::<f64>() / edges.len() as f64;

                patterns.push(DiscoveredPattern {
                    pattern_type: PatternType::ParallelSplit,
                    activities,
                    confidence: avg_prob,
                    description: format!("{} branches to multiple paths", source),
                });
            }
        }

        // Discover loops (A → B → A pattern)
        for edge in &graph.edges {
            if let Some(back_edge) = graph.edges.iter().find(|e| e.from == edge.to && e.to == edge.from) {
                let confidence = (edge.probability + back_edge.probability) / 2.0;

                patterns.push(DiscoveredPattern {
                    pattern_type: PatternType::Loop,
                    activities: vec![edge.from.clone(), edge.to.clone()],
                    confidence,
                    description: format!("Loop between {} and {}", edge.from, edge.to),
                });
            }
        }

        // Discover choices (low probability alternatives)
        for edge in &graph.edges {
            if edge.probability < 0.5 && edge.probability > 0.1 {
                patterns.push(DiscoveredPattern {
                    pattern_type: PatternType::Choice,
                    activities: vec![edge.from.clone(), edge.to.clone()],
                    confidence: edge.probability,
                    description: format!("{} sometimes goes to {}", edge.from, edge.to),
                });
            }
        }

        patterns
    }

    fn print_patterns(patterns: &[DiscoveredPattern]) {
        println!("\n=== Discovered Workflow Patterns ===\n");

        let mut by_type: HashMap<PatternType, Vec<&DiscoveredPattern>> = HashMap::new();
        for pattern in patterns {
            by_type.entry(pattern.pattern_type.clone()).or_default().push(pattern);
        }

        for (pattern_type, patterns) in by_type {
            println!("{:?} Patterns ({}):", pattern_type, patterns.len());
            for pattern in patterns {
                println!("  - {} (confidence: {:.2})", pattern.description, pattern.confidence);
            }
            println!();
        }
    }
}

// ============================================================================
// Pattern Validation
// ============================================================================

#[derive(Debug, Clone)]
struct ExpectedPattern {
    pattern_type: PatternType,
    activities: Vec<String>,
    required: bool,
}

struct PatternValidator {
    expected_patterns: Vec<ExpectedPattern>,
}

impl PatternValidator {
    fn new(expected_patterns: Vec<ExpectedPattern>) -> Self {
        Self { expected_patterns }
    }

    fn validate(&self, discovered_patterns: &[DiscoveredPattern]) -> ValidationReport {
        let mut matched = Vec::new();
        let mut missing = Vec::new();
        let mut unexpected = Vec::new();

        // Check for expected patterns
        for expected in &self.expected_patterns {
            let found = discovered_patterns.iter().any(|d| {
                d.pattern_type == expected.pattern_type
                    && Self::activities_match(&d.activities, &expected.activities)
            });

            if found {
                matched.push(expected.clone());
            } else if expected.required {
                missing.push(expected.clone());
            }
        }

        // Check for unexpected patterns
        for discovered in discovered_patterns {
            let expected = self.expected_patterns.iter().any(|e| {
                e.pattern_type == discovered.pattern_type
                    && Self::activities_match(&discovered.activities, &e.activities)
            });

            if !expected {
                unexpected.push(discovered.clone());
            }
        }

        let conformance = if self.expected_patterns.is_empty() {
            1.0
        } else {
            matched.len() as f64 / self.expected_patterns.len() as f64
        };

        ValidationReport {
            conformance,
            matched,
            missing,
            unexpected,
        }
    }

    fn activities_match(a: &[String], b: &[String]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let set_a: HashSet<_> = a.iter().collect();
        let set_b: HashSet<_> = b.iter().collect();
        set_a == set_b
    }
}

#[derive(Debug)]
struct ValidationReport {
    conformance: f64,
    matched: Vec<ExpectedPattern>,
    missing: Vec<ExpectedPattern>,
    unexpected: Vec<DiscoveredPattern>,
}

impl ValidationReport {
    fn print(&self) {
        println!("\n=== Pattern Validation Report ===\n");

        println!("📊 Conformance Score: {:.1}%\n", self.conformance * 100.0);

        println!("✅ Matched Expected Patterns ({}):", self.matched.len());
        for pattern in &self.matched {
            println!("  - {:?}: {:?}", pattern.pattern_type, pattern.activities);
        }
        println!();

        if !self.missing.is_empty() {
            println!("❌ Missing Required Patterns ({}):", self.missing.len());
            for pattern in &self.missing {
                println!("  - {:?}: {:?}", pattern.pattern_type, pattern.activities);
            }
            println!();
        }

        if !self.unexpected.is_empty() {
            println!("⚠️  Unexpected Patterns Found ({}):", self.unexpected.len());
            for pattern in &self.unexpected {
                println!("  - {}", pattern.description);
            }
            println!();
        }
    }
}

// ============================================================================
// Test Data Generation
// ============================================================================

fn create_sample_event_log() -> EventLog {
    let mut events = Vec::new();
    let base_time = Utc::now();

    // Standard workflow: validate → fetch → process → save
    for i in 0..5 {
        let case_id = format!("case_{:03}", i);
        let offset = Duration::seconds(i * 10);

        events.push(ProcessEvent {
            case_id: case_id.clone(),
            activity: "validate".to_string(),
            timestamp: base_time + offset,
        });

        events.push(ProcessEvent {
            case_id: case_id.clone(),
            activity: "fetch".to_string(),
            timestamp: base_time + offset + Duration::seconds(1),
        });

        events.push(ProcessEvent {
            case_id: case_id.clone(),
            activity: "process".to_string(),
            timestamp: base_time + offset + Duration::seconds(2),
        });

        events.push(ProcessEvent {
            case_id: case_id.clone(),
            activity: "save".to_string(),
            timestamp: base_time + offset + Duration::seconds(3),
        });
    }

    // Variant 1: validation failure → retry
    for i in 5..7 {
        let case_id = format!("case_{:03}", i);
        let offset = Duration::seconds(i * 10);

        events.push(ProcessEvent {
            case_id: case_id.clone(),
            activity: "validate".to_string(),
            timestamp: base_time + offset,
        });

        events.push(ProcessEvent {
            case_id: case_id.clone(),
            activity: "retry".to_string(),
            timestamp: base_time + offset + Duration::seconds(1),
        });

        events.push(ProcessEvent {
            case_id: case_id.clone(),
            activity: "validate".to_string(),
            timestamp: base_time + offset + Duration::seconds(2),
        });

        events.push(ProcessEvent {
            case_id: case_id.clone(),
            activity: "fetch".to_string(),
            timestamp: base_time + offset + Duration::seconds(3),
        });

        events.push(ProcessEvent {
            case_id: case_id.clone(),
            activity: "process".to_string(),
            timestamp: base_time + offset + Duration::seconds(4),
        });

        events.push(ProcessEvent {
            case_id: case_id.clone(),
            activity: "save".to_string(),
            timestamp: base_time + offset + Duration::seconds(5),
        });
    }

    EventLog::new(events)
}

// ============================================================================
// Main Example
// ============================================================================

fn main() {
    println!("=== Process Discovery Example ===\n");

    let start = Instant::now();

    // Create sample event log
    println!("📋 Creating sample event log...");
    let event_log = create_sample_event_log();
    println!("  ✅ Event log created: {} events, {} cases\n", event_log.events.len(), event_log.get_cases().len());

    // Discover process graph
    println!("🔍 Discovering process structure...");
    let process_graph = ProcessGraph::discover(&event_log);
    println!("  ✅ Process discovered\n");

    process_graph.print();

    // Discover patterns
    println!("\n🎯 Discovering workflow patterns...");
    let patterns = PatternDiscovery::discover_patterns(&process_graph);
    println!("  ✅ Found {} patterns\n", patterns.len());

    PatternDiscovery::print_patterns(&patterns);

    // Validate against expected patterns
    println!("✅ Validating against expected process...\n");

    let expected_patterns = vec![
        ExpectedPattern {
            pattern_type: PatternType::Sequence,
            activities: vec!["validate".to_string(), "fetch".to_string()],
            required: true,
        },
        ExpectedPattern {
            pattern_type: PatternType::Sequence,
            activities: vec!["fetch".to_string(), "process".to_string()],
            required: true,
        },
        ExpectedPattern {
            pattern_type: PatternType::Loop,
            activities: vec!["validate".to_string(), "retry".to_string()],
            required: false,
        },
    ];

    let validator = PatternValidator::new(expected_patterns);
    let report = validator.validate(&patterns);

    report.print();

    println!("⏱️  Discovery Time: {:?}\n", start.elapsed());

    println!("=== Key Insights ===\n");
    println!("1. Process Structure:");
    println!("   - Discovered {} activities and {} transitions", process_graph.nodes.len(), process_graph.edges.len());
    println!("   - Identified start and end activities");
    println!("   - Calculated transition probabilities\n");

    println!("2. Pattern Recognition:");
    println!("   - Detected sequences, loops, and choices");
    println!("   - Confidence scores based on frequency");
    println!("   - Revealed actual workflow behavior\n");

    println!("3. Conformance Checking:");
    println!("   - Validated against expected patterns");
    println!("   - Conformance: {:.1}%", report.conformance * 100.0);
    println!("   - Identified deviations from design\n");

    println!("=== Next Steps ===\n");
    println!("1. Export discovered graph to visualization tools");
    println!("2. Compare discovered vs. declared process models");
    println!("3. Investigate unexpected patterns for anomalies");
    println!("4. Update process documentation with actual behavior\n");
}
