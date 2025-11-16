//! Cloud deployment infrastructure

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{MarketplaceError, Result};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum CloudProvider {
    AWS,
    GCP,
    Azure,
}

impl CloudProvider {
    pub fn name(&self) -> &'static str {
        match self {
            CloudProvider::AWS => "Amazon Web Services",
            CloudProvider::GCP => "Google Cloud Platform",
            CloudProvider::Azure => "Microsoft Azure",
        }
    }

    pub fn default_region(&self) -> &'static str {
        match self {
            CloudProvider::AWS => "us-east-1",
            CloudProvider::GCP => "us-central1",
            CloudProvider::Azure => "eastus",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub id: Uuid,
    pub provider: CloudProvider,
    pub regions: Vec<String>,
    pub docker_registry: String,
}

#[derive(Default)]
pub struct DeploymentManager {
    configs: std::collections::HashMap<Uuid, DeploymentConfig>,
}

impl DeploymentManager {
    pub fn new() -> Self {
        Self {
            configs: std::collections::HashMap::new(),
        }
    }

    pub fn create_deployment(&mut self, provider: CloudProvider, registry: String) -> DeploymentConfig {
        let config = DeploymentConfig {
            id: Uuid::new_v4(),
            provider,
            regions: vec![provider.default_region().to_string()],
            docker_registry: registry,
        };

        self.configs.insert(config.id, config.clone());
        config
    }

    pub fn get_deployment(&self, id: Uuid) -> Option<&DeploymentConfig> {
        self.configs.get(&id)
    }

    pub fn count(&self) -> usize {
        self.configs.len()
    }
}

pub struct KubernetesManifestGenerator;

impl KubernetesManifestGenerator {
    pub fn generate_deployment_manifest(config: &DeploymentConfig) -> String {
        format!(
            "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: knhk-workflow\nspec:\n  image: {}",
            config.docker_registry
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_providers() {
        assert_eq!(CloudProvider::AWS.name(), "Amazon Web Services");
        assert_eq!(CloudProvider::GCP.default_region(), "us-central1");
    }

    #[test]
    fn test_deployment_manager() {
        let mut manager = DeploymentManager::new();
        let config = manager.create_deployment(CloudProvider::AWS, "registry.io".to_string());
        assert_eq!(manager.count(), 1);
        assert!(manager.get_deployment(config.id).is_some());
    }
}
