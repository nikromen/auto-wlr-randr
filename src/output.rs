use anyhow::{Context, Result};
use glob::Pattern;
use serde::Deserialize;
use std::fmt;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
pub struct OutputInfo {
    pub name: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
}

impl OutputInfo {
    pub fn build_identifier(&self) -> Option<String> {
        match (&self.make, &self.model, &self.serial) {
            (Some(make), Some(model), Some(serial)) if !serial.is_empty() => {
                Some(format!("{make} {model} {serial}"))
            }
            (Some(make), Some(model), _) => Some(format!("{make} {model}")),
            _ => None,
        }
    }

    pub fn matches_pattern(&self, pattern: &Pattern) -> bool {
        pattern.matches(&self.name)
            || self
                .build_identifier()
                .as_deref()
                .is_some_and(|id| pattern.matches(id))
            || self
                .serial
                .as_deref()
                .is_some_and(|serial| pattern.matches(serial))
    }
}

impl fmt::Display for OutputInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let identifier = self
            .build_identifier()
            .unwrap_or_else(|| "unknown".to_string());
        write!(f, "{} ({})", self.name, identifier)
    }
}

pub fn get_outputs() -> Result<Vec<OutputInfo>> {
    let output = Command::new("wlr-randr")
        .arg("--json")
        .output()
        .context("Failed to run wlr-randr")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("wlr-randr failed: {stderr}");
    }

    let outputs: Vec<OutputInfo> =
        serde_json::from_slice(&output.stdout).context("Failed to parse wlr-randr output")?;

    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_build_identifier_with_serial() {
        let output = make_output(
            "HDMI-1",
            Some("Dell Inc."),
            Some("U2718Q"),
            Some("ABC123456"),
        );
        assert_eq!(
            output.build_identifier(),
            Some("Dell Inc. U2718Q ABC123456".to_string())
        );
    }

    #[test]
    fn test_build_identifier_without_serial() {
        let output = make_output("HDMI-1", Some("Dell Inc."), Some("U2718Q"), None);
        assert_eq!(
            output.build_identifier(),
            Some("Dell Inc. U2718Q".to_string())
        );
    }

    #[test]
    fn test_build_identifier_missing_make_model() {
        let output = make_output("HDMI-1", None, None, None);
        assert_eq!(output.build_identifier(), None);
    }

    #[test]
    fn test_display() {
        let output = make_output("HDMI-1", Some("Dell Inc."), Some("U2718Q"), None);
        assert_eq!(format!("{output}"), "HDMI-1 (Dell Inc. U2718Q)");
    }

    #[test]
    fn test_matches_pattern_name() {
        let output = make_output(
            "HDMI-1",
            Some("Dell Inc."),
            Some("U2718Q"),
            Some("ABC123456"),
        );
        assert!(output.matches_pattern(&Pattern::new("HDMI-1").unwrap()));
        assert!(output.matches_pattern(&Pattern::new("HDMI-*").unwrap()));
    }

    #[test]
    fn test_matches_pattern_identifier() {
        let output = make_output(
            "HDMI-1",
            Some("Dell Inc."),
            Some("U2718Q"),
            Some("ABC123456"),
        );
        assert!(output.matches_pattern(&Pattern::new("Dell Inc. U2718Q ABC123456").unwrap()));
        assert!(output.matches_pattern(&Pattern::new("Dell Inc. * ABC123456").unwrap()));
    }

    #[test]
    fn test_matches_pattern_fields() {
        let output = make_output(
            "HDMI-1",
            Some("Dell Inc."),
            Some("U2718Q"),
            Some("ABC123456"),
        );
        assert!(output.matches_pattern(&Pattern::new("Dell Inc.*").unwrap()));
        assert!(output.matches_pattern(&Pattern::new("*U2718Q*").unwrap()));
        assert!(output.matches_pattern(&Pattern::new("ABC123456").unwrap()));
    }

    #[test]
    fn test_matches_pattern_no_match() {
        let output = make_output(
            "HDMI-1",
            Some("Dell Inc."),
            Some("U2718Q"),
            Some("ABC123456"),
        );
        assert!(!output.matches_pattern(&Pattern::new("NonExistent").unwrap()));
    }
}
