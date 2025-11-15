//! # Resource Handle Pattern - Poka-Yoke Implementation
//!
//! This module implements resource handles with type-safe lifecycle management.
//! Resources cannot be used after being closed or in invalid states.
//!
//! ## Poka-Yoke Principles Applied:
//!
//! 1. **Type States for Lifecycle**: Open/Closed states are distinct types
//! 2. **Consuming Transitions**: Closing a resource consumes it, preventing further use
//! 3. **Compile-Time Prevention**: Cannot read from closed resource - compiler error
//! 4. **Resource Safety**: Ensures proper cleanup and prevents use-after-free bugs
//! 5. **Linear Types Pattern**: Resources can only be used once per operation
//!
//! ## Invalid States Made Impossible:
//!
//! - Cannot add events to closed EventLog - compiler error
//! - Cannot analyze open EventLog - must close first
//! - Cannot re-open closed EventLog - consumed type
//! - Cannot forget to close resources - lint warnings for unused values
//! - Cannot access results before completion - type prevents it

use crate::builders::Event;
use crate::types::{ActivityName, CaseID, Count, Duration, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;

// ============================================================================
// EventLog Resource Handle
// ============================================================================

/// Type state: EventLog is open for writing
pub struct EventLogOpen;

/// Type state: EventLog is closed for analysis
pub struct EventLogClosed;

/// Type-safe event log with lifecycle management.
///
/// ## Poka-Yoke Design:
///
/// The event log has two distinct states:
/// - **Open**: Can add events, cannot analyze
/// - **Closed**: Cannot add events, can analyze
///
/// State transitions are one-way and consuming:
///
/// ```text
/// EventLog<Open>
///     .add_event() -> Result<EventLog<Open>>  (stays open)
///     .close() -> EventLog<Closed>             (consumes, returns closed)
///
/// EventLog<Closed>
///     .analyze() -> ProcessAnalytics           (read-only operations)
/// ```
///
/// ## Compile-Time Guarantees:
///
/// - Cannot add events to closed log
/// - Cannot analyze open log (ensures data consistency)
/// - Cannot re-open closed log (must create new log)
///
/// ## Example:
///
/// ```rust
/// use knhk_process_mining::resource_handles::EventLog;
/// use knhk_process_mining::builders::{Event, EventBuilder};
///
/// // Create open log
/// let mut log = EventLog::new();
///
/// // Add events (log is open)
/// log = log.add_event(event1).unwrap();
/// log = log.add_event(event2).unwrap();
///
/// // Close log (consumes open log, returns closed log)
/// let closed_log = log.close();
///
/// // COMPILER ERROR: log was moved when closed
/// // log.add_event(event3); // ERROR: use of moved value
///
/// // Analyze closed log
/// let analytics = closed_log.analyze();
/// ```
#[derive(Debug, Clone)]
pub struct EventLog<State> {
    events: Vec<Event>,
    metadata: EventLogMetadata,
    _state: PhantomData<State>,
}

/// Metadata about the event log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogMetadata {
    /// When the log was created
    pub created_at: Timestamp,
    /// When the log was closed (if applicable)
    pub closed_at: Option<Timestamp>,
    /// Number of unique cases
    pub case_count: u64,
    /// Number of unique activities
    pub activity_count: u64,
}

impl EventLog<EventLogOpen> {
    /// Creates a new open EventLog.
    ///
    /// ## Poka-Yoke:
    /// This is the ONLY way to create an EventLog, ensuring it starts in Open state.
    pub fn new() -> Self {
        EventLog {
            events: Vec::new(),
            metadata: EventLogMetadata {
                created_at: Timestamp::now(),
                closed_at: None,
                case_count: 0,
                activity_count: 0,
            },
            _state: PhantomData,
        }
    }

    /// Adds an event to the log.
    ///
    /// ## Poka-Yoke:
    /// - Consumes and returns `self`, maintaining open state
    /// - Only available when log is open
    /// - Updates metadata automatically
    ///
    /// ## Errors
    /// Returns error if event is invalid or conflicts with existing events.
    pub fn add_event(mut self, event: Event) -> Result<Self, EventLogError> {
        // Validate event timestamp ordering
        if let Some(last_event) = self.events.last() {
            if event.timestamp < last_event.timestamp {
                return Err(EventLogError::InvalidEventOrder {
                    last_timestamp: last_event.timestamp,
                    new_timestamp: event.timestamp,
                });
            }
        }

        // Update metadata
        let unique_cases: std::collections::HashSet<_> =
            self.events.iter().map(|e| e.case_id).collect();
        let unique_activities: std::collections::HashSet<_> =
            self.events.iter().map(|e| &e.activity).collect();

        self.metadata.case_count = unique_cases.len() as u64;
        self.metadata.activity_count = unique_activities.len() as u64;

        self.events.push(event);

        Ok(self)
    }

    /// Adds multiple events to the log.
    ///
    /// ## Poka-Yoke:
    /// - Validates all events before adding any
    /// - Atomic operation (all or nothing)
    /// - Consumes and returns `self`
    pub fn add_events(mut self, events: Vec<Event>) -> Result<Self, EventLogError> {
        // Validate all events first
        for event in &events {
            if let Some(last_event) = self.events.last() {
                if event.timestamp < last_event.timestamp {
                    return Err(EventLogError::InvalidEventOrder {
                        last_timestamp: last_event.timestamp,
                        new_timestamp: event.timestamp,
                    });
                }
            }
        }

        // All events are valid, add them
        self.events.extend(events);

        // Update metadata
        let unique_cases: std::collections::HashSet<_> =
            self.events.iter().map(|e| e.case_id).collect();
        let unique_activities: std::collections::HashSet<_> =
            self.events.iter().map(|e| &e.activity).collect();

        self.metadata.case_count = unique_cases.len() as u64;
        self.metadata.activity_count = unique_activities.len() as u64;

        Ok(self)
    }

    /// Closes the event log for analysis.
    ///
    /// ## Poka-Yoke:
    /// - Consumes `self` (open log cannot be used after closing)
    /// - Returns new type `EventLog<Closed>` (enables analysis methods)
    /// - Records close timestamp
    /// - Finalizes metadata
    ///
    /// After calling this method, the open log is consumed and cannot be used.
    /// The returned closed log enables analysis operations.
    pub fn close(mut self) -> EventLog<EventLogClosed> {
        self.metadata.closed_at = Some(Timestamp::now());

        EventLog {
            events: self.events,
            metadata: self.metadata,
            _state: PhantomData,
        }
    }

    /// Returns the number of events currently in the log.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Returns a reference to the metadata.
    pub fn metadata(&self) -> &EventLogMetadata {
        &self.metadata
    }
}

impl EventLog<EventLogClosed> {
    /// Analyzes the closed event log.
    ///
    /// ## Poka-Yoke:
    /// - Only available on closed log (data is immutable)
    /// - Guarantees complete dataset (no concurrent modifications)
    /// - Type system prevents analysis of incomplete data
    ///
    /// ## Returns
    /// Process analytics computed from all events in the log.
    pub fn analyze(&self) -> ProcessAnalytics {
        let mut case_durations: HashMap<CaseID, Duration> = HashMap::new();
        let mut activity_counts: HashMap<ActivityName, u32> = HashMap::new();
        let mut case_events: HashMap<CaseID, Vec<&Event>> = HashMap::new();

        // Group events by case
        for event in &self.events {
            case_events
                .entry(event.case_id)
                .or_insert_with(Vec::new)
                .push(event);

            *activity_counts.entry(event.activity.clone()).or_insert(0) += 1;
        }

        // Calculate case durations
        for (case_id, events) in &case_events {
            if events.len() >= 2 {
                let start = events.first().unwrap().timestamp;
                let end = events.last().unwrap().timestamp;
                if let Some(duration) = end.duration_since(start) {
                    case_durations.insert(*case_id, duration);
                }
            }
        }

        // Calculate average duration
        let total_duration: u64 = case_durations.values().map(|d| d.as_millis()).sum();
        let avg_duration = if !case_durations.is_empty() {
            Duration::new(total_duration / case_durations.len() as u64)
        } else {
            Duration::new(0)
        };

        ProcessAnalytics {
            total_events: self.events.len() as u64,
            total_cases: self.metadata.case_count,
            total_activities: self.metadata.activity_count,
            average_case_duration: avg_duration,
            activity_frequencies: activity_counts,
        }
    }

    /// Returns an iterator over the events.
    ///
    /// ## Poka-Yoke:
    /// Returns immutable references (data cannot be modified in closed state).
    pub fn events(&self) -> impl Iterator<Item = &Event> {
        self.events.iter()
    }

    /// Returns the metadata.
    pub fn metadata(&self) -> &EventLogMetadata {
        &self.metadata
    }

    /// Returns the number of events in the log.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

impl Default for EventLog<EventLogOpen> {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur when working with EventLog
#[derive(Debug, Clone, PartialEq)]
pub enum EventLogError {
    /// Events must be in chronological order
    InvalidEventOrder {
        last_timestamp: Timestamp,
        new_timestamp: Timestamp,
    },
    /// Event validation failed
    InvalidEvent { reason: String },
}

impl std::fmt::Display for EventLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventLogError::InvalidEventOrder {
                last_timestamp,
                new_timestamp,
            } => {
                write!(
                    f,
                    "Events must be in chronological order: last={}, new={}",
                    last_timestamp, new_timestamp
                )
            }
            EventLogError::InvalidEvent { reason } => {
                write!(f, "Invalid event: {}", reason)
            }
        }
    }
}

impl std::error::Error for EventLogError {}

/// Process analytics computed from event log
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessAnalytics {
    /// Total number of events
    pub total_events: u64,
    /// Total number of unique cases
    pub total_cases: u64,
    /// Total number of unique activities
    pub total_activities: u64,
    /// Average duration per case
    pub average_case_duration: Duration,
    /// Frequency of each activity
    pub activity_frequencies: HashMap<ActivityName, u32>,
}

// ============================================================================
// ProcessAnalyzer Resource Handle
// ============================================================================

/// Type state: Analyzer is configured but not running
pub struct AnalyzerConfigured;

/// Type state: Analyzer is running
pub struct AnalyzerRunning;

/// Type state: Analyzer has completed
pub struct AnalyzerCompleted;

/// Type-safe process analyzer with lifecycle management.
///
/// ## Poka-Yoke Design:
///
/// The analyzer has three distinct states:
/// - **Configured**: Can modify configuration, cannot get results
/// - **Running**: Cannot modify configuration, cannot get results yet
/// - **Completed**: Cannot modify configuration, can get results
///
/// State transitions are one-way and consuming:
///
/// ```text
/// ProcessAnalyzer<Configured>
///     .run() -> ProcessAnalyzer<Running>
///
/// ProcessAnalyzer<Running>
///     .complete() -> ProcessAnalyzer<Completed>
///
/// ProcessAnalyzer<Completed>
///     .results() -> AnalysisResults
/// ```
#[derive(Debug)]
pub struct ProcessAnalyzer<State> {
    event_log: Option<EventLog<EventLogClosed>>,
    config: AnalyzerConfig,
    results: Option<AnalysisResults>,
    _state: PhantomData<State>,
}

/// Configuration for process analyzer
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Include detailed trace analysis
    pub include_traces: bool,
    /// Calculate performance metrics
    pub calculate_performance: bool,
    /// Maximum number of variants to analyze
    pub max_variants: u32,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        AnalyzerConfig {
            include_traces: true,
            calculate_performance: true,
            max_variants: 100,
        }
    }
}

impl ProcessAnalyzer<AnalyzerConfigured> {
    /// Creates a new analyzer with the given event log.
    ///
    /// ## Poka-Yoke:
    /// - Requires closed event log (ensures complete data)
    /// - Starts in Configured state
    pub fn new(event_log: EventLog<EventLogClosed>) -> Self {
        ProcessAnalyzer {
            event_log: Some(event_log),
            config: AnalyzerConfig::default(),
            results: None,
            _state: PhantomData,
        }
    }

    /// Updates the analyzer configuration.
    ///
    /// ## Poka-Yoke:
    /// - Only available in Configured state
    /// - Consumes and returns `self`
    pub fn with_config(mut self, config: AnalyzerConfig) -> Self {
        self.config = config;
        self
    }

    /// Starts the analysis.
    ///
    /// ## Poka-Yoke:
    /// - Consumes configured analyzer
    /// - Returns running analyzer
    /// - Configuration cannot be changed after this point
    pub fn run(self) -> ProcessAnalyzer<AnalyzerRunning> {
        ProcessAnalyzer {
            event_log: self.event_log,
            config: self.config,
            results: None,
            _state: PhantomData,
        }
    }
}

impl ProcessAnalyzer<AnalyzerRunning> {
    /// Completes the analysis.
    ///
    /// ## Poka-Yoke:
    /// - Consumes running analyzer
    /// - Returns completed analyzer with results
    /// - Analysis cannot be re-run (must create new analyzer)
    pub fn complete(mut self) -> ProcessAnalyzer<AnalyzerCompleted> {
        // Perform analysis
        let analytics = self
            .event_log
            .as_ref()
            .expect("event log should be present")
            .analyze();

        let results = AnalysisResults {
            analytics,
            completed_at: Timestamp::now(),
        };

        ProcessAnalyzer {
            event_log: self.event_log,
            config: self.config,
            results: Some(results),
            _state: PhantomData,
        }
    }
}

impl ProcessAnalyzer<AnalyzerCompleted> {
    /// Gets the analysis results.
    ///
    /// ## Poka-Yoke:
    /// - Only available in Completed state
    /// - Results are guaranteed to exist (type ensures completion)
    pub fn results(&self) -> &AnalysisResults {
        self.results
            .as_ref()
            .expect("results should be present in completed state")
    }

    /// Consumes the analyzer and returns the results.
    pub fn into_results(self) -> AnalysisResults {
        self.results
            .expect("results should be present in completed state")
    }
}

/// Results of process analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisResults {
    /// Process analytics
    pub analytics: ProcessAnalytics,
    /// When the analysis was completed
    pub completed_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builders::EventBuilder;
    use crate::types::{ActivityName, CaseID, Timestamp};

    #[test]
    fn test_event_log_open_add_event() {
        let log = EventLog::new();
        let event = create_test_event(1, "Activity1", 1000);

        let log = log.add_event(event).expect("should add event");
        assert_eq!(log.event_count(), 1);
    }

    #[test]
    fn test_event_log_close() {
        let log = EventLog::new();
        let event = create_test_event(1, "Activity1", 1000);

        let log = log.add_event(event).unwrap();
        let closed_log = log.close();

        assert_eq!(closed_log.event_count(), 1);
        assert!(closed_log.metadata().closed_at.is_some());
    }

    #[test]
    fn test_event_log_analyze() {
        let log = EventLog::new();
        let event1 = create_test_event(1, "Activity1", 1000);
        let event2 = create_test_event(1, "Activity2", 2000);

        let log = log.add_event(event1).unwrap().add_event(event2).unwrap();
        let closed_log = log.close();

        let analytics = closed_log.analyze();
        assert_eq!(analytics.total_events, 2);
        assert_eq!(analytics.total_cases, 1);
    }

    #[test]
    fn test_event_log_invalid_order() {
        let log = EventLog::new();
        let event1 = create_test_event(1, "Activity1", 2000);
        let event2 = create_test_event(1, "Activity2", 1000); // Earlier timestamp

        let log = log.add_event(event1).unwrap();
        let result = log.add_event(event2);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_analyzer_lifecycle() {
        let log = EventLog::new();
        let event = create_test_event(1, "Activity1", 1000);
        let log = log.add_event(event).unwrap();
        let closed_log = log.close();

        let analyzer = ProcessAnalyzer::new(closed_log);
        let analyzer = analyzer.run();
        let analyzer = analyzer.complete();

        let results = analyzer.results();
        assert_eq!(results.analytics.total_events, 1);
    }

    fn create_test_event(case_id: u64, activity: &str, timestamp: u64) -> Event {
        EventBuilder::new()
            .with_case_id(CaseID::new(case_id).unwrap())
            .with_activity(ActivityName::new(activity).unwrap())
            .with_timestamp(Timestamp::new(timestamp))
            .build()
    }
}
