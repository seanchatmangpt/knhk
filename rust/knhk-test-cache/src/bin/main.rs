//! Main binary for test cache daemon
//!
//! CLI interface for managing the autonomic test cache daemon.

use clap::{Parser, Subcommand};
use knhk_test_cache::Daemon;
use std::path::PathBuf;
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "knhk-test-cache")]
#[command(about = "Autonomic test cache daemon - keeps test binaries pre-compiled")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon
    Start {
        /// Project root directory (default: current directory)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },
    /// Stop the daemon
    Stop {
        /// Project root directory (default: current directory)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },
    /// Check daemon status
    Status {
        /// Project root directory (default: current directory)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },
    /// Force rebuild of test binaries
    Rebuild {
        /// Project root directory (default: current directory)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    // Determine project root
    let project_root = match &cli.command {
        Commands::Start { root }
        | Commands::Stop { root }
        | Commands::Status { root }
        | Commands::Rebuild { root } => root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
    };

    let daemon = Daemon::new(project_root);

    match cli.command {
        Commands::Start { .. } => {
            daemon.start().await?;
        }
        Commands::Stop { .. } => {
            daemon.stop().await?;
        }
        Commands::Status { .. } => {
            let status = daemon.status()?;
            println!("Daemon Status:");
            println!("  Running: {}", status.running);
            println!("  Cache entries: {}", status.cache_stats.entry_count);
            println!(
                "  Cache size: {} bytes",
                status.cache_stats.total_size_bytes
            );
            if let Some(hash) = status.current_code_hash {
                println!("  Current code hash: {}", hash);
            }
            println!("  PID file: {}", status.pid_file.display());
            println!("  Log file: {}", status.log_file.display());
        }
        Commands::Rebuild { .. } => {
            daemon.rebuild().await?;
        }
    }

    Ok(())
}
