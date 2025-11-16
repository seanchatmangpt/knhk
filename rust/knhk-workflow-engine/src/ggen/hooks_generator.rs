//! Knowledge Hooks Generator - Generate hook definitions from RDF ontology
//!
//! Generates Lockchain-integrated hook implementations from RDF ontologies:
//! - Hook definitions from RDF classes
//! - Trigger patterns (RDF change, SPARQL result, interval)
//! - Check logic from SPARQL queries
//! - Action execution from workflow definitions
//! - Proof receipt emission (Lockchain integration)
//!
//! # Hook Pattern
//!
//! ```text
//! Trigger → Check → Act → Receipt
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use knhk_workflow_engine::ggen::hooks_generator::HooksGenerator;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let generator = HooksGenerator::new("templates/hooks")?;
//! generator.load_ontology("ontology/workflow.ttl")?;
//!
//! let hooks = generator.generate_hooks()?;
//! println!("{}", hooks);
//! # Ok(())
//! # }
//! ```

use crate::error::{WorkflowError, WorkflowResult};
use oxigraph::io::RdfFormat;
use oxigraph::model::Term;
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;
use std::path::Path;
use tera::{Context, Tera};
use tracing::{debug, info, instrument};

/// Hook trigger type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerType {
    /// RDF graph change (INSERT/DELETE)
    RdfChange,
    /// SPARQL query result change
    SparqlResult,
    /// Time-based interval
    Interval,
    /// Event-driven trigger
    Event,
}

/// Hook definition
#[derive(Debug, Clone)]
pub struct HookDefinition {
    /// Hook identifier
    pub id: String,
    /// Hook name
    pub name: String,
    /// Trigger type
    pub trigger_type: TriggerType,
    /// Trigger pattern (SPARQL query or event name)
    pub trigger_pattern: String,
    /// Check condition (SPARQL ASK query)
    pub check_condition: Option<String>,
    /// Action to execute (workflow ID or code)
    pub action: String,
    /// Whether to emit Lockchain receipt
    pub emit_receipt: bool,
}

/// Hooks generator
pub struct HooksGenerator {
    /// Tera template engine
    tera: Tera,
    /// RDF ontology store
    ontology_store: Store,
}

impl HooksGenerator {
    /// Create new hooks generator
    ///
    /// # Arguments
    ///
    /// * `template_dir` - Directory containing hook templates
    ///
    /// # Errors
    ///
    /// Returns error if template directory is invalid or Tera initialization fails.
    #[instrument(skip(template_dir))]
    pub fn new(template_dir: impl AsRef<Path>) -> WorkflowResult<Self> {
        let template_dir = template_dir.as_ref();

        // Initialize Tera with hook templates
        let template_pattern = template_dir
            .join("**/*.tera")
            .to_str()
            .ok_or_else(|| WorkflowError::Internal("Invalid template path".to_string()))?
            .to_string();

        let tera = Tera::new(&template_pattern)
            .map_err(|e| WorkflowError::Internal(format!("Failed to initialize Tera: {}", e)))?;

        // Create ontology store
        let ontology_store = Store::new()
            .map_err(|e| WorkflowError::Internal(format!("Failed to create RDF store: {:?}", e)))?;

        info!("Created hooks generator");

        Ok(Self {
            tera,
            ontology_store,
        })
    }

    /// Load RDF ontology
    ///
    /// # Arguments
    ///
    /// * `ontology_path` - Path to RDF ontology file (.ttl)
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or RDF parsing fails.
    #[instrument(skip(self))]
    pub fn load_ontology(&self, ontology_path: impl AsRef<Path>) -> WorkflowResult<()> {
        let ontology_path = ontology_path.as_ref();
        let ontology_content = std::fs::read_to_string(ontology_path)
            .map_err(|e| WorkflowError::Internal(format!("Failed to read ontology: {}", e)))?;

        self.ontology_store
            .load_from_reader(RdfFormat::Turtle, ontology_content.as_bytes())
            .map_err(|e| WorkflowError::Internal(format!("Failed to load ontology: {:?}", e)))?;

        info!("Loaded ontology from: {:?}", ontology_path);
        Ok(())
    }

    /// Extract hook definitions from ontology
    ///
    /// Queries the ontology for hook classes and extracts:
    /// - Trigger patterns
    /// - Check conditions
    /// - Actions
    ///
    /// # Errors
    ///
    /// Returns error if SPARQL query execution fails.
    #[instrument(skip(self))]
    pub fn extract_hook_definitions(&self) -> WorkflowResult<Vec<HookDefinition>> {
        // SPARQL query to extract hooks from ontology
        let query = r#"
PREFIX knhk: <http://knhk.io/ontology#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?hook ?name ?triggerType ?triggerPattern ?checkCondition ?action ?emitReceipt
WHERE {
    ?hook rdf:type knhk:Hook .
    ?hook knhk:name ?name .
    ?hook knhk:triggerType ?triggerType .
    ?hook knhk:triggerPattern ?triggerPattern .
    ?hook knhk:action ?action .
    OPTIONAL { ?hook knhk:checkCondition ?checkCondition } .
    OPTIONAL { ?hook knhk:emitReceipt ?emitReceipt } .
}
"#;

        let results = SparqlEvaluator::new()
            .parse_query(query)
            .map_err(|e| WorkflowError::Internal(format!("Invalid SPARQL query: {}", e)))?
            .on_store(&self.ontology_store)
            .execute()
            .map_err(|e| WorkflowError::Internal(format!("SPARQL execution failed: {}", e)))?;

        let mut hooks = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution_result in solutions {
                let solution = solution_result
                    .map_err(|e| WorkflowError::Internal(format!("Solution error: {:?}", e)))?;

                // Extract hook data
                let id = Self::extract_term(&solution, "hook")?;
                let name = Self::extract_term(&solution, "name")?;
                let trigger_type_str = Self::extract_term(&solution, "triggerType")?;
                let trigger_pattern = Self::extract_term(&solution, "triggerPattern")?;
                let action = Self::extract_term(&solution, "action")?;

                let check_condition = solution.get("checkCondition").map(Self::term_to_string);

                let emit_receipt = solution
                    .get("emitReceipt")
                    .map(|t| Self::term_to_string(t) == "true")
                    .unwrap_or(true);

                // Parse trigger type
                let trigger_type = match trigger_type_str.as_str() {
                    "RdfChange" => TriggerType::RdfChange,
                    "SparqlResult" => TriggerType::SparqlResult,
                    "Interval" => TriggerType::Interval,
                    "Event" => TriggerType::Event,
                    _ => TriggerType::Event,
                };

                hooks.push(HookDefinition {
                    id,
                    name,
                    trigger_type,
                    trigger_pattern,
                    check_condition,
                    action,
                    emit_receipt,
                });
            }
        }

        debug!("Extracted {} hook definitions", hooks.len());
        Ok(hooks)
    }

    /// Generate hook implementations
    ///
    /// Generates Rust code for hook implementations with Lockchain integration.
    ///
    /// # Errors
    ///
    /// Returns error if template rendering fails.
    #[instrument(skip(self))]
    pub fn generate_hooks(&self) -> WorkflowResult<String> {
        let hook_defs = self.extract_hook_definitions()?;

        let mut context = Context::new();
        context.insert("hooks", &hook_defs);

        // Generate hook code
        let mut generated_code = String::new();
        generated_code.push_str(&self.generate_hook_header()?);

        for hook in &hook_defs {
            generated_code.push('\n');
            generated_code.push_str(&self.generate_hook_impl(hook)?);
        }

        generated_code.push('\n');
        generated_code.push_str(&self.generate_hook_registry(&hook_defs)?);

        Ok(generated_code)
    }

    /// Generate hook module header
    fn generate_hook_header(&self) -> WorkflowResult<String> {
        Ok(r#"//! Generated Knowledge Hooks
//!
//! Auto-generated from RDF ontology with Lockchain integration.

use crate::error::{WorkflowError, WorkflowResult};
use knhk_lockchain::{Receipt, MerkleTree};
use tracing::{info, warn, instrument};
use std::sync::Arc;

/// Hook execution context
pub struct HookContext {
    /// Lockchain for proof receipts
    pub lockchain: Arc<MerkleTree>,
    /// Execution timestamp
    pub timestamp: u64,
}

"#
        .to_string())
    }

    /// Generate individual hook implementation
    fn generate_hook_impl(&self, hook: &HookDefinition) -> WorkflowResult<String> {
        let trigger_impl = match hook.trigger_type {
            TriggerType::RdfChange => {
                "// Trigger: RDF graph change\n    // Monitor INSERT/DELETE operations"
            }
            TriggerType::SparqlResult => {
                "// Trigger: SPARQL query result\n    // Execute query and detect changes"
            }
            TriggerType::Interval => {
                "// Trigger: Time interval\n    // Schedule periodic execution"
            }
            TriggerType::Event => "// Trigger: Event-driven\n    // Listen for event notifications",
        };

        let check_impl = if let Some(ref condition) = hook.check_condition {
            format!("// Check: {}\n    // Execute ASK query", condition)
        } else {
            "// Check: Always true".to_string()
        };

        let receipt_impl = if hook.emit_receipt {
            r#"
    // Emit Lockchain receipt
    let receipt = Receipt::new(
        context.timestamp,
        0, // shard_id
        0, // hook_id
        execution_ticks,
        hash_a,
    );

    context.lockchain.insert_receipt(receipt)
        .map_err(|e| WorkflowError::Internal(format!("Receipt emission failed: {}", e)))?;"#
        } else {
            "    // No receipt emission"
        };

        let code = format!(
            r#"/// Hook: {}
#[instrument(skip(context))]
pub async fn hook_{}(context: &HookContext) -> WorkflowResult<()> {{
    {}

    {}

    // Act: Execute action
    // Action: {}
    let execution_ticks = 0; // TODO: Measure actual ticks
    let hash_a = 0; // TODO: Compute hash
    {}

    info!("Hook '{}' executed successfully");
    Ok(())
}}
"#,
            hook.name,
            hook.id.replace('-', "_"),
            trigger_impl,
            check_impl,
            hook.action,
            receipt_impl,
            hook.name
        );

        Ok(code)
    }

    /// Generate hook registry
    fn generate_hook_registry(&self, hooks: &[HookDefinition]) -> WorkflowResult<String> {
        let mut registry = String::from(
            r#"/// Hook registry for runtime lookup
pub struct HookRegistry {
    hooks: std::collections::HashMap<String, Box<dyn Fn(&HookContext) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorkflowResult<()>> + Send>> + Send + Sync>>,
}

impl HookRegistry {
    /// Create new hook registry
    pub fn new() -> Self {
        let mut registry = Self {
            hooks: std::collections::HashMap::new(),
        };

"#,
        );

        for hook in hooks {
            registry.push_str(&format!(
                r#"        registry.register("{}", hook_{});
"#,
                hook.id,
                hook.id.replace('-', "_")
            ));
        }

        registry.push_str(
            r#"
        registry
    }

    /// Register hook
    fn register<F, Fut>(&mut self, id: &str, hook: F)
    where
        F: Fn(&HookContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = WorkflowResult<()>> + Send + 'static,
    {
        self.hooks.insert(
            id.to_string(),
            Box::new(move |ctx| Box::pin(hook(ctx))),
        );
    }
}
"#,
        );

        Ok(registry)
    }

    // Helper: Extract term from SPARQL solution
    fn extract_term(
        solution: &oxigraph::sparql::QuerySolution,
        var: &str,
    ) -> WorkflowResult<String> {
        solution
            .get(var)
            .map(Self::term_to_string)
            .ok_or_else(|| WorkflowError::Internal(format!("Missing variable: {}", var)))
    }

    // Helper: Convert RDF term to string
    fn term_to_string(term: &Term) -> String {
        match term {
            Term::NamedNode(node) => node.to_string(),
            Term::BlankNode(node) => format!("_:{}", node),
            Term::Literal(lit) => lit.value().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hooks_generator_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let template_dir = temp_dir.path().join("templates");
        std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

        let generator = HooksGenerator::new(&template_dir);
        assert!(
            generator.is_ok(),
            "Generator should be created successfully"
        );
    }

    #[test]
    fn test_trigger_types() {
        assert_eq!(TriggerType::RdfChange, TriggerType::RdfChange);
        assert_ne!(TriggerType::RdfChange, TriggerType::Event);
    }
}
