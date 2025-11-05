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
/// @param query SPARQL query string
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_query(const char *query, char *result_json, size_t result_size);

/// Execute SPARQL query with data to store first (for stateful operations)
/// @param query SPARQL query string
/// @param turtle_data Turtle data to store before querying (NULL if none)
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_query_with_data(const char *query, const char *turtle_data, char *result_json, size_t result_size);

/// Execute knowledge hook via Rust → unrdf integration layer
/// @param hook_name Name of the hook
/// @param hook_query SPARQL ASK query for hook condition
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_execute_hook(const char *hook_name, const char *hook_query, char *result_json, size_t result_size);

/// Execute knowledge hook with data to store first (for stateful operations)
/// @param hook_name Name of the hook
/// @param hook_query SPARQL ASK query for hook condition
/// @param turtle_data Turtle data to store before hook execution (NULL if none)
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_execute_hook_with_data(const char *hook_name, const char *hook_query, const char *turtle_data, char *result_json, size_t result_size);

// Transaction Management API

/// Begin a new transaction
/// @param actor Actor identifier for the transaction
/// @return Transaction ID on success, -1 on error
int knhk_unrdf_transaction_begin(const char *actor);

/// Add data to a transaction
/// @param transaction_id Transaction ID from begin_transaction
/// @param turtle_data Turtle-formatted RDF data to add
/// @return 0 on success, -1 on error
int knhk_unrdf_transaction_add(int transaction_id, const char *turtle_data);

/// Remove data from a transaction
/// @param transaction_id Transaction ID from begin_transaction
/// @param turtle_data Turtle-formatted RDF data to remove
/// @return 0 on success, -1 on error
int knhk_unrdf_transaction_remove(int transaction_id, const char *turtle_data);

/// Commit a transaction
/// @param transaction_id Transaction ID from begin_transaction
/// @param receipt_json Pre-allocated buffer for JSON receipt
/// @param receipt_size Size of receipt_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_transaction_commit(int transaction_id, char *receipt_json, size_t receipt_size);

/// Rollback a transaction
/// @param transaction_id Transaction ID from begin_transaction
/// @return 0 on success, -1 on error
int knhk_unrdf_transaction_rollback(int transaction_id);

// SHACL Validation API

/// Validate data graph against SHACL shapes graph
/// @param data_turtle Turtle-formatted data graph
/// @param shapes_turtle Turtle-formatted shapes graph
/// @param result_json Pre-allocated buffer for JSON validation result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_validate_shacl(const char *data_turtle, const char *shapes_turtle, char *result_json, size_t result_size);

// RDF Serialization API

/// Serialize unrdf store to Turtle format
/// @param output Pre-allocated buffer for Turtle output
/// @param output_size Size of output buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_to_turtle(char *output, size_t output_size);

/// Serialize unrdf store to JSON-LD format
/// @param output Pre-allocated buffer for JSON-LD output
/// @param output_size Size of output buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_to_jsonld(char *output, size_t output_size);

/// Serialize unrdf store to N-Quads format
/// @param output Pre-allocated buffer for N-Quads output
/// @param output_size Size of output buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_to_nquads(char *output, size_t output_size);

// Hook Management API

/// Register a hook with the system
/// @param hook_json JSON definition of the hook
/// @param hook_id Pre-allocated buffer for hook ID
/// @param id_size Size of hook_id buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_register_hook(const char *hook_json, char *hook_id, size_t id_size);

/// Deregister a hook
/// @param hook_id Hook ID from register_hook
/// @return 0 on success, -1 on error
int knhk_unrdf_deregister_hook(const char *hook_id);

/// List all registered hooks
/// @param hooks_json Pre-allocated buffer for JSON array of hooks
/// @param hooks_size Size of hooks_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_list_hooks(char *hooks_json, size_t hooks_size);

#ifdef __cplusplus
}
#endif

#endif // KNHK_UNRDF_H

