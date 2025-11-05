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

/// Execute SPARQL ASK query via Rust → unrdf integration layer
/// @param query SPARQL ASK query string
/// @param result Pointer to integer to store boolean result (1 = true, 0 = false)
/// @return 0 on success, -1 on error
int knhk_unrdf_query_ask(const char *query, int *result);

/// Execute SPARQL CONSTRUCT query via Rust → unrdf integration layer
/// @param query SPARQL CONSTRUCT query string
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_query_construct(const char *query, char *result_json, size_t result_size);

/// Execute SPARQL DESCRIBE query via Rust → unrdf integration layer
/// @param query SPARQL DESCRIBE query string
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_query_describe(const char *query, char *result_json, size_t result_size);

/// Execute SPARQL UPDATE query via Rust → unrdf integration layer
/// @param query SPARQL UPDATE query string
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_query_update(const char *query, char *result_json, size_t result_size);

/// Generate epistemology (knowledge synthesis) using CONSTRUCT query
/// Implements A = μ(O) - converts observations O into knowledge A via transformation μ
/// @param construct_query SPARQL CONSTRUCT query for knowledge synthesis
/// @param store_triples If non-zero, store generated triples back into unrdf store
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_generate_epistemology(const char *construct_query, int store_triples, char *result_json, size_t result_size);

/// Validate SHACL shapes against data graph via Rust → unrdf integration layer
/// @param data_turtle Turtle-formatted RDF data to validate
/// @param shapes_turtle Turtle-formatted SHACL shapes
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_validate_shacl(const char *data_turtle, const char *shapes_turtle, char *result_json, size_t result_size);

/// Begin a new transaction
/// @param actor Actor identifier for the transaction
/// @return Transaction ID on success, -1 on error
int knhk_unrdf_transaction_begin(const char *actor);

/// Add quads to transaction
/// @param transaction_id Transaction ID
/// @param turtle_data Turtle-formatted RDF data to add
/// @return 0 on success, -1 on error
int knhk_unrdf_transaction_add(int transaction_id, const char *turtle_data);

/// Remove quads from transaction
/// @param transaction_id Transaction ID
/// @param turtle_data Turtle-formatted RDF data to remove
/// @return 0 on success, -1 on error
int knhk_unrdf_transaction_remove(int transaction_id, const char *turtle_data);

/// Commit transaction
/// @param transaction_id Transaction ID
/// @param receipt_json Pre-allocated buffer for JSON receipt
/// @param receipt_size Size of receipt_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_transaction_commit(int transaction_id, char *receipt_json, size_t receipt_size);

/// Rollback transaction
/// @param transaction_id Transaction ID
/// @return 0 on success, -1 on error
int knhk_unrdf_transaction_rollback(int transaction_id);

/// Execute knowledge hook via Rust → unrdf integration layer
/// @param hook_name Name of the hook
/// @param hook_query SPARQL ASK query for hook condition
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_execute_hook(const char *hook_name, const char *hook_query, char *result_json, size_t result_size);

/// Register an autonomous epistemology hook for automatic knowledge generation
/// Implements autonomic epistemology: A = μ(O) triggered by conditions
/// Hook lifecycle: before → when → run (CONSTRUCT) → after
/// @param hook_json JSON string containing AutonomousEpistemologyHook definition
/// @param hook_id Pre-allocated buffer for hook ID
/// @param id_size Size of hook_id buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_register_autonomous_epistemology(const char *hook_json, char *hook_id, size_t id_size);

#ifdef __cplusplus
}
#endif

#endif // KNHK_UNRDF_H

