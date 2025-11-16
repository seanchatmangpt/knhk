//! Reporting and visualization for benchmark results

use crate::{MeasurementResult, OperationType};
use colored::*;
use std::fmt;

/// Comprehensive benchmark report
#[derive(Debug, Clone)]
pub struct BenchmarkReport {
    /// Total number of operations measured
    pub total_operations: usize,
    /// Number of operations that violated bounds
    pub violations: usize,
    /// Number of hot path operations
    pub hot_path_count: usize,
    /// Number of warm path operations
    pub warm_path_count: usize,
    /// Number of cold path operations
    pub cold_path_count: usize,
    /// All measurement results
    pub results: Vec<MeasurementResult>,
    /// Bottlenecks identified
    pub bottlenecks: Vec<Bottleneck>,
    /// Overall pass/fail status
    pub passed: bool,
}

/// Identified performance bottleneck
#[derive(Debug, Clone)]
pub struct Bottleneck {
    /// Operation name
    pub operation: String,
    /// Severity (High, Medium, Low)
    pub severity: Severity,
    /// Description of the issue
    pub description: String,
    /// Actual latency
    pub actual: u64,
    /// Expected latency
    pub expected: u64,
    /// How many times slower
    pub slowdown_factor: f64,
}

/// Bottleneck severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Low => write!(f, "{}", "LOW".green()),
            Severity::Medium => write!(f, "{}", "MEDIUM".yellow()),
            Severity::High => write!(f, "{}", "HIGH".red()),
            Severity::Critical => write!(f, "{}", "CRITICAL".red().bold()),
        }
    }
}

/// Reporter for generating comprehensive reports
pub struct Reporter;

impl Reporter {
    /// Generate a comprehensive benchmark report
    pub fn generate(results: &[MeasurementResult]) -> BenchmarkReport {
        let total_operations = results.len();
        let violations = results.iter().filter(|r| r.bounds_violated).count();

        let hot_path_count = results
            .iter()
            .filter(|r| r.operation_type == OperationType::HotPath)
            .count();

        let warm_path_count = results
            .iter()
            .filter(|r| r.operation_type == OperationType::WarmPath)
            .count();

        let cold_path_count = results
            .iter()
            .filter(|r| r.operation_type == OperationType::ColdPath)
            .count();

        let bottlenecks = Self::identify_bottlenecks(results);

        let passed = violations == 0;

        BenchmarkReport {
            total_operations,
            violations,
            hot_path_count,
            warm_path_count,
            cold_path_count,
            results: results.to_vec(),
            bottlenecks,
            passed,
        }
    }

    /// Identify performance bottlenecks
    fn identify_bottlenecks(results: &[MeasurementResult]) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        for result in results {
            let max_allowed = result.operation_type.max_allowed();
            let actual = result.statistics.p99;

            if result.bounds_violated {
                let slowdown_factor = actual as f64 / max_allowed as f64;

                let severity = if slowdown_factor > 10.0 {
                    Severity::Critical
                } else if slowdown_factor > 5.0 {
                    Severity::High
                } else if slowdown_factor > 2.0 {
                    Severity::Medium
                } else {
                    Severity::Low
                };

                let unit = if result.operation_type.uses_ticks() {
                    "ticks"
                } else {
                    "ns"
                };

                let description = format!(
                    "P99 latency ({} {}) exceeds bound ({} {}) by {:.1}x",
                    actual, unit, max_allowed, unit, slowdown_factor
                );

                bottlenecks.push(Bottleneck {
                    operation: result.operation_name.clone(),
                    severity,
                    description,
                    actual,
                    expected: max_allowed,
                    slowdown_factor,
                });
            }
        }

        // Sort by severity (critical first)
        bottlenecks.sort_by(|a, b| b.severity.cmp(&a.severity));

        bottlenecks
    }

    /// Print report to stdout with colors
    pub fn print_report(report: &BenchmarkReport) {
        println!("\n{}", "=".repeat(80).bright_blue());
        println!(
            "{}",
            "Chicago TDD Performance Report".bright_blue().bold()
        );
        println!("{}", "=".repeat(80).bright_blue());

        // Summary
        println!("\n{}", "Summary:".bright_white().bold());
        println!(
            "  Total Operations: {}",
            report.total_operations.to_string().cyan()
        );
        println!(
            "  Hot Path:         {}",
            report.hot_path_count.to_string().yellow()
        );
        println!(
            "  Warm Path:        {}",
            report.warm_path_count.to_string().green()
        );
        println!(
            "  Cold Path:        {}",
            report.cold_path_count.to_string().blue()
        );
        println!(
            "  Violations:       {}",
            if report.violations == 0 {
                report.violations.to_string().green()
            } else {
                report.violations.to_string().red().bold()
            }
        );

        // Overall status
        println!(
            "\n{} {}",
            "Overall Status:".bright_white().bold(),
            if report.passed {
                "PASS ✓".green().bold()
            } else {
                "FAIL ✗".red().bold()
            }
        );

        // Detailed results
        println!("\n{}", "Detailed Results:".bright_white().bold());
        println!(
            "{:<30} {:<10} {:<12} {:<12} {:<12} {:<12} {}",
            "Operation", "Type", "P50", "P95", "P99", "Bound", "Status"
        );
        println!("{}", "-".repeat(80));

        for result in &report.results {
            let type_str = match result.operation_type {
                OperationType::HotPath => "HOT".yellow(),
                OperationType::WarmPath => "WARM".green(),
                OperationType::ColdPath => "COLD".blue(),
            };

            let unit = if result.operation_type.uses_ticks() {
                "t"
            } else {
                "ns"
            };

            let bound = result.operation_type.max_allowed();
            let status = if result.bounds_violated {
                "FAIL ✗".red().bold()
            } else {
                "PASS ✓".green()
            };

            println!(
                "{:<30} {:<10} {:<12} {:<12} {:<12} {:<12} {}",
                result.operation_name.chars().take(28).collect::<String>(),
                type_str,
                format!("{}{}", result.statistics.p50, unit),
                format!("{}{}", result.statistics.p95, unit),
                format!("{}{}", result.statistics.p99, unit),
                format!("{}{}", bound, unit),
                status
            );
        }

        // Bottlenecks
        if !report.bottlenecks.is_empty() {
            println!("\n{}", "Bottlenecks Identified:".red().bold());
            println!(
                "{:<30} {:<10} {}",
                "Operation", "Severity", "Description"
            );
            println!("{}", "-".repeat(80));

            for bottleneck in &report.bottlenecks {
                println!(
                    "{:<30} {:<10} {}",
                    bottleneck.operation.chars().take(28).collect::<String>(),
                    bottleneck.severity,
                    bottleneck.description
                );
            }
        }

        // Recommendations
        if report.violations > 0 {
            println!("\n{}", "Recommendations:".yellow().bold());

            for bottleneck in &report.bottlenecks {
                match bottleneck.severity {
                    Severity::Critical | Severity::High => {
                        println!(
                            "  {} {}: Consider moving '{}' off critical path or optimizing algorithm",
                            "⚠".red(),
                            bottleneck.severity,
                            bottleneck.operation
                        );
                    }
                    Severity::Medium => {
                        println!(
                            "  {} {}: Review '{}' for optimization opportunities",
                            "⚠".yellow(),
                            bottleneck.severity,
                            bottleneck.operation
                        );
                    }
                    Severity::Low => {
                        println!(
                            "  {} {}: Monitor '{}' for potential regression",
                            "ℹ".blue(),
                            bottleneck.severity,
                            bottleneck.operation
                        );
                    }
                }
            }
        }

        println!("\n{}", "=".repeat(80).bright_blue());
    }

    /// Print a single result in detail
    pub fn print_result(result: &MeasurementResult) {
        println!("\n{}", "─".repeat(60));
        println!("{}: {}", "Operation".bright_white().bold(), result.operation_name);

        let type_color = match result.operation_type {
            OperationType::HotPath => "HOT PATH".yellow(),
            OperationType::WarmPath => "WARM PATH".green(),
            OperationType::ColdPath => "COLD PATH".blue(),
        };
        println!("{}: {}", "Type".bright_white(), type_color);

        let unit = if result.operation_type.uses_ticks() {
            "ticks"
        } else {
            "ns"
        };

        println!("\n{}", "Statistics:".bright_white().bold());
        println!("  Samples:  {}", result.statistics.count);
        println!("  Min:      {} {}", result.statistics.min, unit);
        println!("  Mean:     {:.2} {}", result.statistics.mean, unit);
        println!("  P50:      {} {}", result.statistics.p50, unit);
        println!("  P75:      {} {}", result.statistics.p75, unit);
        println!("  P90:      {} {}", result.statistics.p90, unit);
        println!("  P95:      {} {}", result.statistics.p95, unit);
        println!("  P99:      {} {}", result.statistics.p99, unit);
        println!("  P99.9:    {} {}", result.statistics.p999, unit);
        println!("  Max:      {} {}", result.statistics.max, unit);
        println!("  Std Dev:  {:.2} {}", result.statistics.std_dev, unit);
        println!("  CV:       {:.2}%", result.statistics.cv * 100.0);

        println!(
            "\n{}: {} {}",
            "Bound".bright_white(),
            result.operation_type.max_allowed(),
            unit
        );

        let status = if result.bounds_violated {
            "FAIL ✗".red().bold()
        } else {
            "PASS ✓".green().bold()
        };
        println!("{}: {}", "Status".bright_white(), status);
    }

    /// Export report to JSON
    pub fn export_json(report: &BenchmarkReport) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Export report to CSV
    pub fn export_csv(report: &BenchmarkReport) -> String {
        let mut csv = String::new();
        csv.push_str("Operation,Type,P50,P95,P99,Bound,Status\n");

        for result in &report.results {
            let type_str = match result.operation_type {
                OperationType::HotPath => "HOT",
                OperationType::WarmPath => "WARM",
                OperationType::ColdPath => "COLD",
            };

            let status = if result.bounds_violated { "FAIL" } else { "PASS" };

            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                result.operation_name,
                type_str,
                result.statistics.p50,
                result.statistics.p95,
                result.statistics.p99,
                result.operation_type.max_allowed(),
                status
            ));
        }

        csv
    }
}

// Make BenchmarkReport serializable
impl serde::Serialize for BenchmarkReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("BenchmarkReport", 8)?;
        state.serialize_field("total_operations", &self.total_operations)?;
        state.serialize_field("violations", &self.violations)?;
        state.serialize_field("hot_path_count", &self.hot_path_count)?;
        state.serialize_field("warm_path_count", &self.warm_path_count)?;
        state.serialize_field("cold_path_count", &self.cold_path_count)?;
        state.serialize_field("passed", &self.passed)?;
        state.serialize_field("bottlenecks_count", &self.bottlenecks.len())?;
        state.serialize_field("results_count", &self.results.len())?;
        state.end()
    }
}
