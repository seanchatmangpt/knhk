// rust/knhk-aot/src/prebinding.rs
// Prebinding constants to IR for AOT optimization

use alloc::vec::Vec;
use alloc::string::String;
use crate::template::ConstructTemplate;

#[derive(Debug, Clone)]
pub struct PreboundIr {
    pub s_const: Option<u64>,      // Prebound subject constant (None if variable)
    pub p_const: Option<u64>,      // Prebound predicate constant (None if variable)
    pub o_const: Option<u64>,      // Prebound object constant (None if variable)
    pub s_is_var: bool,            // True if subject is variable
    pub p_is_var: bool,            // True if predicate is variable
    pub o_is_var: bool,            // True if object is variable
    pub len_mask_hint: u64,        // Precomputed length mask
    pub zero_hint: u8,             // Zero-position bitmask hint
}

impl PreboundIr {
    pub fn new() -> Self {
        Self {
            s_const: None,
            p_const: None,
            o_const: None,
            s_is_var: true,
            p_is_var: true,
            o_is_var: true,
            len_mask_hint: 0,
            zero_hint: 0,
        }
    }

    /// Prebind constants from template analysis
    pub fn from_template(template: &ConstructTemplate, len: u64) -> Self {
        let mut ir = Self::new();

        // Precompute length mask hint
        ir.len_mask_hint = if len == 8 {
            0xFF
        } else {
            ((1u64 << len) - 1) & 0xFF
        };

        // Analyze first template triple for constants
        if let Some(triple) = template.template_triples.first() {
            match &triple.subject {
                crate::template::TripleComponent::Constant(s) => {
                    ir.s_const = Some(*s);
                    ir.s_is_var = false;
                }
                _ => {
                    ir.s_is_var = true;
                }
            }

            match &triple.predicate {
                crate::template::TripleComponent::Constant(p) => {
                    ir.p_const = Some(*p);
                    ir.p_is_var = false;
                }
                _ => {
                    ir.p_is_var = true;
                }
            }

            match &triple.object {
                crate::template::TripleComponent::Constant(o) => {
                    ir.o_const = Some(*o);
                    ir.o_is_var = false;
                }
                _ => {
                    ir.o_is_var = true;
                }
            }
        }

        ir
    }

    /// Analyze zero pattern from input subjects
    pub fn analyze_zero_pattern(&mut self, subjects: &[u64; 8]) {
        let mut hint = 0u8;
        for (i, &s) in subjects.iter().enumerate() {
            if s == 0 {
                hint |= 1 << i;
            }
        }
        self.zero_hint = hint;
    }

    /// Check if all subjects are non-zero
    pub fn all_nonzero(&self) -> bool {
        self.zero_hint == 0
    }

    /// Check if all subjects are zero
    pub fn all_zero(&self) -> bool {
        self.zero_hint == 0xFF
    }
}

impl Default for PreboundIr {
    fn default() -> Self {
        Self::new()
    }
}

