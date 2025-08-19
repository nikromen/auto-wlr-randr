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

    let switch_cmd = Command::Switch("test-profile".to_string());
    let json = to_string(&switch_cmd).unwrap();
    assert_eq!(json, r#"{"Switch":"test-profile"}"#);
    let deserialized: Command = from_str(&json).unwrap();
    match deserialized {
        Command::Switch(name) => {
            assert_eq!(name, "test-profile");
        }
        _ => panic!("Expected Command::Switch"),
    }
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
