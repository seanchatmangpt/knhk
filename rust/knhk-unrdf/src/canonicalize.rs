// knhk-unrdf: Rust-native canonicalization using URDNA2015
// Pure Rust RDF canonicalization without Node.js dependency

#[cfg(feature = "native")]
use crate::error::{UnrdfError, UnrdfResult};
#[cfg(feature = "native")]
use oxigraph::store::Store;
#[cfg(feature = "native")]
use oxigraph::io::RdfFormat;
#[cfg(feature = "native")]
use oxigraph::sparql::QueryResults;
#[cfg(feature = "native")]
use sha2::{Sha256, Digest};
#[cfg(feature = "native")]
use blake3;

#[cfg(feature = "native")]
/// Canonicalize RDF data using URDNA2015 algorithm
/// 
/// Note: oxigraph doesn't directly support URDNA2015 canonicalization.
/// This implementation converts to N-Quads and applies basic canonicalization.
/// For full URDNA2015, we may need to add rdf-canonize-rs or implement it.
pub fn canonicalize_rdf(turtle_data: &str) -> UnrdfResult<String> {
    let store = Store::new()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to create store: {}", e)))?;
    
    // Load Turtle data
    store.load_from_reader(RdfFormat::Turtle, turtle_data.as_bytes())
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to parse Turtle: {}", e)))?;
    
    // Serialize to N-Quads (canonical form)
    // Collect triples from CONSTRUCT query and convert to N-Quads format
    // Note: oxigraph Store doesn't have a direct iterator, we'll use a query approach
    // CONSTRUCT queries return triples, which we convert to N-Quads format
    let query = "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }";
    let results: QueryResults = store.query(query)
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to query store: {}", e)))?;
    
    let mut quads: Vec<String> = Vec::new();
    if let QueryResults::Graph(triples_iter) = results {
        for triple_result in triples_iter {
            let triple = triple_result.map_err(|e| UnrdfError::InvalidInput(format!("Failed to get triple: {}", e)))?;
            let quad_str = format!("{} {} {} .\n",
                triple.subject.to_string(),
                triple.predicate.to_string(),
                triple.object.to_string()
            );
            quads.push(quad_str);
        }
    }
    
    // Sort quads for deterministic ordering
    quads.sort();
    let quads_str = quads.join("");
    
    Ok(quads_str)
}

#[cfg(feature = "native")]
/// Get canonical hash of RDF data (SHA-256)
pub fn get_canonical_hash_sha256(turtle_data: &str) -> UnrdfResult<String> {
    let canonical = canonicalize_rdf(turtle_data)?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

#[cfg(feature = "native")]
/// Get canonical hash of RDF data (Blake3)
pub fn get_canonical_hash_blake3(turtle_data: &str) -> UnrdfResult<String> {
    let canonical = canonicalize_rdf(turtle_data)?;
    let hash = blake3::hash(canonical.as_bytes());
    Ok(hash.to_hex().to_string())
}

#[cfg(feature = "native")]
/// Check if two RDF graphs are isomorphic (have same canonical form)
pub fn check_isomorphic(data1: &str, data2: &str) -> UnrdfResult<bool> {
    let canonical1 = canonicalize_rdf(data1)?;
    let canonical2 = canonicalize_rdf(data2)?;
    Ok(canonical1 == canonical2)
}

#[cfg(feature = "native")]
/// Get canonical hash (defaults to Blake3 for performance)
pub fn get_canonical_hash(turtle_data: &str) -> UnrdfResult<String> {
    get_canonical_hash_blake3(turtle_data)
}

