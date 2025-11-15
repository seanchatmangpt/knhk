# How-to Guide: Error Handling and Resilience Patterns

**Goal**: Build resilient systems with comprehensive error handling
**Time**: 30-45 minutes
**Difficulty**: Intermediate

## Error Handling Strategy

### Hierarchy of Error Responses

```
Level 1: Fail Fast (unrecoverable)
  └─ Return Err immediately, no retry

Level 2: Graceful Degradation (partial functionality)
  └─ Use defaults, reduced functionality

Level 3: Retry with Backoff (transient failures)
  └─ Exponential backoff, max retries

Level 4: Circuit Breaker (cascading failures)
  └─ Stop calling failing service
```

### Error Type Definition

```rust
#[derive(Debug)]
pub enum AppError {
    // Recoverable
    TransientError(String),      // Retry
    ValidationError(String),     // User fix
    NotFoundError(String),       // User fix

    // Non-recoverable
    ConfigError(String),         // Operator fix
    InternalError(String),       // Developer fix
    DataIntegrityError(String),  // Data fix
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TransientError(msg) => write!(f, "Transient: {}", msg),
            Self::ValidationError(msg) => write!(f, "Invalid: {}", msg),
            Self::NotFoundError(msg) => write!(f, "Not found: {}", msg),
            Self::ConfigError(msg) => write!(f, "Config: {}", msg),
            Self::InternalError(msg) => write!(f, "Internal: {}", msg),
            Self::DataIntegrityError(msg) => write!(f, "Data: {}", msg),
        }
    }
}

impl AppError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::TransientError(_))
    }
}
```

## Pattern 1: Retry with Exponential Backoff

```rust
pub async fn with_retry<F, T>(
    f: F,
    max_retries: u32
) -> Result<T, AppError>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, AppError>>>>,
{
    let mut attempt = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if !e.is_retryable() => return Err(e),
            Err(e) if attempt >= max_retries => return Err(e),
            Err(e) => {
                attempt += 1;
                let backoff = Duration::from_secs(2_u64.pow(attempt - 1));

                warn!(
                    attempt = attempt,
                    backoff_seconds = backoff.as_secs(),
                    error = %e,
                    "Retrying"
                );

                tokio::time::sleep(backoff).await;
            }
        }
    }
}
```

## Pattern 2: Circuit Breaker

```rust
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

#[derive(Debug)]
enum CircuitState {
    Closed,                                  // Normal operation
    Open { opened_at: Instant },            // Failing, reject calls
    HalfOpen { successes: u32 },            // Testing if recovered
}

impl CircuitBreaker {
    pub async fn execute<F, T>(
        &self,
        f: F
    ) -> Result<T, AppError>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T, AppError>>>>,
    {
        let mut state = self.state.lock().await;

        match *state {
            CircuitState::Closed => {
                match f().await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        *state = CircuitState::Open {
                            opened_at: Instant::now(),
                        };
                        Err(e)
                    }
                }
            }
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() > self.timeout {
                    *state = CircuitState::HalfOpen { successes: 0 };
                    // Fall through to try
                    f().await
                } else {
                    Err(AppError::InternalError("Circuit open".into()))
                }
            }
            CircuitState::HalfOpen { ref mut successes } => {
                match f().await {
                    Ok(result) => {
                        *successes += 1;
                        if *successes >= self.success_threshold {
                            *state = CircuitState::Closed;
                        }
                        Ok(result)
                    }
                    Err(e) => {
                        *state = CircuitState::Open {
                            opened_at: Instant::now(),
                        };
                        Err(e)
                    }
                }
            }
        }
    }
}
```

## Pattern 3: Bulkhead (Resource Isolation)

```rust
pub struct Bulkhead {
    semaphore: Semaphore,
}

impl Bulkhead {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Semaphore::new(max_concurrent),
        }
    }

    pub async fn execute<F, T>(
        &self,
        f: F
    ) -> Result<T, AppError>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T, AppError>>>>,
    {
        let _permit = self.semaphore
            .acquire()
            .await
            .map_err(|_| AppError::InternalError("Bulkhead full".into()))?;

        f().await
    }
}
```

## Pattern 4: Fallback

```rust
pub async fn with_fallback<F, G, T>(
    primary: F,
    fallback: G
) -> Result<T, AppError>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, AppError>>>>,
    G: Fn() -> Pin<Box<dyn Future<Output = Result<T, AppError>>>>,
{
    match primary().await {
        Ok(result) => Ok(result),
        Err(e) => {
            warn!("Primary failed: {}, trying fallback", e);
            fallback().await
        }
    }
}

// Usage
let result = with_fallback(
    || Box::pin(fetch_from_primary()),
    || Box::pin(fetch_from_cache()),
).await?;
```

## Pattern 5: Timeout

```rust
pub async fn with_timeout<F, T>(
    f: F,
    timeout: Duration
) -> Result<T, AppError>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, AppError>>>>,
{
    match tokio::time::timeout(timeout, f()).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(AppError::TransientError("Operation timed out".into())),
    }
}
```

## Complete Resilience Example

```rust
#[instrument]
pub async fn resilient_operation() -> Result<Data, AppError> {
    let bulkhead = Bulkhead::new(10);
    let circuit_breaker = CircuitBreaker::new(5, 2, Duration::from_secs(60));

    // Combine patterns
    bulkhead.execute(|| {
        Box::pin(async {
            circuit_breaker.execute(|| {
                Box::pin(async {
                    with_timeout(
                        || Box::pin(fetch_data()),
                        Duration::from_secs(5)
                    ).await
                })
            }).await
        })
    }).await
}
```

## Error Logging Strategy

```rust
pub fn log_error(e: &AppError) {
    match e {
        AppError::TransientError(msg) => {
            warn!("Transient error: {}", msg);
        }
        AppError::ValidationError(msg) => {
            info!("Validation error: {}", msg);
        }
        AppError::NotFoundError(msg) => {
            debug!("Not found: {}", msg);
        }
        AppError::ConfigError(msg) => {
            error!("Config error: {}", msg);
        }
        AppError::InternalError(msg) => {
            error!("Internal error: {}", msg);
        }
        AppError::DataIntegrityError(msg) => {
            error!("Data integrity error: {}", msg);
        }
    }
}
```

---

**Category**: How-to Guides (Task-oriented)
**Difficulty**: Intermediate
**Related**: Production Validation, Schema-First Development
