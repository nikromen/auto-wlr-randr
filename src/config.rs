use anyhow::{Context, Result};
use glob::Pattern;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct OutputData {
    pub output_name: String,
    pub manufacturer: String,
}

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
    pub fn generate_commands(
        &self,
        manufacturer_output_mapping: &HashMap<String, String>,
    ) -> Vec<String> {
        let mut wlr_randr_command = Vec::new();
        wlr_randr_command.push("wlr-randr".to_string());

        for setting in &self.settings {
            let output_name = manufacturer_output_mapping
                .get(&setting.output)
                .unwrap_or(&setting.output);

            wlr_randr_command.push(format!("--output '{output_name}'"));

            if setting.on {
                wlr_randr_command.push(" --on".to_string());
            } else {
                wlr_randr_command.push(" --off".to_string());
            }

            if let Some(mode) = &setting.mode {
                wlr_randr_command.push(format!(" --mode '{mode}'"));
            }

            if setting.preferred {
                wlr_randr_command.push(" --preferred".to_string());
            }

            if let Some(pos) = &setting.pos {
                wlr_randr_command.push(format!(" --pos '{pos}'"));
            }

            if let Some(left_of) = &setting.left_of {
                wlr_randr_command.push(format!(" --left-of '{left_of}'"));
            }
            if let Some(right_of) = &setting.right_of {
                wlr_randr_command.push(format!(" --right-of '{right_of}'"));
            }
            if let Some(above) = &setting.above {
                wlr_randr_command.push(format!(" --above '{above}'"));
            }
            if let Some(below) = &setting.below {
                wlr_randr_command.push(format!(" --below '{below}'"));
            }

            if let Some(transform) = &setting.transform {
                wlr_randr_command.push(format!(" --transform '{transform}'"));
            }

            if let Some(scale) = setting.scale {
                wlr_randr_command.push(format!(" --scale '{scale}'"));
            }

            if setting.adaptive_sync {
                wlr_randr_command.push(" --adaptive-sync enabled".to_string());
            } else {
                wlr_randr_command.push(" --adaptive-sync disabled".to_string());
            }
        }

        let mut commands = Vec::with_capacity(self.exec.len() + 1);
        if !self.exec.is_empty() {
            commands.push(wlr_randr_command.join(" "));
        }
        commands.extend(self.exec.clone());
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
        let new_config = Config::load_from_file(&self.config_path)?;
        *self = new_config;
        Ok(())
    }

    pub fn find_matching_profile<'a>(
        &'a self,
        connected_outputs: &[OutputData],
    ) -> Option<(String, &'a Profile, HashMap<String, String>)> {
        // TODO: rozdelit, je to slozite
        // TODO: vazne musi vylezt mapy?
        'profile_loop: for (profile_id, profile) in &self.profiles {
            if profile.settings.len() != connected_outputs.len() {
                continue;
            }
            if profile.settings.is_empty() {
                return if connected_outputs.is_empty() {
                    Some((profile_id.clone(), profile, HashMap::new()))
                } else {
                    None
                };
            }

            let mut used_outputs = vec![false; connected_outputs.len()];
            let mut output_name_map = HashMap::with_capacity(profile.settings.len());

            for required_output_setting in &profile.settings {
                let pattern = match Pattern::new(&required_output_setting.output) {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!(
                            "Invalid output pattern '{}': {e}",
                            required_output_setting.output
                        );
                        continue 'profile_loop;
                    }
                };

                let found_match = connected_outputs.iter().enumerate().find(|(i, conn_out)| {
                    !used_outputs[*i]
                        && (pattern.matches(&conn_out.output_name)
                            || pattern.matches(&conn_out.manufacturer))
                });

                if let Some((idx, matched_output)) = found_match {
                    used_outputs[idx] = true;
                    output_name_map.insert(
                        required_output_setting.output.clone(),
                        matched_output.output_name.clone(),
                    );
                } else {
                    continue 'profile_loop;
                }
            }

            return Some((profile_id.clone(), profile, output_name_map));
        }

        None
    }
}
