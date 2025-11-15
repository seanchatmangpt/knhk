# Tutorial: Knowledge Hooks and Workflow Patterns

**Level**: Expert
**Time**: 70-85 minutes
**Learning Objectives**: Master knowledge hooks and implement the 43 workflow patterns

## What You'll Learn

By the end of this tutorial, you'll understand:
- Knowledge hooks (K-hooks) concept and usage
- The 43 fundamental workflow patterns
- Pattern composition and combination
- Real-world pattern applications
- Performance implications
- Best practices and anti-patterns

## Prerequisites

- Completed: [Schema-First Development](06-schema-first-development.md)
- Deep Rust knowledge
- ~85 minutes

## Part 1: Knowledge Hooks Fundamentals

### What Are Knowledge Hooks?

Knowledge Hooks are decision points in workflows that:
- Collect information about system state
- Make intelligent decisions based on context
- Route execution through optimal paths
- Maintain state across transformations
- Enable adaptive workflows

```rust
/// Knowledge Hook: Analyzes input and routes processing
#[derive(Debug, Clone)]
pub struct KnowledgeHook {
    name: String,
    input_schema: Schema,
    output_schema: Schema,
    decision_logic: Box<dyn Fn(&Input) -> Decision>,
    context: HashMap<String, Value>,
}

#[derive(Debug)]
pub enum Decision {
    Route(String),        // Route to named workflow
    Transform(String),    // Apply transformation
    Aggregate,           // Combine with other data
    Cache,               // Cache result
    Skip,                // Skip processing
}

impl KnowledgeHook {
    pub fn new(name: &str, logic: Box<dyn Fn(&Input) -> Decision>) -> Self {
        Self {
            name: name.to_string(),
            input_schema: Schema::new(),
            output_schema: Schema::new(),
            decision_logic: logic,
            context: HashMap::new(),
        }
    }

    pub fn decide(&self, input: &Input) -> Decision {
        (self.decision_logic)(input)
    }

    pub fn with_context(mut self, key: String, value: Value) -> Self {
        self.context.insert(key, value);
        self
    }
}
```

## Part 2: The 43 Workflow Patterns

### Pattern Categories

```
1-8:    Control Flow Patterns
9-20:   Data Patterns
21-30:  Resource Patterns
31-40:  Exception Handling
41-43:  Advanced Patterns
```

### 1-8: Control Flow Patterns

```rust
/// Pattern 1: Sequence - Execute steps in order
pub async fn sequence_workflow(inputs: Vec<Data>) -> Result<Vec<Output>> {
    let mut results = Vec::new();

    for input in inputs {
        let result = step_one(input).await?;
        let result = step_two(result).await?;
        let result = step_three(result).await?;
        results.push(result);
    }

    Ok(results)
}

/// Pattern 2: Parallel Split - Execute steps in parallel
pub async fn parallel_split_workflow(input: Data) -> Result<(Output1, Output2, Output3)> {
    let (result1, result2, result3) = tokio::join!(
        step_one(&input),
        step_two(&input),
        step_three(&input)
    );

    Ok((result1?, result2?, result3?))
}

/// Pattern 3: Synchronization - Wait for all to complete
pub async fn synchronization_workflow(
    inputs1: Vec<Data>,
    inputs2: Vec<Data>
) -> Result<Vec<Output>> {
    // Process in parallel
    let handles1: Vec<_> = inputs1.into_iter()
        .map(|i| tokio::spawn(async move { process_1(i).await }))
        .collect();

    let handles2: Vec<_> = inputs2.into_iter()
        .map(|i| tokio::spawn(async move { process_2(i).await }))
        .collect();

    // Wait for all
    let results1 = futures::future::join_all(handles1).await;
    let results2 = futures::future::join_all(handles2).await;

    // Synchronize
    let combined = combine_results(results1, results2)?;
    Ok(combined)
}

/// Pattern 4: Exclusive Choice - Branch based on condition
pub async fn exclusive_choice_workflow(input: Data) -> Result<Output> {
    let hook = KnowledgeHook::new("route_decision", Box::new(|data| {
        match data.category {
            "urgent" => Decision::Route("fast_path".to_string()),
            "normal" => Decision::Route("normal_path".to_string()),
            "deferred" => Decision::Route("batch_path".to_string()),
            _ => Decision::Skip,
        }
    }));

    match hook.decide(&input) {
        Decision::Route(path) if path == "fast_path" => fast_process(input).await,
        Decision::Route(path) if path == "normal_path" => normal_process(input).await,
        Decision::Route(path) if path == "batch_path" => batch_process(input).await,
        _ => Err("Unknown route".into()),
    }
}

/// Pattern 5: Simple Merge - Combine parallel paths
pub async fn simple_merge_workflow(
    input: Data
) -> Result<Output> {
    let (path1, path2) = tokio::join!(
        process_path_1(&input),
        process_path_2(&input)
    );

    merge_outputs(path1?, path2?)
}

/// Pattern 6: Multi-Choice - Execute multiple matching branches
pub async fn multi_choice_workflow(input: Data) -> Result<Vec<Output>> {
    let mut outputs = Vec::new();

    if input.tags.contains("priority") {
        outputs.push(priority_process(&input).await?);
    }
    if input.tags.contains("archive") {
        outputs.push(archive_process(&input).await?);
    }
    if input.tags.contains("notify") {
        outputs.push(notify_process(&input).await?);
    }

    Ok(outputs)
}

/// Pattern 7: Structured Loop - Iterate with condition
pub async fn structured_loop_workflow(
    items: Vec<Item>,
    threshold: usize
) -> Result<Vec<Output>> {
    let mut results = Vec::new();
    let mut count = 0;

    for item in items {
        if count >= threshold {
            break;
        }

        let result = process_item(item).await?;
        results.push(result);
        count += 1;
    }

    Ok(results)
}

/// Pattern 8: Arbitrary Cycles - Loop with complex condition
pub async fn arbitrary_cycles_workflow(
    mut state: State
) -> Result<Output> {
    loop {
        let next_state = process_iteration(&state).await?;

        if next_state.converged() {
            return Ok(next_state.to_output());
        }

        if next_state.iterations > 1000 {
            return Err("Iteration limit exceeded".into());
        }

        state = next_state;
    }
}
```

### 9-20: Data Patterns

```rust
/// Pattern 9: Pipe and Filter - Chain transformations
pub async fn pipe_and_filter_workflow(data: Data) -> Result<FinalOutput> {
    data
        .filter(|item| item.is_valid())
        .map(|item| transform_1(item))
        .and_then(|item| transform_2(item))
        .and_then(|item| transform_3(item))
}

/// Pattern 10: Aggregation - Combine multiple inputs
pub async fn aggregation_workflow(
    inputs: Vec<Data>
) -> Result<AggregatedOutput> {
    let mut aggregator = Aggregator::new();

    for input in inputs {
        aggregator.add(input).await?;
    }

    aggregator.finalize()
}

/// Pattern 11: Data Enrichment - Add information
pub async fn data_enrichment_workflow(
    input: Data
) -> Result<EnrichedData> {
    let enriched = input
        .with_metadata(fetch_metadata(&input).await?)
        .with_context(fetch_context(&input).await?)
        .with_relationships(fetch_relationships(&input).await?);

    Ok(enriched)
}

/// Pattern 12: Data Validation - Check correctness
pub async fn data_validation_workflow(
    input: Data
) -> Result<ValidatedData> {
    if !input.is_valid() {
        return Err("Invalid data".into());
    }

    if !input.meets_schema() {
        return Err("Schema mismatch".into());
    }

    Ok(input)
}

/// Pattern 13: Splitting - Divide into multiple streams
pub async fn splitting_workflow(
    input: Data
) -> Result<(Stream1, Stream2, Stream3)> {
    let stream1 = input.split_by_category("A").await?;
    let stream2 = input.split_by_category("B").await?;
    let stream3 = input.split_by_category("C").await?;

    Ok((stream1, stream2, stream3))
}
```

### 21-30: Resource Patterns

```rust
/// Pattern 21: Resource Allocation - Manage limited resources
pub struct ResourceAllocator {
    semaphore: Semaphore,
    pool: ResourcePool,
}

impl ResourceAllocator {
    pub async fn allocate(&self) -> Result<ResourceGuard> {
        let permit = self.semaphore.acquire().await?;
        let resource = self.pool.get().await?;

        Ok(ResourceGuard {
            resource,
            _permit: permit,
        })
    }
}

/// Pattern 22: Resource Pooling - Reuse resources
pub struct ResourcePool {
    available: Arc<Mutex<Vec<Resource>>>,
    max_size: usize,
}

impl ResourcePool {
    pub async fn get(&self) -> Result<PooledResource> {
        let mut resources = self.available.lock().await;

        if let Some(resource) = resources.pop() {
            Ok(PooledResource::Pooled(resource))
        } else {
            Ok(PooledResource::New(Resource::new()))
        }
    }

    pub async fn return_resource(&self, resource: Resource) {
        let mut resources = self.available.lock().await;
        if resources.len() < self.max_size {
            resources.push(resource);
        }
    }
}

/// Pattern 23: Caching - Store computed results
pub struct CachingLayer<T> {
    cache: Arc<Mutex<HashMap<String, T>>>,
    ttl: Duration,
}

impl<T: Clone> CachingLayer<T> {
    pub async fn get_or_compute<F>(&self, key: &str, f: F) -> Result<T>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T>>>>,
    {
        // Check cache
        {
            let cache = self.cache.lock().await;
            if let Some(value) = cache.get(key) {
                return Ok(value.clone());
            }
        }

        // Compute
        let value = f().await?;

        // Store
        {
            let mut cache = self.cache.lock().await;
            cache.insert(key.to_string(), value.clone());
        }

        Ok(value)
    }
}
```

### 31-40: Exception Handling

```rust
/// Pattern 31: Error Handling - Catch and handle
pub async fn error_handling_workflow(
    input: Data
) -> Result<Output> {
    match process(input).await {
        Ok(output) => Ok(output),
        Err(e) => {
            error!("Process failed: {}", e);
            fallback_process().await
        }
    }
}

/// Pattern 32: Retry - Attempt multiple times
pub async fn retry_workflow<F, T>(
    f: F,
    max_retries: u32
) -> Result<T>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T>>>>,
{
    let mut retries = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                warn!("Retry {} after: {}", retries, e);
                tokio::time::sleep(Duration::from_secs(2_u64.pow(retries))).await;
            }
            Err(e) => return Err(e),
        }
    }
}

/// Pattern 33: Compensation - Undo on failure
pub async fn compensation_workflow(
    input: Data
) -> Result<Output> {
    let created = create_resource(&input).await?;

    match process(&created).await {
        Ok(output) => Ok(output),
        Err(e) => {
            // Compensate (undo)
            delete_resource(&created).await.ok();
            Err(e)
        }
    }
}
```

### 41-43: Advanced Patterns

```rust
/// Pattern 41: Sagas - Distributed transactions
pub struct Saga {
    steps: Vec<SagaStep>,
    compensations: Vec<Compensation>,
}

impl Saga {
    pub async fn execute(&mut self) -> Result<SagaResult> {
        for (idx, step) in self.steps.iter().enumerate() {
            match step.execute().await {
                Ok(result) => {
                    self.compensations.push(step.compensation());
                }
                Err(e) => {
                    // Compensate backward
                    for comp in self.compensations.iter().rev() {
                        comp.execute().await.ok();
                    }
                    return Err(e);
                }
            }
        }

        Ok(SagaResult::Success)
    }
}

/// Pattern 42: Event Sourcing - Immutable event log
pub struct EventStore {
    events: Arc<Mutex<Vec<Event>>>,
}

impl EventStore {
    pub async fn append(&self, event: Event) -> Result<()> {
        let mut events = self.events.lock().await;
        events.push(event);
        Ok(())
    }

    pub async fn replay(&self) -> Result<State> {
        let events = self.events.lock().await;
        let mut state = State::default();

        for event in events.iter() {
            state = state.apply(event)?;
        }

        Ok(state)
    }
}

/// Pattern 43: CQRS - Separate read/write models
pub struct CQRS {
    write_model: WriteModel,
    read_model: ReadModel,
}

impl CQRS {
    pub async fn command(&self, cmd: Command) -> Result<()> {
        self.write_model.execute(cmd).await?;
        self.read_model.update().await?;
        Ok(())
    }

    pub async fn query(&self, q: Query) -> Result<QueryResult> {
        self.read_model.execute(q).await
    }
}
```

## Part 3: Combining Patterns

Real-world workflows combine multiple patterns:

```rust
/// Example: Complete order processing workflow
pub async fn order_processing_workflow(
    order: Order
) -> Result<ProcessedOrder> {
    // Pattern 4: Exclusive Choice (route by priority)
    let path = if order.is_urgent() {
        "express"
    } else {
        "standard"
    };

    // Pattern 1: Sequence (validate → enrich → process)
    let validated = validate_order(&order).await?;
    let enriched = enrich_with_customer_data(validated).await?;

    // Pattern 2: Parallel Split (process in parallel)
    let (inventory, pricing, shipping) = tokio::join!(
        check_inventory(&enriched),
        calculate_pricing(&enriched),
        estimate_shipping(&enriched)
    );

    // Pattern 10: Aggregation (combine results)
    let combined = aggregate_results(
        inventory?,
        pricing?,
        shipping?
    )?;

    // Pattern 33: Compensation (error handling)
    match reserve_inventory(&combined).await {
        Ok(result) => Ok(result),
        Err(e) => {
            // Cleanup on error
            release_customer_quote(&combined).await.ok();
            Err(e)
        }
    }
}
```

## Part 4: Performance Implications

### Pattern Performance Characteristics

```
Pattern          | Complexity | Latency | Throughput | Best For
Sequence         | O(n)       | High    | Low        | Simple workflows
Parallel Split   | O(1)       | Medium  | High       | Independent tasks
Aggregation      | O(n)       | High    | Medium     | Combining data
CQRS             | O(2n)      | Medium  | Very High  | Read-heavy systems
Event Sourcing   | O(n)       | Medium  | High       | Audit trail needed
```

### Anti-Patterns to Avoid

```rust
// ❌ ANTI-PATTERN: Sequential when parallel possible
for item in items {
    process_slowly(item).await?;  // Each waits for previous
}

// ✅ PATTERN: Use parallel split
let futures = items.iter().map(|i| process_slowly(i));
let results = futures::future::join_all(futures).await;

// ❌ ANTI-PATTERN: Unbounded parallelism
for item in million_items {
    tokio::spawn(async move { process(item).await });
}

// ✅ PATTERN: Use semaphore for resource control
let sem = Semaphore::new(100);
for item in million_items {
    let permit = sem.acquire().await;
    tokio::spawn(async move {
        process(item).await;
        drop(permit);
    });
}
```

## What You've Learned

Congratulations! You now understand:

1. **Knowledge Hooks** - Decision points in workflows
2. **43 Patterns** - Complete pattern library
3. **Pattern Composition** - Combining patterns
4. **Performance Trade-offs** - Choosing optimal patterns
5. **Real-world Applications** - Practical examples
6. **Anti-patterns** - What to avoid

## Next Steps

- **Apply patterns**: Use in your workflows
- **Combine patterns**: Create complex workflows
- **Optimize**: Choose patterns for performance
- **Document**: Record pattern decisions

---

**You are here**: Tutorial (Learning-oriented)
**Framework**: Diátaxis
**Tutorial Duration**: ~85 minutes
**Difficulty**: Expert
**Prerequisites**: Schema-First Development
