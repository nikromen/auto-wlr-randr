use crate::config::{Config, Profile};
use crate::output::{OutputInfo, get_outputs};
use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[cfg(test)]
const NO_MATCH_HOOK_DELAY: Duration = Duration::from_millis(0);
#[cfg(not(test))]
const NO_MATCH_HOOK_DELAY: Duration = Duration::from_millis(500);
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
    pending_no_match_deadline: Option<Instant>,
    last_no_match_outputs: Option<Vec<OutputInfo>>,
    no_match_hook_outputs: Option<Vec<OutputInfo>>,
}

impl WaylandState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            outputs: Vec::new(),
            active_profile_id: None,
            name_map: HashMap::new(),
            pending_update: false,
            pending_no_match_deadline: None,
            last_no_match_outputs: None,
            no_match_hook_outputs: None,
        }
    }

    fn cancel_no_match_hook(&mut self) {
        self.pending_no_match_deadline = None;
        self.last_no_match_outputs = None;
        self.no_match_hook_outputs = None;
    }

    fn schedule_no_match_hook(&mut self, reload: bool) {
        if self.config.on_no_match_exec.is_empty() {
            self.pending_no_match_deadline = None;
            return;
        }

        if self.no_match_hook_outputs.as_ref() == Some(&self.outputs) {
            return;
        }

        if reload {
            self.no_match_hook_outputs = None;
        }

        let outputs_changed = self.last_no_match_outputs.as_ref() != Some(&self.outputs);
        if outputs_changed || reload || self.pending_no_match_deadline.is_none() {
            if outputs_changed || reload {
                self.no_match_hook_outputs = None;
            }
            self.pending_no_match_deadline = Some(Instant::now() + NO_MATCH_HOOK_DELAY);
        }

        self.last_no_match_outputs = Some(self.outputs.clone());
    }

    pub fn next_poll_timeout(&self) -> Option<Duration> {
        self.pending_no_match_deadline
            .map(|deadline| deadline.saturating_duration_since(Instant::now()))
    }

    pub fn poll_no_match_hook(&mut self) {
        if self.pending_no_match_deadline.is_none() {
            return;
        }

        if Instant::now() < self.pending_no_match_deadline.unwrap() {
            return;
        }

        self.pending_no_match_deadline = None;

        if self.config.find_matching_profile(&self.outputs).is_some() {
            return;
        }

        if self.config.on_no_match_exec.is_empty() {
            return;
        }

        if self.no_match_hook_outputs.as_ref() == Some(&self.outputs) {
            return;
        }

        log::info!("No profile match persisted, running on_no_match_exec");
        Self::run_exec(&self.config.on_no_match_exec);
        self.no_match_hook_outputs = Some(self.outputs.clone());
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

    fn run_wlr_randr(args: &[String]) {
        log::debug!("Executing wlr-randr with args: {args:?}");
        if let Err(e) = std::process::Command::new("wlr-randr").args(args).spawn() {
            log::error!("Failed to execute wlr-randr: {e}");
        }
    }

    fn run_exec(commands: &[String]) {
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

    fn activate_profile(&mut self, profile_id: &str, profile: &Profile, force: bool) {
        if self.active_profile_id.as_deref() == Some(profile_id) && !force {
            log::debug!("Profile '{profile_id}' is already active, skipping.");
            return;
        }

        log::info!("Activating profile: '{profile_id}'");
        if let Some(args) = profile.generate_wlr_randr_args(&self.name_map) {
            Self::run_wlr_randr(&args);
        }
        Self::run_exec(&profile.exec);
        self.active_profile_id = Some(profile_id.to_string());
    }

    pub fn evaluate_profiles(&mut self, reload: bool) {
        let matched = self
            .config
            .find_matching_profile(&self.outputs)
            .map(|(id, profile, name_map)| (id.to_string(), profile.clone(), name_map));

        match matched {
            Some((profile_id, profile, name_map)) => {
                self.cancel_no_match_hook();
                let force = reload || self.name_map != name_map;
                self.name_map = name_map;
                self.activate_profile(&profile_id, &profile, force);
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

                self.schedule_no_match_hook(reload);
            }
        }
    }

    pub fn apply_profile_by_name(&mut self, profile_id: &str, force: bool) -> Result<String> {
        let profile = self
            .config
            .profiles
            .get(profile_id)
            .ok_or_else(|| anyhow::anyhow!("Profile '{profile_id}' not found."))?
            .clone();

        let old_name_map = self.name_map.clone();

        if force {
            self.name_map = profile.resolve_name_map(&self.outputs);
        } else {
            self.name_map = profile
                .match_outputs(&self.outputs)
                .ok_or_else(|| {
                    let connected = self.outputs.len();
                    let expected = profile.settings.len();
                    anyhow::anyhow!(
                        "Profile '{profile_id}' does not match current outputs ({connected} connected, profile expects {expected})"
                    )
                })?;
        }

        let apply = force || self.name_map != old_name_map;
        self.cancel_no_match_hook();
        self.activate_profile(profile_id, &profile, apply);
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

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    fn state_with_hook(profiles: IndexMap<String, Profile>) -> WaylandState {
        use assert_fs::TempDir;
        use assert_fs::prelude::*;

        let temp = TempDir::new().unwrap();
        let config_file = temp.child("config.toml");
        config_file
            .write_str("on_no_match_exec = [\"true\"]\n\n[profile.dummy]\n")
            .unwrap();

        let mut config = Config::load_from_file(config_file.path()).unwrap();
        config.profiles = profiles;

        let mut state = WaylandState::new(config);
        state.outputs = vec![OutputInfo {
            name: "TEST-1".to_string(),
            make: None,
            model: None,
            serial: None,
        }];
        state
    }

    #[test]
    fn test_no_match_hook_fires_after_delay() {
        let mut profiles = IndexMap::new();
        profiles.insert(
            "empty".to_string(),
            Profile {
                exec: vec![],
                settings: vec![],
            },
        );

        let mut state = state_with_hook(profiles);
        state.evaluate_profiles(false);

        assert!(state.pending_no_match_deadline.is_some());
        assert!(state.no_match_hook_outputs.is_none());

        state.poll_no_match_hook();

        assert!(state.pending_no_match_deadline.is_none());
        assert_eq!(state.no_match_hook_outputs.as_ref(), Some(&state.outputs));
    }

    #[test]
    fn test_no_match_hook_cancelled_when_profile_matches() {
        let mut profiles = IndexMap::new();
        profiles.insert(
            "empty".to_string(),
            Profile {
                exec: vec![],
                settings: vec![],
            },
        );

        let mut state = state_with_hook(profiles);
        state.evaluate_profiles(false);
        assert!(state.pending_no_match_deadline.is_some());

        state.config.profiles.insert(
            "match".to_string(),
            Profile {
                exec: vec![],
                settings: vec![crate::config::OutputSetting {
                    output: "TEST-1".into(),
                    on: None,
                    mode: None,
                    preferred: false,
                    pos: None,
                    left_of: None,
                    right_of: None,
                    above: None,
                    below: None,
                    transform: None,
                    scale: None,
                    adaptive_sync: None,
                }],
            },
        );
        state.evaluate_profiles(false);

        assert!(state.pending_no_match_deadline.is_none());
        state.poll_no_match_hook();
        assert!(state.no_match_hook_outputs.is_none());
    }

    #[test]
    fn test_no_match_hook_not_fired_twice_for_same_outputs() {
        let mut profiles = IndexMap::new();
        profiles.insert(
            "empty".to_string(),
            Profile {
                exec: vec![],
                settings: vec![],
            },
        );

        let mut state = state_with_hook(profiles);
        state.evaluate_profiles(false);
        state.poll_no_match_hook();

        state.evaluate_profiles(false);
        assert!(state.pending_no_match_deadline.is_none());
    }
}
