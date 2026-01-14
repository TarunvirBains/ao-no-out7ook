use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct State {
    pub version: String,
    pub current_task: Option<CurrentTask>,
    pub last_sync: SyncTimestamps,
    pub work_hours: WorkHoursState,
    /// FR3.3: Mapping between work items and calendar events
    #[serde(default)]
    pub calendar_mappings: Vec<CalendarMapping>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            current_task: None,
            last_sync: SyncTimestamps::default(),
            work_hours: WorkHoursState::default(),
            calendar_mappings: Vec::new(),
        }
    }
}

/// FR3.3: Represents a link between a DevOps work item and a calendar event
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarMapping {
    pub work_item_id: u32,
    pub event_id: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub last_synced: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrentTask {
    pub id: u32,
    pub title: String,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub timer_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SyncTimestamps {
    pub devops: Option<DateTime<Utc>>,
    pub sevenpace: Option<DateTime<Utc>>,
    pub calendar: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WorkHoursState {
    pub start: String,
    pub end: String,
}

impl State {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path).context("Failed to read state file")?;

        // Handle empty file case
        if content.trim().is_empty() {
            return Ok(Self::default());
        }

        serde_json::from_str(&content).context("Failed to parse state JSON")
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self).context("Failed to serialize state")?;

        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Atomic write: write to temp file then rename using standard fs calls
        // For simplicity in MVP/locking context, direct write is acceptable inside lock
        // But atomic write is safer against crashes
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, content)?;
        fs::rename(&temp_path, path)?;

        Ok(())
    }

    // --- FR3.3: Calendar Mapping Operations ---

    /// Add or update a mapping between a work item and calendar event
    pub fn upsert_calendar_mapping(&mut self, work_item_id: u32, event_id: String) {
        let now = Utc::now();

        // Check if mapping already exists
        if let Some(existing) = self
            .calendar_mappings
            .iter_mut()
            .find(|m| m.work_item_id == work_item_id)
        {
            existing.event_id = event_id;
            existing.last_synced = Some(now);
        } else {
            self.calendar_mappings.push(CalendarMapping {
                work_item_id,
                event_id,
                created_at: now,
                last_synced: None,
            });
        }
    }

    /// Get the calendar event ID for a work item, if mapped
    pub fn get_calendar_event(&self, work_item_id: u32) -> Option<&str> {
        self.calendar_mappings
            .iter()
            .find(|m| m.work_item_id == work_item_id)
            .map(|m| m.event_id.as_str())
    }

    /// Remove a calendar mapping for a work item
    pub fn remove_calendar_mapping(&mut self, work_item_id: u32) -> bool {
        let initial_len = self.calendar_mappings.len();
        self.calendar_mappings
            .retain(|m| m.work_item_id != work_item_id);
        self.calendar_mappings.len() < initial_len
    }

    /// Get all calendar mappings (for sync operations)
    pub fn get_all_calendar_mappings(&self) -> &[CalendarMapping] {
        &self.calendar_mappings
    }
}

pub fn with_state_lock<F, R>(lock_path: &Path, state_path: &Path, f: F) -> Result<R>
where
    F: FnOnce(&mut State) -> Result<R>,
{
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(lock_path)
        .context("Failed to open lock file")?;

    file.lock_exclusive().context("Failed to acquire lock")?;

    // Load state
    let mut state = State::load(state_path)?;

    // Execute closure
    let result = f(&mut state);

    // If success, save state
    if result.is_ok() {
        state.save(state_path)?;
    }

    file.unlock().context("Failed to unlock")?;

    result
}
