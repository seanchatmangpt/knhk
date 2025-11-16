//! # KNHK WebAssembly Bindings
//!
//! This crate provides WebAssembly bindings for the KNHK workflow engine,
//! enabling portable workflow execution across browsers, Node.js, Deno, and edge environments.
//!
//! ## Features
//!
//! - **Portable Execution**: Run workflows in any WASM-compatible environment
//! - **Zero-Copy Interop**: Efficient JavaScript/TypeScript integration
//! - **Streaming API**: Process large workflows without blocking
//! - **Type-Safe**: Full TypeScript definitions included
//! - **Size-Optimized**: < 500KB compressed for fast loading
//!
//! ## Example
//!
//! ```javascript
//! import init, { WasmWorkflowEngine } from './knhk_wasm';
//!
//! async function main() {
//!     await init();
//!     const engine = new WasmWorkflowEngine();
//!     const result = await engine.execute_workflow(workflowDef);
//!     console.log('Result:', result);
//! }
//! ```

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::RwLock;

mod runtime;
mod host_functions;
mod error;
mod state;
mod parser;

pub use error::{WasmError, WasmResult};
pub use runtime::WasmWorkflowRuntime;
pub use state::WasmStateStore;

// Set up panic hook for better error messages in WASM
#[cfg(feature = "console_error_panic_hook")]
pub use console_error_panic_hook::set_once as set_panic_hook;

/// Initialize the WASM module
///
/// This should be called once when the module is loaded.
#[wasm_bindgen(start)]
pub fn init_wasm() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // Initialize tracing for WASM
    tracing_wasm::set_as_global_default();

    // Log initialization
    web_sys::console::log_1(&"KNHK WASM module initialized".into());
}

/// Configuration for the WASM workflow engine
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmEngineConfig {
    /// Maximum number of concurrent workflows
    max_workflows: usize,
    /// Enable telemetry (logs to console)
    enable_telemetry: bool,
    /// Workflow execution timeout in milliseconds
    timeout_ms: u32,
    /// Enable strict validation
    strict_validation: bool,
}

#[wasm_bindgen]
impl WasmEngineConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(getter)]
    pub fn max_workflows(&self) -> usize {
        self.max_workflows
    }

    #[wasm_bindgen(setter)]
    pub fn set_max_workflows(&mut self, value: usize) {
        self.max_workflows = value;
    }

    #[wasm_bindgen(getter)]
    pub fn enable_telemetry(&self) -> bool {
        self.enable_telemetry
    }

    #[wasm_bindgen(setter)]
    pub fn set_enable_telemetry(&mut self, value: bool) {
        self.enable_telemetry = value;
    }

    #[wasm_bindgen(getter)]
    pub fn timeout_ms(&self) -> u32 {
        self.timeout_ms
    }

    #[wasm_bindgen(setter)]
    pub fn set_timeout_ms(&mut self, value: u32) {
        self.timeout_ms = value;
    }
}

impl Default for WasmEngineConfig {
    fn default() -> Self {
        Self {
            max_workflows: 100,
            enable_telemetry: true,
            timeout_ms: 30000, // 30 seconds
            strict_validation: true,
        }
    }
}

/// WebAssembly workflow engine
///
/// This is the main entry point for executing KNHK workflows in WASM environments.
#[wasm_bindgen]
pub struct WasmWorkflowEngine {
    runtime: Arc<RwLock<WasmWorkflowRuntime>>,
    config: WasmEngineConfig,
}

#[wasm_bindgen]
impl WasmWorkflowEngine {
    /// Create a new WASM workflow engine with default configuration
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmWorkflowEngine, JsValue> {
        Self::with_config(WasmEngineConfig::default())
    }

    /// Create a new WASM workflow engine with custom configuration
    #[wasm_bindgen]
    pub fn with_config(config: WasmEngineConfig) -> Result<WasmWorkflowEngine, JsValue> {
        let runtime = WasmWorkflowRuntime::new(config.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to initialize runtime: {}", e)))?;

        Ok(WasmWorkflowEngine {
            runtime: Arc::new(RwLock::new(runtime)),
            config,
        })
    }

    /// Execute a workflow from a Turtle/RDF definition
    ///
    /// # Arguments
    ///
    /// * `workflow_def` - The workflow definition in Turtle format
    /// * `input_data` - Input data as JSON string
    ///
    /// # Returns
    ///
    /// A Promise that resolves to the workflow execution result
    #[wasm_bindgen]
    pub async fn execute_workflow(
        &mut self,
        workflow_def: &str,
        input_data: JsValue,
    ) -> Result<JsValue, JsValue> {
        let start = instant::Instant::now();

        // Parse input data
        let input: serde_json::Value = serde_wasm_bindgen::from_value(input_data)
            .map_err(|e| JsValue::from_str(&format!("Invalid input data: {}", e)))?;

        // Execute workflow (unwrap is safe in WASM - single-threaded, no lock poisoning)
        let result = self.runtime.write().unwrap()
            .execute(workflow_def, input)
            .await
            .map_err(|e| JsValue::from_str(&format!("Workflow execution failed: {}", e)))?;

        let elapsed = start.elapsed();

        if self.config.enable_telemetry {
            web_sys::console::log_1(
                &format!("Workflow executed in {:?}", elapsed).into()
            );
        }

        // Convert result to JsValue
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }

    /// Execute a workflow from JSON specification
    #[wasm_bindgen]
    pub async fn execute_workflow_json(
        &mut self,
        workflow_json: &str,
        input_data: JsValue,
    ) -> Result<JsValue, JsValue> {
        let input: serde_json::Value = serde_wasm_bindgen::from_value(input_data)
            .map_err(|e| JsValue::from_str(&format!("Invalid input data: {}", e)))?;

        let result = self.runtime.write().unwrap()
            .execute_json(workflow_json, input)
            .await
            .map_err(|e| JsValue::from_str(&format!("Workflow execution failed: {}", e)))?;

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }

    /// Validate a workflow definition without executing it
    #[wasm_bindgen]
    pub fn validate_workflow(&self, workflow_def: &str) -> Result<bool, JsValue> {
        self.runtime.read().unwrap()
            .validate(workflow_def)
            .map_err(|e| JsValue::from_str(&format!("Validation failed: {}", e)))?;

        Ok(true)
    }

    /// Get the current engine statistics
    #[wasm_bindgen]
    pub fn get_stats(&self) -> Result<JsValue, JsValue> {
        let stats = self.runtime.read().unwrap().get_stats();

        serde_wasm_bindgen::to_value(&stats)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize stats: {}", e)))
    }

    /// Reset the engine state
    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.runtime.write().unwrap()
            .reset()
            .map_err(|e| JsValue::from_str(&format!("Reset failed: {}", e)))
    }

    /// Get the WASM module version
    #[wasm_bindgen]
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

// Note: WorkflowResult is not exposed via wasm_bindgen due to JsValue not being Copy
// Results are returned directly as JsValue from execute methods

/// Engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    /// Total workflows executed
    pub total_executed: u64,
    /// Currently running workflows
    pub running_workflows: usize,
    /// Failed workflows
    pub failed_workflows: u64,
    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,
    /// Memory usage (approximate, in bytes)
    pub memory_usage_bytes: usize,
}

// Re-export for wasm-bindgen
use instant;

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_engine_creation() {
        let engine = WasmWorkflowEngine::new();
        assert!(engine.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_config() {
        let mut config = WasmEngineConfig::new();
        config.set_max_workflows(50);
        assert_eq!(config.max_workflows(), 50);
    }
}
