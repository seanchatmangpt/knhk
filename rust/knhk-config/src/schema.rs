// Configuration schema and types

use serde::{Deserialize, Serialize};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnhkConfig {
    #[serde(default)]
    pub knhk: KnhkSection,
    #[serde(default)]
    pub connectors: BTreeMap<String, ConnectorConfig>,
    #[serde(default)]
    pub epochs: BTreeMap<String, EpochConfig>,
    #[serde(default)]
    pub hooks: HooksSection,
    #[serde(default)]
    pub routes: BTreeMap<String, RouteConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnhkSection {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_context")]
    pub context: String,
}

fn default_version() -> String {
    "0.5.0".to_string()
}

fn default_context() -> String {
    "default".to_string()
}

impl Default for KnhkSection {
    fn default() -> Self {
        Self {
            version: default_version(),
            context: default_context(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub bootstrap_servers: Vec<String>,
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub schema: String,
    #[serde(default = "default_max_run_len")]
    pub max_run_len: u64,
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: u64,
}

fn default_max_run_len() -> u64 {
    8
}

fn default_max_batch_size() -> u64 {
    1000
}

impl Default for ConnectorConfig {
    fn default() -> Self {
        Self {
            r#type: String::new(),
            bootstrap_servers: Vec::new(),
            topic: String::new(),
            schema: String::new(),
            max_run_len: default_max_run_len(),
            max_batch_size: default_max_batch_size(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochConfig {
    #[serde(default = "default_tau")]
    pub tau: u64,
    #[serde(default = "default_ordering")]
    pub ordering: String,
}

fn default_tau() -> u64 {
    8
}

fn default_ordering() -> String {
    "deterministic".to_string()
}

impl Default for EpochConfig {
    fn default() -> Self {
        Self {
            tau: default_tau(),
            ordering: default_ordering(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksSection {
    #[serde(default = "default_max_count")]
    pub max_count: u64,
}

fn default_max_count() -> u64 {
    100
}

impl Default for HooksSection {
    fn default() -> Self {
        Self {
            max_count: default_max_count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub target: String,
    #[serde(default = "default_encode")]
    pub encode: String,
}

fn default_encode() -> String {
    "json-ld".to_string()
}

impl Default for RouteConfig {
    fn default() -> Self {
        Self {
            kind: String::new(),
            target: String::new(),
            encode: default_encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    FileNotFound,
    ParseError(String),
    ValidationError(String),
    IoError(String),
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConfigError::FileNotFound => write!(f, "Configuration file not found"),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

