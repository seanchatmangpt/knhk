// rust/knhk-cli/src/commands/boot.rs
// Boot commands - Initialize Σ and Q

use crate::state::StateManager;
use oxigraph::io::RdfFormat;
use std::fs;
use std::path::PathBuf;

#[cfg(feature = "otel")]
use knhk_otel::{MetricsHelper, Tracer};
#[cfg(feature = "otel")]
use tracing::{debug, error, info, span, Level};

/// Initialize Σ and Q
/// boot(#{sigma => SigmaTTL, q => QTTL})
pub fn init(sigma: String, q: String) -> Result<PathBuf, String> {
    #[cfg(feature = "otel")]
    let _span = span!(Level::INFO, "knhk.boot.init", knhk.operation.name = "boot.init", knhk.operation.type = "system");

    #[cfg(feature = "otel")]
    let _enter = _span.enter();

    println!("Initializing system with Σ and Q...");

    #[cfg(feature = "otel")]
    {
        debug!(sigma = %sigma, q = %q, "initializing_system");
    }

    // Get config directory
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir).map_err(|e| {
        #[cfg(feature = "otel")]
        error!(error = %e, "failed_to_create_config_dir");
        format!("Failed to create config directory: {}", e)
    })?;

    #[cfg(feature = "otel")]
    debug!(config_dir = %config_dir.display(), "config_dir_created");

    // Load schema file
    let sigma_path = PathBuf::from(&sigma);
    if !sigma_path.exists() {
        #[cfg(feature = "otel")]
        error!(sigma = %sigma, "schema_file_not_found");
        return Err(format!("Schema file not found: {}", sigma));
    }
    let sigma_content = fs::read_to_string(&sigma_path).map_err(|e| {
        #[cfg(feature = "otel")]
        error!(error = %e, "failed_to_read_schema");
        format!("Failed to read schema file: {}", e)
    })?;

    #[cfg(feature = "otel")]
    debug!(sigma_size = sigma_content.len(), "schema_loaded");

    // Load invariants file
    let q_path = PathBuf::from(&q);
    if !q_path.exists() {
        #[cfg(feature = "otel")]
        error!(q = %q, "invariants_file_not_found");
        return Err(format!("Invariants file not found: {}", q));
    }
    let q_content = fs::read_to_string(&q_path).map_err(|e| {
        #[cfg(feature = "otel")]
        error!(error = %e, "failed_to_read_invariants");
        format!("Failed to read invariants file: {}", e)
    })?;

    #[cfg(feature = "otel")]
    debug!(q_size = q_content.len(), "invariants_loaded");

    // Validate schema format (basic check - must be non-empty)
    if sigma_content.trim().is_empty() {
        #[cfg(feature = "otel")]
        error!("schema_file_empty");
        return Err("Schema file is empty".to_string());
    }

    // Validate invariants format (basic check - must be non-empty)
    if q_content.trim().is_empty() {
        #[cfg(feature = "otel")]
        error!("invariants_file_empty");
        return Err("Invariants file is empty".to_string());
    }

    // Store schema and invariants in config directory
    let sigma_config = config_dir.join("sigma.ttl");
    let q_config = config_dir.join("q.sparql");

    fs::write(&sigma_config, &sigma_content).map_err(|e| {
        #[cfg(feature = "otel")]
        error!(error = %e, "failed_to_write_schema");
        format!("Failed to write schema config: {}", e)
    })?;
    fs::write(&q_config, &q_content).map_err(|e| {
        #[cfg(feature = "otel")]
        error!(error = %e, "failed_to_write_invariants");
        format!("Failed to write invariants config: {}", e)
    })?;

    // Save Σ and Q to Oxigraph using StateManager
    let state_manager = StateManager::new()?;

    // Load schema file and save to Oxigraph
    let schema_store = oxigraph::store::Store::new()
        .map_err(|e| format!("Failed to create Oxigraph store for schema: {}", e))?;
    schema_store
        .load_from_reader(RdfFormat::Turtle, sigma_content.as_bytes())
        .map_err(|e| format!("Failed to load schema into Oxigraph: {}", e))?;

    // Save schema to StateManager's store
    let mut schema_graph = oxigraph::model::Graph::new();
    for quad_result in schema_store.quads_for_pattern(None, None, None, None) {
        let quad = quad_result.map_err(|e| format!("Failed to query schema store: {}", e))?;
        let triple_ref = oxigraph::model::TripleRef::new(
            quad.subject.as_ref(),
            quad.predicate.as_ref(),
            quad.object.as_ref(),
        );
        schema_graph.insert(triple_ref);
    }

    state_manager
        .ontology_saver()
        .save(&schema_graph, Some(&sigma))?;

    // Load invariants file and save to Oxigraph
    let invariant_store = oxigraph::store::Store::new()
        .map_err(|e| format!("Failed to create Oxigraph store for invariants: {}", e))?;
    invariant_store
        .load_from_reader(RdfFormat::Turtle, q_content.as_bytes())
        .map_err(|e| format!("Failed to load invariants into Oxigraph: {}", e))?;

    // Save invariants to StateManager's store
    let mut invariant_graph = oxigraph::model::Graph::new();
    for quad_result in invariant_store.quads_for_pattern(None, None, None, None) {
        let quad = quad_result.map_err(|e| format!("Failed to query invariant store: {}", e))?;
        let triple_ref = oxigraph::model::TripleRef::new(
            quad.subject.as_ref(),
            quad.predicate.as_ref(),
            quad.object.as_ref(),
        );
        invariant_graph.insert(triple_ref);
    }

    state_manager
        .ontology_saver()
        .save(&invariant_graph, Some(&q))?;

    // Write initialization marker
    let init_marker = config_dir.join(".initialized");
    fs::write(&init_marker, "initialized").map_err(|e| {
        #[cfg(feature = "otel")]
        error!(error = %e, "failed_to_write_init_marker");
        format!("Failed to write init marker: {}", e)
    })?;

    println!("  ✓ Schema loaded: {}", sigma);
    println!("  ✓ Invariants loaded: {}", q);
    println!("  ✓ Config directory: {}", config_dir.display());
    println!("✓ System initialized");

    #[cfg(feature = "otel")]
    {
        info!(
            config_dir = %config_dir.display(),
            sigma_size = sigma_content.len(),
            q_size = q_content.len(),
            "system_initialized"
        );

        // Record metrics
        let mut tracer = Tracer::new();
        MetricsHelper::record_operation(&mut tracer, "boot.init", true);
    }

    Ok(config_dir)
}

fn get_config_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let mut path = PathBuf::from(std::env::var("APPDATA").map_err(|_| "APPDATA not set")?);
        path.push("knhk");
        Ok(path)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").map_err(|_| "HOME not set")?;
        let mut path = PathBuf::from(home);
        path.push(".knhk");
        Ok(path)
    }
}
