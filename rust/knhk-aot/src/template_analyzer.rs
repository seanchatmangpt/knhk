// knhk-aot: CONSTRUCT template analyzer
// Analyzes CONSTRUCT templates to extract constant vs variable patterns

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Template triple pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriplePattern {
    /// Ground triple (all constants)
    Ground { s: u64, p: u64, o: u64 },
    /// Variable triple (has variables)
    Variable {
        s: Variable,
        p: Variable,
        o: Variable,
    },
}

/// Variable binding
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Variable {
    /// Constant value
    Constant(u64),
    /// Variable binding (from WHERE clause)
    Binding(String),
}

/// Template analysis result
#[derive(Debug, Clone)]
pub struct TemplateAnalysis {
    /// Ground triples (can be emitted once, not per solution)
    pub ground_triples: Vec<(u64, u64, u64)>,
    /// Variable triples (emitted per solution)
    pub variable_triples: Vec<TriplePattern>,
    /// Variable bindings map (WHERE clause variable -> template position)
    pub bindings: BTreeMap<String, Vec<BindingInfo>>,
}

/// Binding information
#[derive(Debug, Clone)]
pub struct BindingInfo {
    /// Position in triple (subject, predicate, object)
    pub position: TriplePosition,
    /// Triple index in variable_triples
    pub triple_index: usize,
}

/// Triple position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriplePosition {
    Subject,
    Predicate,
    Object,
}

/// Analyze CONSTRUCT template
/// Extracts ground triples and variable patterns for optimization
pub fn analyze_template(template: &str) -> Result<TemplateAnalysis, String> {
    // Parse CONSTRUCT template
    // Format: CONSTRUCT { triple1 . triple2 . ... } WHERE { ... }

    let mut ground_triples = Vec::new();
    let mut variable_triples = Vec::new();
    let mut bindings = BTreeMap::new();

    // Extract CONSTRUCT clause
    let construct_start = template
        .find("CONSTRUCT")
        .ok_or("No CONSTRUCT clause found")?;
    let construct_end = template.find("WHERE").ok_or("No WHERE clause found")?;
    let construct_clause = &template[construct_start + 9..construct_end].trim();

    // Parse triples in CONSTRUCT clause
    // Simplified parser - in production would use full SPARQL parser
    let triples = parse_triples(construct_clause)?;

    for (idx, triple) in triples.iter().enumerate() {
        match triple {
            TriplePattern::Ground { s, p, o } => {
                ground_triples.push((*s, *p, *o));
            }
            TriplePattern::Variable { s, p, o } => {
                variable_triples.push(triple.clone());

                // Track variable bindings
                if let Variable::Binding(var) = s {
                    bindings
                        .entry(var.clone())
                        .or_insert_with(Vec::new)
                        .push(BindingInfo {
                            position: TriplePosition::Subject,
                            triple_index: idx,
                        });
                }
                if let Variable::Binding(var) = p {
                    bindings
                        .entry(var.clone())
                        .or_insert_with(Vec::new)
                        .push(BindingInfo {
                            position: TriplePosition::Predicate,
                            triple_index: idx,
                        });
                }
                if let Variable::Binding(var) = o {
                    bindings
                        .entry(var.clone())
                        .or_insert_with(Vec::new)
                        .push(BindingInfo {
                            position: TriplePosition::Object,
                            triple_index: idx,
                        });
                }
            }
        }
    }

    Ok(TemplateAnalysis {
        ground_triples,
        variable_triples,
        bindings,
    })
}

/// Parse triples from CONSTRUCT clause
/// Simplified parser - extracts basic patterns
fn parse_triples(clause: &str) -> Result<Vec<TriplePattern>, String> {
    let mut triples = Vec::new();

    // Split by '.' to get individual triples
    let parts: Vec<&str> = clause.split('.').collect();

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Parse triple pattern
        // Format: <s> <p> <o> or ?s <p> ?o etc.
        let triple = parse_triple_pattern(part)?;
        triples.push(triple);
    }

    Ok(triples)
}

/// Parse a single triple pattern
fn parse_triple_pattern(pattern: &str) -> Result<TriplePattern, String> {
    let parts: Vec<&str> = pattern.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(format!("Invalid triple pattern: {}", pattern));
    }

    let s = parse_term(parts[0])?;
    let p = parse_term(parts[1])?;
    let o = parse_term(parts[2])?;

    // Check if all are constants (ground triple)
    if let (Variable::Constant(s_val), Variable::Constant(p_val), Variable::Constant(o_val)) =
        (&s, &p, &o)
    {
        Ok(TriplePattern::Ground {
            s: *s_val,
            p: *p_val,
            o: *o_val,
        })
    } else {
        Ok(TriplePattern::Variable { s, p, o })
    }
}

/// Parse a term (constant IRI or variable)
fn parse_term(term: &str) -> Result<Variable, String> {
    let term = term.trim();

    // Check if it's a variable (starts with ?)
    if let Some(stripped) = term.strip_prefix('?') {
        Ok(Variable::Binding(stripped.to_string()))
    } else if term.starts_with('<') && term.ends_with('>') {
        // IRI constant - hash it (simplified)
        // In production, would use proper IRI hashing
        let iri = &term[1..term.len() - 1];
        let hash = hash_iri(iri);
        Ok(Variable::Constant(hash))
    } else if term.starts_with('"') {
        // Literal constant - hash it (simplified)
        let hash = hash_literal(term);
        Ok(Variable::Constant(hash))
    } else {
        Err(format!("Invalid term: {}", term))
    }
}

/// Hash IRI to u64 (simplified FNV-1a implementation)
fn hash_iri(iri: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in iri.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Hash literal to u64 (simplified)
fn hash_literal(literal: &str) -> u64 {
    hash_iri(literal)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn test_analyze_ground_triple() {
        let template = "CONSTRUCT { <s> <p> <o> } WHERE { ?x <p2> ?y }";
        let analysis =
            analyze_template(template).expect("Failed to analyze ground triple template");

        assert_eq!(analysis.ground_triples.len(), 1);
        assert_eq!(analysis.variable_triples.len(), 0);
    }

    #[test]
    fn test_analyze_variable_triple() {
        let template = "CONSTRUCT { ?x <p> ?y } WHERE { ?x <p2> ?y }";
        let analysis =
            analyze_template(template).expect("Failed to analyze variable triple template");

        assert_eq!(analysis.ground_triples.len(), 0);
        assert_eq!(analysis.variable_triples.len(), 1);
        assert!(analysis.bindings.contains_key("x"));
        assert!(analysis.bindings.contains_key("y"));
    }
}
