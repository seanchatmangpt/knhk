// rust/knhk-sidecar/src/retry.rs
// Retry logic with exponential backoff
// Leverages idempotence property: μ∘μ = μ for safe retries

use std::time::Duration;
use tokio::time::sleep;

pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    pub fn new(max_attempts: u32, initial_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_attempts,
            initial_delay_ms,
            max_delay_ms,
            multiplier: 2.0,
        }
    }
}

pub async fn retry_with_backoff<F, T, E>(
    config: &RetryConfig,
    mut f: F,
) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Debug,
{
    let mut delay_ms = config.initial_delay_ms;
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);

                if attempt < config.max_attempts - 1 {
                    let delay = Duration::from_millis(
                        delay_ms.min(config.max_delay_ms)
                    );
                    sleep(delay).await;
                    delay_ms = ((delay_ms as f64) * config.multiplier) as u64;
                }
            }
        }
    }

    Err(last_error.expect("Should have at least one error"))
}

pub async fn retry_with_backoff_sync<F, T, E>(
    config: &RetryConfig,
    mut f: F,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Debug,
{
    let mut delay_ms = config.initial_delay_ms;
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);

                if attempt < config.max_attempts - 1 {
                    let delay = Duration::from_millis(
                        delay_ms.min(config.max_delay_ms)
                    );
                    sleep(delay).await;
                    delay_ms = ((delay_ms as f64) * config.multiplier) as u64;
                }
            }
        }
    }

    Err(last_error.expect("Should have at least one error"))
}

