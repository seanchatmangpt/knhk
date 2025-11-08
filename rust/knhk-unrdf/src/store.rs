// knhk-unrdf: Data storage operations
// Store RDF data in unrdf

use crate::error::{UnrdfError, UnrdfResult};
use crate::script::execute_unrdf_script;
use crate::state::get_state;
use crate::template::TemplateEngine;
use tera::Context;

/// Store data in unrdf
pub fn store_turtle_data(turtle_data: &str) -> UnrdfResult<()> {
    let state = get_state()?;

    // Use Tera template engine
    let template_engine = TemplateEngine::get()?;
    let mut context = Context::new();
    context.insert("turtle_data", turtle_data);

    let script = template_engine
        .lock()
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to acquire template engine lock: {}", e))
        })?
        .render("store", &context)
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to render store template: {}", e)))?;

    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        // Check if output contains "SUCCESS" (unrdf may print initialization messages first)
        if output.contains("SUCCESS") {
            Ok(())
        } else {
            Err(UnrdfError::StoreFailed(format!(
                "Store operation failed. Output: {}",
                output
            )))
        }
    })
}
