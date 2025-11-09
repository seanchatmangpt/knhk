//! Process Capability Calculations
//!
//! Implements statistical process capability analysis:
//! - Cp (Process Capability): Measures process spread relative to specification
//! - Cpk (Centered Process Capability): Measures process centering and spread
//! - Sigma Level: Converts DPMO to Sigma level
//! - DPMO: Defects Per Million Opportunities

use crate::error::WorkflowResult;
use std::collections::HashMap;

/// Process capability metrics
#[derive(Debug, Clone)]
pub struct ProcessCapability {
    /// Process mean (μ)
    pub mean: f64,
    /// Process standard deviation (σ)
    pub std_dev: f64,
    /// Upper specification limit
    pub usl: f64,
    /// Lower specification limit
    pub lsl: f64,
    /// Process capability (Cp)
    pub cp: f64,
    /// Centered process capability (Cpk)
    pub cpk: f64,
    /// Defects per million opportunities
    pub dpmo: f64,
    /// Sigma level
    pub sigma_level: f64,
}

impl ProcessCapability {
    /// Calculate process capability from performance data
    ///
    /// # Arguments
    /// * `values` - Sample values (e.g., tick counts for hot path operations)
    /// * `usl` - Upper specification limit (e.g., 8 ticks for hot path)
    /// * `lsl` - Lower specification limit (default: 0)
    ///
    /// # Returns
    /// * `ProcessCapability` with Cp, Cpk, DPMO, and Sigma level
    pub fn calculate(values: &[f64], usl: f64, lsl: f64) -> WorkflowResult<Self> {
        if values.is_empty() {
            return Err(crate::error::WorkflowError::Internal(
                "Cannot calculate process capability: no data points".to_string(),
            ));
        }

        // Calculate mean
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate standard deviation
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        // Calculate Cp (Process Capability)
        // Cp = (USL - LSL) / (6σ)
        let cp = if std_dev > 0.0 {
            (usl - lsl) / (6.0 * std_dev)
        } else {
            f64::INFINITY // Perfect consistency
        };

        // Calculate Cpk (Centered Process Capability)
        // Cpk = min[(USL - μ)/(3σ), (μ - LSL)/(3σ)]
        let cpk = if std_dev > 0.0 {
            let cpk_upper = (usl - mean) / (3.0 * std_dev);
            let cpk_lower = (mean - lsl) / (3.0 * std_dev);
            cpk_upper.min(cpk_lower)
        } else {
            f64::INFINITY // Perfect consistency
        };

        // Calculate DPMO (Defects Per Million Opportunities)
        // Count defects (values > USL or < LSL)
        let defects = values.iter().filter(|&&x| x > usl || x < lsl).count();
        let dpmo = (defects as f64 / values.len() as f64) * 1_000_000.0;

        // Calculate Sigma level from DPMO
        // Use lookup table approximation for common values
        let sigma_level = Self::dpmo_to_sigma(dpmo);

        Ok(Self {
            mean,
            std_dev,
            usl,
            lsl,
            cp,
            cpk,
            dpmo,
            sigma_level,
        })
    }

    /// Convert DPMO to Sigma level
    ///
    /// Uses lookup table for common values, with interpolation for others.
    fn dpmo_to_sigma(dpmo: f64) -> f64 {
        // Sigma level lookup table (DPMO → Sigma)
        // Based on standard Six Sigma conversion table
        let lookup_table: [(f64, f64); 7] = [
            (3_000_000.0, 0.0), // 0σ (no process)
            (308_537.0, 2.0),   // 2σ
            (66_807.0, 3.0),    // 3σ
            (6_210.0, 4.0),     // 4σ
            (233.0, 5.0),       // 5σ
            (3.4, 6.0),         // 6σ (world-class)
            (0.0, 7.0),         // 7σ (theoretical)
        ];

        // Handle edge cases
        if dpmo >= lookup_table[0].0 {
            return lookup_table[0].1; // 0σ
        }
        if dpmo <= lookup_table[lookup_table.len() - 1].0 {
            return lookup_table[lookup_table.len() - 1].1; // 7σ
        }

        // Find bounding values and interpolate
        for i in 0..lookup_table.len() - 1 {
            let (dpmo_high, sigma_low) = lookup_table[i];
            let (dpmo_low, sigma_high) = lookup_table[i + 1];

            if dpmo <= dpmo_high && dpmo >= dpmo_low {
                // Linear interpolation
                let ratio = (dpmo_high - dpmo) / (dpmo_high - dpmo_low);
                return sigma_low + ratio * (sigma_high - sigma_low);
            }
        }

        // Fallback: use approximation formula
        // Sigma ≈ 0.8406 + sqrt(29.37 - 2.221 * ln(DPMO))
        if dpmo > 0.0 {
            let ln_dpmo = dpmo.ln();
            if 29.37 - 2.221 * ln_dpmo >= 0.0 {
                0.8406 + (29.37 - 2.221 * ln_dpmo).sqrt()
            } else {
                6.0 // Cap at 6σ
            }
        } else {
            7.0 // Perfect (7σ theoretical)
        }
    }

    /// Calculate process capability from performance benchmark results
    ///
    /// # Arguments
    /// * `operation_ticks` - Map of operation names to tick counts
    /// * `usl` - Upper specification limit (default: 8 ticks)
    ///
    /// # Returns
    /// * `HashMap` of operation names to `ProcessCapability`
    pub fn calculate_from_benchmarks(
        operation_ticks: &HashMap<String, Vec<f64>>,
        usl: f64,
    ) -> WorkflowResult<HashMap<String, Self>> {
        let mut results = HashMap::new();

        for (operation, values) in operation_ticks {
            let capability = Self::calculate(values, usl, 0.0)?;
            results.insert(operation.clone(), capability);
        }

        Ok(results)
    }

    /// Generate process capability report
    pub fn report(&self) -> String {
        format!(
            "Process Capability Analysis:\n\
             Mean (μ): {:.2}\n\
             Std Dev (σ): {:.2}\n\
             USL: {:.2}\n\
             LSL: {:.2}\n\
             Cp: {:.2} {}\n\
             Cpk: {:.2} {}\n\
             DPMO: {:.1}\n\
             Sigma Level: {:.2}σ",
            self.mean,
            self.std_dev,
            self.usl,
            self.lsl,
            self.cp,
            if self.cp >= 2.0 {
                "✅ (Highly Capable)"
            } else if self.cp >= 1.33 {
                "✅ (Capable)"
            } else {
                "⚠️ (Not Capable)"
            },
            self.cpk,
            if self.cpk >= 1.67 {
                "✅ (Well-Centered)"
            } else if self.cpk >= 1.33 {
                "⚠️ (Marginally Centered)"
            } else {
                "❌ (Not Centered)"
            },
            self.dpmo,
            self.sigma_level
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_capability_calculation() {
        // Test data: 18 operations ≤8 ticks, 1 operation at 10 ticks
        let values = vec![
            6.2, 5.1, 4.8, 7.3, 5.9, 6.1, 5.5, 6.8, 5.2, 6.5, 5.7, 6.3, 5.4, 6.7, 5.8, 6.0, 5.6,
            6.4, 10.0, // One outlier
        ];

        let capability = ProcessCapability::calculate(&values, 8.0, 0.0).unwrap();

        // Verify calculations
        assert!(capability.cp > 0.0);
        assert!(capability.cpk > 0.0);
        assert!(capability.dpmo > 0.0);
        assert!(capability.sigma_level > 0.0);

        // Cp should be high (process is capable)
        assert!(capability.cp >= 1.0);

        // Cpk should be lower than Cp (process not well-centered due to outlier)
        assert!(capability.cpk <= capability.cp);
    }

    #[test]
    fn test_dpmo_to_sigma_conversion() {
        // Test standard Six Sigma values
        assert!((ProcessCapability::dpmo_to_sigma(308_537.0) - 2.0).abs() < 0.1);
        assert!((ProcessCapability::dpmo_to_sigma(66_807.0) - 3.0).abs() < 0.1);
        assert!((ProcessCapability::dpmo_to_sigma(6_210.0) - 4.0).abs() < 0.1);
        assert!((ProcessCapability::dpmo_to_sigma(233.0) - 5.0).abs() < 0.1);
        assert!((ProcessCapability::dpmo_to_sigma(3.4) - 6.0).abs() < 0.1);
    }

    #[test]
    fn test_empty_data() {
        let result = ProcessCapability::calculate(&[], 8.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_perfect_process() {
        // All values exactly at target (mean = 4.0, std_dev = 0)
        let values = vec![4.0, 4.0, 4.0, 4.0, 4.0];
        let capability = ProcessCapability::calculate(&values, 8.0, 0.0).unwrap();

        // Perfect consistency: Cp and Cpk should be infinite
        assert!(capability.cp.is_infinite());
        assert!(capability.cpk.is_infinite());
        assert_eq!(capability.dpmo, 0.0);
    }
}
