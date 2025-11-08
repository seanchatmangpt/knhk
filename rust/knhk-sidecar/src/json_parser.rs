// rust/knhk-sidecar/src/json_parser.rs
// JSON parsing module using simdjson for fast JSON → RawTriple conversion
// Provides direct conversion without Turtle intermediate format

use crate::error::{SidecarError, SidecarResult};
use knhk_etl::ingest::RawTriple;
use simd_json::prelude::*;
use std::collections::HashMap;

/// JSON Delta structure for transaction requests
/// Supports both simple JSON format and JSON-LD format
#[derive(Debug, Clone)]
pub struct JsonDelta {
    pub additions: Vec<JsonTriple>,
    pub removals: Vec<JsonTriple>,
}

/// JSON Triple structure
/// Supports multiple formats:
/// - Simple: {"s": "...", "p": "...", "o": "..."}
/// - JSON-LD: {"@id": "...", "@type": "...", ...}
#[derive(Debug, Clone)]
pub struct JsonTriple {
    pub s: String,
    pub p: String,
    pub o: String,
    pub g: Option<String>,
}

/// JSON parsing error
#[derive(Debug, thiserror::Error)]
pub enum JsonParseError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field type: {0}")]
    InvalidFieldType(String),

    #[error("Empty JSON data")]
    EmptyData,
}

impl JsonDelta {
    /// Parse JSON delta from bytes using simdjson
    ///
    /// Supports formats:
    /// - Simple: {"additions": [{"s": "...", "p": "...", "o": "..."}], "removals": []}
    /// - JSON-LD: {"@graph": [...]}
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, JsonParseError> {
        if bytes.is_empty() {
            return Err(JsonParseError::EmptyData);
        }

        // Clone bytes for simdjson (it needs mutable access)
        let mut json_bytes = bytes.to_vec();

        // Parse with simdjson
        let value: simd_json::OwnedValue = simd_json::from_slice(&mut json_bytes)
            .map_err(|e| JsonParseError::InvalidJson(format!("simdjson parse error: {}", e)))?;

        // Try simple format first: {"additions": [...], "removals": [...]}
        if let Some(obj) = value.as_object() {
            let additions = Self::parse_triple_array(
                obj.get("additions")
                    .ok_or_else(|| JsonParseError::MissingField("additions".to_string()))?,
            )?;

            let removals = if let Some(removals_val) = obj.get("removals") {
                Self::parse_triple_array(removals_val)?
            } else {
                Vec::new()
            };

            return Ok(JsonDelta {
                additions,
                removals,
            });
        }

        // Try JSON-LD format: {"@graph": [...]}
        if let Some(obj) = value.as_object() {
            if let Some(graph_val) = obj.get("@graph") {
                let additions = Self::parse_jsonld_array(graph_val)?;
                return Ok(JsonDelta {
                    additions,
                    removals: Vec::new(),
                });
            }
        }

        // Try array format: [{"s": "...", "p": "...", "o": "..."}]
        if let Some(arr) = value.as_array() {
            let additions = Self::parse_triple_array(&simd_json::OwnedValue::Array(
                arr.iter().cloned().collect::<Vec<_>>().into(),
            ))?;
            return Ok(JsonDelta {
                additions,
                removals: Vec::new(),
            });
        }

        Err(JsonParseError::InvalidJson(
            "Expected object with 'additions'/'removals' or '@graph' or array".to_string(),
        ))
    }

    /// Parse array of triples from JSON value
    fn parse_triple_array(
        value: &simd_json::OwnedValue,
    ) -> Result<Vec<JsonTriple>, JsonParseError> {
        let arr = value
            .as_array()
            .ok_or_else(|| JsonParseError::InvalidFieldType("expected array".to_string()))?;

        let mut triples = Vec::new();
        for item in arr {
            let triple = Self::parse_triple(item)?;
            triples.push(triple);
        }

        Ok(triples)
    }

    /// Parse JSON-LD array format
    fn parse_jsonld_array(
        value: &simd_json::OwnedValue,
    ) -> Result<Vec<JsonTriple>, JsonParseError> {
        let arr = value
            .as_array()
            .ok_or_else(|| JsonParseError::InvalidFieldType("expected array".to_string()))?;

        let mut triples = Vec::new();
        for item in arr {
            // JSON-LD format: extract subject, predicate, object from object
            if let Some(obj) = item.as_object() {
                let s = obj
                    .get("@id")
                    .or_else(|| obj.get("subject"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JsonParseError::MissingField("subject/@id".to_string()))?
                    .to_string();

                // For JSON-LD, we need to extract predicates and objects
                // This is a simplified version - full JSON-LD parsing would be more complex
                for (key, val) in obj.iter() {
                    if key != "@id" && key != "@type" && key != "@context" {
                        let p = key.clone();
                        let o = if let Some(s) = val.as_str() {
                            s.to_string()
                        } else if let Some(n) = val.as_u64() {
                            n.to_string()
                        } else if let Some(n) = val.as_i64() {
                            n.to_string()
                        } else if let Some(n) = val.as_f64() {
                            n.to_string()
                        } else if let Some(b) = val.as_bool() {
                            b.to_string()
                        } else if val.is_null() {
                            "null".to_string()
                        } else {
                            // Skip complex nested objects for now
                            continue;
                        };

                        triples.push(JsonTriple {
                            s: s.clone(),
                            p,
                            o,
                            g: None,
                        });
                    }
                }
            }
        }

        Ok(triples)
    }

    /// Parse single triple from JSON value
    fn parse_triple(value: &simd_json::OwnedValue) -> Result<JsonTriple, JsonParseError> {
        let obj = value
            .as_object()
            .ok_or_else(|| JsonParseError::InvalidFieldType("expected object".to_string()))?;

        let s = obj
            .get("s")
            .or_else(|| obj.get("subject"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonParseError::MissingField("s/subject".to_string()))?
            .to_string();

        let p = obj
            .get("p")
            .or_else(|| obj.get("predicate"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonParseError::MissingField("p/predicate".to_string()))?
            .to_string();

        let o = obj
            .get("o")
            .or_else(|| obj.get("object"))
            .and_then(|v| {
                // Support both string and number/boolean for object
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(n) = v.as_u64() {
                    Some(n.to_string())
                } else if let Some(n) = v.as_i64() {
                    Some(n.to_string())
                } else if let Some(n) = v.as_f64() {
                    Some(n.to_string())
                } else if let Some(b) = v.as_bool() {
                    Some(b.to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| JsonParseError::MissingField("o/object".to_string()))?;

        let g = obj
            .get("g")
            .or_else(|| obj.get("graph"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(JsonTriple { s, p, o, g })
    }

    /// Convert JSON delta to RawTriple vector
    /// Direct conversion without Turtle intermediate format
    pub fn to_raw_triples(self) -> Result<Vec<RawTriple>, JsonParseError> {
        let mut raw_triples = Vec::new();

        // Convert additions
        for json_triple in self.additions {
            raw_triples.push(RawTriple {
                subject: json_triple.s,
                predicate: json_triple.p,
                object: json_triple.o,
                graph: json_triple.g,
            });
        }

        // Note: Removals are handled separately in the ETL pipeline
        // For now, we only return additions as RawTriples
        // Removals would need special handling in the pipeline

        Ok(raw_triples)
    }
}

/// Parse JSON triples from bytes (convenience function)
pub fn parse_json_triples(bytes: &[u8]) -> SidecarResult<Vec<RawTriple>> {
    let delta = JsonDelta::from_bytes(bytes)
        .map_err(|e| SidecarError::validation_failed(format!("JSON parsing failed: {}", e)))?;

    delta
        .to_raw_triples()
        .map_err(|e| SidecarError::validation_failed(format!("Triple conversion failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)] // Test code - unwrap is acceptable
    fn test_parse_simple_json_delta() {
        let json = r#"{
            "additions": [
                {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o"}
            ],
            "removals": []
        }"#;

        let delta = JsonDelta::from_bytes(json.as_bytes()).unwrap();
        assert_eq!(delta.additions.len(), 1);
        assert_eq!(delta.removals.len(), 0);
        assert_eq!(delta.additions[0].s, "http://example.org/s");
        assert_eq!(delta.additions[0].p, "http://example.org/p");
        assert_eq!(delta.additions[0].o, "http://example.org/o");
    }

    #[test]
    #[allow(clippy::unwrap_used)] // Test code - unwrap is acceptable
    fn test_parse_json_array() {
        let json = r#"[
            {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o"}
        ]"#;

        let delta = JsonDelta::from_bytes(json.as_bytes()).unwrap();
        assert_eq!(delta.additions.len(), 1);
    }

    #[test]
    #[allow(clippy::unwrap_used)] // Test code - unwrap is acceptable
    fn test_to_raw_triples() {
        let json = r#"{
            "additions": [
                {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o", "g": "http://example.org/g"}
            ],
            "removals": []
        }"#;

        let delta = JsonDelta::from_bytes(json.as_bytes()).unwrap();
        let raw_triples = delta.to_raw_triples().unwrap();
        assert_eq!(raw_triples.len(), 1);
        assert_eq!(raw_triples[0].subject, "http://example.org/s");
        assert_eq!(raw_triples[0].predicate, "http://example.org/p");
        assert_eq!(raw_triples[0].object, "http://example.org/o");
        assert_eq!(
            raw_triples[0].graph,
            Some("http://example.org/g".to_string())
        );
    }

    #[test]
    fn test_parse_empty_json() {
        let json = b"{}";
        let result = JsonDelta::from_bytes(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = b"{ invalid json }";
        let result = JsonDelta::from_bytes(json);
        assert!(result.is_err());
    }
}
