// CLI command modules
use anyhow::Context;

// Helper to convert anyhow::Error to clap_noun_verb error
fn to_cnv_error(e: anyhow::Error) -> clap_noun_verb::NounVerbError {
    clap_noun_verb::NounVerbError::execution_error(e.to_string())
}

pub mod archive;
pub mod autonomics;
pub mod capability;
pub mod charts;
pub mod dflss;
pub mod fortune5;
pub mod innovation;
pub mod metrics;
pub mod mining;
pub mod predictive;
pub mod report;
pub mod validation;
