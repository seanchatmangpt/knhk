//! Process discovery from execution traces
//!
//! This module discovers actual process structure from event logs,
//! validates against expected patterns, and identifies deviations.

use crate::event_log::{EventLog, ProcessEvent};
use crate::{ProcessMiningError, Result};
use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Discovered process graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessGraph {
    /// Nodes (activities)
    pub nodes: Vec<String>,

    /// Edges (transitions between activities)
    pub edges: Vec<ProcessEdge>,

    /// Start activities
    pub start_activities: Vec<String>,

    /// End activities
    pub end_activities: Vec<String>,

    /// Discovered patterns
    pub patterns: Vec<DiscoveredPattern>,
}

/// Edge in the process graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEdge {
    /// Source activity
    pub from: String,

    /// Target activity
    pub to: String,

    /// Frequency (number of times this transition occurred)
    pub frequency: usize,

    /// Probability (frequency / total outgoing from source)
    pub probability: f64,
}

/// Discovered workflow pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPattern {
    /// Pattern type
    pub pattern_type: PatternType,

    /// Involved activities
    pub activities: Vec<String>,

    /// Confidence (0.0 to 1.0)
    pub confidence: f64,

    /// Description
    pub description: String,
}

/// Types of workflow patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    Sequence,
    ParallelSplit,
    ParallelJoin,
    ExclusiveChoice,
    SimpleMerge,
    Loop,
}

/// Process discovery engine
#[derive(Debug, Default)]
pub struct DiscoveryEngine {
    min_frequency: usize,
    min_confidence: f64,
}

impl DiscoveryEngine {
    /// Create new discovery engine
    pub fn new() -> Self {
        Self {
            min_frequency: 2,
            min_confidence: 0.5,
        }
    }

    /// Set minimum frequency threshold
    pub fn with_min_frequency(mut self, freq: usize) -> Self {
        self.min_frequency = freq;
        self
    }

    /// Set minimum confidence threshold
    pub fn with_min_confidence(mut self, conf: f64) -> Self {
        self.min_confidence = conf;
        self
    }

    /// Discover process from event log
    pub fn discover(&self, event_log: &EventLog) -> Result<ProcessGraph> {
        let nodes = self.discover_activities(event_log)?;
        let edges = self.discover_transitions(event_log)?;
        let (start_activities, end_activities) = self.discover_start_end(event_log)?;
        let patterns = self.discover_patterns(event_log, &edges)?;

        Ok(ProcessGraph {
            nodes,
            edges,
            start_activities,
            end_activities,
            patterns,
        })
    }

    /// Discover all activities
    fn discover_activities(&self, event_log: &EventLog) -> Result<Vec<String>> {
        Ok(event_log.activities.clone())
    }

    /// Discover transitions between activities
    fn discover_transitions(&self, event_log: &EventLog) -> Result<Vec<ProcessEdge>> {
        let mut transition_counts: HashMap<(String, String), usize> = HashMap::new();
        let mut outgoing_counts: HashMap<String, usize> = HashMap::new();

        for case_id in &event_log.case_ids {
            let events: Vec<_> = event_log.events_for_case(case_id);

            for window in events.windows(2) {
                let from = window[0].activity.clone();
                let to = window[1].activity.clone();

                *transition_counts.entry((from.clone(), to)).or_insert(0) += 1;
                *outgoing_counts.entry(from).or_insert(0) += 1;
            }
        }

        let mut edges = Vec::new();

        for ((from, to), frequency) in transition_counts {
            if frequency >= self.min_frequency {
                let total_outgoing = *outgoing_counts.get(&from).unwrap_or(&1);
                let probability = frequency as f64 / total_outgoing as f64;

                edges.push(ProcessEdge {
                    from,
                    to,
                    frequency,
                    probability,
                });
            }
        }

        // Sort by frequency (descending)
        edges.sort_by(|a, b| b.frequency.cmp(&a.frequency));

        Ok(edges)
    }

    /// Discover start and end activities
    fn discover_start_end(&self, event_log: &EventLog) -> Result<(Vec<String>, Vec<String>)> {
        let mut start_activities = HashSet::new();
        let mut end_activities = HashSet::new();

        for case_id in &event_log.case_ids {
            let events: Vec<_> = event_log.events_for_case(case_id);

            if let Some(first) = events.first() {
                start_activities.insert(first.activity.clone());
            }

            if let Some(last) = events.last() {
                end_activities.insert(last.activity.clone());
            }
        }

        let mut start: Vec<_> = start_activities.into_iter().collect();
        start.sort();

        let mut end: Vec<_> = end_activities.into_iter().collect();
        end.sort();

        Ok((start, end))
    }

    /// Discover workflow patterns
    fn discover_patterns(
        &self,
        event_log: &EventLog,
        edges: &[ProcessEdge],
    ) -> Result<Vec<DiscoveredPattern>> {
        let mut patterns = Vec::new();

        // Detect sequences
        patterns.extend(self.detect_sequences(edges));

        // Detect parallel patterns
        patterns.extend(self.detect_parallel(edges));

        // Detect loops
        patterns.extend(self.detect_loops(edges));

        // Filter by confidence
        patterns.retain(|p| p.confidence >= self.min_confidence);

        Ok(patterns)
    }

    /// Detect sequence patterns
    fn detect_sequences(&self, edges: &[ProcessEdge]) -> Vec<DiscoveredPattern> {
        let mut patterns = Vec::new();

        for edge in edges {
            if edge.probability > 0.8 {
                patterns.push(DiscoveredPattern {
                    pattern_type: PatternType::Sequence,
                    activities: vec![edge.from.clone(), edge.to.clone()],
                    confidence: edge.probability,
                    description: format!("{} → {} (sequential)", edge.from, edge.to),
                });
            }
        }

        patterns
    }

    /// Detect parallel patterns
    fn detect_parallel(&self, edges: &[ProcessEdge]) -> Vec<DiscoveredPattern> {
        let mut patterns = Vec::new();

        // Group edges by source
        let mut by_source: HashMap<String, Vec<&ProcessEdge>> = HashMap::new();
        for edge in edges {
            by_source.entry(edge.from.clone()).or_default().push(edge);
        }

        // Look for parallel splits (one source → multiple targets)
        for (source, outgoing) in by_source {
            if outgoing.len() > 1 {
                let activities: Vec<_> = outgoing.iter().map(|e| e.to.clone()).collect();
                let avg_prob: f64 =
                    outgoing.iter().map(|e| e.probability).sum::<f64>() / outgoing.len() as f64;

                patterns.push(DiscoveredPattern {
                    pattern_type: PatternType::ParallelSplit,
                    activities,
                    confidence: avg_prob,
                    description: format!("{} splits to multiple branches", source),
                });
            }
        }

        patterns
    }

    /// Detect loop patterns
    fn detect_loops(&self, edges: &[ProcessEdge]) -> Vec<DiscoveredPattern> {
        let mut patterns = Vec::new();

        for edge in edges {
            // Simple loop: A → B → A
            if let Some(back_edge) = edges
                .iter()
                .find(|e| e.from == edge.to && e.to == edge.from)
            {
                let confidence = (edge.probability + back_edge.probability) / 2.0;

                patterns.push(DiscoveredPattern {
                    pattern_type: PatternType::Loop,
                    activities: vec![edge.from.clone(), edge.to.clone()],
                    confidence,
                    description: format!("Loop between {} and {}", edge.from, edge.to),
                });
            }
        }

        patterns
    }
}

/// Pattern validator
#[derive(Debug)]
pub struct PatternValidator {
    expected_patterns: Vec<ExpectedPattern>,
}

/// Expected pattern definition
#[derive(Debug, Clone)]
pub struct ExpectedPattern {
    pub pattern_type: PatternType,
    pub activities: Vec<String>,
    pub required: bool,
}

impl PatternValidator {
    /// Create new validator
    pub fn new(expected_patterns: Vec<ExpectedPattern>) -> Self {
        Self { expected_patterns }
    }

    /// Validate discovered process against expected patterns
    pub fn validate(&self, discovered: &ProcessGraph) -> Result<ValidationReport> {
        let mut matched = Vec::new();
        let mut missing = Vec::new();
        let mut unexpected = Vec::new();

        // Check for expected patterns
        for expected in &self.expected_patterns {
            let found = discovered.patterns.iter().any(|p| {
                p.pattern_type == expected.pattern_type
                    && self.activities_match(&p.activities, &expected.activities)
            });

            if found {
                matched.push(expected.clone());
            } else if expected.required {
                missing.push(expected.clone());
            }
        }

        // Check for unexpected patterns
        for discovered_pattern in &discovered.patterns {
            let expected = self.expected_patterns.iter().any(|e| {
                e.pattern_type == discovered_pattern.pattern_type
                    && self.activities_match(&discovered_pattern.activities, &e.activities)
            });

            if !expected {
                unexpected.push(discovered_pattern.clone());
            }
        }

        let conformance = if self.expected_patterns.is_empty() {
            1.0
        } else {
            matched.len() as f64 / self.expected_patterns.len() as f64
        };

        Ok(ValidationReport {
            conformance,
            matched,
            missing,
            unexpected,
        })
    }

    /// Check if activities match (order-independent)
    fn activities_match(&self, a: &[String], b: &[String]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let set_a: HashSet<_> = a.iter().collect();
        let set_b: HashSet<_> = b.iter().collect();

        set_a == set_b
    }
}

/// Validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Conformance score (0.0 to 1.0)
    pub conformance: f64,

    /// Matched expected patterns
    pub matched: Vec<ExpectedPattern>,

    /// Missing required patterns
    pub missing: Vec<ExpectedPattern>,

    /// Unexpected patterns found
    pub unexpected: Vec<DiscoveredPattern>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_log::EventLogBuilder;
    use chrono::Utc;

    #[test]
    fn test_discovery_engine() {
        let mut builder = EventLogBuilder::new();
        let now = Utc::now();

        // Create simple sequential process
        for i in 0..3 {
            builder.add_span_event(
                format!("case_{}", i),
                "step_1".to_string(),
                now + chrono::Duration::seconds(i * 10),
                None,
                HashMap::new(),
            );

            builder.add_span_event(
                format!("case_{}", i),
                "step_2".to_string(),
                now + chrono::Duration::seconds(i * 10 + 5),
                None,
                HashMap::new(),
            );
        }

        let log = builder.build().unwrap();
        let engine = DiscoveryEngine::new();
        let graph = engine.discover(&log).unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert!(!graph.edges.is_empty());
        assert!(!graph.start_activities.is_empty());
        assert!(!graph.end_activities.is_empty());
    }

    #[test]
    fn test_pattern_validator() {
        let expected = vec![ExpectedPattern {
            pattern_type: PatternType::Sequence,
            activities: vec!["A".to_string(), "B".to_string()],
            required: true,
        }];

        let validator = PatternValidator::new(expected);

        let discovered = ProcessGraph {
            nodes: vec!["A".to_string(), "B".to_string()],
            edges: vec![],
            start_activities: vec!["A".to_string()],
            end_activities: vec!["B".to_string()],
            patterns: vec![DiscoveredPattern {
                pattern_type: PatternType::Sequence,
                activities: vec!["A".to_string(), "B".to_string()],
                confidence: 0.9,
                description: "A → B".to_string(),
            }],
        };

        let report = validator.validate(&discovered).unwrap();
        assert!(report.conformance > 0.9);
        assert_eq!(report.matched.len(), 1);
        assert_eq!(report.missing.len(), 0);
    }
}
