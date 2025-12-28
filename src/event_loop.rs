use crate::config::Config;
use crate::ipc::Command;
use crate::ipc::{self, SocketListener};
use crate::wayland;
use crate::wayland::WaylandState;
use anyhow::Result;
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use std::os::unix::io::{AsFd, AsRawFd};

const WAYLAND_EVENT: Token = Token(0);
const IPC_EVENT: Token = Token(1);

pub fn handle_command(command: Command, state: &mut WaylandState) -> Result<String> {
    match command {
        Command::Reload => {
            log::info!("Reloading configuration...");
            state.config.reload_config()?;
            state.evaluate_profiles(true);
            Ok("Configuration reloaded successfully.".into())
        }
        Command::Status => {
            log::info!("Fetching status...");
            let json_output = serde_json::json!({
                "active_profile": state.active_profile_id.as_deref().unwrap_or("None"),
                "connected_outputs": state.outputs.iter().map(|o| o.to_string()).collect::<Vec<_>>(),
            });
            Ok(serde_json::to_string_pretty(&json_output)?)
        }
        Command::Switch(profile_name) => {
            log::info!("Switching to profile: {profile_name}");
            state.apply_profile_by_name(&profile_name)?;
            Ok(format!("Profile switched successfully to {profile_name}"))
        }
    }
}

pub fn start_event_loop(config: Config) -> Result<()> {
    let (conn, mut state, mut event_queue) = wayland::init_wayland(config)?;
    let wayland_fd = conn.as_fd();

    log::info!("Wayland event loop initialized, starting IPC server...");

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);

    let socket_path = ipc::get_socket_path();
    let mut listener = SocketListener::bind(&socket_path)?;

    poll.registry().register(
        &mut SourceFd(&wayland_fd.as_raw_fd()),
        WAYLAND_EVENT,
        Interest::READABLE,
    )?;

    poll.registry()
        .register(&mut listener.listener, IPC_EVENT, Interest::READABLE)?;

    log::info!("Event loop started, waiting for events...");

    loop {
        poll.poll(&mut events, None)?;
        for event in events.iter() {
            match event.token() {
                WAYLAND_EVENT => {
                    // Process Wayland events by letting the event queue handle a single round of events
                    wayland::process_events(&mut event_queue, &mut state)?;
                }
                IPC_EVENT => {
                    // Handle IPC requests
                    let result = ipc::handle_client_request(&listener, |cmd| {
                        handle_command(cmd, &mut state)
                    });

                    match result {
                        Ok(response) => {
                            log::debug!("IPC request handled successfully: {response}");
                        }
                        Err(e) => {
                            if e.to_string().contains("WouldBlock") {
                                // This is normal for non-blocking sockets, ignore
                                log::debug!("Non-blocking socket error: {e}");
                            } else {
                                log::error!("Error handling IPC request: {e}");
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
