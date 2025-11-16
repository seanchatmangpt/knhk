//! OpenAPI Generator (Π_apis)
//!
//! Generates OpenAPI 3.0 specifications for REST APIs from ontology snapshots.

use crate::determinism::blake3_hash;
use crate::{ProjectionError, Result};
use knhk_ontology::SigmaSnapshot;
use serde_json::{json, Value};
use std::collections::HashSet;
use tracing::instrument;

/// Output from OpenAPI generation
#[derive(Clone, Debug)]
pub struct OpenApiOutput {
    /// OpenAPI 3.0 specification (YAML format)
    pub openapi_spec: String,

    /// Generated API paths
    pub paths: Vec<String>,

    /// Generated schema names
    pub schemas: Vec<String>,

    /// Content hash (for determinism verification)
    pub hash: [u8; 32],
}

/// Generates OpenAPI specifications from ontology snapshots
pub struct OpenApiGenerator;

impl OpenApiGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate OpenAPI specification from snapshot
    #[instrument(skip(self, snapshot))]
    pub async fn generate(&self, snapshot: &SigmaSnapshot) -> Result<OpenApiOutput> {
        let mut spec = json!({
            "openapi": "3.0.0",
            "info": {
                "title": "KNHK API",
                "version": "1.0.0",
                "description": "Auto-generated API from ontology snapshot"
            },
            "paths": {},
            "components": {
                "schemas": {},
                "securitySchemes": {
                    "bearerAuth": {
                        "type": "http",
                        "scheme": "bearer",
                        "bearerFormat": "JWT"
                    }
                }
            },
            "security": [
                {
                    "bearerAuth": []
                }
            ]
        });

        let mut paths = Vec::new();
        let mut schemas = Vec::new();

        // Extract unique classes from RDF triples
        let classes = self.extract_classes(snapshot)?;

        for class in &classes {
            // Generate CRUD paths for each class
            let path = format!("/{}", class.to_lowercase());
            let item_path = format!("/{}/:id", class.to_lowercase());

            paths.push(path.clone());
            paths.push(item_path.clone());
            schemas.push(class.clone());

            // Add paths to spec
            self.add_crud_paths(&mut spec, class, &path, &item_path)?;

            // Add schema to spec
            self.add_schema(&mut spec, class)?;
        }

        // Convert to YAML
        let openapi_yaml = serde_yaml::to_string(&spec)
            .map_err(|e| ProjectionError::Serialization(e.to_string()))?;

        let hash = blake3_hash(openapi_yaml.as_bytes());

        Ok(OpenApiOutput {
            openapi_spec: openapi_yaml,
            paths,
            schemas,
            hash,
        })
    }

    /// Extract unique classes from snapshot
    fn extract_classes(&self, snapshot: &SigmaSnapshot) -> Result<Vec<String>> {
        let triples = snapshot.all_triples();
        let mut classes = HashSet::new();

        for triple in &triples {
            if triple.predicate == "rdf:type" {
                classes.insert(triple.object.clone());
            }
        }

        // If no explicit types, use "Resource" as generic
        if classes.is_empty() {
            classes.insert("Resource".to_string());
        }

        let mut sorted: Vec<_> = classes.into_iter().collect();
        sorted.sort();
        Ok(sorted)
    }

    /// Add CRUD paths for a class
    fn add_crud_paths(
        &self,
        spec: &mut Value,
        class: &str,
        collection_path: &str,
        item_path: &str,
    ) -> Result<()> {
        // Collection endpoints (list, create)
        spec["paths"][collection_path] = json!({
            "get": {
                "summary": format!("List all {}", class),
                "operationId": format!("list{}", class),
                "responses": {
                    "200": {
                        "description": "Success",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "array",
                                    "items": {
                                        "$ref": format!("#/components/schemas/{}", class)
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "post": {
                "summary": format!("Create new {}", class),
                "operationId": format!("create{}", class),
                "requestBody": {
                    "required": true,
                    "content": {
                        "application/json": {
                            "schema": {
                                "$ref": format!("#/components/schemas/{}", class)
                            }
                        }
                    }
                },
                "responses": {
                    "201": {
                        "description": "Created",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": format!("#/components/schemas/{}", class)
                                }
                            }
                        }
                    }
                }
            }
        });

        // Item endpoints (get, update, delete)
        spec["paths"][item_path] = json!({
            "get": {
                "summary": format!("Get {} by ID", class),
                "operationId": format!("get{}", class),
                "parameters": [
                    {
                        "name": "id",
                        "in": "path",
                        "required": true,
                        "schema": {
                            "type": "string"
                        }
                    }
                ],
                "responses": {
                    "200": {
                        "description": "Success",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": format!("#/components/schemas/{}", class)
                                }
                            }
                        }
                    },
                    "404": {
                        "description": "Not found"
                    }
                }
            },
            "put": {
                "summary": format!("Update {} by ID", class),
                "operationId": format!("update{}", class),
                "parameters": [
                    {
                        "name": "id",
                        "in": "path",
                        "required": true,
                        "schema": {
                            "type": "string"
                        }
                    }
                ],
                "requestBody": {
                    "required": true,
                    "content": {
                        "application/json": {
                            "schema": {
                                "$ref": format!("#/components/schemas/{}", class)
                            }
                        }
                    }
                },
                "responses": {
                    "200": {
                        "description": "Success",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": format!("#/components/schemas/{}", class)
                                }
                            }
                        }
                    }
                }
            },
            "delete": {
                "summary": format!("Delete {} by ID", class),
                "operationId": format!("delete{}", class),
                "parameters": [
                    {
                        "name": "id",
                        "in": "path",
                        "required": true,
                        "schema": {
                            "type": "string"
                        }
                    }
                ],
                "responses": {
                    "204": {
                        "description": "Deleted"
                    }
                }
            }
        });

        Ok(())
    }

    /// Add schema for a class
    fn add_schema(&self, spec: &mut Value, class: &str) -> Result<()> {
        spec["components"]["schemas"][class] = json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "Unique identifier"
                }
            }
        });

        Ok(())
    }
}

impl Default for OpenApiGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knhk_ontology::{SnapshotMetadata, Triple, TripleStore};

    #[tokio::test]
    async fn test_generate_openapi_spec() {
        let mut store = TripleStore::new();
        store.add(Triple::new("company1", "rdf:type", "Company"));
        store.add(Triple::new("company1", "name", "TechCorp"));

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default()).unwrap();

        let generator = OpenApiGenerator::new();
        let output = generator.generate(&snapshot).await.unwrap();

        assert!(!output.openapi_spec.is_empty());
        assert!(output.openapi_spec.contains("openapi: 3.0.0"));
        assert!(output.openapi_spec.contains("Company"));
        assert!(output.paths.contains(&"/company".to_string()));
        assert!(output.schemas.contains(&"Company".to_string()));
    }
}
