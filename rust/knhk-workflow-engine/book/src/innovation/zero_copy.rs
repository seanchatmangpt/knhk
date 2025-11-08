//! Zero-copy optimizations
//!
//! Provides zero-copy data structures and operations for hot path operations.

use crate::error::{WorkflowError, WorkflowResult};
use std::borrow::Cow;
use std::ops::Deref;

/// Zero-copy string slice
#[derive(Debug, Clone)]
pub struct ZeroCopyStr<'a> {
    inner: Cow<'a, str>,
}

impl<'a> ZeroCopyStr<'a> {
    /// Create from owned string
    pub fn owned(s: String) -> Self {
        Self {
            inner: Cow::Owned(s),
        }
    }

    /// Create from borrowed string
    pub fn borrowed(s: &'a str) -> Self {
        Self {
            inner: Cow::Borrowed(s),
        }
    }

    /// Check if borrowed
    pub fn is_borrowed(&self) -> bool {
        matches!(self.inner, Cow::Borrowed(_))
    }

    /// Check if owned
    pub fn is_owned(&self) -> bool {
        matches!(self.inner, Cow::Owned(_))
    }
}

impl<'a> Deref for ZeroCopyStr<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> AsRef<str> for ZeroCopyStr<'a> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

/// Zero-copy byte slice
#[derive(Debug, Clone)]
pub struct ZeroCopyBytes<'a> {
    inner: Cow<'a, [u8]>,
}

impl<'a> ZeroCopyBytes<'a> {
    /// Create from owned bytes
    pub fn owned(bytes: Vec<u8>) -> Self {
        Self {
            inner: Cow::Owned(bytes),
        }
    }

    /// Create from borrowed bytes
    pub fn borrowed(bytes: &'a [u8]) -> Self {
        Self {
            inner: Cow::Borrowed(bytes),
        }
    }

    /// Check if borrowed
    pub fn is_borrowed(&self) -> bool {
        matches!(self.inner, Cow::Borrowed(_))
    }

    /// Check if owned
    pub fn is_owned(&self) -> bool {
        matches!(self.inner, Cow::Owned(_))
    }
}

impl<'a> Deref for ZeroCopyBytes<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> AsRef<[u8]> for ZeroCopyBytes<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

/// Zero-copy triple
#[derive(Debug, Clone)]
pub struct ZeroCopyTriple<'a> {
    /// Subject (zero-copy)
    pub subject: ZeroCopyStr<'a>,
    /// Predicate (zero-copy)
    pub predicate: ZeroCopyStr<'a>,
    /// Object (zero-copy)
    pub object: ZeroCopyStr<'a>,
    /// Graph (zero-copy, optional)
    pub graph: Option<ZeroCopyStr<'a>>,
}

impl<'a> ZeroCopyTriple<'a> {
    /// Create from borrowed strings
    pub fn borrowed(
        subject: &'a str,
        predicate: &'a str,
        object: &'a str,
        graph: Option<&'a str>,
    ) -> Self {
        Self {
            subject: ZeroCopyStr::borrowed(subject),
            predicate: ZeroCopyStr::borrowed(predicate),
            object: ZeroCopyStr::borrowed(object),
            graph: graph.map(ZeroCopyStr::borrowed),
        }
    }

    /// Create from owned strings
    pub fn owned(
        subject: String,
        predicate: String,
        object: String,
        graph: Option<String>,
    ) -> Self {
        Self {
            subject: ZeroCopyStr::owned(subject),
            predicate: ZeroCopyStr::owned(predicate),
            object: ZeroCopyStr::owned(object),
            graph: graph.map(ZeroCopyStr::owned),
        }
    }

    /// Check if all fields are borrowed
    pub fn is_fully_borrowed(&self) -> bool {
        self.subject.is_borrowed()
            && self.predicate.is_borrowed()
            && self.object.is_borrowed()
            && self.graph.as_ref().map_or(true, |g| g.is_borrowed())
    }
}

/// Zero-copy triple batch
pub struct ZeroCopyTripleBatch<'a> {
    /// Triples (zero-copy)
    triples: Vec<ZeroCopyTriple<'a>>,
    /// Maximum size
    max_size: usize,
}

impl<'a> ZeroCopyTripleBatch<'a> {
    /// Create new batch
    pub fn new(max_size: usize) -> Self {
        Self {
            triples: Vec::new(),
            max_size,
        }
    }

    /// Add triple (zero-copy)
    pub fn add(&mut self, triple: ZeroCopyTriple<'a>) -> WorkflowResult<()> {
        if self.triples.len() >= self.max_size {
            return Err(WorkflowError::Validation(format!(
                "Batch size {} exceeds maximum {}",
                self.triples.len(),
                self.max_size
            )));
        }
        self.triples.push(triple);
        Ok(())
    }

    /// Get triples (zero-copy)
    pub fn triples(&self) -> &[ZeroCopyTriple<'a>] {
        &self.triples
    }

    /// Get batch size
    pub fn len(&self) -> usize {
        self.triples.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.triples.is_empty()
    }
}
