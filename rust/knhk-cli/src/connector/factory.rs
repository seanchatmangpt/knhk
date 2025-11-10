//! Connector factory - Creates connector instances

#[cfg(feature = "connectors")]
use knhk_connectors::{Connector, DataFormat, SourceType};
use std::collections::BTreeMap;

#[cfg(feature = "connectors")]
/// Connector factory - Creates connector instances from source strings
pub struct ConnectorFactory;

#[cfg(feature = "connectors")]
impl ConnectorFactory {
    /// Create connector from source string
    pub fn create(source: &str) -> Result<Box<dyn Connector>, String> {
        let source_type = Self::parse_source(source)?;

        // Return connector spec as connector
        // The actual connector implementation is handled by knhk-connectors
        // For now, return an error - connector creation needs ConnectorRegistry integration
        match source_type {
            SourceType::Kafka { .. } => {
                Err("Kafka connector creation requires ConnectorRegistry integration".to_string())
            }
            SourceType::Salesforce { .. } => Err(
                "Salesforce connector creation requires ConnectorRegistry integration".to_string(),
            ),
            SourceType::Http { .. } => {
                Err("HTTP connector creation requires ConnectorRegistry integration".to_string())
            }
            SourceType::File { .. } => {
                Err("File connector creation requires ConnectorRegistry integration".to_string())
            }
            SourceType::Sap { .. } => {
                Err("SAP connector creation requires ConnectorRegistry integration".to_string())
            }
        }
    }

    /// Parse source string to SourceType
    pub fn parse_source(source: &str) -> Result<SourceType, String> {
        if let Some(kafka_url) = source.strip_prefix("kafka://") {
            let parts: Vec<&str> = kafka_url.split('/').collect();
            let brokers = if parts.is_empty() {
                vec!["localhost:9092".to_string()]
            } else {
                parts[0].split(',').map(|s| s.to_string()).collect()
            };
            let topic = parts.get(1).unwrap_or(&"triples").to_string();

            Ok(SourceType::Kafka {
                topic,
                format: DataFormat::JsonLd,
                bootstrap_servers: brokers,
            })
        } else if let Some(instance_url) = source.strip_prefix("salesforce://") {
            let instance_url = instance_url.to_string();
            Ok(SourceType::Salesforce {
                instance_url,
                api_version: "v58.0".to_string(),
                object_type: "Triple".to_string(),
            })
        } else if source.starts_with("http://") || source.starts_with("https://") {
            Ok(SourceType::Http {
                url: source.to_string(),
                format: DataFormat::JsonLd,
                headers: BTreeMap::new(),
            })
        } else if source.contains('/') || source.contains('\\') {
            Ok(SourceType::File {
                path: source.to_string(),
                format: DataFormat::RdfTurtle,
            })
        } else {
            Err(format!("Unknown source type: {}", source))
        }
    }
}
