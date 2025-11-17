//! μ-Kernel Core - Main Entry Point
//!
//! Integrates all components into the complete μ-kernel

use crate::guards::GuardRegistry;
use crate::isa::{GuardContext, MuInstruction, TaskResult};
use crate::mape::MapeKColon;
use crate::receipts::{Receipt, ReceiptChain};
use crate::sigma::{SigmaCompiled, SigmaPointer};
use crate::timing::{TickBudget, TickCounter};
use alloc::format;
use alloc::string::String;

/// μ-Kernel state
pub struct MuState {
    /// Active Σ* pointer
    pub sigma: &'static SigmaPointer,
    /// Guard registry
    pub guards: GuardRegistry,
    /// Receipt chain
    pub receipts: ReceiptChain,
    /// Tick counter
    pub ticks: TickCounter,
}

/// μ-Kernel (complete system)
pub struct MuKernel {
    /// Core state
    state: MuState,
    /// MAPE-K control loop
    mape: MapeKColon,
}

impl MuKernel {
    /// Create a new μ-kernel
    pub fn new(sigma_ptr: &'static SigmaPointer) -> Self {
        Self {
            state: MuState {
                sigma: sigma_ptr,
                guards: GuardRegistry::new(),
                receipts: ReceiptChain::new(),
                ticks: TickCounter::new(),
            },
            mape: MapeKColon::new(sigma_ptr),
        }
    }

    /// Execute a task (A = μ(O))
    ///
    /// This is the core execution function that implements:
    /// A = μ(O; Σ*) under Q with τ ≤ 8
    pub fn execute_task(
        &mut self,
        task_id: u64,
        observation: &GuardContext,
    ) -> MuResult<TaskResult> {
        // Start timing
        self.state.ticks.start();

        // Create tick budget (Chatman Constant)
        let mut budget = TickBudget::chatman();

        // Execute via ISA
        let result = MuInstruction::eval_task(task_id, observation, &mut budget)
            .map_err(|e| MuError::Execution(format!("{:?}", e)))?;

        // Measure total ticks
        let total_ticks = self.state.ticks.ticks();

        // Create receipt
        let receipt = Receipt::new(
            0, // Will be assigned by chain
            self.state.sigma.load().unwrap().header.hash,
            [0; 32], // Would compute O_in hash
            result.output_hash,
            total_ticks,
            task_id,
            0, // pattern_id from result
        );

        // Store receipt
        self.state.receipts.append(receipt);

        // Feed to MAPE-K
        self.mape.monitor(receipt);

        Ok(result)
    }

    /// Run MAPE-K autonomic cycle
    pub fn run_mape_cycle(&mut self) -> MuResult<crate::mape::MapeKResult> {
        Ok(self.mape.run_cycle())
    }

    /// Get current Σ*
    pub fn current_sigma(&self) -> Option<&'static SigmaCompiled> {
        self.state.sigma.load()
    }
}

/// μ-Kernel result type
pub type MuResult<T> = Result<T, MuError>;

/// μ-Kernel errors
#[derive(Debug, Clone)]
pub enum MuError {
    /// Execution error
    Execution(String),
    /// No active Σ*
    NoSigma,
    /// Tick budget exceeded
    TickBudgetExceeded,
    /// Guard failed
    GuardFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mu_kernel_creation() {
        let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
        let _kernel = MuKernel::new(sigma_ptr);
        // Kernel created successfully
    }

    #[test]
    fn test_mu_kernel_execution() {
        let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
        let mut kernel = MuKernel::new(sigma_ptr);

        let ctx = GuardContext {
            task_id: 1,
            obs_data: 0,
            params: [0; 4],
        };

        // Would execute if Σ* was loaded
        // let result = kernel.execute_task(1, &ctx);
    }
}
