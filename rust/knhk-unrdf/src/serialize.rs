// knhk-unrdf: RDF serialization
// Serialize unrdf store to various RDF formats

use crate::error::UnrdfResult;
use crate::script::execute_unrdf_script;
use crate::state::get_state;
use crate::types::RdfFormat;

/// Serialize unrdf store to RDF format
pub fn serialize_rdf(format: RdfFormat) -> UnrdfResult<String> {
    let state = get_state()?;
    
    let format_str = match format {
        RdfFormat::Turtle => "turtle",
        RdfFormat::JsonLd => "jsonld",
        RdfFormat::NQuads => "nquads",
    };
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        import {{ toTurtle, toJsonLd, toNQuads }} from './src/knowledge-engine/serialize.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const format = '{}';
            let result;
        
            try {{
                // Get the store from the system
                const store = system.store || system.getStore();
        
                if (format === 'turtle') {{
                    result = await toTurtle(store);
                }} else if (format === 'jsonld') {{
                    result = await toJsonLd(store);
                }} else if (format === 'nquads') {{
                    result = await toNQuads(store);
                }} else {{
                    throw new Error('Unknown format: ' + format);
                }}
        
                console.log(result);
            }} catch (err) {{
                console.error('ERROR:', err.message);
                process.exit(1);
            }}
        }}
        
        main().catch(err => {{
            console.error('ERROR:', err.message);
            process.exit(1);
        }});
        "#,
        format_str
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        Ok(output.trim().to_string())
    })
}

