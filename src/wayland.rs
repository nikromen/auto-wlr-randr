use crate::config::{Config, Profile};
use crate::output::{OutputInfo, get_outputs};
use anyhow::Result;
use std::collections::HashMap;
use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle,
    globals::{GlobalListContents, registry_queue_init},
    protocol::{wl_output, wl_registry},
};

pub struct WaylandState {
    pub config: Config,
    pub outputs: Vec<OutputInfo>,
    pub active_profile_id: Option<String>,
    pub name_map: HashMap<String, String>,
    pending_update: bool,
}

impl WaylandState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            outputs: Vec::new(),
            active_profile_id: None,
            name_map: HashMap::new(),
            pending_update: false,
        }
    }

    pub fn refresh_outputs(&mut self) {
        match get_outputs() {
            Ok(outputs) => {
                log::debug!(
                    "Got {} outputs: {:?}",
                    outputs.len(),
                    outputs.iter().map(|o| &o.name).collect::<Vec<_>>()
                );
                self.outputs = outputs;
                self.evaluate_profiles(false);
            }
            Err(e) => {
                log::error!("Failed to get outputs: {e}");
            }
        }
    }

    fn run_commands(commands: &[String]) {
        for command in commands {
            if command.is_empty() {
                log::warn!("Encountered an empty command, skipping.");
                continue;
            }

            log::debug!("Executing command: {command}");
            if let Err(e) = std::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .spawn()
            {
                log::error!("Failed to execute command '{command}': {e}");
            }
        }
    }

    fn activate_profile(&mut self, profile_id: &str, profile: &Profile, reload: bool) {
        if self.active_profile_id.as_deref() == Some(profile_id) && !reload {
            log::debug!("Profile '{profile_id}' is already active, skipping.");
            return;
        }

        log::info!("Activating profile: '{profile_id}'");
        let commands = profile.generate_commands(&self.name_map);
        Self::run_commands(&commands);
        self.active_profile_id = Some(profile_id.to_string());
    }

    pub fn evaluate_profiles(&mut self, reload: bool) {
        let matched = self
            .config
            .find_matching_profile(&self.outputs)
            .map(|(id, profile, name_map)| (id.to_string(), profile.clone(), name_map));

        match matched {
            Some((profile_id, profile, name_map)) => {
                self.name_map = name_map;
                self.activate_profile(&profile_id, &profile, reload);
            }
            None => {
                if self.active_profile_id.take().is_some() {
                    log::warn!("No matching profile found. Clearing active profile.");
                }

                if self.outputs.is_empty() {
                    log::warn!("No profile matches, and no outputs are connected.");
                } else {
                    let outputs_str = self
                        .outputs
                        .iter()
                        .map(|o| o.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    log::warn!("No profile matches active outputs: [{outputs_str}]");
                }
            }
        }
    }

    pub fn apply_profile_by_name(&mut self, profile_id: &str) -> Result<String> {
        let profile = self
            .config
            .profiles
            .get(profile_id)
            .ok_or_else(|| anyhow::anyhow!("Profile '{profile_id}' not found."))?
            .clone();

        self.activate_profile(profile_id, &profile, false);
        Ok(format!("Profile '{profile_id}' applied successfully."))
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global { interface, .. } if interface == "wl_output" => {
                log::debug!("Output added, scheduling refresh");
                state.pending_update = true;
            }
            wl_registry::Event::GlobalRemove { .. } => {
                log::debug!("Global removed, scheduling refresh");
                state.pending_update = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_output::WlOutput,
        _event: wl_output::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

pub fn init_wayland(
    config: Config,
) -> Result<(Connection, WaylandState, EventQueue<WaylandState>)> {
    let conn = Connection::connect_to_env()?;
    let (_, event_queue) = registry_queue_init::<WaylandState>(&conn)?;

    let mut state = WaylandState::new(config);

    log::info!("Initializing Wayland connection...");

    state.refresh_outputs();

    Ok((conn, state, event_queue))
}

pub fn process_events(
    event_queue: &mut EventQueue<WaylandState>,
    state: &mut WaylandState,
) -> Result<()> {
    event_queue.roundtrip(state)?;

    if state.pending_update {
        state.pending_update = false;
        state.refresh_outputs();
    }

    Ok(())
}
