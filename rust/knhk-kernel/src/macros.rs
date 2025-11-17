// knhk-kernel: Pattern definition macros for code generation
// Generates dispatch code and validates at compile time

/// Macro for defining patterns with automatic dispatch generation
#[macro_export]
macro_rules! define_pattern {
    (
        name: $name:ident,
        type: $pattern_type:expr,
        guards: [$($guard:expr),* $(,)?],
        tick_budget: $budget:expr,
        body: $body:expr
    ) => {
        pub fn $name(ctx: &PatternContext) -> PatternResult {
            // Start tick measurement
            let timer = $crate::timer::HotPathTimer::start();
            let mut budget = $crate::timer::TickBudget::with_budget($budget);

            // Validate pattern type
            debug_assert_eq!(ctx.pattern_type, $pattern_type);

            // Execute pattern body
            let result = {
                // Charge for setup
                budget.charge("setup", 1)?;

                // Execute user code
                $body(ctx, &mut budget)
            };

            // Finalize result with timing
            PatternResult {
                success: result.success,
                output_mask: result.output_mask,
                ticks_used: timer.elapsed_ticks() as u32,
                next_pattern: result.next_pattern,
            }
        }
    };
}

/// Macro for defining guards with compile-time validation
#[macro_export]
macro_rules! define_guard {
    (
        name: $name:ident,
        type: $guard_type:expr,
        predicate: $predicate:expr,
        operands: ($op_a:expr, $op_b:expr)
    ) => {
        pub fn $name() -> $crate::guard::Guard {
            $crate::guard::Guard {
                guard_type: $guard_type,
                predicate: $predicate,
                operand_a: $op_a,
                operand_b: $op_b,
                children: Vec::new(),
            }
        }
    };

    (
        name: $name:ident,
        compound: $compound_type:ident,
        children: [$($child:expr),* $(,)?]
    ) => {
        pub fn $name() -> $crate::guard::Guard {
            $crate::guard::Guard::$compound_type(vec![$($child),*])
        }
    };
}

/// Macro for tick budget validation at compile time
#[macro_export]
macro_rules! validate_tick_budget {
    ($budget:expr) => {
        const _: () = {
            if $budget > 8 {
                panic!("Tick budget exceeds Chatman constant (8 ticks)");
            }
        };
    };
}

/// Macro for generating pattern dispatch table
#[macro_export]
macro_rules! generate_dispatch_table {
    (
        $(
            $pattern_type:expr => $handler:ident
        ),* $(,)?
    ) => {{
        let mut table: [PatternHandler; 44] = [pattern_noop; 44];
        $(
            table[$pattern_type as usize] = $handler;
        )*
        table
    }};
}

/// Macro for defining receipt builders
#[macro_export]
macro_rules! build_receipt {
    (
        pattern: $pattern_id:expr,
        task: $task_id:expr,
        status: $status:expr,
        ticks: $ticks:expr
        $(, guards: [$($guard_id:expr => $passed:expr => $guard_ticks:expr),*])?
        $(, inputs: $inputs:expr)?
        $(, outputs: $outputs:expr)?
        $(, state: ($before:expr, $after:expr))?
    ) => {{
        let mut receipt = $crate::receipt::ReceiptBuilder::new($pattern_id, $task_id);

        receipt = receipt.with_result($status, $ticks);

        $(
            $(
                receipt = receipt.add_guard($guard_id, $passed, $guard_ticks);
            )*
        )?

        $(
            receipt = receipt.with_inputs($inputs);
        )?

        $(
            receipt = receipt.with_outputs($outputs);
        )?

        $(
            receipt = receipt.with_state($before, $after);
        )?

        receipt.build()
    }};
}

/// Macro for atomic state transitions
#[macro_export]
macro_rules! atomic_transition {
    ($task:expr, $new_state:expr) => {{
        let old = $task
            .state
            .swap($new_state as u32, std::sync::atomic::Ordering::AcqRel);
        unsafe { std::mem::transmute::<u32, TaskState>(old) }
    }};
}

/// Macro for pattern configuration validation
#[macro_export]
macro_rules! validate_pattern_config {
    ($config:expr, $pattern_type:expr) => {{
        match $pattern_type {
            PatternType::ParallelSplit | PatternType::MultiChoice => {
                assert!(
                    $config.max_instances > 0 && $config.max_instances <= 64,
                    "Invalid max_instances for split pattern"
                );
            }
            PatternType::Synchronization | PatternType::StructuredSyncMerge => {
                assert!(
                    $config.join_threshold > 0 && $config.join_threshold <= 64,
                    "Invalid join_threshold for sync pattern"
                );
            }
            PatternType::Recursion => {
                assert!(
                    $config.flags.is_cancellable(),
                    "Recursion patterns must be cancellable"
                );
            }
            _ => {}
        }
    }};
}

/// Macro for SIMD-optimized observation matching
#[cfg(target_arch = "x86_64")]
#[macro_export]
macro_rules! simd_match_observations {
    ($observations:expr, $pattern:expr) => {{
        use std::arch::x86_64::*;

        unsafe {
            // Load observations and pattern into SIMD registers
            let obs = _mm256_loadu_si256($observations.as_ptr() as *const __m256i);
            let pat = _mm256_loadu_si256($pattern.as_ptr() as *const __m256i);

            // Compare for equality
            let cmp = _mm256_cmpeq_epi64(obs, pat);

            // Extract mask
            let mask = _mm256_movemask_epi8(cmp);

            // Check if all match
            mask == -1i32
        }
    }};
}

#[cfg(not(target_arch = "x86_64"))]
#[macro_export]
macro_rules! simd_match_observations {
    ($observations:expr, $pattern:expr) => {{
        // Fallback scalar implementation
        $observations
            .iter()
            .zip($pattern.iter())
            .all(|(a, b)| a == b)
    }};
}

/// Macro for generating pattern permutation validators
#[macro_export]
macro_rules! validate_permutation {
    ($source:expr, $target:expr) => {{
        const VALID_PERMUTATIONS: &[(PatternType, PatternType)] = &[
            (PatternType::ParallelSplit, PatternType::Synchronization),
            (PatternType::ExclusiveChoice, PatternType::SimpleMerge),
            (PatternType::MultiChoice, PatternType::StructuredSyncMerge),
            // Add all valid permutations from matrix
        ];

        VALID_PERMUTATIONS
            .iter()
            .any(|&(s, t)| s == $source && t == $target)
    }};
}

/// Macro for hot path assertions (compiled out in release)
#[macro_export]
macro_rules! hot_assert {
    ($cond:expr) => {
        #[cfg(debug_assertions)]
        assert!($cond);
    };
    ($cond:expr, $msg:expr) => {
        #[cfg(debug_assertions)]
        assert!($cond, $msg);
    };
}

/// Macro for measuring operations with automatic tick charging
#[macro_export]
macro_rules! measure_operation {
    ($budget:expr, $op_name:literal, $code:block) => {{
        let start = $crate::timer::read_tsc();
        let result = $code;
        let elapsed = $crate::timer::read_tsc() - start;
        $budget.charge($op_name, elapsed)?;
        result
    }};
}

/// Macro for pattern factory with validation
#[macro_export]
macro_rules! create_pattern {
    (
        type: $pattern_type:expr,
        id: $pattern_id:expr,
        config: {
            max_instances: $max_inst:expr,
            join_threshold: $join_thr:expr,
            timeout_ticks: $timeout:expr,
            flags: $flags:expr
        }
    ) => {{
        validate_tick_budget!($timeout);
        validate_pattern_config!(
            PatternConfig {
                max_instances: $max_inst,
                join_threshold: $join_thr,
                timeout_ticks: $timeout,
                flags: $flags,
            },
            $pattern_type
        );

        $crate::pattern::PatternFactory::create(
            $pattern_type,
            $pattern_id,
            PatternConfig {
                max_instances: $max_inst,
                join_threshold: $join_thr,
                timeout_ticks: $timeout,
                flags: $flags,
            },
        )
    }};
}

/// Macro for generating all 43 W3C pattern handlers
#[macro_export]
macro_rules! generate_all_patterns {
    () => {{
        use $crate::pattern::{PatternType, PatternContext, PatternResult};

        // Generate handlers for all 43 patterns
        define_pattern!(
            name: pattern_sequence,
            type: PatternType::Sequence,
            guards: [],
            tick_budget: 8,
            body: |ctx: &PatternContext, budget: &mut TickBudget| {
                if ctx.input_mask == 0 {
                    return PatternResult {
                        success: false,
                        output_mask: 0,
                        ticks_used: 0,
                        next_pattern: None,
                    };
                }

                PatternResult {
                    success: true,
                    output_mask: ctx.input_mask,
                    ticks_used: 0,
                    next_pattern: Some(ctx.pattern_id + 1),
                }
            }
        );

        // ... Generate all other 42 patterns ...

        // Build dispatch table
        generate_dispatch_table!(
            PatternType::Sequence => pattern_sequence,
            PatternType::ParallelSplit => pattern_parallel_split,
            PatternType::Synchronization => pattern_synchronization,
            // ... all 43 patterns ...
        )
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guard::{GuardType, Predicate};
    use crate::pattern::{PatternConfig, PatternFlags, PatternType};

    #[test]
    fn test_guard_macro() {
        define_guard!(
            name: test_guard,
            type: GuardType::Predicate,
            predicate: Predicate::Equal,
            operands: (0, 42)
        );

        let guard = test_guard();
        assert_eq!(guard.guard_type, GuardType::Predicate);
        assert_eq!(guard.operand_b, 42);
    }

    #[test]
    fn test_receipt_macro() {
        let receipt = build_receipt!(
            pattern: 1,
            task: 100,
            status: crate::receipt::ReceiptStatus::Success,
            ticks: 5,
            guards: [1 => true => 1, 2 => false => 2],
            inputs: &[1, 2, 3],
            state: (0, 1)
        );

        assert_eq!(receipt.pattern_id, 1);
        assert_eq!(receipt.task_id, 100);
        assert_eq!(receipt.ticks_used, 5);
        assert_eq!(receipt.guard_count, 2);
    }

    #[test]
    fn test_pattern_config_validation() {
        let config = PatternConfig {
            max_instances: 4,
            join_threshold: 0,
            timeout_ticks: 8,
            flags: PatternFlags::new(0),
        };

        validate_pattern_config!(config, PatternType::ParallelSplit);
    }

    #[test]
    fn test_permutation_validation() {
        assert!(validate_permutation!(
            PatternType::ParallelSplit,
            PatternType::Synchronization
        ));
    }
}
