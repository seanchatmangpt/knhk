// knhk-unrdf: FFI exports for C integration
// C-compatible function exports for FFI

use crate::error::UnrdfError;
use crate::hooks::{deregister_hook, execute_hook, execute_hook_with_data, list_hooks, register_hook};
use crate::query::query_sparql;
use crate::serialize::serialize_rdf;
use crate::shacl::validate_shacl;
use crate::store::store_turtle_data;
use crate::transaction::{begin_transaction, commit_transaction, rollback_transaction, transaction_add, transaction_remove};
use crate::types::RdfFormat;
use crate::{init_unrdf};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

// Initialization FFI

#[no_mangle]
pub extern "C" fn knhk_unrdf_init(unrdf_path: *const c_char) -> c_int {
    // Validate NULL pointer
    if unrdf_path.is_null() {
        return -1;
    }

    let path = unsafe {
        CStr::from_ptr(unrdf_path)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid path encoding".to_string()))
    };
    
    match path {
        Ok(p) => {
            match init_unrdf(p) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_store_turtle(turtle_data: *const c_char) -> c_int {
    // Validate NULL pointer
    if turtle_data.is_null() {
        return -1;
    }

    let data = unsafe {
        CStr::from_ptr(turtle_data)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid turtle data encoding".to_string()))
    };
    
    match data {
        Ok(d) => {
            match store_turtle_data(d) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_query(query: *const c_char, result_json: *mut c_char, result_size: usize) -> c_int {
    // Validate NULL pointers
    if query.is_null() || result_json.is_null() {
        return -1;
    }

    let q = unsafe {
        CStr::from_ptr(query)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid query encoding".to_string()))
    };
    
    match q {
        Ok(query_str) => {
            match query_sparql(query_str) {
                Ok(result) => {
                    match serde_json::to_string(&result) {
                        Ok(json) => {
                            let json_bytes = json.as_bytes();
                            if json_bytes.len() < result_size {
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        json_bytes.as_ptr(),
                                        result_json as *mut u8,
                                        json_bytes.len()
                                    );
                                    *result_json.add(json_bytes.len()) = 0;
                                }
                                0
                            } else {
                                -1
                            }
                        }
                        Err(_) => -1,
                    }
                }
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

/// Execute SPARQL query with data to store first (for stateful operations)
#[no_mangle]
pub extern "C" fn knhk_unrdf_query_with_data(
    query: *const c_char,
    turtle_data: *const c_char,
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    // Validate NULL pointers
    if query.is_null() || result_json.is_null() {
        return -1;
    }
    
    // If turtle_data is provided, we need to combine store + query in a single script
    // because each script creates a new system instance
    if !turtle_data.is_null() {
        // Use the query module's function that handles both store and query
        use crate::query::query_sparql_with_data;
        
        let q = unsafe {
            CStr::from_ptr(query)
                .to_str()
                .map_err(|_| UnrdfError::InvalidInput("Invalid query encoding".to_string()))
        };
        
        let data = unsafe {
            CStr::from_ptr(turtle_data)
                .to_str()
                .map_err(|_| UnrdfError::InvalidInput("Invalid turtle data encoding".to_string()))
        };
        
        match (q, data) {
            (Ok(query_str), Ok(data_str)) => {
                match query_sparql_with_data(query_str, data_str) {
                    Ok(result) => {
                        match serde_json::to_string(&result) {
                            Ok(json) => {
                                let json_bytes = json.as_bytes();
                                if json_bytes.len() < result_size {
                                    unsafe {
                                        std::ptr::copy_nonoverlapping(
                                            json_bytes.as_ptr(),
                                            result_json as *mut u8,
                                            json_bytes.len()
                                        );
                                        *result_json.add(json_bytes.len()) = 0;
                                    }
                                    0
                                } else {
                                    // Buffer too small - copy error message
                                    let error_msg = format!(r#"{{"success":false,"error":"Result too large: {} bytes, buffer: {} bytes"}}"#, json_bytes.len(), result_size);
                                    let error_bytes = error_msg.as_bytes();
                                    let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                                    unsafe {
                                        std::ptr::copy_nonoverlapping(
                                            error_bytes.as_ptr(),
                                            result_json as *mut u8,
                                            copy_len
                                        );
                                        *result_json.add(copy_len) = 0;
                                    }
                                    -2
                                }
                            }
                            Err(e) => {
                                // Serialization error - copy error message
                                let error_msg = format!(r#"{{"success":false,"error":"JSON serialization failed: {}"}}"#, e);
                                let error_bytes = error_msg.as_bytes();
                                let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        error_bytes.as_ptr(),
                                        result_json as *mut u8,
                                        copy_len
                                    );
                                    *result_json.add(copy_len) = 0;
                                }
                                -7
                            }
                        }
                    }
                    Err(e) => {
                        // Query execution error - copy error message
                        let error_msg = format!(r#"{{"success":false,"error":"{}"}}"#, e);
                        let error_bytes = error_msg.as_bytes();
                        let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                error_bytes.as_ptr(),
                                result_json as *mut u8,
                                copy_len
                            );
                            *result_json.add(copy_len) = 0;
                        }
                        -2
                    }
                }
            }
            (Err(e1), _) | (_, Err(e2)) => {
                // String conversion error
                let error = if let Err(e) = q { e } else { data.unwrap_err() };
                let error_msg = format!(r#"{{"success":false,"error":"Invalid input encoding: {}"}}"#, error);
                let error_bytes = error_msg.as_bytes();
                let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        error_bytes.as_ptr(),
                        result_json as *mut u8,
                        copy_len
                    );
                    *result_json.add(copy_len) = 0;
                }
                -8
            }
        }
    } else {
        // No data to store, just execute query
        knhk_unrdf_query(query, result_json, result_size)
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_execute_hook(
    hook_name: *const c_char,
    hook_query: *const c_char,
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    // Validate NULL pointers
    if hook_name.is_null() || hook_query.is_null() || result_json.is_null() {
        return -1;
    }
    
    let name = unsafe {
        CStr::from_ptr(hook_name)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook name encoding".to_string()))
    };
    
    let query = unsafe {
        CStr::from_ptr(hook_query)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook query encoding".to_string()))
    };
    
    match (name, query) {
        (Ok(n), Ok(q)) => {
            match execute_hook(n, q) {
                Ok(result) => {
                    match serde_json::to_string(&result) {
                        Ok(json) => {
                            let json_bytes = json.as_bytes();
                            if json_bytes.len() < result_size {
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        json_bytes.as_ptr(),
                                        result_json as *mut u8,
                                        json_bytes.len()
                                    );
                                    *result_json.add(json_bytes.len()) = 0;
                                }
                                0
                            } else {
                                -1
                            }
                        }
                        Err(_) => -1,
                    }
                }
                Err(_) => -1,
            }
        }
        _ => -1,
    }
}

/// Execute knowledge hook with data to store first (for stateful operations)
#[no_mangle]
pub extern "C" fn knhk_unrdf_execute_hook_with_data(
    hook_name: *const c_char,
    hook_query: *const c_char,
    turtle_data: *const c_char,
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    // Validate NULL pointers
    if hook_name.is_null() || hook_query.is_null() || result_json.is_null() {
        return -1;
    }
    
    let name = unsafe {
        CStr::from_ptr(hook_name)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook name encoding".to_string()))
    };
    
    let query = unsafe {
        CStr::from_ptr(hook_query)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook query encoding".to_string()))
    };
    
    // If turtle_data is provided, use execute_hook_with_data
    if !turtle_data.is_null() {
        let data = unsafe {
            CStr::from_ptr(turtle_data)
                .to_str()
                .map_err(|_| UnrdfError::InvalidInput("Invalid turtle data encoding".to_string()))
        };
        
        match (name, query, data) {
            (Ok(n), Ok(q), Ok(d)) => {
                match execute_hook_with_data(n, q, d) {
                    Ok(result) => {
                        match serde_json::to_string(&result) {
                            Ok(json) => {
                                let json_bytes = json.as_bytes();
                                if json_bytes.len() < result_size {
                                    unsafe {
                                        std::ptr::copy_nonoverlapping(
                                            json_bytes.as_ptr(),
                                            result_json as *mut u8,
                                            json_bytes.len()
                                        );
                                        *result_json.add(json_bytes.len()) = 0;
                                    }
                                    0
                                } else {
                                    let error_msg = format!(r#"{{"success":false,"error":"Result too large: {} bytes, buffer: {} bytes"}}"#, json_bytes.len(), result_size);
                                    let error_bytes = error_msg.as_bytes();
                                    let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                                    unsafe {
                                        std::ptr::copy_nonoverlapping(
                                            error_bytes.as_ptr(),
                                            result_json as *mut u8,
                                            copy_len
                                        );
                                        *result_json.add(copy_len) = 0;
                                    }
                                    -7
                                }
                            }
                            Err(e) => {
                                let error_msg = format!(r#"{{"success":false,"error":"JSON serialization failed: {}"}}"#, e);
                                let error_bytes = error_msg.as_bytes();
                                let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        error_bytes.as_ptr(),
                                        result_json as *mut u8,
                                        copy_len
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
                                copy_len
                            );
                            *result_json.add(copy_len) = 0;
                        }
                        -4
                    }
                }
            }
            _ => {
                let error_msg = r#"{"success":false,"error":"Invalid input encoding"}"#;
                let error_bytes = error_msg.as_bytes();
                let copy_len = std::cmp::min(error_bytes.len(), result_size.saturating_sub(1));
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        error_bytes.as_ptr(),
                        result_json as *mut u8,
                        copy_len
                    );
                    *result_json.add(copy_len) = 0;
                }
                -8
            }
        }
    } else {
        // No data to store, just execute hook
        match (name, query) {
            (Ok(n), Ok(q)) => {
                match execute_hook(n, q) {
                    Ok(result) => {
                        match serde_json::to_string(&result) {
                            Ok(json) => {
                                let json_bytes = json.as_bytes();
                                if json_bytes.len() < result_size {
                                    unsafe {
                                        std::ptr::copy_nonoverlapping(
                                            json_bytes.as_ptr(),
                                            result_json as *mut u8,
                                            json_bytes.len()
                                        );
                                        *result_json.add(json_bytes.len()) = 0;
                                    }
                                    0
                                } else {
                                    -1
                                }
                            }
                            Err(_) => -1,
                        }
                    }
                    Err(_) => -1,
                }
            }
            _ => -1,
        }
    }
}

// Transaction Management FFI

#[no_mangle]
pub extern "C" fn knhk_unrdf_transaction_begin(actor: *const c_char) -> c_int {
    let actor_str = unsafe {
        CStr::from_ptr(actor)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid actor".to_string()))
    };
    
    match actor_str {
        Ok(a) => {
            match begin_transaction(a) {
                Ok(id) => id as c_int,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_transaction_add(transaction_id: c_int, turtle_data: *const c_char) -> c_int {
    let data = unsafe {
        CStr::from_ptr(turtle_data)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid turtle data".to_string()))
    };
    
    match data {
        Ok(d) => {
            match transaction_add(transaction_id as u32, d) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_transaction_remove(transaction_id: c_int, turtle_data: *const c_char) -> c_int {
    let data = unsafe {
        CStr::from_ptr(turtle_data)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid turtle data".to_string()))
    };
    
    match data {
        Ok(d) => {
            match transaction_remove(transaction_id as u32, d) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_transaction_commit(
    transaction_id: c_int,
    receipt_json: *mut c_char,
    receipt_size: usize
) -> c_int {
    match commit_transaction(transaction_id as u32) {
        Ok(receipt) => {
            match serde_json::to_string(&receipt) {
                Ok(json) => {
                    let json_bytes = json.as_bytes();
                    if json_bytes.len() < receipt_size {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                json_bytes.as_ptr(),
                                receipt_json as *mut u8,
                                json_bytes.len()
                            );
                            *receipt_json.add(json_bytes.len()) = 0;
                        }
                        0
                    } else {
                        -1
                    }
                }
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_transaction_rollback(transaction_id: c_int) -> c_int {
    match rollback_transaction(transaction_id as u32) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

// SHACL Validation FFI

#[no_mangle]
pub extern "C" fn knhk_unrdf_validate_shacl(
    data_turtle: *const c_char,
    shapes_turtle: *const c_char,
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    let data = unsafe {
        CStr::from_ptr(data_turtle)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid data turtle".to_string()))
    };
    
    let shapes = unsafe {
        CStr::from_ptr(shapes_turtle)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid shapes turtle".to_string()))
    };
    
    match (data, shapes) {
        (Ok(d), Ok(s)) => {
            match validate_shacl(d, s) {
                Ok(result) => {
                    match serde_json::to_string(&result) {
                        Ok(json) => {
                            let json_bytes = json.as_bytes();
                            if json_bytes.len() < result_size {
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        json_bytes.as_ptr(),
                                        result_json as *mut u8,
                                        json_bytes.len()
                                    );
                                    *result_json.add(json_bytes.len()) = 0;
                                }
                                0
                            } else {
                                -1
                            }
                        }
                        Err(_) => -1,
                    }
                }
                Err(_) => -1,
            }
        }
        _ => -1,
    }
}

// RDF Serialization FFI

#[no_mangle]
pub extern "C" fn knhk_unrdf_to_turtle(output: *mut c_char, output_size: usize) -> c_int {
    match serialize_rdf(RdfFormat::Turtle) {
        Ok(result) => {
            let result_bytes = result.as_bytes();
            if result_bytes.len() < output_size {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        result_bytes.as_ptr(),
                        output as *mut u8,
                        result_bytes.len()
                    );
                    *output.add(result_bytes.len()) = 0;
                }
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_to_jsonld(output: *mut c_char, output_size: usize) -> c_int {
    match serialize_rdf(RdfFormat::JsonLd) {
        Ok(result) => {
            let result_bytes = result.as_bytes();
            if result_bytes.len() < output_size {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        result_bytes.as_ptr(),
                        output as *mut u8,
                        result_bytes.len()
                    );
                    *output.add(result_bytes.len()) = 0;
                }
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_to_nquads(output: *mut c_char, output_size: usize) -> c_int {
    match serialize_rdf(RdfFormat::NQuads) {
        Ok(result) => {
            let result_bytes = result.as_bytes();
            if result_bytes.len() < output_size {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        result_bytes.as_ptr(),
                        output as *mut u8,
                        result_bytes.len()
                    );
                    *output.add(result_bytes.len()) = 0;
                }
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

// Hook Management FFI

#[no_mangle]
pub extern "C" fn knhk_unrdf_register_hook(
    hook_json: *const c_char,
    hook_id: *mut c_char,
    id_size: usize
) -> c_int {
    let json_str = unsafe {
        CStr::from_ptr(hook_json)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook JSON".to_string()))
    };
    
    match json_str {
        Ok(json) => {
            match register_hook(json) {
                Ok(id) => {
                    let id_bytes = id.as_bytes();
                    if id_bytes.len() < id_size {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                id_bytes.as_ptr(),
                                hook_id as *mut u8,
                                id_bytes.len()
                            );
                            *hook_id.add(id_bytes.len()) = 0;
                        }
                        0
                    } else {
                        -1
                    }
                }
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_deregister_hook(hook_id: *const c_char) -> c_int {
    let id = unsafe {
        CStr::from_ptr(hook_id)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook ID".to_string()))
    };
    
    match id {
        Ok(i) => {
            match deregister_hook(i) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_list_hooks(hooks_json: *mut c_char, hooks_size: usize) -> c_int {
    match list_hooks() {
        Ok(hooks) => {
            match serde_json::to_string(&hooks) {
                Ok(json) => {
                    let json_bytes = json.as_bytes();
                    if json_bytes.len() < hooks_size {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                json_bytes.as_ptr(),
                                hooks_json as *mut u8,
                                json_bytes.len()
                            );
                            *hooks_json.add(json_bytes.len()) = 0;
                        }
                        0
                    } else {
                        -1
                    }
                }
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

