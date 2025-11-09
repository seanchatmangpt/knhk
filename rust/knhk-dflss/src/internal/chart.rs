//! Chart data structures and CSV I/O

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartData {
    pub timestamp: String,
    pub value: f64,
    pub ucl: f64,
    pub cl: f64,
    pub lcl: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subgroup_data: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlLimits {
    pub ucl: f64,
    pub cl: f64,
    pub lcl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpecialCause {
    OutOfControl {
        value: f64,
        ucl: f64,
        lcl: f64,
    },
    Shift {
        direction: ShiftDirection,
        count: usize,
    },
    Trend {
        direction: TrendDirection,
        count: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShiftDirection {
    Above,
    Below,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
}

#[derive(Debug, Error)]
pub enum ChartError {
    #[error("Failed to read chart file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse CSV: {0}")]
    ParseError(#[from] csv::Error),
    #[error("Invalid chart data: {0}")]
    InvalidData(String),
    #[error("Not enough data points: need at least {0}, got {1}")]
    InsufficientData(usize, usize),
}

pub struct ChartManager {
    chart_dir: PathBuf,
}

impl ChartManager {
    pub fn new(chart_dir: PathBuf) -> Self {
        Self { chart_dir }
    }

    pub fn read_chart(&self, chart_file: &str) -> Result<Vec<ChartData>, ChartError> {
        let path = self.chart_dir.join(chart_file);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut reader = csv::Reader::from_path(&path)?;
        let mut data = Vec::new();

        for result in reader.deserialize() {
            let record: ChartData = result?;
            data.push(record);
        }

        Ok(data)
    }

    pub fn write_chart(&self, chart_file: &str, data: &[ChartData]) -> Result<(), ChartError> {
        let path = self.chart_dir.join(chart_file);

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut writer = csv::Writer::from_path(&path)?;
        for record in data {
            writer.serialize(record)?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn append_chart(&self, chart_file: &str, new_data: ChartData) -> Result<(), ChartError> {
        let mut data = self.read_chart(chart_file)?;
        data.push(new_data);
        self.write_chart(chart_file, &data)
    }
}
