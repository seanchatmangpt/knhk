//! Code quality collection logic

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub clippy_errors: u32,
    pub clippy_warnings: u32,
    pub unwrap_count: u32,
    pub println_count: u32,
    pub unimplemented_count: u32,
    pub weighted_total: u32,
    pub categories: QualityCategories,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCategories {
    pub critical: u32,
    pub high: u32,
    pub medium: u32,
    pub low: u32,
}

pub struct QualityCollector {
    rust_dir: PathBuf,
}

impl QualityCollector {
    pub fn new(rust_dir: PathBuf) -> Self {
        Self { rust_dir }
    }

    pub async fn collect(&self) -> Result<QualityMetrics, anyhow::Error> {
        let clippy_errors = self.count_clippy_errors().await?;
        let clippy_warnings = self.count_clippy_warnings().await?;
        let unwrap_count = self.count_unwrap().await?;
        let println_count = self.count_println().await?;
        let unimplemented_count = self.count_unimplemented().await?;

        // Calculate weighted total
        let weights: HashMap<&str, u32> = [
            ("clippy_errors", 10),
            ("unimplemented", 10),
            ("unwrap", 5),
            ("println", 5),
            ("clippy_warnings", 3),
        ]
        .iter()
        .cloned()
        .collect();

        let weighted_total = clippy_errors * weights["clippy_errors"]
            + unimplemented_count * weights["unimplemented"]
            + unwrap_count * weights["unwrap"]
            + println_count * weights["println"]
            + clippy_warnings * weights["clippy_warnings"];

        // Categorize defects
        let categories = QualityCategories {
            critical: clippy_errors + unimplemented_count,
            high: unwrap_count + println_count,
            medium: clippy_warnings,
            low: 0,
        };

        Ok(QualityMetrics {
            clippy_errors,
            clippy_warnings,
            unwrap_count,
            println_count,
            unimplemented_count,
            weighted_total,
            categories,
        })
    }

    async fn count_clippy_errors(&self) -> Result<u32, anyhow::Error> {
        let output = Command::new("cargo")
            .args(&["clippy", "--workspace", "--", "-D", "warnings"])
            .current_dir(&self.rust_dir)
            .output()
            .await?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let errors = stderr.matches("error:").count() as u32;
        Ok(errors)
    }

    async fn count_clippy_warnings(&self) -> Result<u32, anyhow::Error> {
        let output = Command::new("cargo")
            .args(&["clippy", "--workspace"])
            .current_dir(&self.rust_dir)
            .output()
            .await?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let warnings = stderr.matches("warning:").count() as u32;
        Ok(warnings)
    }

    async fn count_unwrap(&self) -> Result<u32, anyhow::Error> {
        let output = Command::new("bash")
            .arg("-c")
            .arg(
                "grep -r '\\.unwrap()\\|\\.expect(' rust/*/src --include='*.rs' | \
                 grep -v test | grep -v cli | grep -v examples | grep -v build.rs | wc -l",
            )
            .current_dir(&self.rust_dir.parent().unwrap())
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.trim().parse::<u32>().unwrap_or(0);
        Ok(count)
    }

    async fn count_println(&self) -> Result<u32, anyhow::Error> {
        let output = Command::new("bash")
            .arg("-c")
            .arg(
                "grep -r 'println!' rust/*/src --include='*.rs' | \
                 grep -v test | grep -v cli | grep -v examples | wc -l",
            )
            .current_dir(&self.rust_dir.parent().unwrap())
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.trim().parse::<u32>().unwrap_or(0);
        Ok(count)
    }

    async fn count_unimplemented(&self) -> Result<u32, anyhow::Error> {
        let output = Command::new("bash")
            .arg("-c")
            .arg("grep -r 'unimplemented!' rust/*/src --include='*.rs' | wc -l")
            .current_dir(&self.rust_dir.parent().unwrap())
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.trim().parse::<u32>().unwrap_or(0);
        Ok(count)
    }
}
