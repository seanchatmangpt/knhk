// knhk-unrdf: SHACL validation
// Validate RDF data graphs against SHACL shapes

use crate::error::{UnrdfError, UnrdfResult};
use crate::script::execute_unrdf_script;
use crate::state::get_state;
use crate::types::ShaclValidationResult;

/// Validate data graph against SHACL shapes graph
pub fn validate_shacl(data_turtle: &str, shapes_turtle: &str) -> UnrdfResult<ShaclValidationResult> {
    let state = get_state()?;
    
    // Escape turtle data
    let escaped_data = data_turtle.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    let escaped_shapes = shapes_turtle.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const dataTurtle = `{}`;
            const shapesTurtle = `{}`;
        
            try {{
                const dataStore = await parseTurtle(dataTurtle);
                const shapesStore = await parseTurtle(shapesTurtle);
        
                const validation = await system.validate({{
                    dataGraph: dataStore,
                    shapesGraph: shapesStore
                }});
        
                const violations = [];
                for (const report of validation.report.results) {{
                    violations.push({{
                        path: report.path ? report.path.value : null,
                        message: report.message ? report.message[0].value : '',
                        severity: report.severity ? report.severity.value : null,
                        focus_node: report.focusNode ? report.focusNode.value : null,
                        value: report.value ? report.value.value : null
                    }});
                }}
        
                console.log(JSON.stringify({{
                    conforms: validation.conforms,
                    violations: violations
                }}));
            }} catch (err) {{
                console.error(JSON.stringify({{
                    conforms: false,
                    violations: [],
                    error: err.message
                }}));
                process.exit(1);
            }}
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{
                conforms: false,
                violations: [],
                error: err.message
            }}));
            process.exit(1);
        }});
        "#,
        escaped_data,
        escaped_shapes
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        let result: ShaclValidationResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::QueryFailed(format!("Failed to parse validation result: {} - output: {}", e, output)))?;
        Ok(result)
    })
}

