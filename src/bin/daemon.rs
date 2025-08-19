use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

use auto_wlr_randr::{config::Config, event_loop};

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Daemon that automatically manages display configurations for Wayland compositors that implement the wlr-output-management protocol"
)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long)]
    config: PathBuf,

    /// Set log verbosity level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&cli.log_level))
        .init();

    let config_path = &cli.config;
    log::info!("Loading configuration from: {:?}", config_path);
    let config = Config::load_from_file(config_path)
        .with_context(|| format!("Failed to load configuration from {:?}", config_path))?;

    log::info!("Configuration loaded successfully.");

    event_loop::start_event_loop(config)?;
    Ok(())
}
