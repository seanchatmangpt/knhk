//! Host function interfaces for WASM
//!
//! This module defines the interface between WASM and the host environment,
//! allowing workflows to interact with external systems through callbacks.

use wasm_bindgen::prelude::*;
use serde_json::Value as JsonValue;

/// Callback function type for logging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn warn(s: &str);
}

/// Host function registry for custom callbacks
#[wasm_bindgen]
pub struct HostFunctionRegistry {
    functions: std::collections::HashMap<String, js_sys::Function>,
}

#[wasm_bindgen]
impl HostFunctionRegistry {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            functions: std::collections::HashMap::new(),
        }
    }

    /// Register a host function
    #[wasm_bindgen]
    pub fn register(&mut self, name: String, func: js_sys::Function) {
        self.functions.insert(name, func);
    }

    /// Call a registered host function
    #[wasm_bindgen]
    pub async fn call(&self, name: &str, args: JsValue) -> Result<JsValue, JsValue> {
        let func = self.functions.get(name)
            .ok_or_else(|| JsValue::from_str(&format!("Host function not found: {}", name)))?;

        let this = JsValue::NULL;
        let result = func.call1(&this, &args)?;

        // If result is a Promise, await it
        if result.is_instance_of::<js_sys::Promise>() {
            let promise: js_sys::Promise = result.dyn_into()?;
            return wasm_bindgen_futures::JsFuture::from(promise).await;
        }

        Ok(result)
    }

    /// Check if a function is registered
    #[wasm_bindgen]
    pub fn has(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Unregister a host function
    #[wasm_bindgen]
    pub fn unregister(&mut self, name: &str) -> bool {
        self.functions.remove(name).is_some()
    }

    /// Get list of registered function names
    #[wasm_bindgen]
    pub fn list_functions(&self) -> Vec<JsValue> {
        self.functions.keys()
            .map(|k| JsValue::from_str(k))
            .collect()
    }
}

/// Sandboxing boundaries for WASM execution
pub struct SandboxLimits {
    /// Maximum memory usage in bytes
    pub max_memory: usize,
    /// Maximum execution time in milliseconds
    pub max_execution_time: u32,
    /// Maximum number of host function calls
    pub max_host_calls: usize,
    /// Allowed host functions
    pub allowed_functions: Vec<String>,
}

impl Default for SandboxLimits {
    fn default() -> Self {
        Self {
            max_memory: 100 * 1024 * 1024, // 100 MB
            max_execution_time: 30000, // 30 seconds
            max_host_calls: 1000,
            allowed_functions: vec![
                "log".to_string(),
                "error".to_string(),
                "warn".to_string(),
            ],
        }
    }
}

/// Sandbox context for tracking resource usage
pub struct SandboxContext {
    limits: SandboxLimits,
    host_calls: usize,
    start_time: instant::Instant,
}

impl SandboxContext {
    pub fn new(limits: SandboxLimits) -> Self {
        Self {
            limits,
            host_calls: 0,
            start_time: instant::Instant::now(),
        }
    }

    /// Check if function call is allowed
    pub fn check_function_allowed(&self, name: &str) -> bool {
        self.limits.allowed_functions.contains(&name.to_string())
    }

    /// Record a host function call
    pub fn record_host_call(&mut self) -> Result<(), String> {
        self.host_calls += 1;

        if self.host_calls > self.limits.max_host_calls {
            return Err(format!(
                "Maximum host calls exceeded: {} > {}",
                self.host_calls, self.limits.max_host_calls
            ));
        }

        Ok(())
    }

    /// Check if execution time limit is exceeded
    pub fn check_timeout(&self) -> Result<(), String> {
        let elapsed = self.start_time.elapsed().as_millis() as u32;

        if elapsed > self.limits.max_execution_time {
            return Err(format!(
                "Execution timeout: {}ms > {}ms",
                elapsed, self.limits.max_execution_time
            ));
        }

        Ok(())
    }
}

use instant;
