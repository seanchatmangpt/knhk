// knhk-unrdf: SHACL validation
// Validate RDF data graphs against SHACL shapes

use crate::error::{UnrdfError, UnrdfResult};
use crate::script::execute_unrdf_script;
use crate::state::get_state;
use crate::template::TemplateEngine;
use crate::types::ShaclValidationResult;
use tera::Context;

/// Validate data graph against SHACL shapes graph
pub fn validate_shacl(
    data_turtle: &str,
    shapes_turtle: &str,
) -> UnrdfResult<ShaclValidationResult> {
    let state = get_state()?;

    // Use Tera template engine
    let template_engine = TemplateEngine::get()?;
    let mut context = Context::new();
    context.insert("data_turtle", data_turtle);
    context.insert("shapes_turtle", shapes_turtle);

    let script = template_engine
        .lock()
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to acquire template engine lock: {}", e))
        })?
        .render("shacl-validate", &context)
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to render shacl-validate template: {}", e))
        })?;

    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        // Extract JSON from output (unrdf prints initialization messages to stdout)
        let json_line = output
            .lines()
            .rev()
            .find(|line| line.trim().starts_with('{') || line.trim().starts_with('['))
            .ok_or_else(|| {
                UnrdfError::QueryFailed(format!("No JSON found in output. Full output: {}", output))
            })?;

        let result: ShaclValidationResult =
            serde_json::from_str(json_line.trim()).map_err(|e| {
                UnrdfError::QueryFailed(format!(
                    "Failed to parse validation result: {} - output: {}",
                    e, output
                ))
            })?;
        Ok(result)
    })
}
