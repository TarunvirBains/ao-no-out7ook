use crate::config::Config;
use anyhow::{Context, Result};

pub fn list(config: &Config) -> Result<()> {
    // Pretty print config as TOML
    // Since Config struct derives Serialize, we can just serialize it
    let toml_str = toml::to_string_pretty(config).context("Failed to serialize config")?;
    println!("{}", toml_str);
    Ok(())
}

pub fn get(key: &str, config: &Config) -> Result<()> {
    // Use serde_json::to_value to inspect fields dynamically by key path
    // Simple implementation: convert to Value and walk path
    let value = serde_json::to_value(config).context("Failed to serialize config")?;

    // Support dot notation: "devops.organization"
    let mut current = &value;
    for part in key.split('.') {
        current = current
            .get(part)
            .context(format!("Key not found: {}", part))?;
    }

    // Print value nicely
    match current {
        serde_json::Value::String(s) => println!("{}", s),
        v => println!("{}", v),
    }

    Ok(())
}

pub fn set(key: &str, value: &str) -> Result<()> {
    // For MVP, implementing "set" is tricky because we need to preserve comments in TOML
    // The `config` crate is mostly for reading.
    // `toml_edit` crate is better for preserving structure, but we didn't add it.
    //
    // Fallback: Load raw TOML string, parse with `toml` (serde), update, save.
    // This loses comments.
    // For Phase 1 MVP, we can warn user or just append/update.

    println!(
        "Config set not fully implemented in MVP. Please edit ~/.ao-no-out7ook/config.toml manually."
    );
    println!("Requested change: {} = {}", key, value);
    Ok(())
}
