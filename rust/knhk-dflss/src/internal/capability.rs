//! Process capability calculations (Cp, Cpk, Sigma level, DPMO)

use crate::internal::statistics::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessCapability {
    pub cp: f64,
    pub cpk: f64,
    pub sigma_level: f64,
    pub dpmo: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub usl: f64,
    pub lsl: f64,
}

#[derive(Debug, Error)]
pub enum CapabilityError {
    #[error("Cannot calculate capability with empty data")]
    EmptyData,
    #[error("Invalid specification limits: USL must be greater than LSL")]
    InvalidLimits,
}

impl ProcessCapability {
    pub fn calculate(data: &[f64], usl: f64, lsl: f64) -> Result<Self, CapabilityError> {
        if data.is_empty() {
            return Err(CapabilityError::EmptyData);
        }

        if usl <= lsl {
            return Err(CapabilityError::InvalidLimits);
        }

        let mean = mean(data);
        let std_dev = std_dev(data);

        if std_dev == 0.0 {
            // If standard deviation is zero, all data points are identical
            let is_within_limits = data.iter().all(|&x| x >= lsl && x <= usl);
            return Ok(Self {
                cp: if is_within_limits { f64::INFINITY } else { 0.0 },
                cpk: if is_within_limits { f64::INFINITY } else { 0.0 },
                sigma_level: if is_within_limits { 6.0 } else { 0.0 },
                dpmo: if is_within_limits { 0.0 } else { 1_000_000.0 },
                mean,
                std_dev,
                usl,
                lsl,
            });
        }

        // Cp: Process Potential Capability
        let cp = (usl - lsl) / (6.0 * std_dev);

        // Cpk: Process Performance Capability
        let cpk_usl = (usl - mean) / (3.0 * std_dev);
        let cpk_lsl = (mean - lsl) / (3.0 * std_dev);
        let cpk = cpk_usl.min(cpk_lsl);

        // DPMO and Sigma Level (simplified approximation)
        let z_usl = (usl - mean) / std_dev;
        let z_lsl = (lsl - mean) / std_dev;

        // Approximate proportion outside limits using normal CDF
        let p_usl = 1.0 - normal_cdf(z_usl);
        let p_lsl = normal_cdf(z_lsl);
        let p_defective = p_usl + p_lsl;

        let dpmo = p_defective * 1_000_000.0;
        let sigma_level = dpmo_to_sigma(dpmo);

        Ok(Self {
            cp,
            cpk,
            sigma_level,
            dpmo,
            mean,
            std_dev,
            usl,
            lsl,
        })
    }

    pub fn calculate_from_benchmarks(
        benchmarks: &HashMap<String, Vec<f64>>,
        usl: f64,
    ) -> Result<HashMap<String, Self>, CapabilityError> {
        let mut results = HashMap::new();
        for (name, data) in benchmarks {
            if !data.is_empty() {
                results.insert(name.clone(), Self::calculate(data, usl, 0.0)?);
            }
        }
        Ok(results)
    }
}

// Simple approximation of standard normal CDF
fn normal_cdf(z: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.2316419 * z.abs());
    let d = 0.39894228 * (-z * z / 2.0).exp();
    let prob = 1.0
        - d * t
            * (0.319381530
                + t * (-0.356563782 + t * (1.781477937 + t * (-1.821255978 + t * 1.330274429))));
    if z > 0.0 {
        prob
    } else {
        1.0 - prob
    }
}

// Convert DPMO to Sigma Level
fn dpmo_to_sigma(dpmo: f64) -> f64 {
    if dpmo <= 0.0 {
        return 6.0;
    }
    if dpmo >= 1_000_000.0 {
        return 0.0;
    }

    let p_defective = dpmo / 1_000_000.0;
    let z_score = inverse_normal_cdf(1.0 - p_defective);
    z_score + 1.5 // Add 1.5 sigma shift for short-term vs long-term
}

// Simple approximation of standard normal inverse CDF
fn inverse_normal_cdf(p: f64) -> f64 {
    if p < 0.0 || p > 1.0 {
        return f64::NAN;
    }
    if p == 0.0 {
        return f64::NEG_INFINITY;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }

    let a1 = 2.50662823884;
    let a2 = -18.6150007218;
    let a3 = 41.3911977353;
    let a4 = -25.4410604963;
    let b1 = -8.4735109309;
    let b2 = 3.2243102832;
    let b3 = -1.2320386856;
    let b4 = -0.1112081967;

    let y = p - 0.5;
    let r = y * y;
    let x =
        y * (((a4 * r + a3) * r + a2) * r + a1) / ((((b4 * r + b3) * r + b2) * r + b1) * r + 1.0);
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_calculation() {
        let data = vec![5.0, 5.5, 6.0, 6.5, 7.0];
        let capability = ProcessCapability::calculate(&data, 8.0, 0.0).unwrap();

        assert!(capability.cp > 0.0);
        assert!(capability.cpk > 0.0);
        assert!(capability.sigma_level > 0.0);
        assert_eq!(capability.usl, 8.0);
        assert_eq!(capability.lsl, 0.0);
    }

    #[test]
    fn test_empty_data() {
        let data = vec![];
        let result = ProcessCapability::calculate(&data, 8.0, 0.0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CapabilityError::EmptyData));
    }
}
