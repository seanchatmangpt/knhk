use crate::{StateError, StateResult, WorkflowSpec, WorkflowSpecId, Case, CaseId};
use sled::Db;

pub struct StateStore { db: Db }

impl StateStore {
    pub fn new(path: impl AsRef<std::path::Path>) -> StateResult<Self> {
        let db = sled::open(path).map_err(|e| StateError::Persistence(format!("{e:?}")))?;
        Ok(Self { db })
    }

    pub fn save_spec(&self, spec: &WorkflowSpec) -> StateResult<()> {
        let key = format!("spec:{}", spec.id);
        let val = serde_json::to_vec(spec).map_err(|e| StateError::Persistence(e.to_string()))?;
        self.db.insert(key.as_bytes(), val).map_err(|e| StateError::Persistence(format!("{e:?}")))?;
        Ok(())
    }

    pub fn load_spec(&self, id: &WorkflowSpecId) -> StateResult<Option<WorkflowSpec>> {
        let key = format!("spec:{id}");
        let Some(v) = self.db.get(key.as_bytes()).map_err(|e| StateError::Persistence(format!("{e:?}")))? else { return Ok(None) };
        let spec = serde_json::from_slice(&v).map_err(|e| StateError::Persistence(e.to_string()))?;
        Ok(Some(spec))
    }

    pub fn save_case(&self, case: &Case) -> StateResult<()> {
        let key = format!("case:{}", case.id);
        let val = serde_json::to_vec(case).map_err(|e| StateError::Persistence(e.to_string()))?;
        self.db.insert(key.as_bytes(), val).map_err(|e| StateError::Persistence(format!("{e:?}")))?;
        Ok(())
    }

    pub fn load_case(&self, id: &CaseId) -> StateResult<Option<Case>> {
        let key = format!("case:{id}");
        let Some(v) = self.db.get(key.as_bytes()).map_err(|e| StateError::Persistence(format!("{e:?}")))? else { return Ok(None) };
        let case = serde_json::from_slice(&v).map_err(|e| StateError::Persistence(e.to_string()))?;
        Ok(Some(case))
    }

    pub fn list_cases_for_spec(&self, spec_id: &WorkflowSpecId) -> StateResult<Vec<CaseId>> {
        let mut out = vec![];
        for kv in self.db.scan_prefix("case:".as_bytes()) {
            let (_, v) = kv.map_err(|e| StateError::Persistence(format!("{e:?}")))?;
            let case: Case = serde_json::from_slice(&v).map_err(|e| StateError::Persistence(e.to_string()))?;
            if &case.spec_id == spec_id { out.push(case.id); }
        }
        Ok(out)
    }
}
crates/admission