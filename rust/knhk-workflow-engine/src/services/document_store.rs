//! YAWL Document Store Port with TRIZ Hyper-Advanced Patterns
//!
//! This module ports Java YAWL's DocumentStore while applying TRIZ principles:
//! - **Principle 2 (Taking Out)**: Extract document storage to external service
//! - **Principle 17 (Another Dimension)**: Store documents in external dimension
//!
//! # Architecture
//!
//! YAWL document store provides:
//! - Case document attachments
//! - Document versioning
//! - Document metadata
//! - Document retrieval
//!
//! # TRIZ Enhancements
//!
//! - Documents stored externally (Principle 2, 17)
//! - Metadata stored in workflow state (lightweight)

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Document identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub String);

impl DocumentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Document ID
    pub id: DocumentId,
    /// Document name
    pub name: String,
    /// MIME type
    pub mime_type: String,
    /// File size (bytes)
    pub size: u64,
    /// Upload timestamp
    pub uploaded_at: DateTime<Utc>,
    /// Case ID this document belongs to
    pub case_id: CaseId,
    /// Document version
    pub version: u32,
}

/// YAWL Document Store
///
/// Manages document attachments for workflow cases.
///
/// # TRIZ Principle 2: Taking Out
///
/// Documents are stored externally (file system or object storage).
/// Only metadata is stored in workflow state.
///
/// # TRIZ Principle 17: Another Dimension
///
/// Document storage is in external dimension (file system), not in workflow engine.
pub struct DocumentStore {
    /// Storage root directory
    storage_root: PathBuf,
    /// Document metadata by document ID
    documents: Arc<RwLock<HashMap<DocumentId, DocumentMetadata>>>,
    /// Documents by case ID
    case_documents: Arc<RwLock<HashMap<CaseId, Vec<DocumentId>>>>,
}

impl DocumentStore {
    /// Create a new document store
    ///
    /// # Errors
    ///
    /// Returns an error if the storage root directory cannot be created.
    pub fn new<P: AsRef<Path>>(storage_root: P) -> WorkflowResult<Self> {
        let root = storage_root.as_ref().to_path_buf();
        std::fs::create_dir_all(&root).map_err(|e| {
            WorkflowError::Internal(format!("Failed to create document store directory: {}", e))
        })?;

        Ok(Self {
            storage_root: root,
            documents: Arc::new(RwLock::new(HashMap::new())),
            case_documents: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Store a document for a case
    ///
    /// Stores the document file and creates metadata.
    ///
    /// # TRIZ Principle 2: Taking Out
    ///
    /// Document content is stored externally, only metadata in memory.
    pub async fn store_document(
        &self,
        case_id: CaseId,
        name: String,
        mime_type: String,
        content: &[u8],
    ) -> WorkflowResult<DocumentId> {
        let doc_id = DocumentId::new();
        let file_path = self.storage_root.join(format!("{}.bin", doc_id.0));

        // Store file (TRIZ Principle 2: Taking Out)
        tokio::fs::write(&file_path, content).await.map_err(|e| {
            WorkflowError::Internal(format!("Failed to write document file: {}", e))
        })?;

        // Create metadata
        let metadata = DocumentMetadata {
            id: doc_id.clone(),
            name,
            mime_type,
            size: content.len() as u64,
            uploaded_at: Utc::now(),
            case_id,
            version: 1,
        };

        // Store metadata
        let mut docs = self.documents.write().await;
        docs.insert(doc_id.clone(), metadata);

        let mut case_docs = self.case_documents.write().await;
        case_docs.entry(case_id).or_insert_with(Vec::new).push(doc_id.clone());

        info!("DocumentStore: Stored document {} for case {}", doc_id.0, case_id);
        Ok(doc_id)
    }

    /// Retrieve a document
    ///
    /// Returns the document content.
    pub async fn get_document(&self, doc_id: &DocumentId) -> WorkflowResult<Vec<u8>> {
        let docs = self.documents.read().await;
        let metadata = docs.get(doc_id).ok_or_else(|| {
            WorkflowError::DocumentNotFound(format!("Document {} not found", doc_id.0))
        })?;

        let file_path = self.storage_root.join(format!("{}.bin", doc_id.0));
        let content = tokio::fs::read(&file_path).await.map_err(|e| {
            WorkflowError::Internal(format!("Failed to read document file: {}", e))
        })?;

        Ok(content)
    }

    /// Get document metadata
    pub async fn get_metadata(&self, doc_id: &DocumentId) -> Option<DocumentMetadata> {
        let docs = self.documents.read().await;
        docs.get(doc_id).cloned()
    }

    /// List documents for a case
    pub async fn list_case_documents(&self, case_id: CaseId) -> Vec<DocumentMetadata> {
        let case_docs = self.case_documents.read().await;
        let docs = self.documents.read().await;

        case_docs
            .get(&case_id)
            .map(|doc_ids| {
                doc_ids
                    .iter()
                    .filter_map(|id| docs.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Delete a document
    pub async fn delete_document(&self, doc_id: &DocumentId) -> WorkflowResult<()> {
        let mut docs = self.documents.write().await;
        let metadata = docs.remove(doc_id).ok_or_else(|| {
            WorkflowError::DocumentNotFound(format!("Document {} not found", doc_id.0))
        })?;

        // Delete file
        let file_path = self.storage_root.join(format!("{}.bin", doc_id.0));
        tokio::fs::remove_file(&file_path).await.map_err(|e| {
            warn!("Failed to delete document file: {}", e);
            // Continue even if file deletion fails
        })?;

        // Remove from case index
        let mut case_docs = self.case_documents.write().await;
        if let Some(doc_ids) = case_docs.get_mut(&metadata.case_id) {
            doc_ids.retain(|id| id != doc_id);
        }

        info!("DocumentStore: Deleted document {}", doc_id.0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_document_store() {
        let temp_dir = TempDir::new().unwrap();
        let store = DocumentStore::new(temp_dir.path()).unwrap();
        let case_id = CaseId::new();

        let content = b"test document content";
        let doc_id = store
            .store_document(case_id, "test.txt".to_string(), "text/plain".to_string(), content)
            .await
            .unwrap();

        let retrieved = store.get_document(&doc_id).await.unwrap();
        assert_eq!(retrieved, content);

        let metadata = store.get_metadata(&doc_id).await.unwrap();
        assert_eq!(metadata.name, "test.txt");
    }
}

