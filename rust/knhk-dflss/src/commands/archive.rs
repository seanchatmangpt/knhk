// Evidence Archive Management
use anyhow::Context;
use chrono::Utc;
use std::fs;
use std::path::Path;

// Helper to convert anyhow::Error to clap_noun_verb error
fn to_cnv_error(e: anyhow::Error) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(e.to_string())
}
// Archive DFLSS metrics and charts for certification

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Serialize)]
pub struct ArchiveResult {
    pub archived: Vec<String>,
    pub location: String,
}

/// Archive current metrics to evidence directory
#[verb("archive metrics")]
pub fn archive_metrics(metrics: PathBuf, dir: PathBuf) -> CnvResult<ArchiveResult> {
    info!(
        "Archiving metrics from: {} to: {}",
        metrics.display(),
        dir.display()
    );

    // Generate timestamped archive directory
    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S");
    let archive_dir = dir.join("archive").join(format!("metrics_{}", timestamp));

    fs::create_dir_all(&archive_dir)
        .context("Failed to create archive directory")
        .map_err(to_cnv_error)?;

    let mut archived = Vec::new();

    // Copy metrics file
    if metrics.exists() {
        let dest = archive_dir.join(metrics.file_name().unwrap_or_default());
        fs::copy(&metrics, &dest)
            .context("Failed to copy metrics file")
            .map_err(to_cnv_error)?;
        archived.push(dest.to_string_lossy().to_string());
        info!("Archived: {}", dest.display());
    } else {
        // If metrics is a directory, copy all JSON files
        if metrics.is_dir() {
            for entry in fs::read_dir(&metrics)
                .context("Failed to read metrics directory")
                .map_err(to_cnv_error)?
            {
                let entry = entry
                    .context("Failed to read directory entry")
                    .map_err(to_cnv_error)?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let dest = archive_dir.join(path.file_name().unwrap_or_default());
                    fs::copy(&path, &dest)
                        .context("Failed to copy metrics file")
                        .map_err(to_cnv_error)?;
                    archived.push(dest.to_string_lossy().to_string());
                    info!("Archived: {}", dest.display());
                }
            }
        } else {
            return Err(to_cnv_error(anyhow::anyhow!(
                "Metrics path does not exist: {}",
                metrics.display()
            )));
        }
    }

    Ok(ArchiveResult {
        archived,
        location: archive_dir.to_string_lossy().to_string(),
    })
}

/// Archive SPC charts
#[verb("archive charts")]
pub fn archive_charts(source: PathBuf, dir: PathBuf) -> CnvResult<ArchiveResult> {
    info!(
        "Archiving charts from: {} to: {}",
        source.display(),
        dir.display()
    );

    // Generate timestamped archive directory
    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S");
    let archive_dir = dir.join("archive").join(format!("charts_{}", timestamp));

    fs::create_dir_all(&archive_dir)
        .context("Failed to create archive directory")
        .map_err(to_cnv_error)?;

    let mut archived = Vec::new();

    // Copy chart files (CSV)
    if source.is_dir() {
        for entry in fs::read_dir(&source)
            .context("Failed to read source directory")
            .map_err(to_cnv_error)?
        {
            let entry = entry
                .context("Failed to read directory entry")
                .map_err(to_cnv_error)?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("csv") {
                let dest = archive_dir.join(path.file_name().unwrap_or_default());
                fs::copy(&path, &dest)
                    .context("Failed to copy chart file")
                    .map_err(to_cnv_error)?;
                archived.push(dest.to_string_lossy().to_string());
                info!("Archived: {}", dest.display());
            }
        }
    } else if source.exists() {
        let dest = archive_dir.join(source.file_name().unwrap_or_default());
        fs::copy(&source, &dest)
            .context("Failed to copy chart file")
            .map_err(to_cnv_error)?;
        archived.push(dest.to_string_lossy().to_string());
        info!("Archived: {}", dest.display());
    } else {
        return Err(to_cnv_error(anyhow::anyhow!(
            "Source path does not exist: {}",
            source.display()
        )));
    }

    Ok(ArchiveResult {
        archived,
        location: archive_dir.to_string_lossy().to_string(),
    })
}

/// Archive all DFLSS evidence
#[verb("archive all")]
pub fn archive_all(dir: PathBuf) -> CnvResult<ArchiveResult> {
    info!("Archiving all DFLSS evidence to: {}", dir.display());

    // Generate timestamped archive directory
    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S");
    let archive_dir = dir.join("archive").join(format!("dflss_{}", timestamp));

    fs::create_dir_all(&archive_dir)
        .context("Failed to create archive directory")
        .map_err(to_cnv_error)?;

    let mut archived = Vec::new();

    // Archive metrics
    let metrics_dir = dir.join("metrics");
    if metrics_dir.exists() {
        let metrics_archive = archive_dir.join("metrics");
        fs::create_dir_all(&metrics_archive)
            .context("Failed to create metrics archive directory")
            .map_err(to_cnv_error)?;

        for entry in fs::read_dir(&metrics_dir)
            .context("Failed to read metrics directory")
            .map_err(to_cnv_error)?
        {
            let entry = entry
                .context("Failed to read directory entry")
                .map_err(to_cnv_error)?;
            let path = entry.path();
            if path.is_file() {
                let dest = metrics_archive.join(path.file_name().unwrap_or_default());
                fs::copy(&path, &dest)
                    .context("Failed to copy metrics file")
                    .map_err(to_cnv_error)?;
                archived.push(dest.to_string_lossy().to_string());
            }
        }
    }

    // Archive charts
    let charts_dir = dir.join("charts");
    if charts_dir.exists() {
        let charts_archive = archive_dir.join("charts");
        fs::create_dir_all(&charts_archive)
            .context("Failed to create charts archive directory")
            .map_err(to_cnv_error)?;

        for entry in fs::read_dir(&charts_dir)
            .context("Failed to read charts directory")
            .map_err(to_cnv_error)?
        {
            let entry = entry
                .context("Failed to read directory entry")
                .map_err(to_cnv_error)?;
            let path = entry.path();
            if path.is_file() {
                let dest = charts_archive.join(path.file_name().unwrap_or_default());
                fs::copy(&path, &dest)
                    .context("Failed to copy chart file")
                    .map_err(to_cnv_error)?;
                archived.push(dest.to_string_lossy().to_string());
            }
        }
    }

    // Archive validation results
    let validation_dir = dir.join("validation");
    if validation_dir.exists() {
        let validation_archive = archive_dir.join("validation");
        fs::create_dir_all(&validation_archive)
            .context("Failed to create validation archive directory")
            .map_err(to_cnv_error)?;

        for entry in fs::read_dir(&validation_dir)
            .context("Failed to read validation directory")
            .map_err(to_cnv_error)?
        {
            let entry = entry
                .context("Failed to read directory entry")
                .map_err(to_cnv_error)?;
            let path = entry.path();
            if path.is_file() {
                let dest = validation_archive.join(path.file_name().unwrap_or_default());
                fs::copy(&path, &dest)
                    .context("Failed to copy validation file")
                    .map_err(to_cnv_error)?;
                archived.push(dest.to_string_lossy().to_string());
            }
        }
    }

    info!(
        "Archived {} files to: {}",
        archived.len(),
        archive_dir.display()
    );

    Ok(ArchiveResult {
        archived,
        location: archive_dir.to_string_lossy().to_string(),
    })
}
