// rust/knhk-etl/src/ingester.rs
// Ingester Pattern - Inspired by Weaver's ingester architecture
// Provides unified interface for multiple input sources (file, stdin, streaming, OTLP)

use crate::error::PipelineError;
use alloc::string::String;
use alloc::vec::Vec;

/// Ingested data result
#[derive(Debug)]
pub struct IngestedData {
    /// Raw data bytes
    pub data: Vec<u8>,
    /// Source identifier
    pub source: String,
    /// Format hint (optional)
    pub format_hint: Option<String>,
    /// Metadata
    pub metadata: alloc::collections::BTreeMap<String, String>,
}

/// Trait for ingesting data from various sources
/// Inspired by Weaver's Ingester pattern
pub trait Ingester: Send + Sync {
    /// Ingest data from source
    fn ingest(&mut self) -> Result<IngestedData, PipelineError>;

    /// Get source identifier
    fn source(&self) -> &str;

    /// Check if ingester supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }
}

/// File-based ingester for RDF/Turtle files
pub struct FileIngester {
    path: String,
    format_hint: Option<String>,
}

impl FileIngester {
    pub fn new(path: String) -> Self {
        Self {
            path,
            format_hint: None,
        }
    }

    pub fn with_format(mut self, format: String) -> Self {
        self.format_hint = Some(format);
        self
    }
}

impl Ingester for FileIngester {
    fn ingest(&mut self) -> Result<IngestedData, PipelineError> {
        #[cfg(feature = "std")]
        {
            use alloc::collections::BTreeMap;
            use std::fs;

            let data = fs::read(&self.path).map_err(|e| {
                PipelineError::IngestError(format!("Failed to read file {}: {}", self.path, e))
            })?;

            let mut metadata = BTreeMap::new();
            metadata.insert("source_type".to_string(), "file".to_string());
            metadata.insert("path".to_string(), self.path.clone());

            Ok(IngestedData {
                data,
                source: self.path.clone(),
                format_hint: self.format_hint.clone(),
                metadata,
            })
        }

        #[cfg(not(feature = "std"))]
        {
            Err(PipelineError::IngestError(
                "File ingester requires std feature".to_string(),
            ))
        }
    }

    fn source(&self) -> &str {
        &self.path
    }
}

/// Stdin-based ingester for streaming input
pub struct StdinIngester {
    format_hint: Option<String>,
}

impl Default for StdinIngester {
    fn default() -> Self {
        Self::new()
    }
}

impl StdinIngester {
    pub fn new() -> Self {
        Self { format_hint: None }
    }

    pub fn with_format(mut self, format: String) -> Self {
        self.format_hint = Some(format);
        self
    }
}

impl Ingester for StdinIngester {
    fn ingest(&mut self) -> Result<IngestedData, PipelineError> {
        #[cfg(feature = "std")]
        {
            use alloc::collections::BTreeMap;
            use std::io::Read;

            let mut data = Vec::new();
            std::io::stdin().read_to_end(&mut data).map_err(|e| {
                PipelineError::IngestError(format!("Failed to read from stdin: {}", e))
            })?;

            let mut metadata = BTreeMap::new();
            metadata.insert("source_type".to_string(), "stdin".to_string());

            Ok(IngestedData {
                data,
                source: "stdin".to_string(),
                format_hint: self.format_hint.clone(),
                metadata,
            })
        }

        #[cfg(not(feature = "std"))]
        {
            Err(PipelineError::IngestError(
                "Stdin ingester requires std feature".to_string(),
            ))
        }
    }

    fn source(&self) -> &str {
        "stdin"
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

/// Memory-based ingester for in-memory data
pub struct MemoryIngester {
    data: Vec<u8>,
    source: String,
    format_hint: Option<String>,
}

impl MemoryIngester {
    pub fn new(data: Vec<u8>, source: String) -> Self {
        Self {
            data,
            source,
            format_hint: None,
        }
    }

    pub fn with_format(mut self, format: String) -> Self {
        self.format_hint = Some(format);
        self
    }
}

impl Ingester for MemoryIngester {
    fn ingest(&mut self) -> Result<IngestedData, PipelineError> {
        use alloc::collections::BTreeMap;

        let mut metadata = BTreeMap::new();
        metadata.insert("source_type".to_string(), "memory".to_string());

        Ok(IngestedData {
            data: self.data.clone(),
            source: self.source.clone(),
            format_hint: self.format_hint.clone(),
            metadata,
        })
    }

    fn source(&self) -> &str {
        &self.source
    }
}

/// Multi-ingester for combining multiple sources
pub struct MultiIngester {
    ingesters: Vec<Box<dyn Ingester>>,
}

impl MultiIngester {
    pub fn new() -> Self {
        Self {
            ingesters: Vec::new(),
        }
    }

    pub fn add_ingester(&mut self, ingester: Box<dyn Ingester>) {
        self.ingesters.push(ingester);
    }

    pub fn ingest_all(&mut self) -> Result<Vec<IngestedData>, PipelineError> {
        let mut results = Vec::new();
        for ingester in &mut self.ingesters {
            results.push(ingester.ingest()?);
        }
        Ok(results)
    }
}

impl Default for MultiIngester {
    fn default() -> Self {
        Self::new()
    }
}
