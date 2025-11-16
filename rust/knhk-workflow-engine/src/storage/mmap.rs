//! Memory-mapped workflow storage for large workflow specifications
//!
//! Uses memory mapping to efficiently load and access large workflow files
//! without loading entire files into memory.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpecId;
use memmap2::{Mmap, MmapOptions};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Memory-mapped workflow specification store
pub struct MmapWorkflowStore {
    /// Memory-mapped file
    mmap: Mmap,
    /// File path
    path: PathBuf,
    /// Workflow index: maps spec ID to (offset, length)
    index: HashMap<WorkflowSpecId, (usize, usize)>,
}

impl MmapWorkflowStore {
    /// Create a new memory-mapped workflow store
    pub fn new<P: AsRef<Path>>(path: P) -> WorkflowResult<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path).map_err(|e| {
            WorkflowError::Internal(format!("Failed to open workflow file: {}", e))
        })?;

        let mmap = unsafe {
            MmapOptions::new().map(&file).map_err(|e| {
                WorkflowError::Internal(format!("Failed to mmap workflow file: {}", e))
            })?
        };

        // Build index from file contents
        let index = Self::build_index(&mmap)?;

        Ok(Self { mmap, path, index })
    }

    /// Get workflow specification by ID
    pub fn get_workflow(&self, id: &WorkflowSpecId) -> Option<&[u8]> {
        self.index.get(id).map(|(offset, length)| {
            let end = offset + length;
            &self.mmap[*offset..end]
        })
    }

    /// Get workflow specification as string
    pub fn get_workflow_str(&self, id: &WorkflowSpecId) -> Option<&str> {
        self.get_workflow(id)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
    }

    /// List all workflow IDs in the store
    pub fn list_workflows(&self) -> Vec<WorkflowSpecId> {
        self.index.keys().copied().collect()
    }

    /// Get number of workflows in store
    pub fn workflow_count(&self) -> usize {
        self.index.len()
    }

    /// Get total size of memory-mapped file
    pub fn total_size(&self) -> usize {
        self.mmap.len()
    }

    /// Build index from file contents
    ///
    /// Expected format: Each workflow has a header:
    /// - 32 bytes: WorkflowSpecId (UUID)
    /// - 8 bytes: length (u64 little-endian)
    /// - N bytes: workflow data
    fn build_index(mmap: &Mmap) -> WorkflowResult<HashMap<WorkflowSpecId, (usize, usize)>> {
        let mut index = HashMap::new();
        let mut offset = 0usize;

        while offset + 40 <= mmap.len() {
            // Read workflow ID (first 16 bytes of UUID)
            let id_bytes = &mmap[offset..offset + 16];
            let id = uuid::Uuid::from_slice(id_bytes)
                .map_err(|e| WorkflowError::Internal(format!("Invalid workflow ID: {}", e)))?;
            let spec_id = WorkflowSpecId(id);

            // Read length (next 8 bytes)
            let length_bytes: [u8; 8] = mmap[offset + 16..offset + 24]
                .try_into()
                .map_err(|_| WorkflowError::Internal("Invalid length field".to_string()))?;
            let length = u64::from_le_bytes(length_bytes) as usize;

            // Store index entry
            let data_offset = offset + 24;
            index.insert(spec_id, (data_offset, length));

            // Move to next workflow
            offset = data_offset + length;
        }

        Ok(index)
    }
}

/// Read-only memory-mapped workflow reader with caching
pub struct MmapWorkflowReader {
    stores: HashMap<PathBuf, Arc<MmapWorkflowStore>>,
}

impl MmapWorkflowReader {
    /// Create a new workflow reader
    pub fn new() -> Self {
        Self {
            stores: HashMap::new(),
        }
    }

    /// Load a workflow store from path (cached)
    pub fn load_store<P: AsRef<Path>>(&mut self, path: P) -> WorkflowResult<Arc<MmapWorkflowStore>> {
        let path = path.as_ref().to_path_buf();

        if let Some(store) = self.stores.get(&path) {
            return Ok(Arc::clone(store));
        }

        let store = Arc::new(MmapWorkflowStore::new(&path)?);
        self.stores.insert(path.clone(), Arc::clone(&store));
        Ok(store)
    }

    /// Get workflow from any loaded store
    pub fn get_workflow(&self, id: &WorkflowSpecId) -> Option<&[u8]> {
        for store in self.stores.values() {
            if let Some(workflow) = store.get_workflow(id) {
                return Some(workflow);
            }
        }
        None
    }

    /// Clear all cached stores
    pub fn clear_cache(&mut self) {
        self.stores.clear();
    }

    /// Get number of cached stores
    pub fn cached_store_count(&self) -> usize {
        self.stores.len()
    }
}

impl Default for MmapWorkflowReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    fn create_test_workflow_file() -> WorkflowResult<NamedTempFile> {
        let mut file = NamedTempFile::new()
            .map_err(|e| WorkflowError::Internal(format!("Failed to create temp file: {}", e)))?;

        // Write workflow 1
        let id1 = Uuid::new_v4();
        let data1 = b"workflow spec 1 data";
        file.write_all(id1.as_bytes())
            .map_err(|e| WorkflowError::Internal(format!("Write failed: {}", e)))?;
        file.write_all(&(data1.len() as u64).to_le_bytes())
            .map_err(|e| WorkflowError::Internal(format!("Write failed: {}", e)))?;
        file.write_all(data1)
            .map_err(|e| WorkflowError::Internal(format!("Write failed: {}", e)))?;

        // Write workflow 2
        let id2 = Uuid::new_v4();
        let data2 = b"workflow spec 2 data longer";
        file.write_all(id2.as_bytes())
            .map_err(|e| WorkflowError::Internal(format!("Write failed: {}", e)))?;
        file.write_all(&(data2.len() as u64).to_le_bytes())
            .map_err(|e| WorkflowError::Internal(format!("Write failed: {}", e)))?;
        file.write_all(data2)
            .map_err(|e| WorkflowError::Internal(format!("Write failed: {}", e)))?;

        file.flush()
            .map_err(|e| WorkflowError::Internal(format!("Flush failed: {}", e)))?;

        Ok(file)
    }

    #[test]
    fn test_mmap_workflow_store() {
        let file = create_test_workflow_file().unwrap();
        let store = MmapWorkflowStore::new(file.path()).unwrap();

        assert_eq!(store.workflow_count(), 2);
        assert!(store.total_size() > 0);

        let workflows = store.list_workflows();
        assert_eq!(workflows.len(), 2);
    }

    #[test]
    fn test_mmap_workflow_reader() {
        let file = create_test_workflow_file().unwrap();
        let mut reader = MmapWorkflowReader::new();

        let store = reader.load_store(file.path()).unwrap();
        assert_eq!(store.workflow_count(), 2);

        // Should use cached version
        let store2 = reader.load_store(file.path()).unwrap();
        assert_eq!(Arc::strong_count(&store), 3); // reader + store + store2

        assert_eq!(reader.cached_store_count(), 1);

        reader.clear_cache();
        assert_eq!(reader.cached_store_count(), 0);
    }
}
