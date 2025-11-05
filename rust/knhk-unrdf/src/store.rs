// knhk-unrdf: Data storage operations
// Store RDF data in unrdf

use crate::error::{UnrdfError, UnrdfResult};
use crate::script::execute_unrdf_script;
use crate::state::get_state;

/// Store data in unrdf
pub fn store_turtle_data(turtle_data: &str) -> UnrdfResult<()> {
    let state = get_state()?;
    
    // Escape backticks and template literals in turtle data
    let escaped_data = turtle_data.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const turtleData = `{}`;
            const store = await parseTurtle(turtleData);
        
            const quads = [];
            store.forEach(q => quads.push(q));
        
            await system.executeTransaction({{
                additions: quads,
                removals: [],
                actor: 'knhk-rust'
            }});
        
            console.log('SUCCESS');
        }}
        
        main().catch(err => {{
            console.error('ERROR:', err.message);
            process.exit(1);
        }});
        "#,
        escaped_data
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        // Check if output contains "SUCCESS" (unrdf may print initialization messages first)
        if output.contains("SUCCESS") {
            Ok(())
        } else {
            Err(UnrdfError::StoreFailed(format!("Store operation failed. Output: {}", output)))
        }
    })
}

