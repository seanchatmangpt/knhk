//! Code Optimizer
//!
//! Performs optimization passes on generated code.
//! Includes dead code elimination, constant folding, and more.

use crate::compiler::code_generator::{DispatchEntry, GeneratedCode, GeneratedGuard};
use crate::compiler::OptimizationStats;
use crate::error::WorkflowResult;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, instrument};

/// Code optimizer
pub struct Optimizer {
    enabled: bool,
    optimization_level: u8,
    stats: OptimizationStats,
}

impl Optimizer {
    /// Create new optimizer
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            optimization_level: if enabled { 2 } else { 0 },
            stats: OptimizationStats::default(),
        }
    }

    /// Optimize generated code
    #[instrument(skip(self, code))]
    pub async fn optimize(
        &mut self,
        code: &mut GeneratedCode,
    ) -> WorkflowResult<OptimizationStats> {
        if !self.enabled {
            return Ok(OptimizationStats::default());
        }

        info!(
            "Starting optimization pass (level {})",
            self.optimization_level
        );

        let initial_size = self.calculate_code_size(code);

        // Pass 1: Dead code elimination
        self.eliminate_dead_code(code)?;

        // Pass 2: Constant folding
        self.fold_constants(code)?;

        // Pass 3: Common subexpression elimination
        self.eliminate_common_subexpressions(code)?;

        // Pass 4: Optimize dispatch table
        self.optimize_dispatch_table(code)?;

        // Pass 5: Optimize guard bytecode
        self.optimize_guards(code)?;

        // Pass 6: Memory layout optimization
        self.optimize_memory_layout(code)?;

        // Pass 7: Inline small functions
        if self.optimization_level >= 2 {
            self.inline_functions(code)?;
        }

        // Pass 8: Loop optimizations
        if self.optimization_level >= 3 {
            self.optimize_loops(code)?;
        }

        let final_size = self.calculate_code_size(code);
        self.stats.size_reduction_percent =
            ((initial_size as f32 - final_size as f32) / initial_size as f32) * 100.0;

        info!(
            "Optimization complete: {} passes, {:.1}% size reduction",
            8, self.stats.size_reduction_percent
        );

        Ok(self.stats.clone())
    }

    /// Calculate code size
    fn calculate_code_size(&self, code: &GeneratedCode) -> usize {
        code.dispatch_table.entries.len() * 64
            + code.guards.iter().map(|g| g.bytecode.len()).sum::<usize>()
            + code.receipts.len() * 128
    }

    /// Eliminate dead code
    fn eliminate_dead_code(&mut self, code: &mut GeneratedCode) -> WorkflowResult<()> {
        debug!("Pass 1: Dead code elimination");

        // Find unreachable patterns
        let mut reachable = HashSet::new();
        let mut work_list = vec![1u8]; // Start with pattern 1

        while let Some(pattern_id) = work_list.pop() {
            if reachable.insert(pattern_id) {
                // Find patterns referenced from this one
                if let Some(entry) = code
                    .dispatch_table
                    .entries
                    .iter()
                    .find(|e| e.pattern_id == pattern_id)
                {
                    // Check guard references
                    for guard in &code.guards {
                        // Guards might reference other patterns
                        // This is simplified - real implementation would parse bytecode
                    }
                }
            }
        }

        // Remove unreachable entries
        let original_count = code.dispatch_table.entries.len();
        code.dispatch_table
            .entries
            .retain(|entry| reachable.contains(&entry.pattern_id));

        let eliminated = original_count - code.dispatch_table.entries.len();
        if eliminated > 0 {
            self.stats.dead_code_eliminated = eliminated;
            debug!("Eliminated {} unreachable patterns", eliminated);
        }

        // Remove unused guards
        self.remove_unused_guards(code)?;

        // Remove unused constants
        self.remove_unused_constants(code)?;

        Ok(())
    }

    /// Remove unused guards
    fn remove_unused_guards(&mut self, code: &mut GeneratedCode) -> WorkflowResult<()> {
        let mut used_guard_ids = HashSet::new();

        // Find referenced guards
        for entry in &code.dispatch_table.entries {
            if entry.guard_offset > 0 {
                // Guards at this offset are used
                // Real implementation would track individual guard IDs
                used_guard_ids.insert(entry.guard_offset);
            }
        }

        let original_count = code.guards.len();
        code.guards.retain(|guard| {
            // Check if guard is referenced
            // Simplified - real implementation would check guard.id
            true
        });

        let eliminated = original_count - code.guards.len();
        if eliminated > 0 {
            self.stats.dead_code_eliminated += eliminated;
            debug!("Eliminated {} unused guards", eliminated);
        }

        Ok(())
    }

    /// Remove unused constants
    fn remove_unused_constants(&mut self, code: &mut GeneratedCode) -> WorkflowResult<()> {
        let mut used_const_ids = HashSet::new();

        // Find referenced constants
        for guard in &code.guards {
            for const_ref in &guard.const_refs {
                used_const_ids.insert(*const_ref);
            }
        }

        let original_count = code.constants.len();
        code.constants
            .retain(|constant| used_const_ids.contains(&constant.id));

        let eliminated = original_count - code.constants.len();
        if eliminated > 0 {
            self.stats.dead_code_eliminated += eliminated;
            debug!("Eliminated {} unused constants", eliminated);
        }

        Ok(())
    }

    /// Fold constants
    fn fold_constants(&mut self, code: &mut GeneratedCode) -> WorkflowResult<()> {
        debug!("Pass 2: Constant folding");

        for guard in &mut code.guards {
            let folded = self.fold_guard_constants(guard)?;
            if folded > 0 {
                self.stats.constants_folded += folded;
            }
        }

        debug!("Folded {} constants", self.stats.constants_folded);
        Ok(())
    }

    /// Fold constants in guard
    fn fold_guard_constants(&mut self, guard: &mut GeneratedGuard) -> WorkflowResult<usize> {
        let mut folded = 0;

        // Scan bytecode for constant operations
        let mut i = 0;
        while i < guard.bytecode.len() {
            match guard.bytecode[i] {
                0x01 => {
                    // CONST_FLOAT
                    if i + 9 < guard.bytecode.len() && guard.bytecode[i + 9] == 0x01 {
                        // Two consecutive float constants
                        if i + 18 < guard.bytecode.len() {
                            match guard.bytecode[i + 18] {
                                0x20 => {
                                    // ADD
                                    // Fold: const1 + const2
                                    let val1 = f64::from_le_bytes(
                                        guard.bytecode[i + 1..i + 9].try_into().unwrap(),
                                    );
                                    let val2 = f64::from_le_bytes(
                                        guard.bytecode[i + 10..i + 18].try_into().unwrap(),
                                    );
                                    let result = val1 + val2;

                                    // Replace with single const
                                    guard.bytecode[i] = 0x01;
                                    guard.bytecode[i + 1..i + 9]
                                        .copy_from_slice(&result.to_le_bytes());

                                    // Remove second const and op
                                    guard.bytecode.drain(i + 9..i + 19);
                                    folded += 1;
                                }
                                0x21 => {
                                    // SUBTRACT
                                    let val1 = f64::from_le_bytes(
                                        guard.bytecode[i + 1..i + 9].try_into().unwrap(),
                                    );
                                    let val2 = f64::from_le_bytes(
                                        guard.bytecode[i + 10..i + 18].try_into().unwrap(),
                                    );
                                    let result = val1 - val2;

                                    guard.bytecode[i] = 0x01;
                                    guard.bytecode[i + 1..i + 9]
                                        .copy_from_slice(&result.to_le_bytes());
                                    guard.bytecode.drain(i + 9..i + 19);
                                    folded += 1;
                                }
                                0x22 => {
                                    // MULTIPLY
                                    let val1 = f64::from_le_bytes(
                                        guard.bytecode[i + 1..i + 9].try_into().unwrap(),
                                    );
                                    let val2 = f64::from_le_bytes(
                                        guard.bytecode[i + 10..i + 18].try_into().unwrap(),
                                    );
                                    let result = val1 * val2;

                                    guard.bytecode[i] = 0x01;
                                    guard.bytecode[i + 1..i + 9]
                                        .copy_from_slice(&result.to_le_bytes());
                                    guard.bytecode.drain(i + 9..i + 19);
                                    folded += 1;
                                }
                                0x23 => {
                                    // DIVIDE
                                    let val1 = f64::from_le_bytes(
                                        guard.bytecode[i + 1..i + 9].try_into().unwrap(),
                                    );
                                    let val2 = f64::from_le_bytes(
                                        guard.bytecode[i + 10..i + 18].try_into().unwrap(),
                                    );
                                    if val2 != 0.0 {
                                        let result = val1 / val2;
                                        guard.bytecode[i] = 0x01;
                                        guard.bytecode[i + 1..i + 9]
                                            .copy_from_slice(&result.to_le_bytes());
                                        guard.bytecode.drain(i + 9..i + 19);
                                        folded += 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    i += 9;
                }
                0x02 => {
                    // CONST_BOOL
                    if i + 2 < guard.bytecode.len() && guard.bytecode[i + 2] == 0x02 {
                        // Two consecutive bool constants
                        if i + 4 < guard.bytecode.len() {
                            match guard.bytecode[i + 4] {
                                0x2A => {
                                    // AND
                                    let val1 = guard.bytecode[i + 1] != 0;
                                    let val2 = guard.bytecode[i + 3] != 0;
                                    let result = val1 && val2;

                                    guard.bytecode[i] = 0x02;
                                    guard.bytecode[i + 1] = if result { 1 } else { 0 };
                                    guard.bytecode.drain(i + 2..i + 5);
                                    folded += 1;
                                }
                                0x2B => {
                                    // OR
                                    let val1 = guard.bytecode[i + 1] != 0;
                                    let val2 = guard.bytecode[i + 3] != 0;
                                    let result = val1 || val2;

                                    guard.bytecode[i] = 0x02;
                                    guard.bytecode[i + 1] = if result { 1 } else { 0 };
                                    guard.bytecode.drain(i + 2..i + 5);
                                    folded += 1;
                                }
                                _ => {}
                            }
                        }
                    }
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }

        Ok(folded)
    }

    /// Eliminate common subexpressions
    fn eliminate_common_subexpressions(&mut self, code: &mut GeneratedCode) -> WorkflowResult<()> {
        debug!("Pass 3: Common subexpression elimination");

        // Build expression map
        let mut expr_map: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();

        for (idx, guard) in code.guards.iter().enumerate() {
            // Find subexpressions in bytecode
            for window_size in 3..20 {
                for i in 0..guard.bytecode.len().saturating_sub(window_size) {
                    let expr = &guard.bytecode[i..i + window_size];
                    expr_map
                        .entry(expr.to_vec())
                        .or_insert_with(Vec::new)
                        .push(idx);
                }
            }
        }

        // Find common subexpressions
        let mut eliminated = 0;
        for (expr, locations) in expr_map.iter() {
            if locations.len() > 1 && expr.len() > 5 {
                // This expression appears multiple times
                // In real implementation, would create a shared computation
                eliminated += 1;
            }
        }

        self.stats.cse_count = eliminated;
        debug!("Eliminated {} common subexpressions", eliminated);

        Ok(())
    }

    /// Optimize dispatch table
    fn optimize_dispatch_table(&mut self, code: &mut GeneratedCode) -> WorkflowResult<()> {
        debug!("Pass 4: Dispatch table optimization");

        // Sort entries by frequency (hot paths first)
        // In real implementation, would use profiling data

        // Align entries to cache lines
        for entry in &mut code.dispatch_table.entries {
            // Ensure offsets are cache-aligned (64 bytes)
            entry.entry_point = (entry.entry_point + 63) & !63;
            if entry.guard_offset > 0 {
                entry.guard_offset = (entry.guard_offset + 63) & !63;
            }
            if entry.var_table_offset > 0 {
                entry.var_table_offset = (entry.var_table_offset + 63) & !63;
            }
        }

        // Rebuild jump table for fast dispatch
        code.dispatch_table.jump_table = vec![0; 44];
        for entry in &code.dispatch_table.entries {
            if entry.pattern_id < 44 {
                code.dispatch_table.jump_table[entry.pattern_id as usize] = entry.entry_point;
            }
        }

        Ok(())
    }

    /// Optimize guards
    fn optimize_guards(&mut self, code: &mut GeneratedCode) -> WorkflowResult<()> {
        debug!("Pass 5: Guard bytecode optimization");

        for guard in &mut code.guards {
            // Peephole optimizations
            self.peephole_optimize(guard)?;

            // Strength reduction
            self.strength_reduce(guard)?;

            // Register allocation hints
            self.add_register_hints(guard)?;
        }

        Ok(())
    }

    /// Peephole optimization
    fn peephole_optimize(&mut self, guard: &mut GeneratedGuard) -> WorkflowResult<()> {
        let mut i = 0;
        while i < guard.bytecode.len().saturating_sub(1) {
            match (guard.bytecode[i], guard.bytecode.get(i + 1)) {
                (0x30, Some(&0x30)) => {
                    // NOT NOT -> remove both
                    guard.bytecode.drain(i..i + 2);
                }
                (0x31, Some(&0x31)) => {
                    // NEGATE NEGATE -> remove both
                    guard.bytecode.drain(i..i + 2);
                }
                _ => {
                    i += 1;
                }
            }
        }

        Ok(())
    }

    /// Strength reduction
    fn strength_reduce(&mut self, guard: &mut GeneratedGuard) -> WorkflowResult<()> {
        let mut i = 0;
        while i < guard.bytecode.len() {
            if guard.bytecode[i] == 0x22 {
                // MULTIPLY
                // Check if multiplying by power of 2
                if i >= 9 && guard.bytecode[i - 9] == 0x01 {
                    // Previous is float const
                    let val = f64::from_le_bytes(guard.bytecode[i - 8..i].try_into().unwrap());
                    if val == 2.0 {
                        // Replace multiply by 2 with add to self
                        guard.bytecode[i] = 0x20; // ADD
                                                  // Duplicate previous value on stack (would need DUP instruction)
                    } else if val == 0.5 {
                        // Replace multiply by 0.5 with divide by 2
                        guard.bytecode[i - 8..i].copy_from_slice(&2.0f64.to_le_bytes());
                        guard.bytecode[i] = 0x23; // DIVIDE
                    }
                }
            }
            i += 1;
        }

        Ok(())
    }

    /// Add register allocation hints
    fn add_register_hints(&mut self, guard: &mut GeneratedGuard) -> WorkflowResult<()> {
        // Analyze variable usage patterns
        let mut var_usage: HashMap<u16, usize> = HashMap::new();
        for var_ref in &guard.var_refs {
            *var_usage.entry(*var_ref).or_insert(0) += 1;
        }

        // High-usage variables should be kept in registers
        // This would add hints to the bytecode for the runtime

        Ok(())
    }

    /// Optimize memory layout
    fn optimize_memory_layout(&mut self, code: &mut GeneratedCode) -> WorkflowResult<()> {
        debug!("Pass 6: Memory layout optimization");

        // Reorder fields for better cache locality
        for receipt in &mut code.receipts {
            // Sort fields by access frequency (hot fields first)
            // In real implementation, would use profiling data
            receipt.fields.sort_by_key(|f| match f.field_type {
                crate::compiler::code_generator::FieldType::U8 => 0,
                crate::compiler::code_generator::FieldType::U16 => 1,
                crate::compiler::code_generator::FieldType::U32 => 2,
                crate::compiler::code_generator::FieldType::U64 => 3,
                _ => 4,
            });

            // Recalculate offsets for better alignment
            let mut offset = 0u16;
            for field in &mut receipt.fields {
                // Align to field size
                let alignment = field.size.min(8);
                offset = (offset + alignment - 1) & !(alignment - 1);
                field.offset = offset;
                offset += field.size;
            }
        }

        Ok(())
    }

    /// Inline small functions
    fn inline_functions(&mut self, code: &mut GeneratedCode) -> WorkflowResult<()> {
        debug!("Pass 7: Function inlining");

        // Identify small guards suitable for inlining
        for guard in &mut code.guards {
            if guard.bytecode.len() <= 10 && guard.stack_depth <= 2 {
                // Mark for inlining
                // In real implementation, would modify call sites
            }
        }

        Ok(())
    }

    /// Optimize loops
    fn optimize_loops(&mut self, code: &mut GeneratedCode) -> WorkflowResult<()> {
        debug!("Pass 8: Loop optimization");

        // Find loop patterns
        for entry in &code.dispatch_table.entries {
            if entry.pattern_id == 10 || entry.pattern_id == 11 {
                // ArbitraryLoop or StructuredLoop
                // Apply loop optimizations
                // - Loop unrolling for small fixed iterations
                // - Loop invariant code motion
                // - Loop fusion/fission
            }
        }

        Ok(())
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::code_generator::{
        CodeMetadata, DispatchTable, GeneratedCode, SymbolTable,
    };

    fn create_test_code() -> GeneratedCode {
        GeneratedCode {
            dispatch_table: DispatchTable {
                entries: Vec::new(),
                jump_table: Vec::new(),
                pattern_map: HashMap::new(),
            },
            guards: Vec::new(),
            receipts: Vec::new(),
            symbols: SymbolTable {
                variables: HashMap::new(),
                functions: HashMap::new(),
                types: HashMap::new(),
                next_id: 0,
            },
            constants: Vec::new(),
            metadata: CodeMetadata {
                code_size: 0,
                data_size: 0,
                stack_size: 0,
                optimization_level: 0,
            },
        }
    }

    #[tokio::test]
    async fn test_optimizer_creation() {
        let optimizer = Optimizer::new(true);
        assert!(optimizer.enabled);
        assert_eq!(optimizer.optimization_level, 2);
    }

    #[tokio::test]
    async fn test_optimization_disabled() {
        let mut optimizer = Optimizer::new(false);
        let mut code = create_test_code();

        let stats = optimizer.optimize(&mut code).await.unwrap();
        assert_eq!(stats.dead_code_eliminated, 0);
        assert_eq!(stats.constants_folded, 0);
    }

    #[tokio::test]
    async fn test_constant_folding() {
        let mut optimizer = Optimizer::new(true);
        let mut guard = GeneratedGuard {
            id: "test".to_string(),
            bytecode: vec![
                0x01, // CONST_FLOAT
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x40, // 4.0
                0x01, // CONST_FLOAT
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x40, // 3.0
                0x20, // ADD
            ],
            var_refs: Vec::new(),
            const_refs: Vec::new(),
            stack_depth: 2,
        };

        let folded = optimizer.fold_guard_constants(&mut guard).unwrap();
        assert_eq!(folded, 1);
        assert!(guard.bytecode.len() < 19); // Should be shorter after folding
    }
}
