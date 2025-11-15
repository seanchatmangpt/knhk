# How-to Guide: Implement Workflow Patterns

**Goal**: Apply the 43 workflow patterns to real problems
**Time**: 30-90 minutes (depends on pattern complexity)
**Difficulty**: Advanced

## Pattern Selection Guide

### When to Use Each Pattern

```
Sequential Tasks?
└─ Pattern 1: Sequence

Independent Tasks?
├─ Pattern 2: Parallel Split
├─ Pattern 5: Simple Merge
└─ Pattern 23: Caching

Multiple Paths?
├─ Pattern 4: Exclusive Choice
└─ Pattern 6: Multi-Choice

Data Processing?
├─ Pattern 9: Pipe and Filter
├─ Pattern 10: Aggregation
└─ Pattern 11: Data Enrichment

Failures Expected?
├─ Pattern 31: Error Handling
├─ Pattern 32: Retry
└─ Pattern 33: Compensation

Distributed?
├─ Pattern 41: Sagas
├─ Pattern 42: Event Sourcing
└─ Pattern 43: CQRS
```

## Step 1: Pattern 1 - Simple Sequence

### Use Case

Processing steps that must happen in order.

### Implementation

```rust
#[instrument]
pub async fn sequential_workflow(input: Input) -> Result<Output> {
    info!("Starting workflow");

    let step1 = step_one(input).await?;
    info!("Completed step 1");

    let step2 = step_two(step1).await?;
    info!("Completed step 2");

    let step3 = step_three(step2).await?;
    info!("Completed step 3");

    Ok(step3)
}
```

### When to Use

- Dependencies between steps
- Must maintain order
- Linear workflow

## Step 2: Pattern 2 - Parallel Split

### Use Case

Independent tasks that can run simultaneously.

### Implementation

```rust
#[instrument]
pub async fn parallel_workflow(input: Input) -> Result<CombinedOutput> {
    let (result1, result2, result3) = tokio::join!(
        process_path_1(&input),
        process_path_2(&input),
        process_path_3(&input)
    );

    let combined = combine(result1?, result2?, result3?)?;
    Ok(combined)
}
```

### When to Use

- Independent operations
- Improve throughput
- Reduce latency

## Step 3: Pattern 4 - Exclusive Choice

### Use Case

Different paths based on input conditions.

### Implementation

```rust
#[instrument]
pub async fn branching_workflow(input: Input) -> Result<Output> {
    match input.category {
        "priority" => {
            info!("Using fast track");
            fast_track_process(input).await
        }
        "standard" => {
            info!("Using standard track");
            standard_process(input).await
        }
        "batch" => {
            info!("Using batch track");
            batch_process(input).await
        }
        _ => {
            warn!("Unknown category");
            Err("Unknown category".into())
        }
    }
}
```

### When to Use

- Different processing paths
- Input-dependent logic
- Multiple strategies

## Step 4: Pattern 9 - Pipe and Filter

### Use Case

Data transformation pipeline.

### Implementation

```rust
#[instrument]
pub async fn pipeline_workflow(items: Vec<Item>) -> Result<Vec<Output>> {
    Ok(items
        .into_iter()
        .filter(|item| {
            info!(id = item.id, "Filtering item");
            item.is_valid()
        })
        .map(|item| {
            info!(id = item.id, "Transforming");
            transform(item)
        })
        .filter_map(Result::ok)
        .map(|item| {
            info!(id = item.id, "Enriching");
            enrich(item)
        })
        .filter_map(Result::ok)
        .collect())
}
```

### When to Use

- Linear transformations
- Multiple filters
- Clear pipeline stages

## Step 5: Pattern 10 - Aggregation

### Use Case

Combining multiple inputs into one output.

### Implementation

```rust
#[instrument]
pub async fn aggregation_workflow(
    input1: Vec<Data>,
    input2: Vec<Data>,
    input3: Vec<Data>
) -> Result<AggregatedOutput> {
    let mut aggregator = Aggregator::new();

    for item in input1 {
        aggregator.add("source1", item).await?;
    }

    for item in input2 {
        aggregator.add("source2", item).await?;
    }

    for item in input3 {
        aggregator.add("source3", item).await?;
    }

    aggregator.finalize()
}
```

### When to Use

- Combining data sources
- Merging results
- Unified output

## Step 6: Pattern 32 - Retry

### Use Case

Transient failures that might succeed on retry.

### Implementation

```rust
#[instrument]
pub async fn retry_workflow<F, T>(
    name: &str,
    mut f: F,
    max_retries: u32
) -> Result<T>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T>>>>,
{
    let mut attempt = 0;

    loop {
        match f().await {
            Ok(result) => {
                info!(name = name, attempts = attempt, "Success");
                return Ok(result);
            }
            Err(e) if attempt < max_retries => {
                attempt += 1;
                let backoff = Duration::from_secs(2_u64.pow(attempt - 1));

                warn!(
                    name = name,
                    attempt = attempt,
                    error = %e,
                    backoff_seconds = backoff.as_secs(),
                    "Retrying"
                );

                tokio::time::sleep(backoff).await;
            }
            Err(e) => {
                error!(name = name, attempts = attempt, error = %e, "Failed");
                return Err(e);
            }
        }
    }
}
```

### When to Use

- Network operations
- Transient failures
- Database connections

## Step 7: Pattern 33 - Compensation

### Use Case

Undo previous work on failure.

### Implementation

```rust
#[instrument]
pub async fn compensation_workflow(input: Input) -> Result<Output> {
    // Step 1: Create
    let created = create_resource(&input).await?;
    info!(id = created.id, "Created resource");

    // Step 2: Process
    match process_resource(&created).await {
        Ok(output) => {
            info!(id = created.id, "Processed successfully");
            Ok(output)
        }
        Err(e) => {
            // Compensate: Undo step 1
            warn!(id = created.id, error = %e, "Processing failed, compensating");
            delete_resource(&created).await
                .map_err(|ce| {
                    error!("Compensation failed: {}", ce);
                    ce
                })?;
            Err(e)
        }
    }
}
```

### When to Use

- Multi-step operations
- Failure rollback needed
- Side effects to undo

## Step 8: Pattern 41 - Sagas

### Use Case

Distributed transactions across multiple services.

### Implementation

```rust
#[instrument]
pub async fn saga_workflow(
    order: Order
) -> Result<CompletedOrder> {
    let mut saga = Saga::new();

    // Step 1: Reserve inventory
    let inventory_id = reserve_inventory(&order).await?;
    saga.add_compensation(|| async {
        release_inventory(inventory_id).await
    });
    info!("Inventory reserved");

    // Step 2: Process payment
    let payment_id = process_payment(&order).await?;
    saga.add_compensation(|| async {
        refund_payment(payment_id).await
    });
    info!("Payment processed");

    // Step 3: Ship order
    match ship_order(&order).await {
        Ok(shipment_id) => {
            info!("Order shipped");
            Ok(CompletedOrder { order, shipment_id })
        }
        Err(e) => {
            // Compensate in reverse order
            warn!("Shipping failed, compensating");
            saga.compensate().await?;
            Err(e)
        }
    }
}
```

### When to Use

- Multi-service workflows
- Distributed transactions
- Compensation needed

## Step 9: Pattern 42 - Event Sourcing

### Use Case

Complete audit trail of all changes.

### Implementation

```rust
#[derive(Debug, Clone)]
pub enum DomainEvent {
    OrderCreated { order_id: String, customer_id: String },
    ItemAdded { order_id: String, item_id: String },
    PaymentProcessed { order_id: String, amount: f64 },
    OrderShipped { order_id: String },
}

#[instrument]
pub async fn event_sourcing_workflow(
    order_id: &str,
    events: Vec<DomainEvent>
) -> Result<OrderState> {
    let mut state = OrderState::default();

    for event in events {
        info!(order_id = order_id, event = ?event, "Applying event");

        state = match event {
            DomainEvent::OrderCreated { customer_id, .. } => {
                state.with_customer(customer_id)
            }
            DomainEvent::ItemAdded { item_id, .. } => {
                state.add_item(item_id)?
            }
            DomainEvent::PaymentProcessed { amount, .. } => {
                state.with_payment(amount)
            }
            DomainEvent::OrderShipped { .. } => {
                state.mark_shipped()
            }
        };
    }

    Ok(state)
}
```

### When to Use

- Audit trail required
- Complete history needed
- Event replay capability

## Step 10: Combining Multiple Patterns

Real workflows use several patterns:

```rust
pub async fn complete_workflow(input: Input) -> Result<Output> {
    // Pattern 4: Exclusive Choice (branch)
    let path = match input.type_ {
        "urgent" => "fast",
        _ => "standard",
    };

    // Pattern 1: Sequence (steps in order)
    let validated = validate(input).await?;
    let enriched = enrich(validated).await?;

    // Pattern 2: Parallel Split (parallel processing)
    let (price, stock, shipping) = tokio::join!(
        get_price(&enriched),
        check_stock(&enriched),
        estimate_shipping(&enriched)
    );

    // Pattern 10: Aggregation (combine)
    let combined = aggregate(price?, stock?, shipping?)?;

    // Pattern 32: Retry (fault tolerance)
    retry_workflow(
        "reserve",
        || Box::pin(reserve_inventory(&combined)),
        3
    ).await?;

    // Pattern 33: Compensation (undo on error)
    match complete_order(&combined).await {
        Ok(result) => Ok(result),
        Err(e) => {
            release_inventory(&combined).await.ok();
            Err(e)
        }
    }
}
```

## Workflow Checklist

When implementing a pattern:

- [ ] Chosen correct pattern for problem
- [ ] All steps identified
- [ ] Dependencies mapped
- [ ] Error cases handled
- [ ] Compensation logic defined
- [ ] Instrumented with telemetry
- [ ] Schema documented
- [ ] Tests written
- [ ] Performance validated
- [ ] Deployed successfully

## Next Steps

- **Apply patterns**: Use in your workflows
- **Combine wisely**: Compose complex workflows
- **Monitor**: Track workflow metrics
- **Optimize**: Based on performance data

---

**Category**: How-to Guides (Task-oriented)
**Framework**: Diátaxis
**Difficulty**: Advanced
**Related**: Workflow Patterns, Schema-First Development
