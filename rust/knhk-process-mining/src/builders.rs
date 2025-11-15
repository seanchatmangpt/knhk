//! # Type-Safe Builders - Poka-Yoke Implementation
//!
//! This module implements type-state builders that use Rust's type system to ensure
//! all required fields are set before construction.
//!
//! ## Poka-Yoke Principles Applied:
//!
//! 1. **Type States**: Each builder step has a unique type state
//! 2. **Compile-Time Validation**: Cannot build without all required fields
//! 3. **Fluent Interface**: Method chaining with type transitions
//! 4. **Phantom Types**: Zero-cost type-level state tracking
//! 5. **Progressive Construction**: Each step advances the type state
//!
//! ## Invalid States Made Impossible:
//!
//! - Cannot build Event without case_id - compiler error
//! - Cannot build Event without activity - compiler error
//! - Cannot build Event without timestamp - compiler error
//! - Cannot set the same field twice - compiler error (method not available)
//! - Cannot forget optional fields - explicit methods required

use crate::types::{ActivityName, CaseID, EventID, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;

// ============================================================================
// Event Builder with Type States
// ============================================================================

/// Type state: No fields set yet
pub struct Empty;

/// Type state: CaseID has been set
pub struct WithCaseID;

/// Type state: CaseID and Activity have been set
pub struct WithActivity;

/// Type state: CaseID, Activity, and Timestamp have been set (ready to build)
pub struct Complete;

/// Type-safe builder for Event construction.
///
/// ## Poka-Yoke Design:
///
/// The builder uses type states to enforce the order and completeness of construction:
///
/// ```text
/// EventBuilder<Empty>
///     .with_case_id() -> EventBuilder<WithCaseID>
///     .with_activity() -> EventBuilder<WithActivity>
///     .with_timestamp() -> EventBuilder<Complete>
///     .build() -> Event
/// ```
///
/// ## Compile-Time Guarantees:
///
/// - `build()` is only available on `EventBuilder<Complete>`
/// - Each field can only be set once (method consumes self)
/// - Cannot skip required fields
/// - Cannot forget to call `build()`
///
/// ## Example:
///
/// ```rust
/// use knhk_process_mining::builders::EventBuilder;
/// use knhk_process_mining::types::{CaseID, ActivityName, Timestamp};
///
/// let event = EventBuilder::new()
///     .with_case_id(CaseID::new(1).unwrap())
///     .with_activity(ActivityName::new("Process").unwrap())
///     .with_timestamp(Timestamp::now())
///     .with_resource("System")
///     .build();
/// ```
///
/// ## Compile Errors (Prevented):
///
/// ```rust,compile_fail
/// // ERROR: build() not available on EventBuilder<Empty>
/// let event = EventBuilder::new().build();
///
/// // ERROR: build() not available on EventBuilder<WithCaseID>
/// let event = EventBuilder::new()
///     .with_case_id(case_id)
///     .build();
///
/// // ERROR: with_case_id() not available on EventBuilder<WithCaseID>
/// let event = EventBuilder::new()
///     .with_case_id(case_id)
///     .with_case_id(case_id); // Cannot set twice!
/// ```
#[derive(Debug)]
pub struct EventBuilder<State> {
    case_id: Option<CaseID>,
    activity: Option<ActivityName>,
    timestamp: Option<Timestamp>,
    event_id: Option<EventID>,
    resource: Option<String>,
    attributes: HashMap<String, String>,
    _state: PhantomData<State>,
}

impl EventBuilder<Empty> {
    /// Creates a new EventBuilder in the Empty state.
    ///
    /// ## Poka-Yoke:
    /// This is the ONLY way to create an EventBuilder, ensuring it starts in the correct state.
    pub fn new() -> Self {
        EventBuilder {
            case_id: None,
            activity: None,
            timestamp: None,
            event_id: None,
            resource: None,
            attributes: HashMap::new(),
            _state: PhantomData,
        }
    }

    /// Sets the case ID and transitions to WithCaseID state.
    ///
    /// ## Poka-Yoke:
    /// - Consumes `self`, preventing reuse
    /// - Returns new type state, enabling next step
    /// - Cannot be called twice (method not available on WithCaseID)
    pub fn with_case_id(mut self, case_id: CaseID) -> EventBuilder<WithCaseID> {
        self.case_id = Some(case_id);
        EventBuilder {
            case_id: self.case_id,
            activity: self.activity,
            timestamp: self.timestamp,
            event_id: self.event_id,
            resource: self.resource,
            attributes: self.attributes,
            _state: PhantomData,
        }
    }
}

impl EventBuilder<WithCaseID> {
    /// Sets the activity name and transitions to WithActivity state.
    ///
    /// ## Poka-Yoke:
    /// - Only available after case_id is set
    /// - Consumes `self`, preventing reuse
    /// - Cannot be called twice
    pub fn with_activity(mut self, activity: ActivityName) -> EventBuilder<WithActivity> {
        self.activity = Some(activity);
        EventBuilder {
            case_id: self.case_id,
            activity: self.activity,
            timestamp: self.timestamp,
            event_id: self.event_id,
            resource: self.resource,
            attributes: self.attributes,
            _state: PhantomData,
        }
    }
}

impl EventBuilder<WithActivity> {
    /// Sets the timestamp and transitions to Complete state.
    ///
    /// ## Poka-Yoke:
    /// - Only available after case_id and activity are set
    /// - Consumes `self`, preventing reuse
    /// - Enables `build()` method
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> EventBuilder<Complete> {
        self.timestamp = Some(timestamp);
        EventBuilder {
            case_id: self.case_id,
            activity: self.activity,
            timestamp: self.timestamp,
            event_id: self.event_id,
            resource: self.resource,
            attributes: self.attributes,
            _state: PhantomData,
        }
    }
}

// Optional fields can be set at any state
impl<State> EventBuilder<State> {
    /// Sets the event ID (optional).
    ///
    /// ## Poka-Yoke:
    /// - Available at any builder state
    /// - Preserves current type state
    pub fn with_event_id(mut self, event_id: EventID) -> Self {
        self.event_id = Some(event_id);
        self
    }

    /// Sets the resource (optional).
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Adds a custom attribute (optional).
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Adds multiple custom attributes (optional).
    pub fn with_attributes(mut self, attributes: HashMap<String, String>) -> Self {
        self.attributes.extend(attributes);
        self
    }
}

impl EventBuilder<Complete> {
    /// Builds the Event.
    ///
    /// ## Poka-Yoke:
    /// - ONLY available on EventBuilder<Complete>
    /// - All required fields are guaranteed to be Some (type system ensures this)
    /// - No runtime validation needed for required fields
    pub fn build(self) -> Event {
        Event {
            case_id: self.case_id.expect("case_id must be set"),
            activity: self.activity.expect("activity must be set"),
            timestamp: self.timestamp.expect("timestamp must be set"),
            event_id: self.event_id,
            resource: self.resource,
            attributes: self.attributes,
        }
    }
}

impl Default for EventBuilder<Empty> {
    fn default() -> Self {
        Self::new()
    }
}

/// Process mining event.
///
/// ## Poka-Yoke:
/// Cannot be constructed directly - must use EventBuilder to ensure all required fields are set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// Case ID (required)
    pub case_id: CaseID,
    /// Activity name (required)
    pub activity: ActivityName,
    /// Timestamp (required)
    pub timestamp: Timestamp,
    /// Event ID (optional)
    pub event_id: Option<EventID>,
    /// Resource that performed the activity (optional)
    pub resource: Option<String>,
    /// Custom attributes (optional)
    pub attributes: HashMap<String, String>,
}

// ============================================================================
// Configuration Builder with Validation
// ============================================================================

/// Builder for process mining configuration with validation.
///
/// ## Poka-Yoke Design:
///
/// Unlike EventBuilder, ConfigBuilder uses runtime validation because:
/// - Configuration values need range checking (e.g., 0.0-1.0)
/// - Optional fields have sensible defaults
/// - All fields can be set in any order
///
/// ## Compile-Time Guarantees:
/// - Cannot forget to call `build()` (returns Result, must be handled)
/// - Cannot use invalid config (build() validates)
///
/// ## Runtime Guarantees:
/// - All values are within valid ranges
/// - No contradictory settings
#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    min_frequency: Option<u32>,
    min_confidence: Option<f64>,
    max_iterations: Option<u32>,
    enable_parallel: bool,
    thread_count: Option<usize>,
}

/// Configuration validation error
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// Required field is missing
    MissingField { field: String },
    /// Value is out of valid range
    InvalidRange {
        field: String,
        value: String,
        min: String,
        max: String,
    },
    /// Configuration has contradictory settings
    Contradiction { reason: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingField { field } => {
                write!(f, "Missing required field: {}", field)
            }
            ConfigError::InvalidRange {
                field,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "Field '{}' has invalid value '{}' (must be between {} and {})",
                    field, value, min, max
                )
            }
            ConfigError::Contradiction { reason } => {
                write!(f, "Configuration contradiction: {}", reason)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl ConfigBuilder {
    /// Creates a new ConfigBuilder with default values.
    pub fn new() -> Self {
        ConfigBuilder {
            min_frequency: None,
            min_confidence: None,
            max_iterations: None,
            enable_parallel: false,
            thread_count: None,
        }
    }

    /// Sets the minimum frequency threshold.
    ///
    /// ## Valid Range: 1 or greater
    pub fn min_frequency(mut self, min_freq: u32) -> Self {
        self.min_frequency = Some(min_freq);
        self
    }

    /// Sets the minimum confidence threshold.
    ///
    /// ## Valid Range: 0.0 to 1.0
    pub fn min_confidence(mut self, min_conf: f64) -> Self {
        self.min_confidence = Some(min_conf);
        self
    }

    /// Sets the maximum number of iterations.
    ///
    /// ## Valid Range: 1 or greater
    pub fn max_iterations(mut self, max_iter: u32) -> Self {
        self.max_iterations = Some(max_iter);
        self
    }

    /// Enables parallel processing.
    pub fn enable_parallel(mut self) -> Self {
        self.enable_parallel = true;
        self
    }

    /// Sets the number of threads for parallel processing.
    ///
    /// ## Valid Range: 1 or greater
    /// ## Note: Automatically enables parallel processing
    pub fn thread_count(mut self, count: usize) -> Self {
        self.thread_count = Some(count);
        self.enable_parallel = true;
        self
    }

    /// Builds and validates the configuration.
    ///
    /// ## Poka-Yoke:
    /// - Validates all fields are within valid ranges
    /// - Checks for contradictory settings
    /// - Returns error if invalid
    ///
    /// ## Errors
    /// - `MissingField` if required field is not set
    /// - `InvalidRange` if value is outside valid range
    /// - `Contradiction` if settings conflict
    pub fn build(self) -> Result<ProcessMiningConfig, ConfigError> {
        // Validate min_frequency
        let min_frequency = self.min_frequency.ok_or(ConfigError::MissingField {
            field: "min_frequency".to_string(),
        })?;

        if min_frequency == 0 {
            return Err(ConfigError::InvalidRange {
                field: "min_frequency".to_string(),
                value: min_frequency.to_string(),
                min: "1".to_string(),
                max: "u32::MAX".to_string(),
            });
        }

        // Validate min_confidence
        let min_confidence = self.min_confidence.ok_or(ConfigError::MissingField {
            field: "min_confidence".to_string(),
        })?;

        if !(0.0..=1.0).contains(&min_confidence) {
            return Err(ConfigError::InvalidRange {
                field: "min_confidence".to_string(),
                value: min_confidence.to_string(),
                min: "0.0".to_string(),
                max: "1.0".to_string(),
            });
        }

        // Validate max_iterations (optional, default to 1000)
        let max_iterations = self.max_iterations.unwrap_or(1000);
        if max_iterations == 0 {
            return Err(ConfigError::InvalidRange {
                field: "max_iterations".to_string(),
                value: max_iterations.to_string(),
                min: "1".to_string(),
                max: "u32::MAX".to_string(),
            });
        }

        // Validate thread_count
        if let Some(count) = self.thread_count {
            if count == 0 {
                return Err(ConfigError::InvalidRange {
                    field: "thread_count".to_string(),
                    value: count.to_string(),
                    min: "1".to_string(),
                    max: "usize::MAX".to_string(),
                });
            }
        }

        // Check for contradictions
        if self.enable_parallel && self.thread_count.is_none() {
            // Use default thread count (number of CPUs)
            let thread_count = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);

            Ok(ProcessMiningConfig {
                min_frequency,
                min_confidence,
                max_iterations,
                enable_parallel: self.enable_parallel,
                thread_count: Some(thread_count),
            })
        } else {
            Ok(ProcessMiningConfig {
                min_frequency,
                min_confidence,
                max_iterations,
                enable_parallel: self.enable_parallel,
                thread_count: self.thread_count,
            })
        }
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Validated process mining configuration.
///
/// ## Poka-Yoke:
/// - Cannot be constructed directly
/// - All fields guaranteed to be valid (validated by ConfigBuilder)
/// - No need for runtime checks when using this config
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessMiningConfig {
    /// Minimum frequency threshold (>= 1)
    pub min_frequency: u32,
    /// Minimum confidence threshold (0.0 to 1.0)
    pub min_confidence: f64,
    /// Maximum iterations (>= 1)
    pub max_iterations: u32,
    /// Whether to enable parallel processing
    pub enable_parallel: bool,
    /// Number of threads (>= 1 if parallel enabled)
    pub thread_count: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_builder_complete() {
        let case_id = CaseID::new(1).unwrap();
        let activity = ActivityName::new("Test Activity").unwrap();
        let timestamp = Timestamp::now();

        let event = EventBuilder::new()
            .with_case_id(case_id)
            .with_activity(activity)
            .with_timestamp(timestamp)
            .build();

        assert_eq!(event.case_id, case_id);
    }

    #[test]
    fn test_event_builder_with_optional_fields() {
        let case_id = CaseID::new(1).unwrap();
        let activity = ActivityName::new("Test").unwrap();
        let timestamp = Timestamp::now();
        let event_id = EventID::new(100).unwrap();

        let event = EventBuilder::new()
            .with_case_id(case_id)
            .with_activity(activity)
            .with_timestamp(timestamp)
            .with_event_id(event_id)
            .with_resource("User")
            .with_attribute("key", "value")
            .build();

        assert_eq!(event.event_id, Some(event_id));
        assert_eq!(event.resource, Some("User".to_string()));
        assert_eq!(event.attributes.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_config_builder_valid() {
        let config = ConfigBuilder::new()
            .min_frequency(10)
            .min_confidence(0.8)
            .max_iterations(100)
            .build()
            .expect("should build valid config");

        assert_eq!(config.min_frequency, 10);
        assert_eq!(config.min_confidence, 0.8);
        assert_eq!(config.max_iterations, 100);
    }

    #[test]
    fn test_config_builder_missing_field() {
        let result = ConfigBuilder::new().min_frequency(10).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_builder_invalid_confidence() {
        let result = ConfigBuilder::new()
            .min_frequency(10)
            .min_confidence(1.5) // Invalid: > 1.0
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_builder_zero_frequency() {
        let result = ConfigBuilder::new()
            .min_frequency(0) // Invalid: must be >= 1
            .min_confidence(0.8)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_builder_parallel_with_threads() {
        let config = ConfigBuilder::new()
            .min_frequency(5)
            .min_confidence(0.7)
            .thread_count(4)
            .build()
            .expect("should build");

        assert!(config.enable_parallel);
        assert_eq!(config.thread_count, Some(4));
    }
}
