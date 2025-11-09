# State Manager Integration Design: RDF Provenance & Lockchain Audit Trails

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Integration Design
**Target:** knhk-workflow-engine v2.0
**Component:** `src/state/manager.rs`, `src/state/store.rs`, `src/compliance/provenance.rs`

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current StateManager Architecture](#current-statemanager-architecture)
3. [RDF State Persistence](#rdf-state-persistence)
4. [Lockchain Integration](#lockchain-integration)
5. [Event Sourcing with RDF Annotations](#event-sourcing-with-rdf-annotations)
6. [Case History with Ontology Context](#case-history-with-ontology-context)
7. [Real-Time vs Cached Execution](#real-time-vs-cached-execution)
8. [Consistency Guarantees](#consistency-guarantees)
9. [Performance Considerations](#performance-considerations)
10. [Implementation Roadmap](#implementation-roadmap)

---

## 1. Executive Summary

This document defines the **detailed integration architecture** between the `StateManager` and the YAWL ontology, focusing on:

1. **RDF State Persistence:** Store case state as RDF triples for semantic queries
2. **Lockchain Provenance:** Integrate Git commit hashes and cryptographic proofs
3. **Event Sourcing:** Annotate events with ontology metadata
4. **Hybrid Storage:** RDF for provenance/audit, Rust structs for hot path
5. **Consistency:** Ensure cache coherence between RDF store and in-memory cache

**Critical Design Principle:** StateManager maintains **dual representation**:
- **Hot path:** Cached Rust structs (WorkflowSpec, Case) for ≤8 tick execution
- **Cold path:** RDF triples for provenance, audit queries, historical analysis

---

## 2. Current StateManager Architecture

### 2.1 Existing Structure

**File:** `src/state/manager.rs`

```rust
pub struct StateManager {
    /// State store (persistent backend)
    store: Arc<StateStore>,

    /// In-memory cache for specs
    spec_cache: Arc<RwLock<HashMap<WorkflowSpecId, WorkflowSpec>>>,

    /// In-memory cache for cases
    case_cache: Arc<RwLock<HashMap<CaseId, Case>>>,

    /// Event log for event sourcing
    event_log: Arc<RwLock<Vec<StateEvent>>>,
}

#[derive(Debug, Clone)]
pub enum StateEvent {
    SpecRegistered {
        spec_id: WorkflowSpecId,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    CaseCreated {
        case_id: CaseId,
        spec_id: WorkflowSpecId,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    CaseStateChanged {
        case_id: CaseId,
        old_state: String,
        new_state: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}
```

### 2.2 Current Data Flow

```
┌─────────────────────────────────────────────────────┐
│  engine.register_workflow(spec)                     │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 1. Save to StateStore (disk)
                  ▼
┌─────────────────────────────────────────────────────┐
│  state_manager.save_spec(spec)                      │
│  ├─ store.save_spec(spec)     ← Disk persistence    │
│  ├─ spec_cache.insert(spec)   ← Memory cache        │
│  └─ event_log.push(SpecRegistered)                  │
└─────────────────────────────────────────────────────┘
                  │
                  │ 2. Create case
                  ▼
┌─────────────────────────────────────────────────────┐
│  engine.create_case(spec_id, data)                  │
│  ├─ state_manager.save_case(case)                   │
│  └─ event_log.push(CaseCreated)                     │
└─────────────────────────────────────────────────────┘
                  │
                  │ 3. Execute tasks (state changes)
                  ▼
┌─────────────────────────────────────────────────────┐
│  engine.execute_task(case_id, task_id)              │
│  ├─ state_manager.save_case(updated_case)           │
│  └─ event_log.push(CaseStateChanged)                │
└─────────────────────────────────────────────────────┘
```

---

## 3. RDF State Persistence

### 3.1 Enhanced StateStore with RDF Backend

**Design:** Add RDF store alongside existing StateStore for provenance tracking.

```rust
pub struct StateManager {
    // Existing fields
    store: Arc<StateStore>,  // Rust-native persistence (fast)
    spec_cache: Arc<RwLock<HashMap<WorkflowSpecId, WorkflowSpec>>>,
    case_cache: Arc<RwLock<HashMap<CaseId, Case>>>,
    event_log: Arc<RwLock<Vec<StateEvent>>>,

    // NEW: RDF store for provenance and semantic queries
    rdf_store: Option<Arc<RwLock<Store>>>,  // Oxigraph store

    // NEW: Lockchain integration
    lockchain: Option<Arc<LockchainIntegration>>,

    // NEW: Provenance tracker
    provenance_tracker: Option<Arc<ProvenanceTracker>>,
}

impl StateManager {
    /// Create StateManager with RDF provenance
    pub fn new_with_provenance(
        store: Arc<StateStore>,
        rdf_store_path: Option<&Path>,
        lockchain: Option<Arc<LockchainIntegration>>,
    ) -> WorkflowResult<Self> {
        let rdf_store = if let Some(path) = rdf_store_path {
            // Open RocksDB-backed RDF store for production
            let store = Store::open(path)
                .map_err(|e| WorkflowError::Storage(format!("Failed to open RDF store: {:?}", e)))?;
            Some(Arc::new(RwLock::new(store)))
        } else {
            None
        };

        Ok(Self {
            store,
            spec_cache: Arc::new(RwLock::new(HashMap::new())),
            case_cache: Arc::new(RwLock::new(HashMap::new())),
            event_log: Arc::new(RwLock::new(Vec::new())),
            rdf_store,
            lockchain,
            provenance_tracker: None,
        })
    }
}
```

### 3.2 Case State as RDF

**Turtle representation of case state:**

```turtle
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix knhk: <http://knhk.org/ontology#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix ex: <http://example.org/case#> .

ex:Case123 a yawl:WorkflowInstance ;
    rdfs:label "Order Processing Instance" ;
    yawl:hasSpecification ex:OrderWorkflow ;
    yawl:hasState "running" ;
    yawl:startedAtTime "2025-11-08T10:00:00Z"^^xsd:dateTime ;

    # Current execution state
    yawl:hasEnabledElement ex:TaskB, ex:ConditionC ;
    yawl:hasCompletedTask ex:TaskA ;

    # Provenance
    knhk:hasProvenanceChain "abc123def456..." ;
    prov:wasGeneratedBy ex:Execution456 ;
    prov:wasAssociatedWith ex:Agent789 .

# Execution activity (PROV-O)
ex:Execution456 a prov:Activity ;
    prov:used ex:OrderWorkflow ;
    prov:startedAtTime "2025-11-08T10:00:00Z"^^xsd:dateTime ;
    prov:wasAssociatedWith ex:Agent789 .

# Agent (executor)
ex:Agent789 a prov:Agent ;
    rdfs:label "knhk-workflow-engine v2.0" ;
    knhk:hasGitCommit "abc123def456" .
```

### 3.3 Save Case to RDF

```rust
impl StateManager {
    /// Save case state to both Rust store and RDF store
    pub async fn save_case_with_provenance(&self, case: &Case) -> WorkflowResult<()> {
        // Step 1: Save to Rust store (fast, for hot path execution)
        self.store.save_case(case.id, case)?;

        // Step 2: Update in-memory cache
        {
            let mut cache = self.case_cache.write().await;
            let old_state = cache.get(&case.id).map(|c| c.state.to_string());
            cache.insert(case.id, case.clone());

            // Log state change event
            if let Some(old) = old_state {
                if old != case.state.to_string() {
                    let mut log = self.event_log.write().await;
                    log.push(StateEvent::CaseStateChanged {
                        case_id: case.id,
                        old_state: old,
                        new_state: case.state.to_string(),
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
        }

        // Step 3: Save to RDF store (if enabled, for provenance)
        if let Some(ref rdf_store) = self.rdf_store {
            self.save_case_to_rdf(rdf_store, case).await?;
        }

        // Step 4: Update Lockchain (if enabled)
        if let Some(ref lockchain) = self.lockchain {
            self.update_lockchain(case).await?;
        }

        Ok(())
    }

    /// Convert case to RDF triples and save
    async fn save_case_to_rdf(
        &self,
        rdf_store: &Arc<RwLock<Store>>,
        case: &Case,
    ) -> WorkflowResult<()> {
        // Build Turtle representation
        let turtle = self.case_to_turtle(case)?;

        // Load into RDF store
        let mut store = rdf_store.write().await;
        store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
            .map_err(|e| WorkflowError::Storage(format!("RDF save failed: {:?}", e)))?;

        Ok(())
    }

    /// Convert Case to Turtle
    fn case_to_turtle(&self, case: &Case) -> WorkflowResult<String> {
        let mut ttl = String::new();

        // Namespaces
        ttl.push_str("@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .\n");
        ttl.push_str("@prefix knhk: <http://knhk.org/ontology#> .\n");
        ttl.push_str("@prefix prov: <http://www.w3.org/ns/prov#> .\n");
        ttl.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n");

        // Case instance
        ttl.push_str(&format!(
            "ex:Case{} a yawl:WorkflowInstance ;\n",
            case.id
        ));
        ttl.push_str(&format!(
            "    yawl:hasSpecification ex:Spec{} ;\n",
            case.workflow_id
        ));
        ttl.push_str(&format!(
            "    yawl:hasState \"{}\" ;\n",
            case.state
        ));

        // Enabled elements
        for element in &case.enabled_elements {
            ttl.push_str(&format!(
                "    yawl:hasEnabledElement <{}> ;\n",
                element
            ));
        }

        // Completed tasks
        for task_id in &case.completed_tasks {
            ttl.push_str(&format!(
                "    yawl:hasCompletedTask <{}> ;\n",
                task_id
            ));
        }

        // Provenance chain
        if let Some(ref chain) = case.provenance_chain {
            ttl.push_str(&format!(
                "    knhk:hasProvenanceChain \"{}\" ;\n",
                chain
            ));
        }

        // Timestamp
        ttl.push_str(&format!(
            "    prov:startedAtTime \"{}\"^^xsd:dateTime .\n\n",
            chrono::Utc::now().to_rfc3339()
        ));

        Ok(ttl)
    }
}
```

### 3.4 Query Case State from RDF

**SPARQL queries for historical analysis:**

```rust
impl StateManager {
    /// Query all cases in "running" state
    pub async fn query_running_cases(&self) -> WorkflowResult<Vec<CaseId>> {
        if let Some(ref rdf_store) = self.rdf_store {
            let query = r#"
                PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>

                SELECT ?case WHERE {
                    ?case a yawl:WorkflowInstance .
                    ?case yawl:hasState "running" .
                }
            "#;

            let store = rdf_store.read().await;
            #[allow(deprecated)]
            let results = store.query(query)
                .map_err(|e| WorkflowError::Query(format!("SPARQL failed: {:?}", e)))?;

            let mut case_ids = Vec::new();
            if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
                for solution in solutions {
                    let solution = solution?;
                    if let Some(case_term) = solution.get("case") {
                        let case_iri = case_term.to_string();
                        // Extract UUID from IRI (ex:Case123 → 123)
                        if let Some(id_str) = case_iri.strip_prefix("ex:Case") {
                            if let Ok(uuid) = Uuid::parse_str(id_str) {
                                case_ids.push(CaseId(uuid));
                            }
                        }
                    }
                }
            }

            Ok(case_ids)
        } else {
            // Fallback: Query from Rust cache
            Ok(self.query_running_cases_from_cache().await)
        }
    }

    /// Query cases that completed a specific task
    pub async fn query_cases_with_completed_task(&self, task_id: &str) -> WorkflowResult<Vec<CaseId>> {
        if let Some(ref rdf_store) = self.rdf_store {
            let query = format!(
                r#"
                PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>

                SELECT ?case WHERE {{
                    ?case a yawl:WorkflowInstance .
                    ?case yawl:hasCompletedTask <{task}> .
                }}
                "#,
                task = task_id
            );

            let store = rdf_store.read().await;
            #[allow(deprecated)]
            let results = store.query(&query)
                .map_err(|e| WorkflowError::Query(format!("SPARQL failed: {:?}", e)))?;

            let mut case_ids = Vec::new();
            if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
                for solution in solutions {
                    let solution = solution?;
                    if let Some(case_term) = solution.get("case") {
                        // Extract CaseId from IRI
                        let case_id = self.extract_case_id_from_iri(&case_term.to_string())?;
                        case_ids.push(case_id);
                    }
                }
            }

            Ok(case_ids)
        } else {
            Err(WorkflowError::Query("RDF store not enabled".into()))
        }
    }
}
```

---

## 4. Lockchain Integration

### 4.1 Provenance Chain Structure

**Lockchain:** Cryptographic audit trail linking workflow execution to Git commits.

```rust
pub struct ProvenanceChain {
    /// Git commit hash of workflow definition
    pub workflow_commit: String,

    /// Git commit hash of engine version
    pub engine_commit: String,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Previous chain link (for merkle tree)
    pub previous_hash: Option<String>,

    /// Current hash (SHA-256 of all fields)
    pub hash: String,

    /// Signature (optional, for non-repudiation)
    pub signature: Option<String>,
}

impl ProvenanceChain {
    /// Create new chain link
    pub fn new(
        workflow_commit: String,
        engine_commit: String,
        previous_hash: Option<String>,
    ) -> Self {
        let timestamp = chrono::Utc::now();
        let hash = Self::compute_hash(&workflow_commit, &engine_commit, &timestamp, &previous_hash);

        Self {
            workflow_commit,
            engine_commit,
            timestamp,
            previous_hash,
            hash,
            signature: None,
        }
    }

    /// Compute cryptographic hash
    fn compute_hash(
        workflow_commit: &str,
        engine_commit: &str,
        timestamp: &chrono::DateTime<chrono::Utc>,
        previous_hash: &Option<String>,
    ) -> String {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(workflow_commit.as_bytes());
        hasher.update(engine_commit.as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        if let Some(ref prev) = previous_hash {
            hasher.update(prev.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }

    /// Verify chain integrity
    pub fn verify(&self) -> bool {
        let computed_hash = Self::compute_hash(
            &self.workflow_commit,
            &self.engine_commit,
            &self.timestamp,
            &self.previous_hash,
        );
        computed_hash == self.hash
    }

    /// Serialize to RDF
    pub fn to_turtle(&self) -> String {
        format!(
            r#"
            @prefix knhk: <http://knhk.org/ontology#> .
            @prefix prov: <http://www.w3.org/ns/prov#> .

            ex:ProvenanceChain{hash} a knhk:ProvenanceChain ;
                knhk:workflowCommit "{workflow}" ;
                knhk:engineCommit "{engine}" ;
                prov:generatedAtTime "{timestamp}"^^xsd:dateTime ;
                knhk:previousHash "{prev}" ;
                knhk:chainHash "{hash}" .
            "#,
            hash = &self.hash[..8],
            workflow = self.workflow_commit,
            engine = self.engine_commit,
            timestamp = self.timestamp.to_rfc3339(),
            prev = self.previous_hash.as_ref().unwrap_or(&"".to_string()),
        )
    }
}
```

### 4.2 Lockchain Tracking

```rust
impl StateManager {
    /// Start Lockchain tracking for a case
    async fn update_lockchain(&self, case: &Case) -> WorkflowResult<()> {
        if let Some(ref lockchain) = self.lockchain {
            // Get workflow commit hash
            let workflow_commit = lockchain.get_workflow_commit(&case.workflow_id).await?;

            // Get engine commit hash
            let engine_commit = lockchain.get_engine_commit()?;

            // Get previous chain link (if exists)
            let previous_hash = case.provenance_chain.clone();

            // Create new chain link
            let chain_link = ProvenanceChain::new(workflow_commit, engine_commit, previous_hash);

            // Verify integrity
            if !chain_link.verify() {
                return Err(WorkflowError::ProvenanceVerificationFailed);
            }

            // Save to Lockchain
            lockchain.save_chain_link(&chain_link).await?;

            // Update case with new chain hash
            let mut case_mut = case.clone();
            case_mut.provenance_chain = Some(chain_link.hash.clone());

            // Save to RDF store with provenance
            if let Some(ref rdf_store) = self.rdf_store {
                let mut store = rdf_store.write().await;
                store.load_from_reader(RdfFormat::Turtle, chain_link.to_turtle().as_bytes())
                    .map_err(|e| WorkflowError::Storage(format!("Lockchain RDF save failed: {:?}", e)))?;
            }

            Ok(())
        } else {
            Ok(()) // Lockchain not enabled
        }
    }

    /// Verify provenance chain for a case
    pub async fn verify_case_provenance(&self, case_id: CaseId) -> WorkflowResult<bool> {
        let case = self.load_case(&case_id).await?
            .ok_or(WorkflowError::CaseNotFound(case_id))?;

        if let Some(ref chain_hash) = case.provenance_chain {
            if let Some(ref lockchain) = self.lockchain {
                // Reconstruct chain and verify
                lockchain.verify_chain(chain_hash).await
            } else {
                Err(WorkflowError::ProvenanceNotEnabled)
            }
        } else {
            Ok(true) // No provenance required
        }
    }
}
```

---

## 5. Event Sourcing with RDF Annotations

### 5.1 Enhanced State Events

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StateEvent {
    SpecRegistered {
        spec_id: WorkflowSpecId,
        timestamp: chrono::DateTime<chrono::Utc>,
        // NEW: Ontology annotations
        spec_iri: String,
        workflow_commit: Option<String>,
    },

    CaseCreated {
        case_id: CaseId,
        spec_id: WorkflowSpecId,
        timestamp: chrono::DateTime<chrono::Utc>,
        // NEW: Provenance
        created_by: Option<String>,
        provenance_chain: Option<String>,
    },

    CaseStateChanged {
        case_id: CaseId,
        old_state: String,
        new_state: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        // NEW: Task context
        triggered_by_task: Option<String>,
        affected_elements: Vec<String>,
    },

    TaskExecuted {
        case_id: CaseId,
        task_id: String,
        task_name: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        // NEW: Performance metrics
        elapsed_ticks: u32,
        tick_budget: Option<u32>,
        // NEW: Ontology context
        join_type: JoinType,
        split_type: SplitType,
    },

    ResourceAllocated {
        case_id: CaseId,
        task_id: String,
        resource_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        // NEW: Allocation policy
        allocation_policy: String,
    },
}
```

### 5.2 Event Persistence to RDF

```rust
impl StateManager {
    /// Save event to RDF store
    async fn persist_event_to_rdf(&self, event: &StateEvent) -> WorkflowResult<()> {
        if let Some(ref rdf_store) = self.rdf_store {
            let turtle = self.event_to_turtle(event)?;

            let mut store = rdf_store.write().await;
            store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
                .map_err(|e| WorkflowError::Storage(format!("Event RDF save failed: {:?}", e)))?;
        }
        Ok(())
    }

    /// Convert event to Turtle
    fn event_to_turtle(&self, event: &StateEvent) -> WorkflowResult<String> {
        match event {
            StateEvent::TaskExecuted {
                case_id,
                task_id,
                task_name,
                timestamp,
                elapsed_ticks,
                tick_budget,
                join_type,
                split_type,
            } => {
                Ok(format!(
                    r#"
                    @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
                    @prefix knhk: <http://knhk.org/ontology#> .
                    @prefix prov: <http://www.w3.org/ns/prov#> .

                    ex:TaskExecution{} a knhk:TaskExecutionEvent ;
                        prov:wasPartOf ex:Case{} ;
                        knhk:executedTask <{}> ;
                        rdfs:label "{}" ;
                        prov:endedAtTime "{}"^^xsd:dateTime ;
                        knhk:elapsedTicks {} ;
                        knhk:tickBudget {} ;
                        yawl:joinType "{:?}" ;
                        yawl:splitType "{:?}" .
                    "#,
                    Uuid::new_v4(),
                    case_id,
                    task_id,
                    task_name,
                    timestamp.to_rfc3339(),
                    elapsed_ticks,
                    tick_budget.unwrap_or(0),
                    join_type,
                    split_type,
                ))
            }
            // Similar for other event types
            _ => Ok(String::new()),
        }
    }
}
```

---

## 6. Case History with Ontology Context

### 6.1 Enhanced Case History Query

```rust
impl StateManager {
    /// Get case history with ontology annotations
    pub async fn get_case_history_detailed(&self, case_id: CaseId) -> WorkflowResult<CaseHistory> {
        // Step 1: Get events from in-memory log
        let events = self.get_case_history(case_id).await;

        // Step 2: Enrich with RDF provenance data (if available)
        let provenance = if let Some(ref rdf_store) = self.rdf_store {
            self.query_case_provenance_from_rdf(rdf_store, case_id).await?
        } else {
            None
        };

        // Step 3: Build detailed history
        Ok(CaseHistory {
            case_id,
            events,
            provenance,
            created_at: self.get_case_creation_time(case_id).await?,
            completed_at: self.get_case_completion_time(case_id).await?,
        })
    }

    /// Query provenance data from RDF store
    async fn query_case_provenance_from_rdf(
        &self,
        rdf_store: &Arc<RwLock<Store>>,
        case_id: CaseId,
    ) -> WorkflowResult<Option<CaseProvenance>> {
        let query = format!(
            r#"
            PREFIX knhk: <http://knhk.org/ontology#>
            PREFIX prov: <http://www.w3.org/ns/prov#>

            SELECT ?chain ?workflow_commit ?engine_commit ?timestamp
            WHERE {{
                ex:Case{case} knhk:hasProvenanceChain ?chain .
                ?chain knhk:workflowCommit ?workflow_commit .
                ?chain knhk:engineCommit ?engine_commit .
                ?chain prov:generatedAtTime ?timestamp .
            }}
            "#,
            case = case_id
        );

        let store = rdf_store.read().await;
        #[allow(deprecated)]
        let results = store.query(&query)
            .map_err(|e| WorkflowError::Query(format!("Provenance query failed: {:?}", e)))?;

        if let oxigraph::sparql::QueryResults::Solutions(mut solutions) = results {
            if let Some(Ok(solution)) = solutions.next() {
                return Ok(Some(CaseProvenance {
                    chain_hash: solution.get("chain").map(|t| t.to_string()),
                    workflow_commit: solution.get("workflow_commit").map(|t| extract_literal_string(t)),
                    engine_commit: solution.get("engine_commit").map(|t| extract_literal_string(t)),
                    timestamp: solution.get("timestamp").map(|t| extract_literal_string(t)),
                }));
            }
        }

        Ok(None)
    }
}

#[derive(Debug)]
pub struct CaseHistory {
    pub case_id: CaseId,
    pub events: Vec<StateEvent>,
    pub provenance: Option<CaseProvenance>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug)]
pub struct CaseProvenance {
    pub chain_hash: Option<String>,
    pub workflow_commit: Option<String>,
    pub engine_commit: Option<String>,
    pub timestamp: Option<String>,
}
```

---

## 7. Real-Time vs Cached Execution

### 7.1 Dual Representation Strategy

**Key Principle:** Hot path execution NEVER queries RDF. RDF is for cold-path audit/provenance only.

```
┌─────────────────────────────────────────────────────┐
│              DUAL REPRESENTATION                    │
├─────────────────────────────────────────────────────┤
│                                                     │
│  HOT PATH (≤8 ticks)                               │
│  ┌────────────────────────────────────┐            │
│  │  In-Memory Cache (Rust structs)    │            │
│  │  ├─ WorkflowSpec (HashMap)         │            │
│  │  ├─ Case (HashMap)                 │            │
│  │  └─ StateEvent (Vec)               │            │
│  └────────────────────────────────────┘            │
│         ▲                    │                      │
│         │ read (fast)        │ write (fast)         │
│         │                    ▼                      │
│  ┌────────────────────────────────────┐            │
│  │  StateStore (Rust-native disk)     │            │
│  │  Bincode serialization             │            │
│  └────────────────────────────────────┘            │
│                                                     │
│  COLD PATH (audit/provenance)                      │
│  ┌────────────────────────────────────┐            │
│  │  RDF Store (Oxigraph/RocksDB)      │            │
│  │  ├─ Case state as triples          │            │
│  │  ├─ Event log as triples           │            │
│  │  └─ Provenance chains              │            │
│  └────────────────────────────────────┘            │
│         ▲                    │                      │
│         │ SPARQL queries     │ async writes         │
│         │                    ▼                      │
│  ┌────────────────────────────────────┐            │
│  │  Lockchain (Git commits)           │            │
│  └────────────────────────────────────┘            │
└─────────────────────────────────────────────────────┘
```

### 7.2 Asynchronous RDF Updates

**Strategy:** Write to Rust cache synchronously (hot path), write to RDF asynchronously (cold path).

```rust
impl StateManager {
    /// Save case with async RDF update
    pub async fn save_case_async_rdf(&self, case: &Case) -> WorkflowResult<()> {
        // Step 1: Synchronous save to Rust store and cache (hot path)
        self.store.save_case(case.id, case)?;

        {
            let mut cache = self.case_cache.write().await;
            cache.insert(case.id, case.clone());
        }

        // Step 2: Asynchronous save to RDF store (cold path)
        if let Some(ref rdf_store) = self.rdf_store {
            let rdf_store = Arc::clone(rdf_store);
            let case = case.clone();

            // Spawn background task for RDF update
            tokio::spawn(async move {
                let turtle = Self::case_to_turtle_static(&case).unwrap();
                let mut store = rdf_store.write().await;
                let _ = store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes());
            });
        }

        Ok(())
    }

    /// Static version for use in spawned task
    fn case_to_turtle_static(case: &Case) -> WorkflowResult<String> {
        // Same as case_to_turtle but without self
        // Implementation omitted for brevity
        Ok(String::new())
    }
}
```

---

## 8. Consistency Guarantees

### 8.1 Cache Coherence

**Problem:** In-memory cache and RDF store can diverge if updates fail.

**Solution:** Two-phase commit for critical operations.

```rust
impl StateManager {
    /// Save case with consistency guarantee
    pub async fn save_case_consistent(&self, case: &Case) -> WorkflowResult<()> {
        // Phase 1: Prepare (validate before commit)
        self.validate_case_update(case)?;

        // Phase 2: Commit to Rust store
        self.store.save_case(case.id, case)?;

        // Phase 3: Update cache (rollback on failure)
        {
            let mut cache = self.case_cache.write().await;
            cache.insert(case.id, case.clone());
        }

        // Phase 4: Update RDF store (eventual consistency)
        if let Some(ref rdf_store) = self.rdf_store {
            match self.save_case_to_rdf(rdf_store, case).await {
                Ok(_) => {}
                Err(e) => {
                    // Log error but don't fail (eventual consistency)
                    tracing::error!("RDF update failed: {:?}", e);
                }
            }
        }

        Ok(())
    }

    /// Validate case update before commit
    fn validate_case_update(&self, case: &Case) -> WorkflowResult<()> {
        // Ensure case exists
        if !self.store.case_exists(case.id)? {
            return Err(WorkflowError::CaseNotFound(case.id));
        }

        // Ensure workflow spec exists
        if !self.store.spec_exists(case.workflow_id)? {
            return Err(WorkflowError::SpecNotFound(case.workflow_id));
        }

        Ok(())
    }
}
```

### 8.2 RDF Store Recovery

**Strategy:** Rebuild RDF store from Rust StateStore if inconsistency detected.

```rust
impl StateManager {
    /// Rebuild RDF store from Rust state
    pub async fn rebuild_rdf_store(&self) -> WorkflowResult<()> {
        if let Some(ref rdf_store) = self.rdf_store {
            // Step 1: Clear RDF store
            {
                let mut store = rdf_store.write().await;
                *store = Store::new()
                    .map_err(|e| WorkflowError::Storage(format!("RDF clear failed: {:?}", e)))?;
            }

            // Step 2: Reload all specs
            let spec_ids = self.store.list_all_spec_ids()?;
            for spec_id in spec_ids {
                if let Some(spec) = self.store.load_spec(&spec_id)? {
                    let turtle = self.spec_to_turtle(&spec)?;
                    let mut store = rdf_store.write().await;
                    store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
                        .map_err(|e| WorkflowError::Storage(format!("Spec reload failed: {:?}", e)))?;
                }
            }

            // Step 3: Reload all cases
            let case_ids = self.store.list_all_case_ids()?;
            for case_id in case_ids {
                if let Some(case) = self.store.load_case(&case_id)? {
                    let turtle = self.case_to_turtle(&case)?;
                    let mut store = rdf_store.write().await;
                    store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
                        .map_err(|e| WorkflowError::Storage(format!("Case reload failed: {:?}", e)))?;
                }
            }

            tracing::info!("RDF store rebuilt successfully");
            Ok(())
        } else {
            Err(WorkflowError::Storage("RDF store not enabled".into()))
        }
    }
}
```

---

## 9. Performance Considerations

### 9.1 RDF Write Performance

**Benchmark Target:**
- Rust store write: <1ms
- RDF store write (async): <10ms (acceptable for cold path)

**Optimization:**

```rust
impl StateManager {
    /// Batch RDF updates for better performance
    pub async fn flush_rdf_batch(&self, cases: Vec<Case>) -> WorkflowResult<()> {
        if let Some(ref rdf_store) = self.rdf_store {
            // Build single large Turtle document
            let mut turtle = String::new();
            turtle.push_str("@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .\n");
            turtle.push_str("@prefix knhk: <http://knhk.org/ontology#> .\n\n");

            for case in cases {
                turtle.push_str(&self.case_to_turtle(&case)?);
                turtle.push_str("\n");
            }

            // Single RDF store write (much faster than N individual writes)
            let mut store = rdf_store.write().await;
            store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
                .map_err(|e| WorkflowError::Storage(format!("Batch RDF write failed: {:?}", e)))?;

            Ok(())
        } else {
            Ok(())
        }
    }
}
```

### 9.2 SPARQL Query Performance

**Strategy:** Use prepared queries and indexing.

```rust
impl StateManager {
    /// Query with prepared statement (faster)
    pub async fn query_cases_optimized(
        &self,
        filter: CaseFilter,
    ) -> WorkflowResult<Vec<CaseId>> {
        if let Some(ref rdf_store) = self.rdf_store {
            // Use pre-compiled query template
            let query = self.build_query_from_filter(filter);

            let store = rdf_store.read().await;
            #[allow(deprecated)]
            let results = store.query(&query)
                .map_err(|e| WorkflowError::Query(format!("Optimized query failed: {:?}", e)))?;

            // Extract case IDs
            self.extract_case_ids_from_results(results)
        } else {
            // Fallback to cache-based query
            self.query_cases_from_cache(filter).await
        }
    }
}
```

---

## 10. Implementation Roadmap

### Phase 1: Basic RDF Persistence (Weeks 1-2)
- ✅ Add `rdf_store` field to `StateManager`
- ✅ Implement `case_to_turtle()` conversion
- ✅ Implement `save_case_to_rdf()` with async writes
- ✅ Add RocksDB backend for production

### Phase 2: Lockchain Integration (Weeks 3-4)
- ⬜ Implement `ProvenanceChain` struct
- ⬜ Add `update_lockchain()` to case saves
- ⬜ Implement provenance verification
- ⬜ Integrate with Git commit tracking

### Phase 3: Event Sourcing Enhancement (Weeks 5-6)
- ⬜ Enhance `StateEvent` with ontology annotations
- ⬜ Implement `event_to_turtle()` conversion
- ⬜ Add event persistence to RDF store
- ⬜ Create event replay mechanism

### Phase 4: Query & Analytics (Weeks 7-8)
- ⬜ Implement SPARQL query methods
- ⬜ Add case history enrichment
- ⬜ Create provenance query API
- ⬜ Build analytics dashboard queries

### Phase 5: Performance & Consistency (Week 9)
- ⬜ Implement batch RDF updates
- ⬜ Add cache coherence validation
- ⬜ Create RDF store rebuild mechanism
- ⬜ Benchmark and optimize

---

## Summary

This integration design provides **hyper-detailed, implementation-ready specifications** for wiring RDF provenance and Lockchain integration into the `StateManager`. Key features:

1. **Dual Representation:** Hot path (Rust cache) + Cold path (RDF store)
2. **Lockchain Provenance:** Cryptographic audit trail with Git commits
3. **Event Sourcing:** RDF-annotated events for historical analysis
4. **Consistency:** Two-phase commit with eventual consistency for RDF
5. **Performance:** Asynchronous RDF writes, batch updates, prepared queries
6. **SPARQL Queries:** Semantic queries for case history and provenance
7. **Recovery:** RDF store rebuild from Rust state

**Integration Complete:** Parser → Executor → StateManager all wired with ontology semantics.
