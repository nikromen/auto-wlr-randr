use auto_wlr_randr::config::{Config, OutputSetting, Profile};
use auto_wlr_randr::event_loop::handle_command;
use auto_wlr_randr::ipc::Command;
use auto_wlr_randr::output::OutputInfo;
use auto_wlr_randr::wayland::WaylandState;
use indexmap::IndexMap;
use std::collections::HashMap;

fn make_test_output() -> OutputInfo {
    OutputInfo {
        name: "TEST-1".to_string(),
        make: Some("Test Inc.".to_string()),
        model: Some("TestModel".to_string()),
        serial: None,
    }
}

fn create_test_state(profiles: IndexMap<String, Profile>) -> WaylandState {
    let mut config = Config::load_from_file("config.toml").unwrap_or_else(|_| {
        use assert_fs::TempDir;
        use assert_fs::prelude::*;

        let temp = TempDir::new().unwrap();
        let config_file = temp.child("config.toml");
        config_file.write_str("[profile.test]\n").unwrap();
        Config::load_from_file(config_file.path()).expect("Failed to load test config")
    });

    config.profiles = profiles;

    let mut state = WaylandState::new(config);
    state.outputs = vec![make_test_output()];
    state
}

fn matching_test_profile() -> Profile {
    Profile {
        exec: vec![],
        settings: vec![OutputSetting {
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
    }
}

fn non_matching_test_profile() -> Profile {
    Profile {
        exec: vec![],
        settings: vec![],
    }
}

fn serial_test_profile() -> Profile {
    Profile {
        exec: vec![],
        settings: vec![OutputSetting {
            output: "ABC".into(),
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
    }
}

fn make_output_with_serial(name: &str, serial: &str) -> OutputInfo {
    OutputInfo {
        name: name.to_string(),
        make: Some("Test Inc.".to_string()),
        model: Some("TestModel".to_string()),
        serial: Some(serial.to_string()),
    }
}

#[test]
fn test_handle_command_status() {
    let mut profiles = IndexMap::new();
    profiles.insert("test".to_string(), matching_test_profile());

    let mut state = create_test_state(profiles);
    state.active_profile_id = Some("test".to_string());

    let result = handle_command(Command::Status, &mut state);

    assert!(result.is_ok());
    let json_str = result.unwrap();

    assert!(json_str.contains("test"));
    assert!(json_str.contains("TEST-1"));
}

#[test]
fn test_handle_command_switch_valid() {
    let mut profiles = IndexMap::new();
    profiles.insert("test".to_string(), matching_test_profile());

    let mut state = create_test_state(profiles);

    let result = handle_command(
        Command::Switch {
            profile: "test".to_string(),
            force: false,
        },
        &mut state,
    );

    assert!(result.is_ok());
    assert_eq!(state.active_profile_id, Some("test".to_string()));
}

#[test]
fn test_handle_command_switch_invalid_profile() {
    let mut profiles = IndexMap::new();
    profiles.insert("test".to_string(), matching_test_profile());

    let mut state = create_test_state(profiles);

    let result = handle_command(
        Command::Switch {
            profile: "nonexistent".to_string(),
            force: false,
        },
        &mut state,
    );

    assert!(result.is_err());
}

#[test]
fn test_handle_command_switch_no_match() {
    let mut profiles = IndexMap::new();
    profiles.insert("empty".to_string(), non_matching_test_profile());

    let mut state = create_test_state(profiles);

    let result = handle_command(
        Command::Switch {
            profile: "empty".to_string(),
            force: false,
        },
        &mut state,
    );

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("does not match current outputs")
    );
    assert_eq!(state.active_profile_id, None);
}

#[test]
fn test_handle_command_switch_force() {
    let mut profiles = IndexMap::new();
    profiles.insert("empty".to_string(), non_matching_test_profile());

    let mut state = create_test_state(profiles);

    let result = handle_command(
        Command::Switch {
            profile: "empty".to_string(),
            force: true,
        },
        &mut state,
    );

    assert!(result.is_ok());
    assert_eq!(state.active_profile_id, Some("empty".to_string()));
}

#[test]
fn test_evaluate_profiles_clears_active_profile_on_no_match() {
    let mut profiles = IndexMap::new();
    profiles.insert("test".to_string(), non_matching_test_profile());

    let mut state = create_test_state(profiles);
    state.active_profile_id = Some("test".to_string());

    state.evaluate_profiles(false);

    assert_eq!(state.active_profile_id, None);
}

#[test]
fn test_evaluate_profiles_updates_name_map_for_same_profile() {
    let mut profiles = IndexMap::new();
    profiles.insert("serial".to_string(), serial_test_profile());

    let mut state = create_test_state(profiles);
    state.outputs = vec![make_output_with_serial("DP-1", "ABC")];

    state.evaluate_profiles(false);
    assert_eq!(state.active_profile_id, Some("serial".to_string()));
    assert_eq!(state.name_map.get("ABC"), Some(&"DP-1".to_string()));

    state.outputs = vec![make_output_with_serial("HDMI-1", "ABC")];
    state.evaluate_profiles(false);

    assert_eq!(state.active_profile_id, Some("serial".to_string()));
    assert_eq!(state.name_map.get("ABC"), Some(&"HDMI-1".to_string()));
}

#[test]
fn test_apply_profile_by_name_reapplies_when_name_map_changes() {
    let mut profiles = IndexMap::new();
    profiles.insert("serial".to_string(), serial_test_profile());

    let mut state = create_test_state(profiles);
    state.outputs = vec![make_output_with_serial("DP-1", "ABC")];
    state.active_profile_id = Some("serial".to_string());
    state.name_map = HashMap::from([("ABC".to_string(), "DP-1".to_string())]);

    state.outputs = vec![make_output_with_serial("HDMI-1", "ABC")];

    let result = handle_command(
        Command::Switch {
            profile: "serial".to_string(),
            force: false,
        },
        &mut state,
    );

    assert!(result.is_ok());
    assert_eq!(state.name_map.get("ABC"), Some(&"HDMI-1".to_string()));
}
