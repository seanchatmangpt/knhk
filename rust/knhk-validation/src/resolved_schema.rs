// knhk-validation: Resolved Schema
// Self-contained resolved schema pattern for RDF schemas with version management
// Inspired by Weaver's ResolvedTelemetrySchema

#![cfg(feature = "schema-resolution")]

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use serde::{Deserialize, Serialize};

/// Schema version identifier
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
}

impl SchemaVersion {
    /// Create new schema version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// Parse version from string (e.g., "1.2.3")
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", s));
        }
        Ok(Self {
            major: parts[0].parse().map_err(|_| format!("Invalid major version: {}", parts[0]))?,
            minor: parts[1].parse().map_err(|_| format!("Invalid minor version: {}", parts[1]))?,
            patch: parts[2].parse().map_err(|_| format!("Invalid patch version: {}", parts[2]))?,
        })
    }

    /// Format as string
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Schema dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDependency {
    /// Dependency schema ID
    pub schema_id: String,
    /// Dependency version
    pub version: SchemaVersion,
    /// Dependency URL or path
    pub url: String,
}

/// Schema catalog entry (shared definitions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCatalogEntry {
    /// Entry ID
    pub id: String,
    /// Entry type (predicate, class, property, etc.)
    pub entry_type: String,
    /// Entry definition
    pub definition: BTreeMap<String, String>,
}

/// Schema catalog (shared definitions across schemas)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCatalog {
    /// Catalog entries
    pub entries: Vec<SchemaCatalogEntry>,
}

impl SchemaCatalog {
    /// Create new catalog
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add catalog entry
    pub fn add_entry(&mut self, entry: SchemaCatalogEntry) {
        self.entries.push(entry);
    }

    /// Find entry by ID
    pub fn find_entry(&self, id: &str) -> Option<&SchemaCatalogEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
}

impl Default for SchemaCatalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolved RDF schema
/// A resolved schema is self-contained and doesn't contain any external references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRdfSchema {
    /// Schema format version
    pub file_format: String,
    /// Schema URL that this file is published at
    pub schema_url: String,
    /// The ID of the schema
    pub schema_id: String,
    /// Schema version
    pub version: SchemaVersion,
    /// Schema name
    pub name: String,
    /// Schema description
    pub description: Option<String>,
    /// Catalog of shared definitions
    pub catalog: SchemaCatalog,
    /// Schema definitions (predicates, classes, properties)
    pub definitions: BTreeMap<String, BTreeMap<String, String>>,
    /// List of dependencies (resolved)
    pub dependencies: Vec<SchemaDependency>,
    /// Schema metadata
    pub metadata: BTreeMap<String, String>,
}

impl ResolvedRdfSchema {
    /// Create new resolved schema
    pub fn new(
        schema_id: String,
        version: SchemaVersion,
        name: String,
        schema_url: String,
    ) -> Self {
        Self {
            file_format: "1.0".to_string(),
            schema_url,
            schema_id,
            version,
            name,
            description: None,
            catalog: SchemaCatalog::new(),
            definitions: BTreeMap::new(),
            dependencies: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    /// Add schema definition
    pub fn add_definition(&mut self, key: String, definition: BTreeMap<String, String>) {
        self.definitions.insert(key, definition);
    }

    /// Add dependency
    pub fn add_dependency(&mut self, dependency: SchemaDependency) {
        self.dependencies.push(dependency);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Check if schema is compatible with another version
    pub fn is_compatible_with(&self, other: &SchemaVersion) -> bool {
        // Same major version means compatible
        self.version.major == other.major
    }

    /// Get schema identifier (schema_id:version)
    pub fn identifier(&self) -> String {
        format!("{}:{}", self.schema_id, self.version.to_string())
    }
}

/// Schema resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaResolutionResult {
    /// Resolved schema
    pub schema: ResolvedRdfSchema,
    /// Resolution lineage (which schemas were resolved)
    pub lineage: Vec<String>,
    /// Resolution errors (if any)
    pub errors: Vec<String>,
}

impl SchemaResolutionResult {
    /// Create successful resolution result
    pub fn success(schema: ResolvedRdfSchema, lineage: Vec<String>) -> Self {
        Self {
            schema,
            lineage,
            errors: Vec::new(),
        }
    }

    /// Create failed resolution result
    pub fn failure(schema: ResolvedRdfSchema, errors: Vec<String>) -> Self {
        Self {
            schema,
            lineage: Vec::new(),
            errors,
        }
    }

    /// Check if resolution was successful
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version() {
        let v1 = SchemaVersion::new(1, 2, 3);
        assert_eq!(v1.major, 1);
        assert_eq!(v1.minor, 2);
        assert_eq!(v1.patch, 3);
        assert_eq!(v1.to_string(), "1.2.3");
    }

    #[test]
    fn test_schema_version_parse() {
        let v1 = match SchemaVersion::parse("1.2.3") {
            Ok(v) => v,
            Err(e) => panic!("Failed to parse version: {}", e),
        };
        assert_eq!(v1.major, 1);
        assert_eq!(v1.minor, 2);
        assert_eq!(v1.patch, 3);
    }

    #[test]
    fn test_resolved_schema() {
        let mut schema = ResolvedRdfSchema::new(
            "test-schema".to_string(),
            SchemaVersion::new(1, 0, 0),
            "Test Schema".to_string(),
            "https://example.com/schema".to_string(),
        );
        schema.add_metadata("author".to_string(), "Test Author".to_string());
        assert_eq!(schema.schema_id, "test-schema");
        assert_eq!(schema.version.major, 1);
    }

    #[test]
    fn test_schema_compatibility() {
        let schema = ResolvedRdfSchema::new(
            "test-schema".to_string(),
            SchemaVersion::new(1, 0, 0),
            "Test Schema".to_string(),
            "https://example.com/schema".to_string(),
        );
        assert!(schema.is_compatible_with(&SchemaVersion::new(1, 1, 0)));
        assert!(!schema.is_compatible_with(&SchemaVersion::new(2, 0, 0)));
    }

    #[test]
    fn test_schema_catalog() {
        let mut catalog = SchemaCatalog::new();
        catalog.add_entry(SchemaCatalogEntry {
            id: "pred1".to_string(),
            entry_type: "predicate".to_string(),
            definition: BTreeMap::new(),
        });
        assert_eq!(catalog.entries.len(), 1);
        assert!(catalog.find_entry("pred1").is_some());
    }
}

