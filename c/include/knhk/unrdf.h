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

/// Execute knowledge hook via Rust → unrdf integration layer
/// @param hook_name Name of the hook
/// @param hook_query SPARQL ASK query for hook condition
/// @param result_json Pre-allocated buffer for JSON result
/// @param result_size Size of result_json buffer
/// @return 0 on success, -1 on error
int knhk_unrdf_execute_hook(const char *hook_name, const char *hook_query, char *result_json, size_t result_size);

#ifdef __cplusplus
}
#endif

#endif // KNHK_UNRDF_H

