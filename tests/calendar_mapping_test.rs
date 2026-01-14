use ao_no_out7ook::platform::{ensure_writable, get_state_dir, state_paths};
use ao_no_out7ook::state::{CalendarMapping, State};
use chrono::Utc;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_calendar_mapping_upsert_new() {
    let mut state = State::default();
    assert!(state.calendar_mappings.is_empty());

    state.upsert_calendar_mapping(123, "event-abc".to_string());

    assert_eq!(state.calendar_mappings.len(), 1);
    assert_eq!(state.calendar_mappings[0].work_item_id, 123);
    assert_eq!(state.calendar_mappings[0].event_id, "event-abc");
}

#[test]
fn test_calendar_mapping_upsert_existing() {
    let mut state = State::default();
    state.upsert_calendar_mapping(123, "event-old".to_string());
    state.upsert_calendar_mapping(123, "event-new".to_string());

    // Should not add duplicate, just update
    assert_eq!(state.calendar_mappings.len(), 1);
    assert_eq!(state.calendar_mappings[0].event_id, "event-new");
    assert!(state.calendar_mappings[0].last_synced.is_some());
}

#[test]
fn test_calendar_mapping_get() {
    let mut state = State::default();
    state.upsert_calendar_mapping(100, "event-100".to_string());
    state.upsert_calendar_mapping(200, "event-200".to_string());

    assert_eq!(state.get_calendar_event(100), Some("event-100"));
    assert_eq!(state.get_calendar_event(200), Some("event-200"));
    assert_eq!(state.get_calendar_event(999), None);
}

#[test]
fn test_calendar_mapping_remove() {
    let mut state = State::default();
    state.upsert_calendar_mapping(100, "event-100".to_string());
    state.upsert_calendar_mapping(200, "event-200".to_string());

    let removed = state.remove_calendar_mapping(100);
    assert!(removed);
    assert_eq!(state.calendar_mappings.len(), 1);
    assert_eq!(state.get_calendar_event(100), None);
    assert_eq!(state.get_calendar_event(200), Some("event-200"));

    // Removing non-existent should return false
    let removed_again = state.remove_calendar_mapping(100);
    assert!(!removed_again);
}

#[test]
fn test_calendar_mapping_serialization() {
    let mut state = State::default();
    state.upsert_calendar_mapping(123, "event-xyz".to_string());

    let json = serde_json::to_string(&state).unwrap();
    assert!(json.contains("calendar_mappings"));
    assert!(json.contains("event-xyz"));

    // Deserialize back
    let loaded: State = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.calendar_mappings.len(), 1);
    assert_eq!(loaded.get_calendar_event(123), Some("event-xyz"));
}

#[test]
fn test_backward_compatibility_missing_field() {
    // Old state file without calendar_mappings field
    let old_json = r#"{
        "version": "1.0.0",
        "current_task": null,
        "last_sync": {},
        "work_hours": {"start": "", "end": ""}
    }"#;

    let state: State = serde_json::from_str(old_json).unwrap();

    // Should default to empty vec
    assert!(state.calendar_mappings.is_empty());
}

#[test]
fn test_state_dir_override() {
    let temp = TempDir::new().unwrap();
    let override_path = temp.path().to_path_buf();

    let dir = get_state_dir(Some(&override_path)).unwrap();
    assert_eq!(dir, override_path);
}

#[test]
fn test_ensure_writable_nested() {
    let temp = TempDir::new().unwrap();
    let nested = temp.path().join("deep").join("nested").join("dir");

    let result = ensure_writable(&nested);
    assert!(result.is_ok());
    assert!(nested.exists());
}

#[test]
fn test_state_paths_resolution() {
    let temp = TempDir::new().unwrap();
    let override_path = temp.path().to_path_buf();

    let (lock, state) = state_paths(Some(&override_path)).unwrap();

    assert!(lock.ends_with("state.lock"));
    assert!(state.ends_with("state.json"));
    assert_eq!(lock.parent(), state.parent());
}

#[test]
fn test_state_save_and_load_with_mappings() {
    let temp = TempDir::new().unwrap();
    let state_path = temp.path().join("state.json");

    // Create state with mappings
    let mut state = State::default();
    state.upsert_calendar_mapping(111, "event-a".to_string());
    state.upsert_calendar_mapping(222, "event-b".to_string());
    state.save(&state_path).unwrap();

    // Load back
    let loaded = State::load(&state_path).unwrap();
    assert_eq!(loaded.calendar_mappings.len(), 2);
    assert_eq!(loaded.get_calendar_event(111), Some("event-a"));
    assert_eq!(loaded.get_calendar_event(222), Some("event-b"));
}
