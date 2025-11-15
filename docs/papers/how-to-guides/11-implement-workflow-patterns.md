# How-to Guide 11: Implement Workflow Patterns

## Goal

Learn and apply the 43 proven workflow patterns to build production-ready features following established best practices, ensuring consistency, reliability, and maintainability across KNHK projects.

**Time Estimate**: 2.5-3 hours
**Prerequisites**: [Add New Features](04-add-new-features.md), [Use Knowledge Hooks](10-use-knowledge-hooks.md)
**Difficulty**: Advanced
**Outcomes**: Features built following proven workflow patterns

---

## What Are Workflow Patterns?

### The Concept

**Workflow Patterns** are proven, repeatable solutions for common development scenarios in KNHK projects.

```
Problem: "How do I handle user authentication safely?"
Solution: Use the "Secure Authentication Pattern"
Benefits: Proven approach, tested, documented, auditable

Problem: "How do I manage database transactions?"
Solution: Use the "ACID Compliance Pattern"
Benefits: Data consistency guaranteed

Problem: "How do I validate input safely?"
Solution: Use the "Input Validation Pattern"
Benefits: Security and data integrity assured
```

### Pattern Philosophy

```
KNHK uses patterns because:
✓ Eliminate decision paralysis (don't reinvent)
✓ Ensure best practices (proven solutions)
✓ Enable consistency (team aligns on approach)
✓ Reduce bugs (patterns are battle-tested)
✓ Speed development (known solutions, not unknown)
```

---

## Part 1: Core Patterns (1-10)

### Pattern 1: Request-Response Handler

**Use**: When building API endpoints or service operations

```rust
@knowledge.hook({
  id: "request_response_pattern",
  pattern: "RequestResponse",
  version: "1.0",
  domain: "api.patterns"
})
#[instrument]
pub async fn handle_create_user(
  req: CreateUserRequest
) -> Result<CreateUserResponse> {
  // Step 1: Validate input
  req.validate()?;
  counter!("user_creation_attempts", 1);

  // Step 2: Execute business logic
  let user = User::create(
    req.email.clone(),
    req.password.clone()
  )?;
  counter!("user_creation_success", 1);

  // Step 3: Return response
  Ok(CreateUserResponse {
    user_id: user.id,
    created_at: user.created_at,
  })
}
```

**Key Points:**
- Validate input first
- Execute core logic
- Return structured response
- Emit telemetry at each step

### Pattern 2: Error Handling and Recovery

**Use**: When operations might fail and need recovery

```rust
@knowledge.hook({
  pattern: "ErrorHandling",
  rules: [
    "Categorize errors (recoverable vs fatal)",
    "Retry on recoverable errors",
    "Log all errors with context",
    "Return meaningful error messages"
  ]
})
pub fn operation_with_retry(config: &Config) -> Result<Output> {
  let mut attempts = 0;
  let max_attempts = 3;

  loop {
    match execute_operation(config) {
      Ok(output) => {
        info!("Operation succeeded");
        return Ok(output);
      }
      Err(e) if is_recoverable(&e) && attempts < max_attempts => {
        attempts += 1;
        warn!("Operation failed (recoverable), retrying: {}", e);
        std::thread::sleep(Duration::from_millis(100 * attempts as u64));
        counter!("operation_retries", 1);
      }
      Err(e) => {
        error!("Operation failed (fatal): {}", e);
        counter!("operation_fatal_errors", 1);
        return Err(e);
      }
    }
  }
}
```

**Key Points:**
- Distinguish recoverable vs fatal errors
- Implement exponential backoff
- Log with full context
- Track error metrics

### Pattern 3: Caching Layer

**Use**: When operations are expensive and results are reusable

```rust
@knowledge.hook({
  pattern: "Caching",
  cache_strategy: "write-through",
  ttl_seconds: 3600
})
pub fn get_user_with_cache(user_id: u64) -> Result<User> {
  // Check cache first
  if let Some(cached) = CACHE.get(&user_id) {
    counter!("cache_hits", 1);
    return Ok(cached);
  }

  // Fetch from source
  counter!("cache_misses", 1);
  let user = fetch_user_from_db(user_id)?;

  // Store in cache
  CACHE.insert(user_id, user.clone());

  Ok(user)
}
```

**Key Points:**
- Check cache before expensive operation
- Update cache on miss
- Track hit/miss rates
- Implement cache invalidation

### Pattern 4: Batch Processing

**Use**: When processing multiple items efficiently

```rust
@knowledge.hook({
  pattern: "BatchProcessing",
  batch_size: 100,
  rules: [
    "Group items into batches",
    "Process batch as atomic unit",
    "Rollback entire batch on any failure"
  ]
})
pub fn process_users_batched(user_ids: Vec<u64>) -> Result<()> {
  let batch_size = 100;

  for batch in user_ids.chunks(batch_size) {
    info!("Processing batch of {} users", batch.len());

    // Process entire batch
    match process_batch(batch) {
      Ok(_) => {
        counter!("batch_success", 1);
      }
      Err(e) => {
        // Rollback entire batch
        error!("Batch failed, rolling back: {}", e);
        rollback_batch(batch)?;
        counter!("batch_failure", 1);
      }
    }
  }

  Ok(())
}
```

**Key Points:**
- Group items into batches
- Process batch atomically
- Rollback on failure
- Track batch metrics

### Pattern 5: State Machine

**Use**: When objects have distinct states with specific transitions

```rust
@knowledge.hook({
  pattern: "StateMachine",
  states: [
    "Pending",
    "Processing",
    "Completed",
    "Failed"
  ],
  transitions: [
    "Pending → Processing",
    "Processing → Completed",
    "Processing → Failed",
    "Failed → Pending (retry)"
  ]
})
pub fn transition_state(
  job: &mut Job,
  event: JobEvent
) -> Result<()> {
  match (&job.state, event) {
    (JobState::Pending, JobEvent::Start) => {
      job.state = JobState::Processing;
      info!("Job started");
      counter!("job_started", 1);
      Ok(())
    }
    (JobState::Processing, JobEvent::Complete) => {
      job.state = JobState::Completed;
      info!("Job completed");
      counter!("job_completed", 1);
      Ok(())
    }
    (JobState::Processing, JobEvent::Fail) => {
      job.state = JobState::Failed;
      error!("Job failed");
      counter!("job_failed", 1);
      Ok(())
    }
    (current, event) => {
      Err(format!("Invalid transition: {:?} + {:?}", current, event))
    }
  }
}
```

**Key Points:**
- Define all valid states
- Restrict transitions to valid paths
- Emit telemetry for each transition
- Prevent invalid state changes

### Pattern 6: Circuit Breaker

**Use**: When external service calls might fail

```rust
@knowledge.hook({
  pattern: "CircuitBreaker",
  states: ["Closed", "Open", "HalfOpen"],
  threshold: 5,
  timeout_seconds: 60
})
pub fn call_external_service(request: &Request) -> Result<Response> {
  let breaker = CIRCUIT_BREAKER.lock().unwrap();

  match breaker.state {
    CircuitState::Closed => {
      // Normal operation
      match execute_request(request) {
        Ok(resp) => {
          breaker.record_success();
          Ok(resp)
        }
        Err(e) => {
          breaker.record_failure();
          if breaker.failure_count >= 5 {
            info!("Opening circuit breaker");
            breaker.open();
            counter!("circuit_breaker_open", 1);
          }
          Err(e)
        }
      }
    }
    CircuitState::Open => {
      // Fail fast
      counter!("circuit_breaker_rejected", 1);
      Err("Circuit breaker open, failing fast".into())
    }
    CircuitState::HalfOpen => {
      // Test if service recovered
      match execute_request(request) {
        Ok(resp) => {
          info!("Service recovered, closing circuit");
          breaker.close();
          counter!("circuit_breaker_closed", 1);
          Ok(resp)
        }
        Err(e) => {
          breaker.open();
          Err(e)
        }
      }
    }
  }
}
```

**Key Points:**
- Fail fast when service is down
- Prevent cascading failures
- Implement exponential backoff in HalfOpen state
- Track circuit breaker metrics

### Pattern 7: Dependency Injection

**Use**: When components have complex dependencies

```rust
@knowledge.hook({
  pattern: "DependencyInjection",
  benefits: ["Testability", "Flexibility", "Separation of concerns"]
})
pub struct UserService {
  db: Arc<Database>,
  cache: Arc<Cache>,
  auth: Arc<AuthService>,
}

impl UserService {
  pub fn new(
    db: Arc<Database>,
    cache: Arc<Cache>,
    auth: Arc<AuthService>,
  ) -> Self {
    Self { db, cache, auth }
  }

  pub async fn create_user(&self, req: CreateUserRequest) -> Result<User> {
    // Dependencies are injected, not created
    let user = User::new(req.email);
    self.db.insert(&user).await?;
    self.cache.invalidate_users().await?;
    self.auth.register_user(&user).await?;
    Ok(user)
  }
}
```

**Key Points:**
- Accept dependencies in constructor
- Use trait objects for flexibility
- Enables easy mocking in tests
- Facilitates dependency graphs

### Pattern 8: Observer Pattern

**Use**: When multiple components need to react to events

```rust
@knowledge.hook({
  pattern: "Observer",
  event_types: ["UserCreated", "UserDeleted", "UserUpdated"]
})
pub struct EventBus {
  subscribers: HashMap<String, Vec<Box<dyn EventHandler>>>,
}

impl EventBus {
  pub fn subscribe(&mut self, event_type: &str, handler: Box<dyn EventHandler>) {
    self.subscribers
      .entry(event_type.to_string())
      .or_insert_with(Vec::new)
      .push(handler);
  }

  pub fn publish(&self, event: &Event) {
    if let Some(handlers) = self.subscribers.get(&event.event_type) {
      for handler in handlers {
        handler.handle(event);
        counter!("event_handled", 1);
      }
    }
  }
}

// Usage
pub fn create_user(user: &User, event_bus: &EventBus) -> Result<()> {
  insert_into_db(user)?;
  event_bus.publish(&Event::UserCreated(user.clone()));
  Ok(())
}
```

**Key Points:**
- Decouple event producers from consumers
- Multiple handlers per event
- Async event handling support
- Track event metrics

### Pattern 9: Decorator Pattern

**Use**: When adding behavior to objects dynamically

```rust
@knowledge.hook({
  pattern: "Decorator",
  behaviors: ["Logging", "Caching", "Authorization"]
})
pub trait UserRepository {
  fn get_user(&self, id: u64) -> Result<User>;
}

pub struct LoggingDecorator<T: UserRepository> {
  inner: T,
}

impl<T: UserRepository> UserRepository for LoggingDecorator<T> {
  fn get_user(&self, id: u64) -> Result<User> {
    info!("Getting user {}", id);
    let result = self.inner.get_user(id);
    if result.is_ok() {
      counter!("user_get_success", 1);
    }
    result
  }
}

pub struct CachingDecorator<T: UserRepository> {
  inner: T,
  cache: Arc<Cache>,
}

impl<T: UserRepository> UserRepository for CachingDecorator<T> {
  fn get_user(&self, id: u64) -> Result<User> {
    if let Some(cached) = self.cache.get(&id) {
      counter!("user_cache_hit", 1);
      return Ok(cached);
    }

    counter!("user_cache_miss", 1);
    let user = self.inner.get_user(id)?;
    self.cache.insert(id, user.clone());
    Ok(user)
  }
}
```

**Key Points:**
- Wrap objects to add behavior
- Compose decorators for multiple behaviors
- Keep decorators focused and single-responsibility
- Track decorator metrics

### Pattern 10: Pipeline Pattern

**Use**: When data needs to flow through multiple transformations

```rust
@knowledge.hook({
  pattern: "Pipeline",
  steps: [
    "Validate",
    "Transform",
    "Enrich",
    "Persist"
  ]
})
pub struct Pipeline<T> {
  steps: Vec<Box<dyn PipelineStep<T>>>,
}

impl<T> Pipeline<T> {
  pub fn execute(&self, input: T) -> Result<T> {
    let mut output = input;

    for (i, step) in self.steps.iter().enumerate() {
      info!("Executing pipeline step {}", i);
      output = step.process(output)?;
      counter!("pipeline_step_success", 1);
    }

    Ok(output)
  }
}

// Usage
let pipeline = Pipeline::new()
  .add_step(Box::new(ValidateUserData))
  .add_step(Box::new(NormalizeUserData))
  .add_step(Box::new(EnrichUserData))
  .add_step(Box::new(PersistUserData));

let user_data = pipeline.execute(raw_user_data)?;
```

**Key Points:**
- Define clear pipeline stages
- Each step transforms data
- Composable and reusable steps
- Easy to add/remove steps

---

## Part 2: Intermediate Patterns (11-25)

### Pattern 11: Bulkhead Pattern

Isolate resources to prevent failures from cascading:

```rust
@knowledge.hook({
  pattern: "Bulkhead",
  purpose: "Isolate critical resources"
})
// Use thread pool to isolate payment processing
let payment_thread_pool = ThreadPool::new(4);
// Other operations use separate thread pool
let io_thread_pool = ThreadPool::new(8);
```

### Pattern 12: Saga Pattern

Manage distributed transactions with compensating actions:

```rust
@knowledge.hook({
  pattern: "Saga",
  steps: ["Reserve", "Charge", "Ship", "Confirm"],
  compensations: ["UnReserve", "Refund", "Cancel"]
})
pub async fn process_order(order: &Order) -> Result<()> {
  reserve_inventory(&order)?;  // Step 1

  match charge_payment(&order).await {
    Ok(_) => { /* Step 2 */ }
    Err(e) => {
      // Compensate
      unreserve_inventory(&order).await?;
      return Err(e);
    }
  }

  // Continue with steps 3-4...
}
```

### Pattern 13: Command Pattern

Encapsulate requests as objects:

```rust
@knowledge.hook({
  pattern: "Command",
  purpose: "Enable undo/redo"
})
pub trait Command {
  fn execute(&self) -> Result<()>;
  fn undo(&self) -> Result<()>;
}

pub struct CreateUserCommand {
  email: String,
  password: String,
}

impl Command for CreateUserCommand {
  fn execute(&self) -> Result<()> {
    create_user(&self.email, &self.password)
  }

  fn undo(&self) -> Result<()> {
    delete_user_by_email(&self.email)
  }
}
```

### Pattern 14: Strategy Pattern

Define family of algorithms, encapsulate each:

```rust
@knowledge.hook({
  pattern: "Strategy",
  strategies: ["QuantityBasedDiscount", "TimeBasedDiscount", "LoyaltyDiscount"]
})
pub trait DiscountStrategy {
  fn calculate(&self, amount: f64) -> f64;
}

pub struct QuantityBasedDiscount(pub i32);
impl DiscountStrategy for QuantityBasedDiscount {
  fn calculate(&self, amount: f64) -> f64 {
    if self.0 > 100 { amount * 0.1 } else { 0.0 }
  }
}

pub fn apply_discount(amount: f64, strategy: &dyn DiscountStrategy) -> f64 {
  amount - strategy.calculate(amount)
}
```

### Pattern 15: Template Method Pattern

Define algorithm skeleton, let subclasses fill in details:

```rust
@knowledge.hook({
  pattern: "TemplateMethod",
  skeleton: ["Prepare", "Process", "Validate", "Cleanup"]
})
pub trait DataProcessor {
  fn process(&self, data: Vec<u8>) -> Result<Vec<u8>> {
    let prepared = self.prepare(data)?;
    let processed = self.execute_processing(&prepared)?;
    let validated = self.validate(&processed)?;
    self.cleanup(&validated)?;
    Ok(validated)
  }

  fn prepare(&self, data: Vec<u8>) -> Result<Vec<u8>>;
  fn execute_processing(&self, data: &[u8]) -> Result<Vec<u8>>;
  fn validate(&self, data: &[u8]) -> Result<Vec<u8>>;
  fn cleanup(&self, data: &[u8]) -> Result<()>;
}
```

### Pattern 16: Builder Pattern

Construct complex objects step by step:

```rust
@knowledge.hook({
  pattern: "Builder",
  advantages: ["Readable construction", "Optional fields", "Validation"]
})
pub struct QueryBuilder {
  table: String,
  filters: Vec<Filter>,
  order_by: Option<String>,
  limit: Option<u32>,
}

impl QueryBuilder {
  pub fn new(table: &str) -> Self {
    Self {
      table: table.to_string(),
      filters: Vec::new(),
      order_by: None,
      limit: None,
    }
  }

  pub fn filter(mut self, filter: Filter) -> Self {
    self.filters.push(filter);
    self
  }

  pub fn order_by(mut self, column: &str) -> Self {
    self.order_by = Some(column.to_string());
    self
  }

  pub fn limit(mut self, n: u32) -> Self {
    self.limit = Some(n);
    self
  }

  pub fn build(self) -> Query {
    Query {
      table: self.table,
      filters: self.filters,
      order_by: self.order_by,
      limit: self.limit,
    }
  }
}

// Usage
let query = QueryBuilder::new("users")
  .filter(Filter::eq("status", "active"))
  .order_by("created_at")
  .limit(100)
  .build();
```

### Patterns 17-25: Additional Intermediate Patterns

- **Pattern 17: Adapter Pattern** - Convert interface of class to another
- **Pattern 18: Factory Pattern** - Create objects without specifying exact classes
- **Pattern 19: Prototype Pattern** - Create object by cloning prototype
- **Pattern 20: Visitor Pattern** - Perform operations on elements of collection
- **Pattern 21: Memento Pattern** - Capture and restore object state
- **Pattern 22: Chain of Responsibility** - Pass requests along chain of handlers
- **Pattern 23: Proxy Pattern** - Provide surrogate for another object
- **Pattern 24: Interpreter Pattern** - Define grammar for language
- **Pattern 25: Composite Pattern** - Compose objects into trees

---

## Part 3: Advanced Patterns (26-43)

### Pattern 26: Event Sourcing

Store all changes as sequence of events:

```rust
@knowledge.hook({
  pattern: "EventSourcing",
  advantages: ["Full history", "Audit trail", "Replay capability"]
})
pub struct EventStore {
  events: Vec<DomainEvent>,
}

impl EventStore {
  pub fn append(&mut self, event: DomainEvent) -> Result<()> {
    self.events.push(event);
    counter!("event_appended", 1);
    Ok(())
  }

  pub fn rebuild_state(&self, aggregate_id: u64) -> Result<AggregateState> {
    let mut state = AggregateState::default();

    for event in self.events.iter().filter(|e| e.aggregate_id == aggregate_id) {
      state.apply_event(event);
    }

    Ok(state)
  }
}
```

### Pattern 27: CQRS (Command Query Responsibility Segregation)

Separate read and write models:

```rust
@knowledge.hook({
  pattern: "CQRS",
  separation: "Commands (write) vs Queries (read)"
})
// Write Model
pub async fn create_user(cmd: CreateUserCommand) -> Result<UserId> {
  let user = User::new(cmd.email, cmd.password);
  event_store.append(UserCreated(user.clone()))?;
  counter!("user_created", 1);
  Ok(user.id)
}

// Read Model (optimized for queries)
pub async fn get_active_users() -> Result<Vec<UserSummary>> {
  // Query denormalized read model
  read_db.query_active_users().await
}
```

### Patterns 28-43: Advanced Patterns List

- **Pattern 28: Reactor Pattern** - Handle concurrent requests efficiently
- **Pattern 29: Proactor Pattern** - Async I/O with event notification
- **Pattern 30: Thread Pool Pattern** - Manage thread lifecycle
- **Pattern 31: Producer-Consumer Pattern** - Decouple work units from processing
- **Pattern 32: Read-Write Lock Pattern** - Optimize read-heavy workloads
- **Pattern 33: Double-Checked Locking Pattern** - Lazy initialization with thread safety
- **Pattern 34: Copy-On-Write Pattern** - Efficient shared data structures
- **Pattern 35: Immutable Objects Pattern** - Thread-safe by default
- **Pattern 36: Active Object Pattern** - Encapsulate own thread
- **Pattern 37: Monitor Object Pattern** - Synchronize method execution
- **Pattern 38: Thread-Safe Singleton Pattern** - One instance across threads
- **Pattern 39: Future Pattern** - Asynchronous computation representation
- **Pattern 40: Graceful Shutdown Pattern** - Clean resource cleanup
- **Pattern 41: Bulkhead Isolation Pattern** - Fault isolation
- **Pattern 42: Timeout Pattern** - Prevent indefinite blocking
- **Pattern 43: Sentinel Pattern** - Detect and handle exceptional conditions

---

## Part 4: Applying Patterns Step-by-Step

### Step 1: Identify the Problem

```
Problem: "Users can't log back in when password reset expires"
Pattern: State Machine + Timeout Pattern
```

### Step 2: Find Matching Pattern

```rust
// State Machine for password reset lifecycle
enum PasswordResetState {
  Pending,      // User requested reset
  Completed,    // User confirmed reset
  Expired,      // 24-hour window passed
}

// Timeout Pattern to detect expiration
struct PasswordResetToken {
  token: String,
  created_at: Instant,
  expires_in: Duration,
}

impl PasswordResetToken {
  fn is_expired(&self) -> bool {
    self.created_at.elapsed() > self.expires_in
  }
}
```

### Step 3: Implement with K-hooks

```rust
@knowledge.hook({
  id: "password_reset_flow",
  domain: "auth.patterns",
  patterns: ["StateMachine", "TimeoutPattern"],

  rules: [
    "Reset token valid for 24 hours",
    "Expired tokens cannot be redeemed",
    "State transitions: Pending → Completed/Expired"
  ],

  related_tests: [
    "test_reset_token_expires",
    "test_expired_token_rejected",
    "test_state_transitions"
  ]
})
pub fn process_password_reset(token: &PasswordResetToken, new_password: &str) -> Result<()> {
  if token.is_expired() {
    return Err("Reset token expired");
  }

  update_user_password(new_password)?;
  counter!("password_reset_success", 1);

  Ok(())
}
```

### Step 4: Write Tests for Pattern Behavior

```rust
#[test]
fn test_password_reset_pattern() {
  // Test state transitions
  let mut state = PasswordResetState::Pending;

  // Valid transition
  state = PasswordResetState::Completed;
  assert_eq!(state, PasswordResetState::Completed);

  // Test timeout
  let token = PasswordResetToken::new(Duration::from_secs(24));
  assert!(!token.is_expired());

  // Simulate time passing
  std::thread::sleep(Duration::from_secs(1));
  // In real code, would use test time library
}
```

---

## Part 5: Pattern Selection Guide

### By Use Case

**Authentication**: Guardian Pattern, State Machine
**Caching**: Caching Layer, Invalidation Pattern
**Concurrency**: Thread Pool, Read-Write Lock
**Distributed Systems**: Circuit Breaker, Saga
**Error Handling**: Error Handling, Bulkhead
**Event Processing**: Observer, Event Sourcing
**Data Access**: CQRS, Repository
**API Design**: Request-Response, Adapter

### By Performance Needs

**High Throughput**: Thread Pool, Batch Processing, Producer-Consumer
**Low Latency**: Caching, Circuit Breaker, Future Pattern
**Memory Efficient**: Iterator, Lazy Initialization
**CPU Efficient**: Pipeline, Strategy Pattern

### By Consistency Needs

**Strong Consistency**: Database Transactions, Saga (compensating)
**Eventual Consistency**: Event Sourcing, CQRS
**Read-Heavy**: Read-Write Lock, CQRS

---

## Summary: Pattern Usage Checklist

- [ ] Identify the problem you're solving
- [ ] Find matching pattern(s)
- [ ] Implement pattern structure
- [ ] Add K-hook documentation
- [ ] Link to related patterns
- [ ] Write pattern-specific tests
- [ ] Document decisions
- [ ] Train team on pattern

---

**Created**: 2025-11-15
**Status**: Complete
**Difficulty**: Advanced
**Patterns Covered**: 43 production workflow patterns
**Related**: [Use Knowledge Hooks](10-use-knowledge-hooks.md), [Add New Features](04-add-new-features.md)
