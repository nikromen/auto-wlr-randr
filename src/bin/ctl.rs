use auto_wlr_randr::ipc::{Command, get_socket_path};
use clap::{Parser, Subcommand};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

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
    /// Changes the current output configuration to the specified profile
    /// defined in the configuration file.
    Switch {
        /// Name of the profile to switch to
        profile_name: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let socket_path = get_socket_path();

    if !socket_path.exists() {
        anyhow::bail!(
            "Error: Daemon socket not found at {}.\nIs the auto-wlr-randr daemon running?",
            socket_path.display()
        );
    }

    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|e| anyhow::anyhow!("Failed to connect to daemon socket: {}", e))?;

    let command = match cli.command {
        CliCommand::Reload => Command::Reload,
        CliCommand::Status => Command::Status,
        CliCommand::Switch { profile_name } => Command::Switch(profile_name),
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
