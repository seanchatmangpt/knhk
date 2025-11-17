//! Enterprise resilience patterns for Fortune 500-level reliability
//!
//! # YAWL Exception Handling
//!
//! - `yawl_exception.rs`: YAWL exception taxonomy and handlers with TRIZ enhancements

pub mod circuit_breaker;
pub mod dlq;
pub mod rate_limit;
pub mod retry;
pub mod timeout;
pub mod yawl_exception;

pub use circuit_breaker::CircuitBreaker;
pub use dlq::DeadLetterQueue;
pub use rate_limit::{KeyedRateLimiter, RateLimitConfig, RateLimiter};
pub use retry::{RetryConfig, RetryPolicy};
pub use timeout::{PathType, TimeoutConfig, TimeoutManager};
pub use yawl_exception::{
    CompensationHandler, ExceptionAnalytics, ExceptionCategory, ExceptionHandler,
    ExceptionSeverity, RetryHandler, YawlException, YawlExceptionManager,
};
