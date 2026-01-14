use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub devops: DevOpsConfig,
    #[serde(default)]
    pub graph: GraphConfig,
    #[serde(default)]
    pub work_hours: WorkHoursConfig,
    #[serde(default)]
    pub focus_blocks: FocusBlocksConfig,
    #[serde(default)]
    pub state: StateConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DevOpsConfig {
    pub pat: Option<String>, // Can be optional if loading from env/keyring
    pub organization: String,
    pub project: String,
    /// States to skip during markdown import (case-insensitive)
    #[serde(default = "default_skip_states")]
    pub skip_states: Vec<String>,
    /// Optional API URL override for testing (e.g. mocking)
    pub api_url: Option<String>,
    /// Optional 7Pace API URL override for testing
    pub pace_api_url: Option<String>,
}

fn default_skip_states() -> Vec<String> {
    vec![
        "Completed".to_string(),
        "Resolved".to_string(),
        "Closed".to_string(),
        "Removed".to_string(),
    ]
}

impl Default for DevOpsConfig {
    fn default() -> Self {
        Self {
            pat: None,
            organization: String::new(),
            project: String::new(),
            skip_states: default_skip_states(),
            api_url: None,
            pace_api_url: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GraphConfig {
    pub client_id: String,
    #[serde(default = "default_tenant_id")]
    pub tenant_id: String,
}

fn default_tenant_id() -> String {
    "common".to_string()
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            tenant_id: "common".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct WorkHoursConfig {
    pub start: String,
    pub end: String,
    pub timezone: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FocusBlocksConfig {
    pub duration_minutes: u32,
    pub interval_minutes: u32,
    pub teams_presence_sync: bool,
}

impl Default for FocusBlocksConfig {
    fn default() -> Self {
        Self {
            duration_minutes: 45,
            interval_minutes: 15,
            teams_presence_sync: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StateConfig {
    pub task_expiry_hours: u32,
    /// Optional override for state directory (for testing)
    pub state_dir_override: Option<PathBuf>,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            task_expiry_hours: 24,
            state_dir_override: None,
        }
    }
}

impl WorkHoursConfig {
    /// Validate work hours configuration
    pub fn validate(&self) -> Result<()> {
        if self.start.is_empty() || self.end.is_empty() {
            return Ok(()); // Empty is fine (no validation)
        }

        // Validate format HH:MM
        let parse_time = |s: &str| -> Result<(u32, u32)> {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() != 2 {
                anyhow::bail!("Invalid time format '{}', expected HH:MM", s);
            }
            let hours: u32 = parts[0].parse().context("Invalid hour")?;
            let minutes: u32 = parts[1].parse().context("Invalid minute")?;
            if hours >= 24 || minutes >= 60 {
                anyhow::bail!("Invalid time '{}'", s);
            }
            Ok((hours, minutes))
        };

        let (start_h, start_m) = parse_time(&self.start)?;
        let (end_h, end_m) = parse_time(&self.end)?;

        let start_mins = start_h * 60 + start_m;
        let end_mins = end_h * 60 + end_m;

        if start_mins >= end_mins {
            anyhow::bail!("Work hours start time must be before end time");
        }

        Ok(())
    }
}

impl FocusBlocksConfig {
    /// Validate focus blocks configuration
    pub fn validate(&self) -> Result<()> {
        if self.duration_minutes == 0 {
            anyhow::bail!("Focus block duration must be greater than 0");
        }

        // Warn if interval is not a common value
        let common_intervals = [15, 25, 30, 50, 60];
        if !common_intervals.contains(&self.interval_minutes) {
            eprintln!(
                "Warning: Interval {} is unusual. Common values: {:?}",
                self.interval_minutes, common_intervals
            );
        }

        Ok(())
    }
}

impl Config {
    /// Get DevOps PAT from keyring or config (with migration)
    pub fn get_devops_pat(&self) -> Result<String> {
        // Try keyring first
        if let Ok(pat) = crate::keyring::get_devops_pat() {
            return Ok(pat);
        }

        // Fall back to config file (legacy)
        if let Some(pat) = &self.devops.pat {
            return Ok(pat.clone());
        }

        anyhow::bail!("DevOps PAT not found. Run 'ano7 config set devops.pat <PAT>' to configure")
    }

    /// Validate all configuration
    pub fn validate(&self) -> Result<()> {
        self.work_hours.validate()?;
        self.focus_blocks.validate()?;
        Ok(())
    }

    /// Migrate plain-text PAT to keyring
    pub fn migrate_credentials(&mut self) -> Result<bool> {
        let mut migrated = false;

        if let Some(pat) = &self.devops.pat {
            // Store in keyring
            crate::keyring::store_devops_pat(pat).context("Failed to store PAT in keyring")?;

            // Clear from config
            self.devops.pat = None;
            migrated = true;
        }

        Ok(migrated)
    }
}

pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Config> {
    let loader = ConfigBuilder::builder()
        .add_source(File::from(path.as_ref()).format(FileFormat::Toml))
        .build()
        .context("Failed to build config loader")?;

    loader
        .try_deserialize()
        .context("Failed to parse config file")
}

pub fn load() -> Result<Config> {
    let config_dir = home::home_dir()
        .context("Could not find home directory")?
        .join(".ao-no-out7ook");
    let config_path = config_dir.join("config.toml");

    let mut config = load_from_path(&config_path)?;

    // Validate configuration
    config.validate()?;

    // Auto-migrate credentials on load if needed
    if config.migrate_credentials()? {
        println!("Migrated credentials to secure storage.");
        // Save config without PAT
        save_to_path(&config, &config_path)?;
    }

    Ok(config)
}

pub fn save_to_path<P: AsRef<Path>>(config: &Config, path: P) -> Result<()> {
    let toml_string = toml::to_string_pretty(config).context("Failed to serialize config")?;

    std::fs::write(path.as_ref(), toml_string).context("Failed to write config file")?;

    Ok(())
}
