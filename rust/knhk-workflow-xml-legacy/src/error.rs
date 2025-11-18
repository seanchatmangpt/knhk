//! Error types for legacy XML parsing

use thiserror::Error;

/// Result type for legacy operations
pub type LegacyResult<T> = Result<T, LegacyError>;

/// Errors that can occur during XML to TTL conversion
#[derive(Error, Debug)]
pub enum LegacyError {
    /// XML parsing error
    #[error("XML parsing failed: {0}")]
    XmlParse(String),

    /// TTL serialization error
    #[error("TTL serialization failed: {0}")]
    TtlSerialize(String),

    /// Validation error
    #[error("Validation failed: {0}")]
    Validation(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(String),

    /// Unsupported XML feature
    #[error("Unsupported XML feature: {0}")]
    Unsupported(String),

    /// Missing required element
    #[error("Missing required XML element: {0}")]
    MissingElement(String),

    /// Invalid attribute value
    #[error("Invalid attribute value for {0}: {1}")]
    InvalidAttribute(String, String),
}

impl From<std::io::Error> for LegacyError {
    fn from(e: std::io::Error) -> Self {
        LegacyError::Io(e.to_string())
    }
}

impl From<quick_xml::Error> for LegacyError {
    fn from(e: quick_xml::Error) -> Self {
        LegacyError::XmlParse(e.to_string())
    }
}

impl From<roxmltree::Error> for LegacyError {
    fn from(e: roxmltree::Error) -> Self {
        LegacyError::XmlParse(e.to_string())
    }
}
