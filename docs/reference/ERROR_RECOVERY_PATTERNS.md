# Error Recovery Patterns

How to build resilient systems that recover from failures gracefully.

---

## 1. Retry with Exponential Backoff

### Automatic Retries with Increasing Delays

```rust
pub async fn with_exponential_backoff<F, T>(
    mut f: F,
    max_retries: u32,
) -> Result<T, Error>
where
    F: FnMut() -> futures::future::BoxFuture<'static, Result<T, Error>>,
{
    let mut backoff_ms = 100;
    
    for attempt in 0..max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if is_retryable(&e) => {
                if attempt < max_retries - 1 {
                    tokio::time::sleep(
                        Duration::from_millis(backoff_ms)
                    ).await;
                    backoff_ms = (backoff_ms * 2).min(10000); // Cap at 10s
                    continue;
                }
                return Err(e);
            }
            Err(e) => return Err(e), // Non-retryable error
        }
    }
    
    Err(Error::MaxRetriesExceeded)
}

fn is_retryable(error: &Error) -> bool {
    matches!(error, 
        Error::Timeout | Error::NetworkUnreachable | Error::ServiceUnavailable
    )
}
```

---

## 2. Circuit Breaker Pattern

### Prevent Cascading Failures

```rust
pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Failing, reject requests
    HalfOpen,    // Testing recovery
}

pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: usize,
    failure_threshold: usize,
    last_failure_time: Option<Instant>,
    timeout_secs: u64,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&mut self, f: F) -> Result<T, Error>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, Error>>,
    {
        match self.state {
            CircuitState::Closed => {
                match f().await {
                    Ok(result) => {
                        self.failure_count = 0;
                        Ok(result)
                    }
                    Err(e) => {
                        self.failure_count += 1;
                        if self.failure_count >= self.failure_threshold {
                            self.state = CircuitState::Open;
                            self.last_failure_time = Some(Instant::now());
                        }
                        Err(e)
                    }
                }
            }
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() > Duration::from_secs(self.timeout_secs) {
                        self.state = CircuitState::HalfOpen;
                        self.call(f).await
                    } else {
                        Err(Error::CircuitOpen)
                    }
                } else {
                    Err(Error::CircuitOpen)
                }
            }
            CircuitState::HalfOpen => {
                match f().await {
                    Ok(result) => {
                        self.state = CircuitState::Closed;
                        self.failure_count = 0;
                        Ok(result)
                    }
                    Err(e) => {
                        self.state = CircuitState::Open;
                        self.last_failure_time = Some(Instant::now());
                        Err(e)
                    }
                }
            }
        }
    }
}
```

---

## 3. Bulkhead Pattern

### Isolate Failures to Prevent Cascade

```rust
pub struct Bulkhead {
    active_requests: Arc<AtomicUsize>,
    max_concurrent: usize,
}

impl Bulkhead {
    pub async fn execute<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, Error>>,
    {
        let current = self.active_requests.fetch_add(1, Ordering::SeqCst);
        
        if current >= self.max_concurrent {
            self.active_requests.fetch_sub(1, Ordering::SeqCst);
            return Err(Error::BulkheadFull);
        }
        
        let result = f().await;
        
        self.active_requests.fetch_sub(1, Ordering::SeqCst);
        result
    }
}
```

---

## 4. Graceful Degradation

### Provide Reduced Functionality Under Load

```rust
pub enum ServiceMode {
    Full,      // All features available
    Reduced,   // Non-critical features disabled
    Essential, // Only critical operations
}

impl Service {
    pub fn determine_mode(&self) -> ServiceMode {
        let load = self.current_load();
        let capacity = self.capacity();
        let utilization = load as f32 / capacity as f32;
        
        match utilization {
            0.0..=0.7 => ServiceMode::Full,
            0.7..=0.9 => ServiceMode::Reduced,
            _ => ServiceMode::Essential,
        }
    }
    
    pub async fn handle_request(&mut self, req: Request) -> Response {
        match self.determine_mode() {
            ServiceMode::Full => self.process_fully(req).await,
            ServiceMode::Reduced => self.process_critical_only(req).await,
            ServiceMode::Essential => self.process_emergency(req).await,
        }
    }
}
```

---

## 5. Fallback Values

### Provide Safe Defaults When Services Fail

```rust
pub async fn get_user_preferences(user_id: u64) -> Preferences {
    match fetch_from_service(user_id).await {
        Ok(prefs) => prefs,
        Err(e) => {
            // Try cache
            if let Ok(cached) = get_from_cache(user_id).await {
                return cached;
            }
            
            // Fall back to defaults
            log::warn!("Using default preferences for user {}: {}", user_id, e);
            Preferences::default()
        }
    }
}
```

---

## 6. Bulkhead with Timeout

### Combined Pattern for Resilience

```rust
pub async fn execute_with_timeout<F, T>(
    bulkhead: &Bulkhead,
    timeout_secs: u64,
    f: F,
) -> Result<T, Error>
where
    F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, Error>>,
{
    let result = bulkhead.execute(|| {
        Box::pin(async {
            tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                f(),
            )
            .await
            .map_err(|_| Error::Timeout)?
        })
    }).await;
    
    result
}
```

---

## 7. Health Checks

### Detect Problems Before Users Notice

```rust
pub trait HealthCheck {
    async fn is_healthy(&self) -> bool;
}

pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl HealthChecker {
    pub async fn check_all(&self) -> HealthStatus {
        let results = futures::future::join_all(
            self.checks.iter().map(|c| c.is_healthy())
        ).await;
        
        let healthy = results.iter().all(|&h| h);
        
        HealthStatus {
            healthy,
            timestamp: Utc::now(),
            details: results,
        }
    }
}
```

---

## 8. Dead Letter Queue

### Handle Unprocessable Messages

```rust
pub struct MessageProcessor {
    processing_queue: Queue<Message>,
    dead_letter_queue: Queue<(Message, Error)>,
}

impl MessageProcessor {
    pub async fn process_message(&mut self, msg: Message) {
        match self.handle_message(&msg).await {
            Ok(_) => {},
            Err(e) if self.is_retryable(&e) => {
                // Requeue for retry
                self.processing_queue.push(msg);
            }
            Err(e) => {
                // Move to DLQ for investigation
                self.dead_letter_queue.push((msg, e));
                log::error!("Message moved to DLQ: {:?}", e);
            }
        }
    }
}
```

---

## 9. Request Deduplication

### Prevent Duplicate Processing

```rust
pub struct RequestDeduplicator {
    processed: HashSet<String>,
    ttl_secs: u64,
}

impl RequestDeduplicator {
    pub async fn process_once<F, T>(
        &mut self,
        request_id: String,
        f: F,
    ) -> Result<T, Error>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, Error>>,
    {
        if self.processed.contains(&request_id) {
            return Err(Error::DuplicateRequest);
        }
        
        let result = f().await;
        
        if result.is_ok() {
            self.processed.insert(request_id);
        }
        
        result
    }
}
```

---

## 10. Error Aggregation

### Collect and Report Multiple Errors

```rust
pub struct ErrorAggregator {
    errors: Vec<Error>,
}

impl ErrorAggregator {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }
    
    pub fn add(&mut self, error: Error) {
        self.errors.push(error);
    }
    
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
    
    pub fn into_result(self) -> Result<(), AggregatedError> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(AggregatedError {
                errors: self.errors,
            })
        }
    }
}
```

---

## Pattern Selection Guide

| Pattern | Problem | Solution | Trade-off |
|---------|---------|----------|-----------|
| **Retry** | Transient failures | Automatic retry | May mask real issues |
| **Circuit Breaker** | Cascading failures | Stop calling failing service | Requires manual reset |
| **Bulkhead** | Resource exhaustion | Limit concurrent requests | Less throughput |
| **Graceful Degradation** | Overload | Reduce features | User experience impact |
| **Fallback** | Service unavailable | Use cached/default | Stale data |
| **Health Check** | Undetected failures | Monitor continuously | Performance cost |
| **Dead Letter** | Unprocessable messages | Queue for investigation | Extra complexity |
| **Deduplication** | Duplicate processing | Track processed requests | Storage overhead |

---

**Last Updated**: 2025-11-15
**Version**: v1.1.0
**Framework**: Error Recovery Patterns
