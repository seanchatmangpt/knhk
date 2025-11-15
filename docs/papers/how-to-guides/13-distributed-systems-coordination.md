# How-to Guide: Distributed Systems Coordination

**Goal**: Coordinate work across multiple services
**Time**: 45-60 minutes
**Difficulty**: Advanced

## Service Communication Patterns

### Pattern 1: Synchronous RPC

```rust
pub struct RpcClient {
    base_url: String,
    client: reqwest::Client,
}

impl RpcClient {
    pub async fn call_service(
        &self,
        service: &str,
        method: &str,
        params: serde_json::Value
    ) -> Result<serde_json::Value, AppError> {
        let url = format!("{}/{}/{}", self.base_url, service, method);

        self.client
            .post(&url)
            .json(&params)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AppError::TransientError(e.to_string()))?
            .json()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))
    }
}
```

### Pattern 2: Asynchronous Messaging

```rust
pub struct MessageQueue {
    queue: Arc<Mutex<VecDeque<Message>>>,
}

pub struct Message {
    id: String,
    service: String,
    method: String,
    payload: Vec<u8>,
    timestamp: SystemTime,
}

impl MessageQueue {
    pub async fn publish(&self, msg: Message) -> Result<()> {
        let mut queue = self.queue.lock().await;
        queue.push_back(msg);
        Ok(())
    }

    pub async fn consume(&self) -> Result<Option<Message>> {
        let mut queue = self.queue.lock().await;
        Ok(queue.pop_front())
    }
}
```

### Pattern 3: Service Discovery

```rust
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, Vec<ServiceEndpoint>>>>,
}

#[derive(Clone, Debug)]
pub struct ServiceEndpoint {
    host: String,
    port: u16,
    healthy: bool,
}

impl ServiceRegistry {
    pub async fn register_service(
        &self,
        name: &str,
        endpoint: ServiceEndpoint
    ) -> Result<()> {
        let mut services = self.services.write().await;
        services
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(endpoint);
        Ok(())
    }

    pub async fn get_endpoint(
        &self,
        service: &str
    ) -> Result<ServiceEndpoint> {
        let services = self.services.read().await;

        services
            .get(service)
            .and_then(|endpoints| {
                endpoints.iter()
                    .find(|e| e.healthy)
                    .cloned()
            })
            .ok_or_else(|| AppError::NotFoundError("Service not available".into()))
    }
}
```

### Pattern 4: Distributed Tracing

```rust
pub async fn traced_call(
    span_context: TraceContext,
    service: &str,
    method: &str,
    data: Vec<u8>
) -> Result<Vec<u8>> {
    let span = Span::new(service, method)
        .with_trace_id(span_context.trace_id)
        .with_parent_span_id(span_context.span_id);

    info!(
        trace_id = %span.trace_id,
        span_id = %span.span_id,
        "Calling {} {}", service, method
    );

    let result = make_call(service, method, data).await?;

    info!(
        trace_id = %span.trace_id,
        "Call completed"
    );

    Ok(result)
}
```

## Coordination Patterns

### Pattern: Two-Phase Commit

```rust
pub struct TwoPhaseCommit {
    coordinator: String,
    participants: Vec<String>,
}

#[derive(Debug)]
enum Phase1Response {
    Yes,
    No(String),  // reason
}

impl TwoPhaseCommit {
    pub async fn execute(&self) -> Result<()> {
        // Phase 1: Prepare
        let mut votes = Vec::new();

        for participant in &self.participants {
            let vote = prepare(participant).await?;
            votes.push((participant.clone(), vote));
        }

        let all_yes = votes.iter().all(|(_, v)| matches!(v, Phase1Response::Yes));

        // Phase 2: Commit or Abort
        if all_yes {
            for (participant, _) in votes {
                commit(&participant).await?;
            }
            Ok(())
        } else {
            for (participant, _) in votes {
                abort(&participant).await?;
            }
            Err("Transaction aborted".into())
        }
    }
}
```

### Pattern: Consensus (Raft)

```rust
pub struct RaftNode {
    state: Arc<Mutex<RaftState>>,
    log: Arc<Mutex<Vec<LogEntry>>>,
    peers: Vec<String>,
}

#[derive(Clone)]
enum RaftState {
    Follower { leader: Option<String> },
    Candidate { votes: u32 },
    Leader,
}

impl RaftNode {
    pub async fn append_entry(&self, entry: LogEntry) -> Result<()> {
        let state = self.state.lock().await;

        if let RaftState::Leader = *state {
            let mut log = self.log.lock().await;
            log.push(entry);

            // Replicate to followers
            self.replicate_to_followers().await?;
            Ok(())
        } else {
            Err("Not leader".into())
        }
    }

    async fn replicate_to_followers(&self) -> Result<()> {
        for peer in &self.peers {
            // Send log entries to peer
            replicate_log(peer).await?;
        }
        Ok(())
    }
}
```

## Failure Scenarios

### Network Partition

```rust
pub async fn handle_partition() -> Result<()> {
    // Detect partition
    if is_partitioned().await {
        // Go read-only or fail
        enter_readonly_mode().await?;

        // Detect recovery
        wait_for_recovery().await?;

        // Resync
        resync_state().await?;
    }

    Ok(())
}
```

### Cascading Failure

```rust
pub async fn protect_against_cascade() -> Result<()> {
    // Use circuit breaker
    let cb = CircuitBreaker::new(5, 2, Duration::from_secs(60));

    // Limit parallelism
    let bulkhead = Bulkhead::new(10);

    // Set timeout
    let timeout = Duration::from_secs(5);

    bulkhead.execute(|| {
        Box::pin(async {
            cb.execute(|| {
                Box::pin(with_timeout(
                    || Box::pin(call_service()),
                    timeout
                ))
            }).await
        })
    }).await
}
```

## Complete Distributed Example

```rust
pub struct DistributedWorkflow {
    registry: ServiceRegistry,
    tracer: Tracer,
}

impl DistributedWorkflow {
    pub async fn process_order(&self, order_id: &str) -> Result<()> {
        let trace = Tracer::start(order_id);

        // Step 1: Inventory Service
        let inventory = self.registry.get_endpoint("inventory").await?;
        let inv_result = self.traced_call(
            &trace,
            "inventory",
            "reserve",
            order_id.as_bytes().to_vec()
        ).await?;

        // Step 2: Payment Service
        let payment = self.registry.get_endpoint("payment").await?;
        let pay_result = self.traced_call(
            &trace,
            "payment",
            "process",
            order_id.as_bytes().to_vec()
        ).await?;

        // Step 3: Shipping Service
        let shipping = self.registry.get_endpoint("shipping").await?;
        let ship_result = self.traced_call(
            &trace,
            "shipping",
            "schedule",
            order_id.as_bytes().to_vec()
        ).await?;

        info!(
            trace_id = %trace.trace_id,
            "Order processed successfully"
        );

        Ok(())
    }

    async fn traced_call(
        &self,
        trace: &TraceContext,
        service: &str,
        method: &str,
        data: Vec<u8>
    ) -> Result<Vec<u8>> {
        let endpoint = self.registry.get_endpoint(service).await?;

        self.tracer.span(service, method, || {
            Box::pin(async {
                let url = format!("http://{}:{}/{}", endpoint.host, endpoint.port, method);
                // Call endpoint
                Ok(vec![])
            })
        }).await
    }
}
```

---

**Category**: How-to Guides (Task-oriented)
**Difficulty**: Advanced
**Related**: Error Handling, Workflow Patterns
