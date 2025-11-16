//! Workflow template marketplace

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{MarketplaceError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub content: String,
    pub dependencies: Vec<String>,
    pub rating: f32,
    pub rating_count: u32,
    pub download_count: u32,
}

impl WorkflowTemplate {
    pub fn new(name: String, version: String, description: String, author: String, content: String) -> Result<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            name,
            version,
            description,
            author,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            content,
            dependencies: Vec::new(),
            rating: 0.0,
            rating_count: 0,
            download_count: 0,
        })
    }

    pub fn add_dependency(&mut self, dep: String) {
        self.dependencies.push(dep);
    }

    pub fn add_rating(&mut self, stars: f32) -> Result<()> {
        if !(0.0..=5.0).contains(&stars) {
            return Err(MarketplaceError::Marketplace("Invalid rating".to_string()));
        }

        let total = self.rating * self.rating_count as f32 + stars;
        self.rating_count += 1;
        self.rating = total / self.rating_count as f32;

        Ok(())
    }

    pub fn increment_download(&mut self) {
        self.download_count += 1;
    }
}

pub struct TemplateRegistry {
    templates: HashMap<Uuid, WorkflowTemplate>,
    by_name: HashMap<String, Vec<Uuid>>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    pub fn register(&mut self, template: WorkflowTemplate) -> Result<Uuid> {
        let id = template.id;
        self.by_name
            .entry(template.name.clone())
            .or_insert_with(Vec::new)
            .push(id);

        self.templates.insert(id, template);
        Ok(id)
    }

    pub fn find(&self, name: &str, _version: Option<&str>) -> Option<&WorkflowTemplate> {
        let ids = self.by_name.get(name)?;
        ids.iter().find_map(|id| self.templates.get(id))
    }

    pub fn list_all(&self) -> Vec<&WorkflowTemplate> {
        self.templates.values().collect()
    }

    pub fn count(&self) -> usize {
        self.templates.len()
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TemplatePublisher {
    registry: TemplateRegistry,
    versions: HashMap<String, Vec<String>>,
}

impl TemplatePublisher {
    pub fn new() -> Self {
        Self {
            registry: TemplateRegistry::new(),
            versions: HashMap::new(),
        }
    }

    pub fn publish(&mut self, template: WorkflowTemplate) -> Result<Uuid> {
        let versions = self.versions.entry(template.name.clone()).or_insert_with(Vec::new);
        if versions.contains(&template.version) {
            return Err(MarketplaceError::Marketplace("Version already exists".to_string()));
        }

        versions.push(template.version.clone());
        let id = self.registry.register(template)?;
        Ok(id)
    }

    pub fn get_versions(&self, name: &str) -> Option<Vec<&str>> {
        self.versions.get(name).map(|v| v.iter().map(|s| s.as_str()).collect())
    }

    pub fn registry(&self) -> &TemplateRegistry {
        &self.registry
    }
}

impl Default for TemplatePublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = WorkflowTemplate::new(
            "test".to_string(),
            "1.0.0".to_string(),
            "Test".to_string(),
            "Author".to_string(),
            "content".to_string(),
        );

        assert!(template.is_ok());
    }

    #[test]
    fn test_registry() {
        let mut registry = TemplateRegistry::new();
        let template = WorkflowTemplate::new(
            "test".to_string(),
            "1.0.0".to_string(),
            "Test".to_string(),
            "Author".to_string(),
            "content".to_string(),
        )
        .unwrap();

        assert!(registry.register(template).is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_publisher() {
        let mut publisher = TemplatePublisher::new();
        let template = WorkflowTemplate::new(
            "test".to_string(),
            "1.0.0".to_string(),
            "Test".to_string(),
            "Author".to_string(),
            "content".to_string(),
        )
        .unwrap();

        assert!(publisher.publish(template).is_ok());
    }
}
