use assert_fs::TempDir;
use assert_fs::prelude::*;
use auto_wlr_randr::config::{Config, OutputSetting, Profile, ValidationLevel, ValidationReport};
use auto_wlr_randr::output::OutputInfo;
use indexmap::IndexMap;
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

#[test]
fn test_config_load_on_no_match_exec() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.child("config.toml");

    config_file
        .write_str(
            r#"
on_no_match_exec = ["notify-send 'No matching profile'"]

[profile.laptop]
[[profile.laptop.settings]]
output = "eDP-1"
"#,
        )
        .unwrap();

    let config = Config::load_from_file(config_file.path()).unwrap();

    assert_eq!(config.on_no_match_exec.len(), 1);
    assert_eq!(
        config.on_no_match_exec[0],
        "notify-send 'No matching profile'"
    );
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
    let mut profiles = IndexMap::new();

    let laptop_profile = Profile {
        exec: vec![],
        settings: vec![OutputSetting {
            output: "eDP-1".into(),
            on: Some(true),
            mode: Some("1920x1080".into()),
            preferred: false,
            pos: Some("0,0".into()),
            left_of: None,
            right_of: None,
            above: None,
            below: None,
            transform: None,
            scale: Some(1.0),
            adaptive_sync: None,
        }],
    };
    profiles.insert("laptop".to_string(), laptop_profile);

    let docked_settings = vec![
        OutputSetting {
            output: "eDP-1".into(),
            on: Some(false),
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
        },
        OutputSetting {
            output: "HDMI-*".into(),
            on: Some(true),
            mode: Some("2560x1440".into()),
            preferred: false,
            pos: Some("0,0".into()),
            left_of: None,
            right_of: None,
            above: None,
            below: None,
            transform: None,
            scale: Some(1.0),
            adaptive_sync: None,
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
fn test_profile_generate_wlr_randr_args() {
    let profile = Profile {
        exec: vec!["echo 'Profile activated'".into()],
        settings: vec![OutputSetting {
            output: "HDMI-1".into(),
            on: Some(true),
            mode: Some("1920x1080".into()),
            preferred: false,
            pos: Some("0,0".into()),
            left_of: None,
            right_of: None,
            above: None,
            below: None,
            transform: None,
            scale: Some(1.0),
            adaptive_sync: Some(true),
        }],
    };

    let mut name_map = HashMap::new();
    name_map.insert("HDMI-1".to_string(), "HDMI-A-1".to_string());

    let args = profile.generate_wlr_randr_args(&name_map).unwrap();

    assert!(args.contains(&"--output".to_string()));
    assert!(args.contains(&"HDMI-A-1".to_string()));
    assert!(args.contains(&"--on".to_string()));
    assert!(args.contains(&"--mode".to_string()));
    assert!(args.contains(&"1920x1080".to_string()));
    assert!(args.contains(&"--pos".to_string()));
    assert!(args.contains(&"0,0".to_string()));
    assert!(args.contains(&"--scale".to_string()));
    assert!(args.contains(&"1".to_string()));
    assert!(args.contains(&"--adaptive-sync".to_string()));
    assert!(args.contains(&"enabled".to_string()));
}

#[test]
fn test_profile_order_from_config_file() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.child("config.toml");

    config_file
        .write_str(
            r#"
[profile.first]
[[profile.first.settings]]
output = "eDP-1"

[profile.second]
[[profile.second.settings]]
output = "eDP-1"
"#,
        )
        .unwrap();

    let config = Config::load_from_file(config_file.path()).unwrap();
    let outputs = vec![make_output("eDP-1", Some("Laptop"), Some("Screen"), None)];

    let (profile_id, _, _) = config.find_matching_profile(&outputs).unwrap();
    assert_eq!(profile_id, "first");

    config_file
        .write_str(
            r#"
[profile.second]
[[profile.second.settings]]
output = "eDP-1"

[profile.first]
[[profile.first.settings]]
output = "eDP-1"
"#,
        )
        .unwrap();

    let config = Config::load_from_file(config_file.path()).unwrap();
    let (profile_id, _, _) = config.find_matching_profile(&outputs).unwrap();
    assert_eq!(profile_id, "second");
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

#[test]
fn test_validate_reports_invalid_pattern_and_transform() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.child("config.toml");

    config_file
        .write_str(
            r#"
[profile.broken]
[[profile.broken.settings]]
output = "[invalid"
transform = "45"
"#,
        )
        .unwrap();

    let report = Config::load_from_file(config_file.path())
        .unwrap()
        .validate();

    assert!(report.has_errors());
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.level == ValidationLevel::Error
                && issue.message.contains("invalid output pattern"))
    );
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.level == ValidationLevel::Error
                && issue.message.contains("invalid transform"))
    );
}

#[test]
fn test_validate_warns_on_duplicate_and_identical_profiles() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.child("config.toml");

    config_file
        .write_str(
            r#"
[profile.a]
[[profile.a.settings]]
output = "eDP-1"
[[profile.a.settings]]
output = "eDP-1"

[profile.b]
[[profile.b.settings]]
output = "eDP-1"

[profile.c]
[[profile.c.settings]]
output = "eDP-1"
"#,
        )
        .unwrap();

    let report = Config::load_from_file(config_file.path())
        .unwrap()
        .validate();

    assert!(!report.has_errors());
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.level == ValidationLevel::Warning
                && issue.message.contains("duplicate output pattern"))
    );
    assert!(report.issues.iter().any(|issue| {
        issue.level == ValidationLevel::Warning
            && issue.message.contains("identical output patterns")
            && issue.message.contains("'b'")
            && issue.message.contains("'c'")
    }));
}

#[test]
fn test_validate_valid_config_has_no_errors() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.child("config.toml");

    config_file
        .write_str(
            r#"
[profile.laptop]
[[profile.laptop.settings]]
output = "eDP-1"
transform = "normal"
"#,
        )
        .unwrap();

    let report = Config::load_from_file(config_file.path())
        .unwrap()
        .validate();

    assert_eq!(report, ValidationReport::default());
}

#[test]
fn test_dry_run_returns_matching_profile() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.child("config.toml");

    config_file
        .write_str(
            r#"
[profile.laptop]
exec = ["echo laptop"]

[[profile.laptop.settings]]
output = "eDP-1"
on = true
mode = "1920x1080"
"#,
        )
        .unwrap();

    let config = Config::load_from_file(config_file.path()).unwrap();
    let outputs = vec![make_output("eDP-1", Some("Laptop"), Some("Screen"), None)];
    let result = config.dry_run(&outputs);

    assert_eq!(result.matched_profile.as_deref(), Some("laptop"));
    assert_eq!(result.exec, vec!["echo laptop".to_string()]);
    assert_eq!(
        result.output_name_map.get("eDP-1"),
        Some(&"eDP-1".to_string())
    );
    assert!(result.wlr_randr_args.is_some());
    assert!(result.on_no_match_exec.is_empty());
}

#[test]
fn test_dry_run_without_match_includes_on_no_match_exec() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.child("config.toml");

    config_file
        .write_str(
            r#"
on_no_match_exec = ["notify-send no match"]

[profile.dual]
[[profile.dual.settings]]
output = "eDP-1"
[[profile.dual.settings]]
output = "HDMI-*"
"#,
        )
        .unwrap();

    let config = Config::load_from_file(config_file.path()).unwrap();
    let outputs = vec![make_output("eDP-1", Some("Laptop"), Some("Screen"), None)];
    let result = config.dry_run(&outputs);

    assert!(result.matched_profile.is_none());
    assert!(result.wlr_randr_args.is_none());
    assert_eq!(
        result.on_no_match_exec,
        vec!["notify-send no match".to_string()]
    );
}

#[test]
fn test_ensure_valid_fails_on_validation_errors() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.child("config.toml");

    config_file
        .write_str(
            r#"
[profile.broken]
[[profile.broken.settings]]
output = "[invalid"
"#,
        )
        .unwrap();

    let config = Config::load_from_file(config_file.path()).unwrap();
    assert!(config.ensure_valid().is_err());
}

#[test]
fn test_ensure_valid_succeeds_with_warnings_only() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.child("config.toml");

    config_file
        .write_str(
            r#"
[profile.empty]
"#,
        )
        .unwrap();

    let config = Config::load_from_file(config_file.path()).unwrap();
    assert!(config.ensure_valid().is_ok());
}

#[test]
fn test_reload_config_keeps_old_config_on_validation_failure() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.child("config.toml");

    config_file
        .write_str(
            r#"
[profile.laptop]
[[profile.laptop.settings]]
output = "eDP-1"
"#,
        )
        .unwrap();

    let mut config = Config::load_from_file(config_file.path()).unwrap();
    assert_eq!(config.profiles.len(), 1);

    config_file
        .write_str(
            r#"
[profile.broken]
[[profile.broken.settings]]
output = "[invalid"
"#,
        )
        .unwrap();

    assert!(config.reload_config().is_err());
    assert_eq!(config.profiles.len(), 1);
    assert!(config.profiles.contains_key("laptop"));
}
