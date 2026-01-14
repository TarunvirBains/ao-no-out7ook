use ao_no_out7ook::devops::models::WorkItem;
use ao_no_out7ook::utils::markdown::{from_markdown, to_markdown};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use tempfile::NamedTempFile;

fn create_mock_work_item(id: u32, title: &str, state: &str) -> WorkItem {
    let mut fields = HashMap::new();
    fields.insert("System.Title".to_string(), json!(title));
    fields.insert("System.State".to_string(), json!(state));
    fields.insert("System.WorkItemType".to_string(), json!("Task"));

    WorkItem {
        id,
        rev: 1,
        fields,
        relations: None,
        url: format!("https://dev.azure.com/test/{}", id),
    }
}

#[test]
fn test_markdown_export_creates_valid_format() {
    let work_item = create_mock_work_item(100, "Test Task", "Active");
    let markdown = to_markdown(&work_item);

    assert!(markdown.contains("#### Task:"));
    assert!(markdown.contains("Test Task"));
    assert!(markdown.contains("#100"));
    assert!(markdown.contains("**State:** Active"));
}

#[test]
fn test_markdown_import_parses_work_item() {
    let markdown = r#"#### Task: Import Test (#0)
**State:** New | **Effort:** 2h

This is a new task description.
"#;

    let items = from_markdown(markdown).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Import Test");
    assert!(items[0].description.contains("new task"));
}

#[test]
fn test_markdown_round_trip_consistency() {
    let original = create_mock_work_item(200, "Round Trip Test", "Active");

    // Export to markdown
    let markdown = to_markdown(&original);

    // Verify markdown contains key fields
    assert!(markdown.contains("Round Trip Test"));
    assert!(markdown.contains("#200"));
}

#[test]
fn test_markdown_export_file_operations() {
    let temp_file = NamedTempFile::new().unwrap();
    let work_item = create_mock_work_item(300, "File Test", "Active");

    // Export to file
    let markdown = to_markdown(&work_item);
    fs::write(temp_file.path(), &markdown).unwrap();

    // Read back
    let content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("File Test"));
    assert!(content.contains("#300"));
}

#[test]
fn test_markdown_import_from_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let markdown = r#"#### Task: File Import (#0)
**State:** New

From file test.
"#;

    fs::write(temp_file.path(), markdown).unwrap();

    // Read and parse
    let content = fs::read_to_string(temp_file.path()).unwrap();
    let items = from_markdown(&content).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "File Import");
}

#[test]
fn test_markdown_multiple_work_items() {
    let markdown = r#"#### Task: First Task (#1)
**State:** Active

First description.

---

#### Task: Second Task (#2)
**State:** New

Second description.
"#;

    let items = from_markdown(markdown).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].title, "First Task");
    assert_eq!(items[1].title, "Second Task");
}

#[test]
fn test_markdown_preserves_work_item_type() {
    let mut fields = HashMap::new();
    fields.insert("System.Title".to_string(), json!("Bug Test"));
    fields.insert("System.State".to_string(), json!("Active"));
    fields.insert("System.WorkItemType".to_string(), json!("Bug"));

    let work_item = WorkItem {
        id: 400,
        rev: 1,
        fields,
        relations: None,
        url: "https://dev.azure.com/test/400".to_string(),
    };

    let markdown = to_markdown(&work_item);
    assert!(markdown.contains("#### Bug:"));
}
