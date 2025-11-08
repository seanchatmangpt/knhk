// knhk-unrdf: RDF serialization
// Serialize unrdf store to various RDF formats

use crate::error::{UnrdfError, UnrdfResult};
use crate::script::execute_unrdf_script;
use crate::state::get_state;
use crate::template::TemplateEngine;
use crate::types::RdfFormat;
use tera::Context;

/// Serialize unrdf store to RDF format
pub fn serialize_rdf(format: RdfFormat) -> UnrdfResult<String> {
    let state = get_state()?;

    let format_str = match format {
        RdfFormat::Turtle => "turtle",
        RdfFormat::JsonLd => "jsonld",
        RdfFormat::NQuads => "nquads",
    };

    // Use Tera template engine
    let template_engine = TemplateEngine::get()?;
    let mut context = Context::new();
    context.insert("format", &format_str);

    let script = template_engine
        .lock()
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to acquire template engine lock: {}", e))
        })?
        .render("serialize", &context)
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to render serialize template: {}", e))
        })?;

    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        Ok(output.trim().to_string())
    })
}
