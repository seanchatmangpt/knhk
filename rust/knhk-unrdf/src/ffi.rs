// FFI exports for C integration

use crate::errors::UnrdfError;
use crate::hooks::{deregister_hook, list_hooks, register_hook};
use crate::query::{execute_hook, query_sparql, store_turtle_data};
use crate::serialization::{serialize_to_jsonld, serialize_to_nquads, serialize_to_turtle};
use crate::state::init_unrdf;
use crate::transaction::execute_transaction;
use crate::validation::validate_shacl;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

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

#[no_mangle]
pub extern "C" fn knhk_unrdf_execute_transaction(
    additions_turtle: *const c_char,
    removals_turtle: *const c_char,
    actor: *const c_char,
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    let additions = unsafe {
        CStr::from_ptr(additions_turtle)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid additions turtle".to_string()))
    };
    
    let removals = unsafe {
        CStr::from_ptr(removals_turtle)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid removals turtle".to_string()))
    };
    
    let actor_str = unsafe {
        CStr::from_ptr(actor)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid actor".to_string()))
    };
    
    match (additions, removals, actor_str) {
        (Ok(a), Ok(r), Ok(act)) => {
            match execute_transaction(a, r, act) {
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

#[no_mangle]
pub extern "C" fn knhk_unrdf_serialize_to_turtle(
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    match serialize_to_turtle() {
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

#[no_mangle]
pub extern "C" fn knhk_unrdf_serialize_to_jsonld(
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    match serialize_to_jsonld() {
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

#[no_mangle]
pub extern "C" fn knhk_unrdf_serialize_to_nquads(
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    match serialize_to_nquads() {
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

#[no_mangle]
pub extern "C" fn knhk_unrdf_register_hook(
    hook_json: *const c_char,
    hook_id: *mut c_char,
    hook_id_size: usize
) -> c_int {
    let hook = unsafe {
        CStr::from_ptr(hook_json)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook JSON".to_string()))
    };
    
    match hook {
        Ok(h) => {
            match register_hook(h) {
                Ok(id) => {
                    let id_bytes = id.as_bytes();
                    if id_bytes.len() < hook_id_size {
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
pub extern "C" fn knhk_unrdf_list_hooks(
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    match list_hooks() {
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

