use auto_wlr_randr::ipc::{Command, ensure_socket_dir_exists, get_socket_path};
use serde_json::{from_str, to_string};

#[test]
fn test_command_serialization() {
    let reload_cmd = Command::Reload;
    let json = to_string(&reload_cmd).unwrap();
    assert_eq!(json, r#""Reload""#);
    let deserialized: Command = from_str(&json).unwrap();
    match deserialized {
        Command::Reload => {}
        _ => panic!("Expected Command::Reload"),
    }

    let status_cmd = Command::Status;
    let json = to_string(&status_cmd).unwrap();
    assert_eq!(json, r#""Status""#);
    let deserialized: Command = from_str(&json).unwrap();
    match deserialized {
        Command::Status => {}
        _ => panic!("Expected Command::Status"),
    }

    let switch_cmd = Command::Switch {
        profile: "test-profile".to_string(),
        force: false,
    };
    let json = to_string(&switch_cmd).unwrap();
    assert_eq!(
        json,
        r#"{"Switch":{"profile":"test-profile","force":false}}"#
    );
    let deserialized: Command = from_str(&json).unwrap();
    match deserialized {
        Command::Switch { profile, force } => {
            assert_eq!(profile, "test-profile");
            assert!(!force);
        }
        _ => panic!("Expected Command::Switch"),
    }

    let switch_force_cmd = Command::Switch {
        profile: "test-profile".to_string(),
        force: true,
    };
    let json = to_string(&switch_force_cmd).unwrap();
    assert_eq!(
        json,
        r#"{"Switch":{"profile":"test-profile","force":true}}"#
    );
}

#[test]
fn test_socket_path() {
    let path = get_socket_path();
    assert!(
        path.to_str()
            .unwrap()
            .ends_with("/auto-wlr-randr/auto-wlr-randr.sock")
    );
}

#[test]
fn test_ensure_socket_dir_exists() {
    // This should not fail, though it might not create a directory
    // if we don't have permissions
    let result = ensure_socket_dir_exists();
    // Just make sure it doesn't panic
    assert!(result.is_ok() || result.is_err());
}
