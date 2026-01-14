use ao_no_out7ook::commands::agent;
use ao_no_out7ook::config::{Config, DevOpsConfig};
use ao_no_out7ook::devops::client::DevOpsClient;
use ao_no_out7ook::devops::models::WorkItem;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::NamedTempFile;

fn create_test_config() -> Config {
    let mut config = Config::default();
    config.devops = DevOpsConfig {
        pat: Some("test-pat".to_string()),
        organization: "test-org".to_string(),
        project: "test-project".to_string(),
        skip_states: vec![],
    };
    config
}

fn create_mock_work_item(id: u32, title: &str, state: &str, wi_type: &str) -> WorkItem {
    let mut fields = HashMap::new();
    fields.insert("System.Title".to_string(), json!(title));
    fields.insert("System.State".to_string(), json!(state));
    fields.insert("System.WorkItemType".to_string(), json!(wi_type));

    WorkItem {
        id,
        rev: 1,
        fields,
        relations: None,
        url: format!("https://dev.azure.com/test/{}", id),
    }
}

#[test]
fn test_context_command_no_active_task() {
    // This test verifies behavior when no task is active
    // Since we can't easily mock the state file, we'll test the logic
    // in a more controlled way by checking error handling

    // For now, this is a placeholder that documents expected behavior
    // Real implementation would need state file mocking
    assert!(true); // Placeholder
}

#[test]
fn test_decompose_valid_json() {
    let config = create_test_config();

    // Create temporary JSON file
    let temp_file = NamedTempFile::new().unwrap();
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

    fs::write(temp_file.path(), json_content.to_string()).unwrap();

    // Note: This test requires mocking DevOpsClient
    // For now, we verify JSON parsing works
    let content = fs::read_to_string(temp_file.path()).unwrap();
    let parsed: agent::DecomposeInput = serde_json::from_str(&content).unwrap();

    assert_eq!(parsed.parent_id, 123);
    assert_eq!(parsed.tasks.len(), 2);
    assert_eq!(parsed.tasks[0].title, "Test Task 1");
    assert_eq!(parsed.tasks[0].effort, Some(3.0));
    assert_eq!(parsed.tasks[1].work_item_type, None); // Uses default
}

#[test]
fn test_decompose_invalid_json() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), "{ invalid json }").unwrap();

    let content = fs::read_to_string(temp_file.path()).unwrap();
    let result: Result<agent::DecomposeInput, _> = serde_json::from_str(&content);

    assert!(result.is_err());
}

#[test]
fn test_decompose_missing_required_fields() {
    let temp_file = NamedTempFile::new().unwrap();
    let json_content = json!({
        "parent_id": 123,
        "tasks": [
            {
                // Missing required "title" field
                "description": "Test"
            }
        ]
    });

    fs::write(temp_file.path(), json_content.to_string()).unwrap();

    let content = fs::read_to_string(temp_file.path()).unwrap();
    let result: Result<agent::DecomposeInput, _> = serde_json::from_str(&content);

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
fn test_doc_command_embedded_content() {
    // Verify that the embedded workflow content exists
    // The actual `doc` command in main.rs uses include_str!
    let workflow_content = include_str!("../.agent/workflows/breakdown_story.md");

    assert!(workflow_content.contains("description:"));
    assert!(workflow_content.contains("ano7"));
    assert!(workflow_content.len() > 100); // Has substantial content
}
