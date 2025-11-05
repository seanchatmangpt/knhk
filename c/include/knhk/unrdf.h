// knhk_unrdf.h
// FFI declarations for Rust unrdf integration layer
// This header provides C interface to Rust integration layer for unrdf

#ifndef KNHK_UNRDF_H
#define KNHK_UNRDF_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/// Initialize unrdf integration layer
/// @param unrdf_path Path to unrdf vendor directory
/// @return 0 on success, -1 on error
int knhk_unrdf_init(const char *unrdf_path);

/// Store Turtle data in unrdf via Rust integration layer
/// @param turtle_data Turtle-formatted RDF data
/// @return 0 on success, -1 on error
int knhk_unrdf_store_turtle(const char *turtle_data);

/// Execute SPARQL query via Rust → unrdf integration layer
/// Supports all query types: SELECT, ASK, CONSTRUCT, DESCRIBE, UPDATE
/// @param query SPARQL query string
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_query(const char *query, char *result_json, size_t result_size);

/// Execute knowledge hook via Rust → unrdf integration layer
/// @param hook_name Name of the hook
/// @param hook_query SPARQL ASK query for hook condition
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_execute_hook(const char *hook_name, const char *hook_query, char *result_json, size_t result_size);

/// Validate SHACL shapes against data graph
/// @param data_turtle Turtle-formatted data graph
/// @param shapes_turtle Turtle-formatted shapes graph
/// @param result_json Pre-allocated buffer for JSON validation result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_validate_shacl(const char *data_turtle, const char *shapes_turtle, char *result_json, size_t result_size);

/// Execute transaction with additions and removals
/// @param additions_turtle Turtle-formatted additions (can be empty string "")
/// @param removals_turtle Turtle-formatted removals (can be empty string "")
/// @param actor Actor identifier for the transaction
/// @param result_json Pre-allocated buffer for JSON transaction result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_execute_transaction(const char *additions_turtle, const char *removals_turtle, const char *actor, char *result_json, size_t result_size);

/// Serialize current store to Turtle format
/// @param result_json Pre-allocated buffer for JSON serialization result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_serialize_to_turtle(char *result_json, size_t result_size);

/// Serialize current store to JSON-LD format
/// @param result_json Pre-allocated buffer for JSON serialization result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_serialize_to_jsonld(char *result_json, size_t result_size);

/// Serialize current store to N-Quads format
/// @param result_json Pre-allocated buffer for JSON serialization result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_serialize_to_nquads(char *result_json, size_t result_size);

/// Register a hook with the system
/// @param hook_json JSON string defining the hook (see unrdf hook definition format)
/// @param hook_id Pre-allocated buffer for hook ID
/// @param hook_id_size Size of hook_id buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_register_hook(const char *hook_json, char *hook_id, size_t hook_id_size);

/// Deregister a hook from the system
/// @param hook_id Hook ID to deregister
/// @return 0 on success, -1 on error
int knhk_unrdf_deregister_hook(const char *hook_id);

/// List all registered hooks
/// @param result_json Pre-allocated buffer for JSON hook list result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_list_hooks(char *result_json, size_t result_size);

#ifdef __cplusplus
}
#endif

#endif // KNHK_UNRDF_H

