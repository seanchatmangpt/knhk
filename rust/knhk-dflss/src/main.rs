//! KNHK DFLSS CLI
//! Design for Lean Six Sigma metrics collection and SPC chart management

use clap_noun_verb::Result;

// Import all command modules for auto-discovery
mod commands {
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
}

// Import internal modules
mod internal {
    pub mod capability;
    pub mod chart;
    pub mod metrics;
    pub mod quality;
    pub mod rules;
    pub mod statistics;
    pub mod validation;
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
