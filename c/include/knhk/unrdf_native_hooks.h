#ifndef KNHK_UNRDF_NATIVE_HOOKS_H
#define KNHK_UNRDF_NATIVE_HOOKS_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>

/**
 * Execute a hook by name (native Rust implementation)
 * Use case 1: Single hook execution
 * 
 * @param hook_name Hook name/identifier
 * @param hook_query SPARQL ASK query for hook condition
 * @param turtle_data Turtle data to evaluate against
 * @param result_json Buffer to write JSON result
 * @param result_size Size of result buffer
 * @return 0 on success, negative on error
 */
int knhk_unrdf_execute_hook_native(
    const char *hook_name,
    const char *hook_query,
    const char *turtle_data,
    char *result_json,
    size_t result_size
);

/**
 * Execute multiple hooks in batch (native Rust implementation)
 * Use case 2: Batch hook evaluation for efficiency
 * 
 * @param hooks_json JSON array of hook definitions
 * @param turtle_data Turtle data to evaluate against
 * @param result_json Buffer to write JSON result
 * @param result_size Size of result buffer
 * @return 0 on success, negative on error
 */
int knhk_unrdf_execute_hooks_batch_native(
    const char *hooks_json,
    const char *turtle_data,
    char *result_json,
    size_t result_size
);

/**
 * Register a hook in the native registry
 * 
 * @param hook_json JSON hook definition
 * @return 0 on success, negative on error
 */
int knhk_unrdf_register_hook_native(const char *hook_json);

/**
 * Deregister a hook from the native registry
 * 
 * @param hook_id Hook identifier
 * @return 0 on success, negative on error
 */
int knhk_unrdf_deregister_hook_native(const char *hook_id);

#ifdef __cplusplus
}
#endif

#endif /* KNHK_UNRDF_NATIVE_HOOKS_H */

