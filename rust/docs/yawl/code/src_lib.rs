mod reflex;
mod legacy;
mod error;

use knhk_admission::AdmissionGate;
use knhk_patterns::{PatternRegistry, RegisterAllExt, PatternId, PatternExecutionContext};
use knhk_state::{StateStore, WorkflowSpec, WorkflowSpecId, Case, CaseState};
use knhk_workitems::WorkItemService;
use knhk_timer::{TimerService, TimerFired};
use knhk_integration::{Fortune5Integration, Fortune5Config};
use knhk_sidecar::EventSidecar;

use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

pub use error::EngineError;

#[derive(Clone)]
pub struct EngineHandle { inner: Arc<Engine> }

struct Engine {
    registry: PatternRegistry,
    store: StateStore,
    admit: AdmissionGate,
    work: WorkItemService,
    timer: TimerService,
    f5: Option<Fortune5Integration>,
    events_rx: mpsc::Receiver<serde_json::Value>,
}

impl EngineHandle {
    pub async fn new(db_path: &str, f5: Option<Fortune5Config>) -> Result<(Self, EventSidecar), EngineError> {
        let store = StateStore::new(db_path)?;
        let mut registry = PatternRegistry::new();
        registry.register_all_patterns();

        let admit = AdmissionGate::new();
        let work = WorkItemService::new();

        // event bus channel
        let (evt_tx, evt_rx) = mpsc::channel::<serde_json::Value>(1024);
        let sidecar = EventSidecar::new(evt_tx);

        // timer -> engine loopback
        let (timer_tx, mut timer_rx) = mpsc::channel::<TimerFired>(1024);
        let timer = TimerService::new(timer_tx);

        let f5i = f5.map(Fortune5Integration::new);

        let inner = Arc::new(Engine { registry, store, admit, work, timer, f5: f5i, events_rx: evt_rx });
        let handle = Self { inner: inner.clone() };

        // pump timer fired → execute patterns 30/31
        let h = handle.clone();
        tokio::spawn(async move {
            while let Some(tf) = timer_rx.recv().await {
                let _ = h.execute_pattern(PatternId(tf.pattern_id), PatternExecutionContext {
                    case_id: tf.case_id, workflow_id: tf.workflow_id, variables: serde_json::json!({"key": tf.key})
                }).await;
            }
        });

        // pump external events → deferred choice (16) by default
        let h2 = handle.clone();
        tokio::spawn(async move {
            let mut rx = inner.events_rx;
            while let Some(evt) = rx.recv().await {
                let _ = h2.execute_pattern(PatternId(16), PatternExecutionContext {
                    case_id: evt.get("case_id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    workflow_id: evt.get("workflow_id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    variables: evt,
                }).await;
            }
        });

        Ok((handle, sidecar))
    }

    pub async fn register_workflow(&self, spec_json: serde_json::Value) -> Result<(), EngineError> {
        // legacy façade parsing (Turtle/RDF → JSON model) would sit before this
        let spec = WorkflowSpec {
            id: spec_json.get("id").and_then(|v| v.as_str()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string().as_str()).to_string(),
            name: spec_json.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed").to_string(),
            tasks: spec_json.get("tasks").cloned().unwrap_or_default(),
            conditions: spec_json.get("conditions").cloned().unwrap_or_default(),
            start_condition: spec_json.get("start_condition").and_then(|v| v.as_str()).map(|s| s.to_string()),
            end_condition: spec_json.get("end_condition").and_then(|v| v.as_str()).map(|s| s.to_string()),
        };
        self.inner.store.save_spec(&spec)?;
        Ok(())
    }

    pub async fn create_case(&self, spec_id: String, data: serde_json::Value) -> Result<String, EngineError> {
        self.inner.admit.admit(&data).map_err(|e| EngineError::Admission(format!("{e}")))?;
        let mut case = Case::new(spec_id, data);
        case.start();
        self.inner.store.save_case(&case)?;
        Ok(case.id)
    }

    pub async fn execute_pattern(&self, pid: PatternId, ctx: PatternExecutionContext) -> Result<(), EngineError> {
        if let Some(res) = self.inner.registry.execute(&pid, &ctx) {
            let r = res.map_err(|e| EngineError::Pattern(format!("{e}")))?;
            // persist case updates here; schedule next activities, etc.
            if let Some(f5) = &self.inner.f5 {
                f5.record_slo("pattern", 2_000).await;
            }
            // reflex bridge would promote pattern subgraphs to hot path here
            let _ = r; // placeholder
            Ok(())
        } else {
            Err(EngineError::NotFound(format!("pattern {}", pid.0)))
        }
    }
}