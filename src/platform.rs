//! Cross-platform utilities for directory resolution and file operations.
//!
//! This module provides OS-agnostic state directory resolution with
//! permission fallback chains to handle various deployment scenarios.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Get the state directory with cross-platform fallback chain.
///
/// Priority order:
/// 1. Explicit override (for testing/CI)
/// 2. Home directory (~/.ao-no-out7ook)
/// 3. Platform-specific data directory (XDG on Linux, AppData on Windows)
/// 4. Current working directory (last resort)
///
/// Each directory is validated for write access before being selected.
pub fn get_state_dir(override_dir: Option<&PathBuf>) -> Result<PathBuf> {
    // Priority 1: Explicit override (testing/CI)
    if let Some(dir) = override_dir {
        ensure_writable(dir)?;
        return Ok(dir.clone());
    }

    // Priority 2: Home directory (traditional Unix-style)
    if let Some(home) = home::home_dir() {
        let dir = home.join(".ao-no-out7ook");
        if ensure_writable(&dir).is_ok() {
            return Ok(dir);
        }
        // Log but continue to fallback
        eprintln!(
            "Warning: Cannot write to {}. Trying fallback locations.",
            dir.display()
        );
    }

    // Priority 3: Platform-specific data directory
    // - Linux: ~/.local/share/ao-no-out7ook
    // - macOS: ~/Library/Application Support/ao-no-out7ook
    // - Windows: C:\Users\<User>\AppData\Local\ao-no-out7ook
    if let Some(data) = dirs::data_local_dir() {
        let dir = data.join("ao-no-out7ook");
        if ensure_writable(&dir).is_ok() {
            return Ok(dir);
        }
    }

    // Priority 4: Current working directory (absolute last resort)
    let dir = PathBuf::from(".ao-no-out7ook");
    ensure_writable(&dir).context(
        "Cannot create state directory in any location. \
         Check file permissions or set state_dir_override in config.",
    )?;
    Ok(dir)
}

/// Ensure a directory exists and is writable by the current user.
///
/// Creates the directory if it doesn't exist, then tests write access
/// by creating and removing a temporary file.
pub fn ensure_writable(dir: &PathBuf) -> Result<()> {
    // Create directory structure
    fs::create_dir_all(dir)
        .with_context(|| format!("Failed to create directory: {}", dir.display()))?;

    // Test write access with a temporary file
    let test_path = dir.join(".write_test");
    fs::write(&test_path, b"test")
        .with_context(|| format!("Directory {} is not writable", dir.display()))?;

    // Cleanup test file, ignore errors (file might be held by antivirus on Windows)
    let _ = fs::remove_file(&test_path);

    Ok(())
}

/// Get the state file paths (lock and state JSON) for the given config.
///
/// This is the canonical way to get state paths, respecting config overrides.
pub fn state_paths(state_dir_override: Option<&PathBuf>) -> Result<(PathBuf, PathBuf)> {
    let state_dir = get_state_dir(state_dir_override)?;
    Ok((state_dir.join("state.lock"), state_dir.join("state.json")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_override_dir_takes_priority() {
        let temp = TempDir::new().unwrap();
        let override_path = temp.path().to_path_buf();

        let result = get_state_dir(Some(&override_path));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), override_path);
    }

    #[test]
    fn test_ensure_writable_creates_dir() {
        let temp = TempDir::new().unwrap();
        let nested = temp.path().join("a").join("b").join("c");

        let result = ensure_writable(&nested);
        assert!(result.is_ok());
        assert!(nested.exists());
    }

    #[test]
    fn test_state_paths_with_override() {
        let temp = TempDir::new().unwrap();
        let override_path = temp.path().to_path_buf();

        let (lock, state) = state_paths(Some(&override_path)).unwrap();
        assert_eq!(lock, override_path.join("state.lock"));
        assert_eq!(state, override_path.join("state.json"));
    }

    #[test]
    fn test_fallback_when_home_unavailable() {
        // This test simulates the fallback by not providing an override
        // The actual fallback behavior depends on the environment
        let result = get_state_dir(None);
        assert!(result.is_ok(), "Should always find a writable location");
    }
}
