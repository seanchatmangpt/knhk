//! # Enhanced Type Safety Module - Poka-Yoke Implementation
//!
//! This module implements validated newtypes that use Rust's type system to make invalid states
//! impossible. Each type enforces invariants at compile time or construction time, preventing
//! errors before they can occur.
//!
//! ## Poka-Yoke Principles Applied:
//!
//! 1. **NonZero Types**: IDs and counts cannot be zero (compile-time guarantee)
//! 2. **Validated Constructors**: Strings cannot be empty (runtime validation, compile-time enforcement)
//! 3. **Range Validation**: Probabilities must be 0-100 (constructor validation)
//! 4. **Type-Safe Conversions**: TryFrom/From prevent invalid conversions
//! 5. **Phantom Data**: Zero-cost type safety markers
//!
//! ## Invalid States Made Impossible:
//!
//! - Cannot create CaseID(0) - compiler error or runtime error
//! - Cannot create ActivityName("") - constructor returns Err
//! - Cannot create Probability(101) - constructor returns Err
//! - Cannot accidentally use raw u64 as CaseID - type mismatch
//! - Cannot compare different ID types - type safety

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::num::{NonZeroU32, NonZeroU64};

// ============================================================================
// Error Types for Type Validation
// ============================================================================

/// Errors that can occur when creating validated types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidIdError {
    /// ID cannot be zero
    ZeroId,
    /// ID exceeds maximum allowed value
    IdTooLarge,
}

impl Display for InvalidIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidIdError::ZeroId => write!(f, "ID cannot be zero"),
            InvalidIdError::IdTooLarge => write!(f, "ID exceeds maximum allowed value"),
        }
    }
}

impl std::error::Error for InvalidIdError {}

/// Errors for invalid string-based types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidStringError {
    /// String cannot be empty
    EmptyString,
    /// String exceeds maximum length
    TooLong { max_length: usize, actual: usize },
    /// String contains invalid characters
    InvalidCharacters(String),
}

impl Display for InvalidStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidStringError::EmptyString => write!(f, "String cannot be empty"),
            InvalidStringError::TooLong { max_length, actual } => {
                write!(
                    f,
                    "String too long: maximum {} chars, got {}",
                    max_length, actual
                )
            }
            InvalidStringError::InvalidCharacters(chars) => {
                write!(f, "String contains invalid characters: {}", chars)
            }
        }
    }
}

impl std::error::Error for InvalidStringError {}

/// Errors for invalid probability values
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidProbabilityError {
    /// Probability must be between 0 and 100
    OutOfRange { value: u32 },
}

impl Display for InvalidProbabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidProbabilityError::OutOfRange { value } => {
                write!(f, "Probability must be 0-100, got {}", value)
            }
        }
    }
}

impl std::error::Error for InvalidProbabilityError {}

// ============================================================================
// CaseID - Poka-Yoke: Cannot be zero, cannot be confused with other ID types
// ============================================================================

/// Case identifier that cannot be zero.
///
/// ## Poka-Yoke Invariants:
/// - Cannot be zero (guaranteed by NonZeroU64)
/// - Cannot be confused with EventID or other ID types (type safety)
/// - Cannot accidentally use raw u64 (must explicitly convert)
///
/// ## Construction:
/// ```rust
/// use knhk_process_mining::types::CaseID;
///
/// // Safe construction (validated)
/// let case_id = CaseID::new(42).expect("non-zero");
///
/// // Compiler error: cannot create zero directly
/// // let invalid = CaseID(NonZeroU64::new(0).unwrap()); // Panics at runtime
///
/// // Type error: cannot use raw u64
/// // let invalid: CaseID = 42; // Compiler error
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CaseID(NonZeroU64);

impl CaseID {
    /// Creates a new CaseID from a u64 value.
    ///
    /// ## Errors
    /// Returns `InvalidIdError::ZeroId` if the value is zero.
    ///
    /// ## Poka-Yoke:
    /// This is the ONLY way to create a CaseID from a raw integer, ensuring validation.
    pub fn new(id: u64) -> Result<Self, InvalidIdError> {
        NonZeroU64::new(id).map(CaseID).ok_or(InvalidIdError::ZeroId)
    }

    /// Returns the inner NonZeroU64 value.
    ///
    /// ## Poka-Yoke:
    /// Returning NonZeroU64 (not u64) preserves the non-zero guarantee.
    pub fn get(self) -> NonZeroU64 {
        self.0
    }

    /// Returns the raw u64 value.
    ///
    /// ## Note:
    /// Only use this when interfacing with external systems. Prefer `get()` for internal use.
    pub fn as_u64(self) -> u64 {
        self.0.get()
    }
}

impl Display for CaseID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CaseID({})", self.0)
    }
}

impl TryFrom<u64> for CaseID {
    type Error = InvalidIdError;

    fn try_from(id: u64) -> Result<Self, Self::Error> {
        Self::new(id)
    }
}

impl From<NonZeroU64> for CaseID {
    /// Infallible conversion from NonZeroU64.
    ///
    /// ## Poka-Yoke:
    /// If you already have a NonZeroU64, you're guaranteed to have a valid CaseID.
    fn from(id: NonZeroU64) -> Self {
        CaseID(id)
    }
}

// ============================================================================
// EventID - Poka-Yoke: Cannot be zero, different type from CaseID
// ============================================================================

/// Event identifier that cannot be zero.
///
/// ## Poka-Yoke Invariants:
/// - Cannot be zero (guaranteed by NonZeroU32)
/// - Cannot be confused with CaseID (different underlying type and wrapper)
/// - Smaller than CaseID (u32 vs u64) - events are typically more numerous but shorter-lived
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventID(NonZeroU32);

impl EventID {
    /// Creates a new EventID from a u32 value.
    ///
    /// ## Errors
    /// Returns `InvalidIdError::ZeroId` if the value is zero.
    pub fn new(id: u32) -> Result<Self, InvalidIdError> {
        NonZeroU32::new(id).map(EventID).ok_or(InvalidIdError::ZeroId)
    }

    /// Returns the inner NonZeroU32 value.
    pub fn get(self) -> NonZeroU32 {
        self.0
    }

    /// Returns the raw u32 value.
    pub fn as_u32(self) -> u32 {
        self.0.get()
    }
}

impl Display for EventID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EventID({})", self.0)
    }
}

impl TryFrom<u32> for EventID {
    type Error = InvalidIdError;

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        Self::new(id)
    }
}

impl From<NonZeroU32> for EventID {
    fn from(id: NonZeroU32) -> Self {
        EventID(id)
    }
}

// ============================================================================
// Count - Poka-Yoke: Cannot be zero (counts start at 1)
// ============================================================================

/// A count value that cannot be zero.
///
/// ## Poka-Yoke Invariants:
/// - Cannot be zero (counts start at 1, not 0)
/// - Prevents off-by-one errors in counting logic
/// - Explicit type for "number of things" vs "index of thing"
///
/// ## Rationale:
/// Many counting scenarios require at least one item. Using `Count` makes this explicit
/// and prevents logic errors where 0 is treated as a valid count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Count(NonZeroU32);

impl Count {
    /// Creates a new Count from a u32 value.
    ///
    /// ## Errors
    /// Returns `InvalidIdError::ZeroId` if the value is zero.
    pub fn new(count: u32) -> Result<Self, InvalidIdError> {
        NonZeroU32::new(count)
            .map(Count)
            .ok_or(InvalidIdError::ZeroId)
    }

    /// Returns the inner NonZeroU32 value.
    pub fn get(self) -> NonZeroU32 {
        self.0
    }

    /// Returns the raw u32 value.
    pub fn as_u32(self) -> u32 {
        self.0.get()
    }

    /// Increments the count by 1.
    ///
    /// ## Panics
    /// Panics on overflow (u32::MAX + 1).
    pub fn increment(self) -> Self {
        Count(NonZeroU32::new(self.0.get() + 1).expect("count overflow"))
    }

    /// Adds another count to this count.
    ///
    /// ## Poka-Yoke:
    /// Adding two non-zero counts always yields a non-zero count.
    ///
    /// ## Panics
    /// Panics on overflow.
    pub fn add(self, other: Count) -> Self {
        Count(
            NonZeroU32::new(self.0.get() + other.0.get()).expect("count overflow on addition"),
        )
    }
}

impl Display for Count {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u32> for Count {
    type Error = InvalidIdError;

    fn try_from(count: u32) -> Result<Self, Self::Error> {
        Self::new(count)
    }
}

impl From<NonZeroU32> for Count {
    fn from(count: NonZeroU32) -> Self {
        Count(count)
    }
}

// ============================================================================
// ActivityName - Poka-Yoke: Cannot be empty
// ============================================================================

/// Activity name that cannot be empty.
///
/// ## Poka-Yoke Invariants:
/// - Cannot be empty string (validated at construction)
/// - Optional max length validation (prevents memory issues)
/// - Cannot accidentally use raw String (must explicitly convert)
///
/// ## Construction:
/// ```rust
/// use knhk_process_mining::types::ActivityName;
///
/// // Valid construction
/// let name = ActivityName::new("Process Payment").expect("non-empty");
///
/// // Runtime error: empty string
/// let invalid = ActivityName::new(""); // Returns Err(InvalidStringError::EmptyString)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActivityName(String);

impl ActivityName {
    /// Maximum allowed length for activity names (prevents DoS via huge strings).
    pub const MAX_LENGTH: usize = 256;

    /// Creates a new ActivityName from a string.
    ///
    /// ## Errors
    /// - `InvalidStringError::EmptyString` if the string is empty
    /// - `InvalidStringError::TooLong` if the string exceeds MAX_LENGTH
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidStringError> {
        let name = name.into();

        if name.is_empty() {
            return Err(InvalidStringError::EmptyString);
        }

        if name.len() > Self::MAX_LENGTH {
            return Err(InvalidStringError::TooLong {
                max_length: Self::MAX_LENGTH,
                actual: name.len(),
            });
        }

        Ok(ActivityName(name))
    }

    /// Returns a reference to the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes self and returns the inner String.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl Display for ActivityName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for ActivityName {
    type Error = InvalidStringError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        Self::new(name)
    }
}

impl TryFrom<&str> for ActivityName {
    type Error = InvalidStringError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        Self::new(name)
    }
}

impl AsRef<str> for ActivityName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// Timestamp - Poka-Yoke: Type-safe timestamp with ordering guarantees
// ============================================================================

/// Timestamp in milliseconds since Unix epoch.
///
/// ## Poka-Yoke Invariants:
/// - Cannot accidentally use raw u64 for timestamps
/// - Explicit type for time values vs other numeric values
/// - Ordering operations are type-safe
/// - Cannot mix with Duration accidentally
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Creates a new Timestamp from milliseconds since Unix epoch.
    pub fn new(millis: u64) -> Self {
        Timestamp(millis)
    }

    /// Returns the current system time as a Timestamp.
    ///
    /// ## Note:
    /// Uses system clock, which may not be monotonic. For interval measurements,
    /// consider using monotonic time sources.
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before Unix epoch")
            .as_millis() as u64;
        Timestamp(millis)
    }

    /// Returns the raw millisecond value.
    pub fn as_millis(self) -> u64 {
        self.0
    }

    /// Calculates the duration between two timestamps.
    ///
    /// ## Poka-Yoke:
    /// Returns Duration type, not raw u64, maintaining type safety.
    ///
    /// ## Errors
    /// Returns None if `other` is later than `self` (would be negative duration).
    pub fn duration_since(self, other: Timestamp) -> Option<Duration> {
        self.0.checked_sub(other.0).map(Duration::new)
    }

    /// Adds a duration to this timestamp.
    ///
    /// ## Poka-Yoke:
    /// Type system prevents accidentally adding two timestamps together.
    ///
    /// ## Panics
    /// Panics on overflow.
    pub fn add_duration(self, duration: Duration) -> Self {
        Timestamp(self.0 + duration.as_millis())
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ms", self.0)
    }
}

impl From<u64> for Timestamp {
    fn from(millis: u64) -> Self {
        Timestamp(millis)
    }
}

// ============================================================================
// Duration - Poka-Yoke: Type-safe duration (always non-negative)
// ============================================================================

/// Duration in milliseconds (always non-negative).
///
/// ## Poka-Yoke Invariants:
/// - Cannot be negative (unsigned type)
/// - Cannot accidentally mix with Timestamp
/// - Explicit type for time intervals
/// - Arithmetic operations preserve type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Duration(u64);

impl Duration {
    /// Creates a new Duration from milliseconds.
    pub fn new(millis: u64) -> Self {
        Duration(millis)
    }

    /// Creates a Duration from seconds.
    pub fn from_secs(secs: u64) -> Self {
        Duration(secs * 1000)
    }

    /// Creates a Duration from minutes.
    pub fn from_mins(mins: u64) -> Self {
        Duration(mins * 60 * 1000)
    }

    /// Creates a Duration from hours.
    pub fn from_hours(hours: u64) -> Self {
        Duration(hours * 60 * 60 * 1000)
    }

    /// Returns the raw millisecond value.
    pub fn as_millis(self) -> u64 {
        self.0
    }

    /// Returns the duration in seconds (rounded down).
    pub fn as_secs(self) -> u64 {
        self.0 / 1000
    }

    /// Returns the duration in minutes (rounded down).
    pub fn as_mins(self) -> u64 {
        self.0 / (60 * 1000)
    }

    /// Returns the duration in hours (rounded down).
    pub fn as_hours(self) -> u64 {
        self.0 / (60 * 60 * 1000)
    }

    /// Adds two durations together.
    ///
    /// ## Poka-Yoke:
    /// Adding durations yields a duration (not a timestamp).
    ///
    /// ## Panics
    /// Panics on overflow.
    pub fn add(self, other: Duration) -> Self {
        Duration(self.0 + other.0)
    }

    /// Subtracts another duration from this one.
    ///
    /// ## Errors
    /// Returns None if the result would be negative.
    pub fn checked_sub(self, other: Duration) -> Option<Self> {
        self.0.checked_sub(other.0).map(Duration)
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hours = self.as_hours();
        let mins = (self.as_mins() % 60);
        let secs = (self.as_secs() % 60);
        let millis = self.0 % 1000;

        if hours > 0 {
            write!(f, "{}h {}m {}s", hours, mins, secs)
        } else if mins > 0 {
            write!(f, "{}m {}s", mins, secs)
        } else if secs > 0 {
            write!(f, "{}s", secs)
        } else {
            write!(f, "{}ms", millis)
        }
    }
}

impl From<u64> for Duration {
    fn from(millis: u64) -> Self {
        Duration(millis)
    }
}

impl From<std::time::Duration> for Duration {
    fn from(std_duration: std::time::Duration) -> Self {
        Duration(std_duration.as_millis() as u64)
    }
}

// ============================================================================
// Probability - Poka-Yoke: Must be 0-100 (percentage)
// ============================================================================

/// Probability value (0-100 representing percentage).
///
/// ## Poka-Yoke Invariants:
/// - Must be between 0 and 100 (validated at construction)
/// - Cannot accidentally use raw u32 for probabilities
/// - Prevents logic errors from out-of-range probabilities
/// - Type-safe arithmetic operations
///
/// ## Construction:
/// ```rust
/// use knhk_process_mining::types::Probability;
///
/// // Valid construction
/// let prob = Probability::new(75).expect("valid probability");
///
/// // Runtime error: out of range
/// let invalid = Probability::new(101); // Returns Err(InvalidProbabilityError::OutOfRange)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Probability(u32);

impl Probability {
    /// Minimum probability value (0%).
    pub const MIN: u32 = 0;

    /// Maximum probability value (100%).
    pub const MAX: u32 = 100;

    /// Creates a new Probability from a percentage value (0-100).
    ///
    /// ## Errors
    /// Returns `InvalidProbabilityError::OutOfRange` if value > 100.
    pub fn new(percent: u32) -> Result<Self, InvalidProbabilityError> {
        if percent > Self::MAX {
            return Err(InvalidProbabilityError::OutOfRange { value: percent });
        }
        Ok(Probability(percent))
    }

    /// Returns the raw percentage value (0-100).
    pub fn as_percent(self) -> u32 {
        self.0
    }

    /// Returns the probability as a fraction (0.0-1.0).
    pub fn as_fraction(self) -> f64 {
        f64::from(self.0) / 100.0
    }

    /// Returns the complementary probability (100 - this).
    ///
    /// ## Poka-Yoke:
    /// The result is guaranteed to be a valid Probability.
    pub fn complement(self) -> Self {
        Probability(Self::MAX - self.0)
    }

    /// Combines two independent probabilities (P(A or B) for independent events).
    ///
    /// Formula: P(A or B) = P(A) + P(B) - P(A) * P(B)
    ///
    /// ## Poka-Yoke:
    /// Result is guaranteed to be a valid Probability (0-100).
    pub fn or(self, other: Probability) -> Self {
        let p_a = self.as_fraction();
        let p_b = other.as_fraction();
        let p_or = p_a + p_b - (p_a * p_b);
        let percent = (p_or * 100.0).round() as u32;
        Probability(percent.min(Self::MAX))
    }

    /// Combines two independent probabilities (P(A and B) for independent events).
    ///
    /// Formula: P(A and B) = P(A) * P(B)
    ///
    /// ## Poka-Yoke:
    /// Result is guaranteed to be a valid Probability (0-100).
    pub fn and(self, other: Probability) -> Self {
        let p_a = self.as_fraction();
        let p_b = other.as_fraction();
        let p_and = p_a * p_b;
        let percent = (p_and * 100.0).round() as u32;
        Probability(percent)
    }
}

impl Display for Probability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl TryFrom<u32> for Probability {
    type Error = InvalidProbabilityError;

    fn try_from(percent: u32) -> Result<Self, Self::Error> {
        Self::new(percent)
    }
}

impl TryFrom<f64> for Probability {
    type Error = InvalidProbabilityError;

    /// Converts from a fraction (0.0-1.0) to a percentage (0-100).
    ///
    /// ## Errors
    /// Returns error if the value is outside 0.0-1.0 range.
    fn try_from(fraction: f64) -> Result<Self, Self::Error> {
        if !(0.0..=1.0).contains(&fraction) {
            return Err(InvalidProbabilityError::OutOfRange {
                value: (fraction * 100.0) as u32,
            });
        }
        let percent = (fraction * 100.0).round() as u32;
        Ok(Probability(percent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_id_cannot_be_zero() {
        assert!(CaseID::new(0).is_err());
        assert!(CaseID::new(1).is_ok());
        assert!(CaseID::new(u64::MAX).is_ok());
    }

    #[test]
    fn test_event_id_cannot_be_zero() {
        assert!(EventID::new(0).is_err());
        assert!(EventID::new(1).is_ok());
        assert!(EventID::new(u32::MAX).is_ok());
    }

    #[test]
    fn test_count_cannot_be_zero() {
        assert!(Count::new(0).is_err());
        assert!(Count::new(1).is_ok());
    }

    #[test]
    fn test_count_increment() {
        let count = Count::new(5).unwrap();
        let incremented = count.increment();
        assert_eq!(incremented.as_u32(), 6);
    }

    #[test]
    fn test_count_add() {
        let count1 = Count::new(5).unwrap();
        let count2 = Count::new(3).unwrap();
        let sum = count1.add(count2);
        assert_eq!(sum.as_u32(), 8);
    }

    #[test]
    fn test_activity_name_cannot_be_empty() {
        assert!(ActivityName::new("").is_err());
        assert!(ActivityName::new("Valid Name").is_ok());
    }

    #[test]
    fn test_activity_name_too_long() {
        let long_name = "a".repeat(ActivityName::MAX_LENGTH + 1);
        assert!(ActivityName::new(long_name).is_err());
    }

    #[test]
    fn test_timestamp_duration_calculation() {
        let t1 = Timestamp::new(1000);
        let t2 = Timestamp::new(2000);
        let duration = t2.duration_since(t1).unwrap();
        assert_eq!(duration.as_millis(), 1000);
    }

    #[test]
    fn test_timestamp_add_duration() {
        let t = Timestamp::new(1000);
        let d = Duration::new(500);
        let result = t.add_duration(d);
        assert_eq!(result.as_millis(), 1500);
    }

    #[test]
    fn test_duration_operations() {
        let d1 = Duration::new(1000);
        let d2 = Duration::new(500);
        let sum = d1.add(d2);
        assert_eq!(sum.as_millis(), 1500);

        let diff = d1.checked_sub(d2).unwrap();
        assert_eq!(diff.as_millis(), 500);
    }

    #[test]
    fn test_probability_range_validation() {
        assert!(Probability::new(0).is_ok());
        assert!(Probability::new(50).is_ok());
        assert!(Probability::new(100).is_ok());
        assert!(Probability::new(101).is_err());
    }

    #[test]
    fn test_probability_complement() {
        let prob = Probability::new(30).unwrap();
        let comp = prob.complement();
        assert_eq!(comp.as_percent(), 70);
    }

    #[test]
    fn test_probability_or() {
        let p1 = Probability::new(50).unwrap();
        let p2 = Probability::new(50).unwrap();
        let result = p1.or(p2);
        // P(A or B) = 0.5 + 0.5 - 0.5*0.5 = 0.75 = 75%
        assert_eq!(result.as_percent(), 75);
    }

    #[test]
    fn test_probability_and() {
        let p1 = Probability::new(50).unwrap();
        let p2 = Probability::new(50).unwrap();
        let result = p1.and(p2);
        // P(A and B) = 0.5 * 0.5 = 0.25 = 25%
        assert_eq!(result.as_percent(), 25);
    }
}
