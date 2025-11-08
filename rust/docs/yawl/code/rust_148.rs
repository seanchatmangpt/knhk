impl WorkflowEngine {
    /// Create a new workflow engine
    pub fn new(state_store: StateStore) -> Self;
    
    /// Register a workflow specification (validates for deadlocks)
    pub async fn register_workflow(&self, spec: WorkflowSpec) -> WorkflowResult<()>;
    
    /// Get workflow specification
    pub async fn get_workflow(&self, spec_id: WorkflowSpecId) -> WorkflowResult<WorkflowSpec>;
    
    /// Create a new case
    pub async fn create_case(
        &self,
        spec_id: WorkflowSpecId,
        data: serde_json::Value
    ) -> WorkflowResult<CaseId>;
    
    /// Start a case
    pub async fn start_case(&self, case_id: CaseId) -> WorkflowResult<()>;
    
    /// Execute a case
    pub async fn execute_case(&self, case_id: CaseId) -> WorkflowResult<()>;
    
    /// Cancel a case
    pub async fn cancel_case(&self, case_id: CaseId) -> WorkflowResult<()>;
    
    /// Get case status
    pub async fn get_case(&self, case_id: CaseId) -> WorkflowResult<Case>;
    
    /// Execute a pattern
    pub async fn execute_pattern(
        &self,
        pattern_id: PatternId,
        context: PatternExecutionContext
    ) -> WorkflowResult<PatternExecutionResult>;
    
    /// Get pattern registry
    pub fn pattern_registry(&self) -> &PatternRegistry;
    
    /// Get resource allocator
    pub fn resource_allocator(&self) -> &ResourceAllocator;
    
    /// Get worklet repository
    pub fn worklet_repository(&self) -> &WorkletRepository;
    
    /// Get worklet executor
    pub fn worklet_executor(&self) -> &WorkletExecutor;
}