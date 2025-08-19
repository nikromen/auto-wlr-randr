use crate::config::{Config, OutputData, Profile};
use crate::output::{OutputInfo, PendingOutputInfo, clean_description};
use anyhow::Result;
use std::collections::HashMap;
use wayland_client::EventQueue;
use wayland_client::{
    Connection, Dispatch, QueueHandle,
    globals::{GlobalListContents, registry_queue_init},
    protocol::{wl_output, wl_registry},
};

pub struct WaylandState {
    pub config: Config,
    pub pending_outputs: HashMap<u32, PendingOutputInfo>,
    pub outputs: HashMap<u32, OutputInfo>,
    pub active_profile_name: Option<String>,
    pub name_map: HashMap<String, String>,
}

impl WaylandState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            pending_outputs: HashMap::new(),
            outputs: HashMap::new(),
            active_profile_name: None,
            name_map: HashMap::new(),
        }
    }

    pub fn add_pending_output(
        &mut self,
        id: u32,
        version: u32,
        qh: &QueueHandle<Self>,
        registry: &wl_registry::WlRegistry,
    ) {
        let output_proxy = registry.bind::<wl_output::WlOutput, _, _>(id, version, qh, id);
        let pending_info = PendingOutputInfo {
            proxy: output_proxy,
            name: None,
            description: None,
        };
        log::debug!(
            "Detected new output with ID #{id} {:?}, waiting for details.",
            &pending_info
        );
        self.pending_outputs.insert(id, pending_info);
    }

    pub fn finalize_output(&mut self, output_id: u32) {
        if self.outputs.contains_key(&output_id) {
            log::debug!("Ignoring repeated Done event for output ID #{output_id}");
            return;
        }

        let Some(pending) = self.pending_outputs.get(&output_id) else {
            log::warn!("Received Done event for unknown output ID #{output_id}");
            return;
        };

        let (Some(name), Some(description)) = (&pending.name, &pending.description) else {
            log::warn!(
                "Output (ID: #{output_id}) received 'Done' event but is missing name or description. Discarding."
            );
            return;
        };

        let cleaned_description = clean_description(description, name);

        let complete_info = OutputInfo {
            id: output_id,
            name: name.clone(),
            description: cleaned_description,
        };

        log::debug!("Output fully identified: {complete_info}");
        self.outputs.insert(output_id, complete_info);

        if let Some(pending) = self.pending_outputs.remove(&output_id) {
            pending.proxy.release();
        }

        self.evaluate_profiles(false);
    }

    pub fn run_commands(&self, commands: &[String]) {
        if commands.is_empty() {
            log::debug!("No commands to execute.");
            return;
        }

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

    pub fn active_profile(
        &mut self,
        profile: &Profile,
        name_map: &HashMap<String, String>,
        reload: bool,
    ) {
        log::debug!("Activating profile: {}", profile.name);
        if self.active_profile_name.as_ref() != Some(&profile.name) || reload {
            log::info!(
                "Found matching profile: '{}'. Applying changes.",
                profile.name
            );
            let commands = profile.generate_commands(name_map);
            self.run_commands(&commands);
            self.active_profile_name = Some(profile.name.clone());
        } else {
            log::debug!(
                "Profile '{}' is already active, skipping activation.",
                profile.name
            );
        }
    }

    pub fn evaluate_profiles(&mut self, reload: bool) {
        let connected_outputs: Vec<OutputData> = self
            .outputs
            .values()
            .map(|output| OutputData {
                output_name: output.name.clone(),
                manufacturer: output.description.clone(),
            })
            .collect();

        if let Some((profile, name_map)) = self.config.find_matching_profile(&connected_outputs) {
            self.name_map = name_map.clone();
            let profile_copy = profile.clone();
            let name_map_copy = self.name_map.clone();
            self.active_profile(&profile_copy, &name_map_copy, reload);
        } else {
            if self.active_profile_name.is_some() {
                log::warn!("No matching profile found. Clearing active profile.");
                self.active_profile_name = None;
            }
            let outputs_str = self
                .outputs
                .values()
                .map(|o| o.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            if !outputs_str.is_empty() {
                log::warn!("No profile matches active outputs: [{outputs_str}]");
            } else {
                log::warn!("No profile matches, and no outputs are connected.");
            }
        }
    }

    pub fn apply_profile_by_name(&mut self, profile_name: &str) -> Result<String> {
        let profile = self
            .config
            .profiles
            .get(profile_name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found.", profile_name))?
            .clone();

        let name_map_clone = self.name_map.clone();
        self.active_profile(&profile, &name_map_clone, false);
        Ok(format!("Profile '{profile_name}' applied successfully."))
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == "wl_output" {
                state.add_pending_output(name, version, qh, registry);
            }
        } else if let wl_registry::Event::GlobalRemove { name } = event
            && let Some(removed_output) = state.outputs.remove(&name)
        {
            log::info!("Output removed: {removed_output}");
            state.evaluate_profiles(false);
        }
    }
}

impl Dispatch<wl_output::WlOutput, u32> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &wl_output::WlOutput,
        event: wl_output::Event,
        output_id: &u32,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            wl_output::Event::Name { name } => {
                if let Some(pending_info) = state.pending_outputs.get_mut(output_id) {
                    log::debug!("Name received for output: {pending_info}");
                    pending_info.name = Some(name);
                }
            }
            wl_output::Event::Description { description } => {
                if let Some(pending_info) = state.pending_outputs.get_mut(output_id) {
                    log::debug!("Description received for output: {pending_info}");
                    pending_info.description = Some(description);
                }
            }
            wl_output::Event::Done => {
                log::debug!("Done event received for output ID #{output_id}");
                state.finalize_output(*output_id);
            }
            _ => {}
        }
    }
}

pub fn init_wayland(
    config: Config,
) -> Result<(
    Connection,
    WaylandState,
    wayland_client::EventQueue<WaylandState>,
)> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init::<WaylandState>(&conn)?;
    let qh = event_queue.handle();

    let mut state = WaylandState::new(config);

    log::info!("Initializing Wayland connection...");

    let registry = globals.registry();
    for global in globals.contents().clone_list() {
        if global.interface == "wl_output" {
            state.add_pending_output(global.name, global.version, &qh, registry);
        }
    }

    event_queue.roundtrip(&mut state)?;

    Ok((conn, state, event_queue))
}

pub fn process_events(
    event_queue: &mut EventQueue<WaylandState>,
    state: &mut WaylandState,
) -> Result<()> {
    event_queue.dispatch_pending(state)?;
    event_queue.roundtrip(state)?;
    Ok(())
}
