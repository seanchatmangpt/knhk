//! Legacy XML/YAWL parser for migration to KNHK TTL-only architecture
//!
//! # ⚠️ DEPRECATION NOTICE
//!
//! This crate provides XML parsing ONLY for migration from legacy XML YAWL files.
//! KNHK v4.0+ uses **TTL/Turtle exclusively** per DOCTRINE Covenant 1.
//!
//! DO NOT use this crate for new workflows. Use TTL format directly.
//!
//! # Usage
//!
//! ```no_run
//! use knhk_workflow_xml_legacy::XmlToTtlConverter;
//!
//! let xml_content = std::fs::read_to_string("workflow.yawl")?;
//! let converter = XmlToTtlConverter::new();
//! let ttl = converter.convert(&xml_content)?;
//! std::fs::write("workflow.ttl", ttl)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(missing_docs)]

pub mod parser;
pub mod serializer;
pub mod error;

pub use parser::XmlParser;
pub use serializer::TurtleSerializer;
pub use error::{LegacyError, LegacyResult};

use std::path::Path;

/// Main converter from XML YAWL to TTL format
pub struct XmlToTtlConverter {
    parser: XmlParser,
    serializer: TurtleSerializer,
}

impl XmlToTtlConverter {
    /// Create new converter
    pub fn new() -> Self {
        Self {
            parser: XmlParser::new(),
            serializer: TurtleSerializer::new(),
        }
    }

    /// Convert XML YAWL string to TTL string
    pub fn convert(&self, xml: &str) -> LegacyResult<String> {
        // Parse XML to intermediate representation
        let workflow = self.parser.parse(xml)?;

        // Serialize to TTL
        let ttl = self.serializer.serialize(&workflow)?;

        Ok(ttl)
    }

    /// Convert XML YAWL file to TTL file
    pub fn convert_file(&self, input: &Path, output: &Path) -> LegacyResult<()> {
        let xml = std::fs::read_to_string(input)
            .map_err(|e| LegacyError::Io(format!("Failed to read input: {}", e)))?;

        let ttl = self.convert(&xml)?;

        std::fs::write(output, ttl)
            .map_err(|e| LegacyError::Io(format!("Failed to write output: {}", e)))?;

        Ok(())
    }

    /// Validate converted TTL output
    pub fn validate_ttl(&self, ttl: &str) -> LegacyResult<()> {
        // Basic TTL syntax validation using oxigraph
        #[cfg(feature = "rdf")]
        {
            use oxigraph::store::Store;
            use oxigraph::io::RdfFormat;

            let store = Store::new()
                .map_err(|e| LegacyError::Validation(format!("Failed to create RDF store: {:?}", e)))?;

            store
                .load_from_reader(RdfFormat::Turtle, ttl.as_bytes())
                .map_err(|e| {
                    LegacyError::Validation(format!("Invalid TTL syntax: {}", e))
                })?;
        }

        #[cfg(not(feature = "rdf"))]
        {
            // Basic syntax check - ensure it has prefix declarations and triples
            if !ttl.contains("@prefix") {
                return Err(LegacyError::Validation(
                    "TTL missing @prefix declarations".to_string()
                ));
            }
        }

        Ok(())
    }
}

impl Default for XmlToTtlConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_conversion() {
        let xml = r#"<?xml version="1.0"?>
<specification uri="http://example.org/test">
  <name>Test Workflow</name>
</specification>"#;

        let converter = XmlToTtlConverter::new();
        let result = converter.convert(xml);

        assert!(result.is_ok(), "Conversion should succeed");

        let ttl = result.unwrap();
        assert!(ttl.contains("yawl:Specification"), "Should contain YAWL type");
        assert!(ttl.contains("Test Workflow"), "Should contain workflow name");
    }

    #[test]
    fn test_validation() {
        let valid_ttl = r#"
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
<http://example.org/test> a yawl:Specification .
"#;

        let converter = XmlToTtlConverter::new();
        let result = converter.validate_ttl(valid_ttl);

        assert!(result.is_ok(), "Valid TTL should pass validation");
    }

    #[test]
    fn test_invalid_ttl_validation() {
        let invalid_ttl = "This is not valid TTL syntax @#$%";

        let converter = XmlToTtlConverter::new();
        let result = converter.validate_ttl(invalid_ttl);

        assert!(result.is_err(), "Invalid TTL should fail validation");
    }
}
