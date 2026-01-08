use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::path::Path;

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

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DevOpsConfig {
    pub organization: String,
    pub project: String,
    pub pat: Option<String>, // Can be optional if loading from env/keyring
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
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            task_expiry_hours: 24,
        }
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

    load_from_path(config_path)
}
