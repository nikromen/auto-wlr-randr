use auto_wlr_randr::config::{Config, Profile};
use auto_wlr_randr::event_loop::handle_command;
use auto_wlr_randr::ipc::Command;
use auto_wlr_randr::output::OutputInfo;
use auto_wlr_randr::wayland::WaylandState;
use std::collections::HashMap;

fn create_test_state() -> WaylandState {
    let mut profiles = HashMap::new();
    profiles.insert(
        "test".to_string(),
        Profile {
            name: "test".to_string(),
            exec: vec![],
            settings: vec![],
        },
    );

    let mut config = Config::load_from_file("config.toml").unwrap_or_else(|_| {
        use assert_fs::TempDir;
        use assert_fs::prelude::*;

        let temp = TempDir::new().unwrap();
        let config_file = temp.child("config.toml");

        let config_content = r#"
[profile.test]
name = "test"
"#;
        config_file.write_str(config_content).unwrap();
        Config::load_from_file(config_file.path()).expect("Failed to load test config")
    });

    config.profiles = profiles;

    let mut state = WaylandState::new(config);

    let output = OutputInfo {
        id: 1,
        name: "TEST-1".to_string(),
        description: "Test Monitor".to_string(),
    };

    state.outputs.insert(1, output);

    state
}

#[test]
fn test_handle_command_status() {
    let mut state = create_test_state();
    state.active_profile_name = Some("test".to_string());

    let result = handle_command(Command::Status, &mut state);

    assert!(result.is_ok());
    let json_str = result.unwrap();

    assert!(json_str.contains("test"));
    assert!(json_str.contains("TEST-1"));
}

#[test]
fn test_handle_command_switch_valid() {
    let mut state = create_test_state();

    let result = handle_command(Command::Switch("test".to_string()), &mut state);

    assert!(result.is_ok());
    assert_eq!(state.active_profile_name, Some("test".to_string()));
}

#[test]
fn test_handle_command_switch_invalid() {
    let mut state = create_test_state();

    let result = handle_command(Command::Switch("nonexistent".to_string()), &mut state);

    assert!(result.is_err());
}
