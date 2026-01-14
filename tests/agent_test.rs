use ao_no_out7ook::commands::agent;
use ao_no_out7ook::state::{CurrentTask, State};
use chrono::Utc;
use serde_json::json;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_decompose_valid_json_structure() {
    let json_content = json!({
        "parent_id": 123,
        "tasks": [
            {
                "title": "Test Task 1",
                "description": "Description 1",
                "effort": 3.0,
                "work_item_type": "Task"
            },
            {
                "title": "Test Task 2",
                "description": "Description 2",
                "effort": 5.0
            }
        ]
    });

    let parsed: agent::DecomposeInput = serde_json::from_value(json_content).unwrap();

    assert_eq!(parsed.parent_id, 123);
    assert_eq!(parsed.tasks.len(), 2);
    assert_eq!(parsed.tasks[0].title, "Test Task 1");
    assert_eq!(parsed.tasks[0].effort, Some(3.0));
    assert_eq!(parsed.tasks[1].work_item_type, None); // Uses default
}

#[test]
fn test_decompose_invalid_json() {
    let result: Result<agent::DecomposeInput, _> = serde_json::from_str("{ invalid json }");
    assert!(result.is_err());
}

#[test]
fn test_decompose_missing_required_fields() {
    let json_content = json!({
        "parent_id": 123,
        "tasks": [
            {
                // Missing required "title" field
                 "description": "Test"
            }
        ]
    });

    let result: Result<agent::DecomposeInput, _> = serde_json::from_value(json_content);
    assert!(result.is_err());
}

#[test]
fn test_decompose_json_schema_validation() {
    // Valid minimal JSON
    let json_content = json!({
        "parent_id": 123,
        "tasks": []
    });

    let parsed: Result<agent::DecomposeInput, _> = serde_json::from_value(json_content);

    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap().tasks.len(), 0);
}

#[test]
fn test_decompose_task_structure() {
    let task_json = json!({
        "title": "Test Task",
        "description": "Test Description",
        "effort": 2.5,
        "work_item_type": "Bug"
    });

    let task: agent::DecomposeTask = serde_json::from_value(task_json).unwrap();

    assert_eq!(task.title, "Test Task");
    assert_eq!(task.description, Some("Test Description".to_string()));
    assert_eq!(task.effort, Some(2.5));
    assert_eq!(task.work_item_type, Some("Bug".to_string()));
}

#[test]
fn test_decompose_task_with_defaults() {
    let task_json = json!({
        "title": "Minimal Task"
    });

    let task: agent::DecomposeTask = serde_json::from_value(task_json).unwrap();

    assert_eq!(task.title, "Minimal Task");
    assert_eq!(task.description, None);
    assert_eq!(task.effort, None);
    assert_eq!(task.work_item_type, None);
}

#[test]
fn test_decompose_invalid_parent_id() {
    let json_content = json!({
        "parent_id": "not_a_number",
        "tasks": []
    });

    let result: Result<agent::DecomposeInput, _> = serde_json::from_value(json_content);
    assert!(result.is_err());
}

#[test]
fn test_doc_command_embedded_content() {
    // Verify that the embedded workflow content exists
    let workflow_content = include_str!("../.agent/workflows/breakdown_story.md");

    assert!(workflow_content.contains("description:"));
    assert!(workflow_content.contains("ano7"));
    assert!(workflow_content.len() > 100);
}

#[test]
fn test_context_state_file_loading() {
    // Test that State can be loaded from a temp file
    // This validates the state management that context command uses
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("state.json");

    let state = State {
        version: "1.0.0".to_string(),
        current_task: Some(CurrentTask {
            id: 123,
            title: "Test Task".to_string(),
            started_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(24),
            timer_id: Some("timer-123".to_string()),
        }),
        last_sync: Default::default(),
        work_hours: Default::default(),
    };

    // Save state
    let json = serde_json::to_string_pretty(&state).unwrap();
    fs::write(&state_path, json).unwrap();

    // Load state
    let loaded = State::load(&state_path).unwrap();
    assert_eq!(loaded.current_task.unwrap().id, 123);
}

#[test]
fn test_decompose_json_file_parsing() {
    // Test that decompose can parse from actual file
    let temp_file = NamedTempFile::new().unwrap();
    let json_input = json!({
        "parent_id": 456,
        "tasks": [
            {
                "title": "File Task 1",
                "effort": 2.0
            }
        ]
    });
    fs::write(temp_file.path(), json_input.to_string()).unwrap();

    // Read and parse
    let content = fs::read_to_string(temp_file.path()).unwrap();
    let parsed: agent::DecomposeInput = serde_json::from_str(&content).unwrap();

    assert_eq!(parsed.parent_id, 456);
    assert_eq!(parsed.tasks[0].title, "File Task 1");
}

#[test]
fn test_context_no_current_task() {
    // Test state with no current task
    let state = State::default();
    assert!(state.current_task.is_none());
}
