use auto_wlr_randr::config::{Config, ValidationLevel};
use auto_wlr_randr::ipc::{Command, get_socket_path};
use auto_wlr_randr::output::get_outputs;
use clap::{Parser, Subcommand};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Control tool for auto-wlr-randr daemon which automatically manages display configurations for Wayland compositors that implement the wlr-output-management protocol"
)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    /// Reload the configuration file
    ///
    /// Forces the daemon to reload its configuration file, applying any changes
    /// made since the daemon was started or the config was last reloaded.
    Reload,

    /// Display current status information
    ///
    /// Shows information about the currently active profile, connected
    /// outputs, and daemon state.
    Status,

    /// Switch to a specific profile
    ///
    /// The profile must match currently connected outputs (same rules as
    /// automatic profile matching). Use --force to apply regardless.
    Switch {
        /// Name of the profile to switch to
        profile_name: String,

        /// Apply the profile even if it does not match current outputs
        #[arg(long)]
        force: bool,
    },

    /// Validate a configuration file
    ///
    /// Performs static checks on the configuration without starting the daemon
    /// or changing display settings.
    Validate {
        /// Path to configuration file
        #[arg(short, long)]
        config: PathBuf,
    },

    /// Show what would be applied for current outputs
    ///
    /// Evaluates profile matching against currently connected outputs and
    /// prints the matching profile, wlr-randr arguments, and exec commands
    /// without applying them. Does not require a running daemon.
    DryRun {
        /// Path to configuration file
        #[arg(short, long)]
        config: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CliCommand::Validate { config } => validate_config(&config),
        CliCommand::DryRun { config } => dry_run_config(&config),
        command => send_daemon_command(command),
    }
}

fn validate_config(config_path: &PathBuf) -> anyhow::Result<()> {
    let config = Config::load_from_file(config_path)?;
    let report = config.validate();

    for issue in &report.issues {
        let prefix = match issue.level {
            ValidationLevel::Warning => "warning",
            ValidationLevel::Error => "error",
        };
        eprintln!("{prefix}: {}", issue.message);
    }

    if report.has_errors() {
        std::process::exit(1);
    }

    if report.issues.is_empty() {
        println!("Configuration is valid.");
    } else {
        println!("Configuration is valid with warnings.");
    }

    Ok(())
}

fn dry_run_config(config_path: &PathBuf) -> anyhow::Result<()> {
    let config = Config::load_from_file(config_path)?;
    let connected_outputs = get_outputs()?;
    let result = config.dry_run(&connected_outputs);
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn send_daemon_command(command: CliCommand) -> anyhow::Result<()> {
    let socket_path = get_socket_path();

    if !socket_path.exists() {
        anyhow::bail!(
            "Error: Daemon socket not found at {}.\nIs the auto-wlr-randr daemon running?",
            socket_path.display()
        );
    }

    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|e| anyhow::anyhow!("Failed to connect to daemon socket: {}", e))?;

    let command = match command {
        CliCommand::Reload => Command::Reload,
        CliCommand::Status => Command::Status,
        CliCommand::Switch {
            profile_name,
            force,
        } => Command::Switch {
            profile: profile_name,
            force,
        },
        CliCommand::Validate { .. } | CliCommand::DryRun { .. } => {
            unreachable!("handled before daemon command dispatch")
        }
    };

    let request = serde_json::to_vec(&command)?;
    stream.write_all(&request)?;
    // Shut down the write half to signal the end of the request.
    stream.shutdown(std::net::Shutdown::Write)?;

    let mut response_bytes = Vec::new();
    stream.read_to_end(&mut response_bytes)?;

    let response: Result<String, String> = serde_json::from_slice(&response_bytes)?;
    match response {
        Ok(success_message) => {
            println!("{success_message}");
            Ok(())
        }
        Err(error_message) => {
            eprintln!("Error: {error_message}");
            std::process::exit(1);
        }
    }
}
