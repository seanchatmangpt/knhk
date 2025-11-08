// knhk-unrdf: FFI exports for Rust-native hooks
// C-compatible function exports for native hook execution

#[cfg(feature = "native")]
use crate::hooks_native::{
    evaluate_hooks_batch_native, execute_hook_by_name_native, NativeHookRegistry,
};
#[cfg(feature = "native")]
use crate::types::HookDefinition;
#[cfg(feature = "native")]
use serde_json::Value as JsonValue;
#[cfg(feature = "native")]
use std::ffi::CStr;
#[cfg(feature = "native")]
use std::os::raw::{c_char, c_int};
#[cfg(feature = "native")]
use std::sync::{Mutex, OnceLock};

#[cfg(feature = "native")]
// Global hook registry singleton
static NATIVE_HOOK_REGISTRY: OnceLock<Mutex<NativeHookRegistry>> = OnceLock::new();

#[cfg(feature = "native")]
/// Get or initialize the native hook registry
fn get_native_hook_registry() -> &'static Mutex<NativeHookRegistry> {
    NATIVE_HOOK_REGISTRY.get_or_init(|| Mutex::new(NativeHookRegistry::new()))
}

#[cfg(feature = "native")]
/// Execute a hook by name (native Rust implementation)
/// Use case 1: Single hook execution
#[no_mangle]
pub unsafe extern "C" fn knhk_unrdf_execute_hook_native(
    hook_name: *const c_char,
    hook_query: *const c_char,
    turtle_data: *const c_char,
    result_json: *mut c_char,
    result_size: usize,
) -> c_int {
    // Validate NULL pointers
    if hook_name.is_null() || hook_query.is_null() || turtle_data.is_null() || result_json.is_null()
    {
        return -1;
    }

    let hook_name_str = unsafe {
        match CStr::from_ptr(hook_name).to_str() {
            Ok(s) => s,
            Err(_) => {
                let error_msg = r#"{"success":false,"error":"Invalid hook_name encoding"}"#;
                let error_bytes = error_msg.as_bytes();
                let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        error_bytes.as_ptr(),
                        result_json as *mut u8,
                        copy_len,
                    );
                    *result_json.add(copy_len) = 0;
                }
                return -1;
            }
        }
    };

    let hook_query_str = unsafe {
        match CStr::from_ptr(hook_query).to_str() {
            Ok(s) => s,
            Err(_) => {
                let error_msg = r#"{"success":false,"error":"Invalid hook_query encoding"}"#;
                let error_bytes = error_msg.as_bytes();
                let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        error_bytes.as_ptr(),
                        result_json as *mut u8,
                        copy_len,
                    );
                    *result_json.add(copy_len) = 0;
                }
                return -1;
            }
        }
    };

    let turtle_data_str = unsafe {
        match CStr::from_ptr(turtle_data).to_str() {
            Ok(s) => s,
            Err(_) => {
                let error_msg = r#"{"success":false,"error":"Invalid turtle_data encoding"}"#;
                let error_bytes = error_msg.as_bytes();
                let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        error_bytes.as_ptr(),
                        result_json as *mut u8,
                        copy_len,
                    );
                    *result_json.add(copy_len) = 0;
                }
                return -1;
            }
        }
    };

    match execute_hook_by_name_native(hook_name_str, hook_query_str, turtle_data_str) {
        Ok(result) => match serde_json::to_string(&result) {
            Ok(json) => {
                let json_bytes = json.as_bytes();
                if json_bytes.len() < result_size {
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            json_bytes.as_ptr(),
                            result_json as *mut u8,
                            json_bytes.len(),
                        );
                        *result_json.add(json_bytes.len()) = 0;
                    }
                    0
                } else {
                    let error_msg = format!(
                        r#"{{"success":false,"error":"Result too large: {} bytes, buffer: {} bytes"}}"#,
                        json_bytes.len(),
                        result_size
                    );
                    let error_bytes = error_msg.as_bytes();
                    let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            error_bytes.as_ptr(),
                            result_json as *mut u8,
                            copy_len,
                        );
                        *result_json.add(copy_len) = 0;
                    }
                    -2
                }
            }
            Err(e) => {
                let error_msg = format!(
                    r#"{{"success":false,"error":"JSON serialization failed: {}"}}"#,
                    e
                );
                let error_bytes = error_msg.as_bytes();
                let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        error_bytes.as_ptr(),
                        result_json as *mut u8,
                        copy_len,
                    );
                    *result_json.add(copy_len) = 0;
                }
                -7
            }
        },
        Err(e) => {
            let error_msg = format!(r#"{{"success":false,"error":"{}"}}"#, e);
            let error_bytes = error_msg.as_bytes();
            let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
            unsafe {
                std::ptr::copy_nonoverlapping(
                    error_bytes.as_ptr(),
                    result_json as *mut u8,
                    copy_len,
                );
                *result_json.add(copy_len) = 0;
            }
            -2
        }
    }
}

#[cfg(feature = "native")]
/// Execute multiple hooks in batch (native Rust implementation)
/// Use case 2: Batch hook evaluation for efficiency
#[no_mangle]
pub extern "C" fn knhk_unrdf_execute_hooks_batch_native(
    hooks_json: *const c_char,
    turtle_data: *const c_char,
    result_json: *mut c_char,
    result_size: usize,
) -> c_int {
    // Validate NULL pointers
    if hooks_json.is_null() || turtle_data.is_null() || result_json.is_null() {
        return -1;
    }

    let hooks_json_str = unsafe {
        match CStr::from_ptr(hooks_json).to_str() {
            Ok(s) => s,
            Err(_) => {
                let error_msg = r#"{"success":false,"error":"Invalid hooks_json encoding"}"#;
                let error_bytes = error_msg.as_bytes();
                let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        error_bytes.as_ptr(),
                        result_json as *mut u8,
                        copy_len,
                    );
                    *result_json.add(copy_len) = 0;
                }
                return -1;
            }
        }
    };

    let turtle_data_str = unsafe {
        match CStr::from_ptr(turtle_data).to_str() {
            Ok(s) => s,
            Err(_) => {
                let error_msg = r#"{"success":false,"error":"Invalid turtle_data encoding"}"#;
                let error_bytes = error_msg.as_bytes();
                let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        error_bytes.as_ptr(),
                        result_json as *mut u8,
                        copy_len,
                    );
                    *result_json.add(copy_len) = 0;
                }
                return -1;
            }
        }
    };

    // Parse hooks JSON array
    let hooks: Vec<HookDefinition> = match serde_json::from_str(hooks_json_str) {
        Ok(h) => h,
        Err(e) => {
            let error_msg = format!(r#"{{"success":false,"error":"Invalid hooks JSON: {}"}}"#, e);
            let error_bytes = error_msg.as_bytes();
            let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
            unsafe {
                std::ptr::copy_nonoverlapping(
                    error_bytes.as_ptr(),
                    result_json as *mut u8,
                    copy_len,
                );
                *result_json.add(copy_len) = 0;
            }
            return -1;
        }
    };

    match evaluate_hooks_batch_native(&hooks, turtle_data_str) {
        Ok(results) => {
            // Create result object with batch results
            let mut result_obj = serde_json::Map::new();
            result_obj.insert("success".to_string(), JsonValue::Bool(true));
            result_obj.insert("count".to_string(), JsonValue::Number(results.len().into()));
            let results_value = match serde_json::to_value(&results) {
                Ok(v) => v,
                Err(e) => {
                    let error_msg = format!(
                        r#"{{"success":false,"error":"Failed to serialize results: {}"}}"#,
                        e
                    );
                    let error_bytes = error_msg.as_bytes();
                    let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            error_bytes.as_ptr(),
                            result_json as *mut u8,
                            copy_len,
                        );
                        *result_json.add(copy_len) = 0;
                    }
                    return -7;
                }
            };
            result_obj.insert("results".to_string(), results_value);

            match serde_json::to_string(&JsonValue::Object(result_obj)) {
                Ok(json) => {
                    let json_bytes = json.as_bytes();
                    if json_bytes.len() < result_size {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                json_bytes.as_ptr(),
                                result_json as *mut u8,
                                json_bytes.len(),
                            );
                            *result_json.add(json_bytes.len()) = 0;
                        }
                        0
                    } else {
                        let error_msg = format!(
                            r#"{{"success":false,"error":"Result too large: {} bytes, buffer: {} bytes"}}"#,
                            json_bytes.len(),
                            result_size
                        );
                        let error_bytes = error_msg.as_bytes();
                        let copy_len =
                            std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                error_bytes.as_ptr(),
                                result_json as *mut u8,
                                copy_len,
                            );
                            *result_json.add(copy_len) = 0;
                        }
                        -2
                    }
                }
                Err(e) => {
                    let error_msg = format!(
                        r#"{{"success":false,"error":"JSON serialization failed: {}"}}"#,
                        e
                    );
                    let error_bytes = error_msg.as_bytes();
                    let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            error_bytes.as_ptr(),
                            result_json as *mut u8,
                            copy_len,
                        );
                        *result_json.add(copy_len) = 0;
                    }
                    -7
                }
            }
        }
        Err(e) => {
            let error_msg = format!(r#"{{"success":false,"error":"{}"}}"#, e);
            let error_bytes = error_msg.as_bytes();
            let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
            unsafe {
                std::ptr::copy_nonoverlapping(
                    error_bytes.as_ptr(),
                    result_json as *mut u8,
                    copy_len,
                );
                *result_json.add(copy_len) = 0;
            }
            -2
        }
    }
}

#[cfg(feature = "native")]
/// Register a hook in the native registry
#[no_mangle]
pub unsafe extern "C" fn knhk_unrdf_register_hook_native(hook_json: *const c_char) -> c_int {
    if hook_json.is_null() {
        return -1;
    }

    let hook_json_str = unsafe {
        match CStr::from_ptr(hook_json).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    let hook: HookDefinition = match serde_json::from_str(hook_json_str) {
        Ok(h) => h,
        Err(_) => return -1,
    };

    let registry = get_native_hook_registry();
    match registry.lock() {
        Ok(reg) => match reg.register(hook) {
            Ok(_) => 0,
            Err(_) => -1,
        },
        Err(_) => -1,
    }
}

#[cfg(feature = "native")]
/// Deregister a hook from the native registry
#[no_mangle]
pub extern "C" fn knhk_unrdf_deregister_hook_native(hook_id: *const c_char) -> c_int {
    if hook_id.is_null() {
        return -1;
    }

    let hook_id_str = unsafe {
        match CStr::from_ptr(hook_id).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    let registry = get_native_hook_registry();
    match registry.lock() {
        Ok(reg) => match reg.deregister(hook_id_str) {
            Ok(_) => 0,
            Err(_) => -1,
        },
        Err(_) => -1,
    }
}
