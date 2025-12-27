use crate::output::OutputInfo;
use anyhow::{Context, Result};
use glob::Pattern;
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
    pub on: bool,

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
    pub adaptive_sync: bool,
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
    #[serde(rename = "profile")]
    pub profiles: HashMap<String, Profile>,

    #[serde(skip)]
    config_path: String,
}

impl Profile {
    pub fn generate_commands(&self, output_name_map: &HashMap<String, String>) -> Vec<String> {
        let mut commands = Vec::with_capacity(self.exec.len() + 1);

        if self.settings.is_empty() {
            commands.extend(self.exec.iter().cloned());
            return commands;
        }

        let mut args = vec!["wlr-randr".to_string()];

        for setting in &self.settings {
            let output_name = output_name_map
                .get(&setting.output)
                .unwrap_or(&setting.output);

            args.push(format!("--output '{output_name}'"));

            if setting.on {
                args.push("--on".to_string());
            } else {
                args.push("--off".to_string());
            }

            if let Some(mode) = &setting.mode {
                args.push(format!("--mode '{mode}'"));
            }

            if setting.preferred {
                args.push("--preferred".to_string());
            }

            if let Some(pos) = &setting.pos {
                args.push(format!("--pos '{pos}'"));
            }

            if let Some(left_of) = &setting.left_of {
                args.push(format!("--left-of '{left_of}'"));
            }
            if let Some(right_of) = &setting.right_of {
                args.push(format!("--right-of '{right_of}'"));
            }
            if let Some(above) = &setting.above {
                args.push(format!("--above '{above}'"));
            }
            if let Some(below) = &setting.below {
                args.push(format!("--below '{below}'"));
            }

            if let Some(transform) = &setting.transform {
                args.push(format!("--transform '{transform}'"));
            }

            if let Some(scale) = setting.scale {
                args.push(format!("--scale '{scale}'"));
            }

            if setting.adaptive_sync {
                args.push("--adaptive-sync enabled".to_string());
            } else {
                args.push("--adaptive-sync disabled".to_string());
            }
        }

        commands.push(args.join(" "));
        commands.extend(self.exec.iter().cloned());
        commands
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
        'profile_loop: for (profile_id, profile) in &self.profiles {
            if profile.settings.len() != connected_outputs.len() {
                continue;
            }

            if profile.settings.is_empty() {
                return if connected_outputs.is_empty() {
                    Some((profile_id, profile, HashMap::new()))
                } else {
                    None
                };
            }

            let mut used_outputs = vec![false; connected_outputs.len()];
            let mut output_name_map = HashMap::with_capacity(profile.settings.len());

            for setting in &profile.settings {
                let pattern = match Pattern::new(&setting.output) {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!("Invalid output pattern '{}': {e}", setting.output);
                        continue 'profile_loop;
                    }
                };

                let found = connected_outputs
                    .iter()
                    .enumerate()
                    .find(|(i, out)| !used_outputs[*i] && out.matches_pattern(&pattern));

                log::debug!(
                    "Pattern '{}' against outputs: {:?} => {:?}",
                    setting.output,
                    connected_outputs
                        .iter()
                        .map(|o| o.to_string())
                        .collect::<Vec<_>>(),
                    found.map(|(i, _)| i)
                );

                if let Some((idx, matched)) = found {
                    used_outputs[idx] = true;
                    output_name_map.insert(setting.output.clone(), matched.name.clone());
                } else {
                    continue 'profile_loop;
                }
            }

            return Some((profile_id, profile, output_name_map));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_commands_with_settings() {
        let profile = Profile {
            exec: vec!["echo 'done'".into()],
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
        assert!(commands[0].contains("--scale '1'"));
        assert!(commands[0].contains("--adaptive-sync enabled"));
        assert_eq!(commands[1], "echo 'done'");
    }

    #[test]
    fn test_generate_commands_empty_settings() {
        let profile = Profile {
            exec: vec!["echo 'test'".into()],
            settings: vec![],
        };

        let commands = profile.generate_commands(&HashMap::new());
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "echo 'test'");
    }
}
