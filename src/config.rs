use crate::output::OutputInfo;
use anyhow::{Context, Result};
use glob::Pattern;
use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// mirrors wlr-randr's output settings
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct OutputSetting {
    pub output: String,

    #[serde(default)]
    pub on: Option<bool>,

    #[serde(default)]
    pub mode: Option<String>,

    #[serde(default)]
    pub preferred: bool,

    #[serde(default)]
    pub pos: Option<String>,

    #[serde(default)]
    pub left_of: Option<String>,
    #[serde(default)]
    pub right_of: Option<String>,
    #[serde(default)]
    pub above: Option<String>,
    #[serde(default)]
    pub below: Option<String>,

    #[serde(default)]
    pub transform: Option<String>,

    #[serde(default)]
    pub scale: Option<f32>,

    #[serde(default)]
    pub adaptive_sync: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    #[serde(default)]
    pub exec: Vec<String>,

    #[serde(default)]
    pub settings: Vec<OutputSetting>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub on_no_match_exec: Vec<String>,

    #[serde(rename = "profile")]
    pub profiles: IndexMap<String, Profile>,

    #[serde(skip)]
    config_path: String,
}

impl Profile {
    pub fn generate_wlr_randr_args(
        &self,
        output_name_map: &HashMap<String, String>,
    ) -> Option<Vec<String>> {
        if self.settings.is_empty() {
            return None;
        }

        let mut args = Vec::new();

        for setting in &self.settings {
            let output_name = output_name_map
                .get(&setting.output)
                .unwrap_or(&setting.output);

            args.push("--output".to_string());
            args.push(output_name.clone());

            if let Some(on) = setting.on {
                if on {
                    args.push("--on".to_string());
                } else {
                    args.push("--off".to_string());
                }
            }

            if let Some(mode) = &setting.mode {
                args.push("--mode".to_string());
                args.push(mode.clone());
            }

            if setting.preferred {
                args.push("--preferred".to_string());
            }

            if let Some(pos) = &setting.pos {
                args.push("--pos".to_string());
                args.push(pos.clone());
            }

            if let Some(left_of) = &setting.left_of {
                args.push("--left-of".to_string());
                args.push(left_of.clone());
            }
            if let Some(right_of) = &setting.right_of {
                args.push("--right-of".to_string());
                args.push(right_of.clone());
            }
            if let Some(above) = &setting.above {
                args.push("--above".to_string());
                args.push(above.clone());
            }
            if let Some(below) = &setting.below {
                args.push("--below".to_string());
                args.push(below.clone());
            }

            if let Some(transform) = &setting.transform {
                args.push("--transform".to_string());
                args.push(transform.clone());
            }

            if let Some(scale) = setting.scale {
                args.push("--scale".to_string());
                args.push(scale.to_string());
            }

            if let Some(adaptive_sync) = setting.adaptive_sync {
                if adaptive_sync {
                    args.push("--adaptive-sync".to_string());
                    args.push("enabled".to_string());
                } else {
                    args.push("--adaptive-sync".to_string());
                    args.push("disabled".to_string());
                }
            }
        }

        Some(args)
    }

    pub fn match_outputs(
        &self,
        connected_outputs: &[OutputInfo],
    ) -> Option<HashMap<String, String>> {
        if self.settings.len() != connected_outputs.len() {
            return None;
        }

        if self.settings.is_empty() {
            return if connected_outputs.is_empty() {
                Some(HashMap::new())
            } else {
                None
            };
        }

        let mut used_outputs = vec![false; connected_outputs.len()];
        let mut output_name_map = HashMap::with_capacity(self.settings.len());

        for setting in &self.settings {
            let pattern = match Pattern::new(&setting.output) {
                Ok(p) => p,
                Err(e) => {
                    log::error!("Invalid output pattern '{}': {e}", setting.output);
                    return None;
                }
            };

            let found = connected_outputs
                .iter()
                .enumerate()
                .find(|(i, out)| !used_outputs[*i] && out.matches_pattern(&pattern));

            if let Some((idx, matched)) = found {
                used_outputs[idx] = true;
                output_name_map.insert(setting.output.clone(), matched.name.clone());
            } else {
                return None;
            }
        }

        Some(output_name_map)
    }

    /// Resolve output patterns to connector names for manual profile application.
    /// Unlike `match_outputs`, this does not require an exact output count match.
    /// serial:ABCD1234 -> HDMI-A-1
    pub fn resolve_name_map(&self, connected_outputs: &[OutputInfo]) -> HashMap<String, String> {
        let mut used_outputs = vec![false; connected_outputs.len()];
        let mut output_name_map = HashMap::new();

        for setting in &self.settings {
            let pattern = match Pattern::new(&setting.output) {
                Ok(p) => p,
                Err(e) => {
                    log::warn!("Invalid output pattern '{}': {e}", setting.output);
                    continue;
                }
            };

            if let Some((idx, matched)) = connected_outputs
                .iter()
                .enumerate()
                .find(|(i, out)| !used_outputs[*i] && out.matches_pattern(&pattern))
            {
                used_outputs[idx] = true;
                output_name_map.insert(setting.output.clone(), matched.name.clone());
            }
        }

        output_name_map
    }
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(anyhow::anyhow!("Config file not found at {path:?}"));
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file at {path:?}"))?;

        let mut config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file at {path:?}"))?;

        config.config_path = path.to_string_lossy().to_string();
        Ok(config)
    }

    pub fn reload_config(&mut self) -> Result<()> {
        *self = Self::load_from_file(&self.config_path)?;
        Ok(())
    }

    pub fn find_matching_profile(
        &self,
        connected_outputs: &[OutputInfo],
    ) -> Option<(&str, &Profile, HashMap<String, String>)> {
        for (profile_id, profile) in &self.profiles {
            if let Some(name_map) = profile.match_outputs(connected_outputs) {
                log::debug!("Profile '{profile_id}' matches current outputs");
                return Some((profile_id, profile, name_map));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_wlr_randr_args_with_settings() {
        let profile = Profile {
            exec: vec!["echo 'done'".into()],
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

        assert_eq!(
            args,
            vec![
                "--output".to_string(),
                "HDMI-A-1".to_string(),
                "--on".to_string(),
                "--mode".to_string(),
                "1920x1080".to_string(),
                "--pos".to_string(),
                "0,0".to_string(),
                "--scale".to_string(),
                "1".to_string(),
                "--adaptive-sync".to_string(),
                "enabled".to_string(),
            ]
        );
    }

    #[test]
    fn test_generate_wlr_randr_args_omits_unset_on_and_adaptive_sync() {
        let profile = Profile {
            exec: vec![],
            settings: vec![OutputSetting {
                output: "HDMI-1".into(),
                on: None,
                mode: Some("1920x1080".into()),
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
        };

        let args = profile.generate_wlr_randr_args(&HashMap::new()).unwrap();

        assert!(!args.contains(&"--on".to_string()));
        assert!(!args.contains(&"--off".to_string()));
        assert!(!args.iter().any(|arg| arg == "--adaptive-sync"));
    }

    #[test]
    fn test_generate_wlr_randr_args_empty_settings() {
        let profile = Profile {
            exec: vec!["echo 'test'".into()],
            settings: vec![],
        };

        assert!(profile.generate_wlr_randr_args(&HashMap::new()).is_none());
    }
}
