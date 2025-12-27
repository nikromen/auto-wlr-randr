use assert_fs::TempDir;
use assert_fs::prelude::*;
use auto_wlr_randr::config::{Config, OutputSetting, Profile};
use auto_wlr_randr::output::OutputInfo;
use rstest::*;
use std::collections::HashMap;

#[test]
fn test_config_load_from_file() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.child("config.toml");

    config_file
        .write_str(
            r#"
[profile.laptop]
exec = ["notify-send 'Laptop profile activated'"]

[[profile.laptop.settings]]
output = "eDP-1"
on = true
mode = "1920x1080@60Hz"
pos = "0,0"
scale = 1.0

[profile.docked]
exec = ["notify-send 'Docked profile activated'"]

[[profile.docked.settings]]
output = "eDP-1"
on = false

[[profile.docked.settings]]
output = "HDMI-*"
on = true
mode = "2560x1440@144Hz"
pos = "0,0"
scale = 1.0
"#,
        )
        .unwrap();

    let config = Config::load_from_file(config_file.path()).unwrap();

    assert_eq!(config.profiles.len(), 2);
    assert!(config.profiles.contains_key("laptop"));
    assert!(config.profiles.contains_key("docked"));

    let laptop_profile = &config.profiles["laptop"];
    assert_eq!(laptop_profile.exec.len(), 1);
    assert_eq!(laptop_profile.settings.len(), 1);

    let docked_profile = &config.profiles["docked"];
    assert_eq!(docked_profile.exec.len(), 1);
    assert_eq!(docked_profile.settings.len(), 2);
}

#[test]
fn test_config_load_nonexistent_file() {
    let result = Config::load_from_file("/path/to/nonexistent/config.toml");
    assert!(result.is_err());
}

fn make_output(
    name: &str,
    make: Option<&str>,
    model: Option<&str>,
    serial: Option<&str>,
) -> OutputInfo {
    OutputInfo {
        name: name.to_string(),
        make: make.map(String::from),
        model: model.map(String::from),
        serial: serial.map(String::from),
    }
}

#[rstest]
#[case(
    vec![make_output("eDP-1", Some("Laptop"), Some("Screen"), None)],
    "laptop",
    true
)]
#[case(
    vec![
        make_output("HDMI-1", Some("Dell"), Some("Monitor"), None),
        make_output("eDP-1", Some("Laptop"), Some("Screen"), None)
    ],
    "docked",
    true
)]
#[case(
    vec![make_output("DP-1", Some("Unknown"), Some("Monitor"), None)],
    "",
    false
)]
fn test_find_matching_profile(
    #[case] connected_outputs: Vec<OutputInfo>,
    #[case] expected_profile_name: &str,
    #[case] should_match: bool,
) {
    let mut profiles = HashMap::new();

    let laptop_profile = Profile {
        exec: vec![],
        settings: vec![OutputSetting {
            output: "eDP-1".into(),
            on: true,
            mode: Some("1920x1080".into()),
            preferred: false,
            pos: Some("0,0".into()),
            left_of: None,
            right_of: None,
            above: None,
            below: None,
            transform: None,
            scale: Some(1.0),
            adaptive_sync: false,
        }],
    };
    profiles.insert("laptop".to_string(), laptop_profile);

    let docked_settings = vec![
        OutputSetting {
            output: "eDP-1".into(),
            on: false,
            mode: None,
            preferred: false,
            pos: None,
            left_of: None,
            right_of: None,
            above: None,
            below: None,
            transform: None,
            scale: None,
            adaptive_sync: false,
        },
        OutputSetting {
            output: "HDMI-*".into(),
            on: true,
            mode: Some("2560x1440".into()),
            preferred: false,
            pos: Some("0,0".into()),
            left_of: None,
            right_of: None,
            above: None,
            below: None,
            transform: None,
            scale: Some(1.0),
            adaptive_sync: false,
        },
    ];

    let docked_profile = Profile {
        exec: vec![],
        settings: docked_settings,
    };
    profiles.insert("docked".to_string(), docked_profile);

    let mut config = Config::load_from_file("config.toml").unwrap_or_else(|_| {
        let temp = TempDir::new().unwrap();
        let config_file = temp.child("config.toml");
        config_file.write_str("[profile.dummy]\n").unwrap();
        Config::load_from_file(config_file.path()).expect("Failed to load test config")
    });

    config.profiles = profiles;

    let result = config.find_matching_profile(&connected_outputs);

    if should_match {
        assert!(
            result.is_some(),
            "Expected to find a matching profile but found none"
        );
        let (profile_id, _, _) = result.unwrap();
        assert_eq!(profile_id, expected_profile_name);
    } else {
        assert!(
            result.is_none(),
            "Expected to not find a matching profile but found one"
        );
    }
}

#[test]
fn test_profile_generate_commands() {
    let profile = Profile {
        exec: vec!["echo 'Profile activated'".into()],
        settings: vec![OutputSetting {
            output: "HDMI-1".into(),
            on: true,
            mode: Some("1920x1080".into()),
            preferred: false,
            pos: Some("0,0".into()),
            left_of: None,
            right_of: None,
            above: None,
            below: None,
            transform: None,
            scale: Some(1.0),
            adaptive_sync: true,
        }],
    };

    let mut name_map = HashMap::new();
    name_map.insert("HDMI-1".to_string(), "HDMI-A-1".to_string());

    let commands = profile.generate_commands(&name_map);

    assert_eq!(commands.len(), 2);
    assert!(commands[0].starts_with("wlr-randr"));
    assert!(commands[0].contains("--output 'HDMI-A-1'"));
    assert!(commands[0].contains("--on"));
    assert!(commands[0].contains("--mode '1920x1080'"));
    assert!(commands[0].contains("--pos '0,0'"));
    assert!(commands[0].contains("--scale '1'"));
    assert!(commands[0].contains("--adaptive-sync enabled"));
    assert_eq!(commands[1], "echo 'Profile activated'");
}

#[test]
fn test_reload_config() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.child("config.toml");

    config_file.write_str("[profile.laptop]\n").unwrap();

    let mut config = Config::load_from_file(config_file.path()).unwrap();
    assert_eq!(config.profiles.len(), 1);

    config_file
        .write_str("[profile.laptop]\n\n[profile.docked]\n")
        .unwrap();

    config.reload_config().unwrap();

    assert_eq!(config.profiles.len(), 2);
    assert!(config.profiles.contains_key("docked"));
}
