// knhk-unrdf: FFI exports for C integration
// C-compatible function exports for FFI

use crate::error::UnrdfError;
use crate::hooks::{deregister_hook, execute_hook, list_hooks, register_hook};
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
    let path = unsafe {
        CStr::from_ptr(unrdf_path)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid path".to_string()))
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
    let data = unsafe {
        CStr::from_ptr(turtle_data)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid turtle data".to_string()))
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
    let q = unsafe {
        CStr::from_ptr(query)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid query".to_string()))
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

#[no_mangle]
pub extern "C" fn knhk_unrdf_execute_hook(
    hook_name: *const c_char,
    hook_query: *const c_char,
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    let name = unsafe {
        CStr::from_ptr(hook_name)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook name".to_string()))
    };
    
    let query = unsafe {
        CStr::from_ptr(hook_query)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook query".to_string()))
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

