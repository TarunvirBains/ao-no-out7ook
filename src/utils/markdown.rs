use crate::devops::models::WorkItem;
use anyhow::Result;
use std::collections::HashMap;

// Simple Frontmatter + Body format
// ---
// id: 123
// title: Title
// state: Active
// assigned_to: Person
// ---
// Description...

/// Generate Markdown for a work item (FR4.1 - Enhanced)
/// Supports both simple frontmatter and hierarchical header formats
pub fn to_markdown(item: &WorkItem) -> String {
    // Enhanced format: Use headers for hierarchy
    let mut md = String::new();

    // Determine work item type and header level
    let work_item_type = item
        .get_work_item_type()
        .unwrap_or_else(|| "Work Item".to_string());
    let header_level = match work_item_type.as_str() {
        "Epic" => "#",
        "Feature" => "##",
        "User Story" => "###",
        "Task" | "Bug" => "####",
        _ => "###", // Default to User Story level
    };

    // Title line: "# Epic: Title (#ID)"
    let title = item.get_title().unwrap_or("Untitled");
    let id = item.id;
    md.push_str(&format!(
        "{} {}: {} (#{})\n",
        header_level, work_item_type, title, id
    ));

    // Metadata line
    let mut metadata = Vec::new();

    if let Some(state) = item.get_state() {
        metadata.push(format!("**State:** {}", state));
    }

    if let Some(assigned_to) = item.get_assigned_to() {
        metadata.push(format!("**Assigned:** {}", assigned_to));
    }

    if let Some(priority) = item
        .fields
        .get("Microsoft.VSTS.Common.Priority")
        .and_then(|v| v.as_i64())
    {
        metadata.push(format!("**Priority:** {}", priority));
    }

    // Parent reference
    if let Some(parent_id) = item.get_parent_id() {
        metadata.push(format!("**Parent:** #{}", parent_id));
    }

    // Effort/work
    if let Some(effort) = item
        .fields
        .get("Microsoft.VSTS.Scheduling.Effort")
        .and_then(|v| v.as_f64())
    {
        metadata.push(format!("**Effort:** {}h", effort));
    }

    // Tags
    if let Some(tags) = item.get_tags() {
        if !tags.is_empty() {
            metadata.push(format!("**Tags:** {}", tags.join(", ")));
        }
    }

    if !metadata.is_empty() {
        md.push_str(&format!("{}\n", metadata.join(" | ")));
    }

    // Description (if exists)
    md.push('\n');
    if let Some(desc) = item
        .fields
        .get("System.Description")
        .and_then(|v| v.as_str())
    {
        let cleaned_desc = strip_html_tags(desc);
        md.push_str(&cleaned_desc);
        md.push('\n');
    }

    md
}

/// Strip HTML tags from description (simple implementation)
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result.trim().to_string()
}

pub struct ParsedWorkItem {
    pub id: Option<u32>,
    pub fields: HashMap<String, String>,
    pub description: String,
}

pub fn from_markdown(content: &str) -> Result<ParsedWorkItem> {
    // Very basic parser for MVP.
    // Split by "---"
    let parts: Vec<&str> = content.splitn(3, "---").collect();

    if parts.len() < 3 {
        // Maybe no frontmatter?
        return Ok(ParsedWorkItem {
            id: None,
            fields: HashMap::new(),
            description: content.to_string(),
        });
    }

    let frontmatter = parts[1];
    let body = parts[2].trim();

    let mut fields = HashMap::new();
    let mut id = None;

    for line in frontmatter.lines() {
        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim();
            let val = val.trim();
            match key {
                "id" => id = val.parse().ok(),
                "title" => {
                    fields.insert("System.Title".to_string(), val.to_string());
                }
                "state" => {
                    fields.insert("System.State".to_string(), val.to_string());
                }
                "assigned_to" => {
                    // Assigning by display name is tricky, usually need email or ID.
                    // For now, ignore or store as hint?
                }
                _ => {}
            }
        }
    }

    Ok(ParsedWorkItem {
        id,
        fields,
        description: body.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devops::models::WorkItemRelation;
    use serde_json::json;

    fn create_test_work_item(work_item_type: &str, id: u32) -> WorkItem {
        let fields_map = json!({
            "System.WorkItemType": work_item_type,
            "System.Title": format!("Test {}", work_item_type),
            "System.State": "Active",
        });

        WorkItem {
            id,
            rev: 1,
            url: format!("https://dev.azure.com/test/_apis/wit/workItems/{}", id),
            fields: fields_map
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            relations: None,
        }
    }

    #[test]
    fn test_markdown_epic_header() {
        let item = create_test_work_item("Epic", 456);
        let md = to_markdown(&item);

        assert!(md.starts_with("# Epic: Test Epic (#456)"));
        assert!(md.contains("**State:** Active"));
    }

    #[test]
    fn test_markdown_feature_header() {
        let item = create_test_work_item("Feature", 123);
        let md = to_markdown(&item);

        assert!(md.starts_with("## Feature: Test Feature (#123)"));
    }

    #[test]
    fn test_markdown_user_story_header() {
        let item = create_test_work_item("User Story", 789);
        let md = to_markdown(&item);

        assert!(md.starts_with("### User Story: Test User Story (#789)"));
    }

    #[test]
    fn test_markdown_task_header() {
        let item = create_test_work_item("Task", 101);
        let md = to_markdown(&item);

        assert!(md.starts_with("#### Task: Test Task (#101)"));
    }

    #[test]
    fn test_markdown_with_metadata() {
        let mut item = create_test_work_item("User Story", 200);
        item.fields.insert(
            "System.AssignedTo".to_string(),
            json!({"displayName": "John Doe"}),
        );
        item.fields
            .insert("Microsoft.VSTS.Common.Priority".to_string(), json!(1));
        item.fields
            .insert("Microsoft.VSTS.Scheduling.Effort".to_string(), json!(5.0));
        item.fields
            .insert("System.Tags".to_string(), json!("frontend; ux; important"));

        let md = to_markdown(&item);

        assert!(md.contains("**State:** Active"));
        assert!(md.contains("**Assigned:** John Doe"));
        assert!(md.contains("**Priority:** 1"));
        assert!(md.contains("**Effort:** 5h"));
        assert!(md.contains("**Tags:** frontend, ux, important"));
    }

    #[test]
    fn test_markdown_with_parent() {
        let mut item = create_test_work_item("User Story", 300);
        item.relations = Some(vec![WorkItemRelation {
            rel: "System.LinkTypes.Hierarchy-Reverse".to_string(),
            url: "https://dev.azure.com/test/_apis/wit/workItems/250".to_string(),
            attributes: None,
        }]);

        let md = to_markdown(&item);

        assert!(md.contains("**Parent:** #250"));
    }

    #[test]
    fn test_markdown_with_description() {
        let mut item = create_test_work_item("Task", 400);
        item.fields.insert(
            "System.Description".to_string(),
            json!("<p>This is a <strong>test</strong> description</p>"),
        );

        let md = to_markdown(&item);

        assert!(md.contains("This is a test description"));
        assert!(!md.contains("<p>"));
        assert!(!md.contains("<strong>"));
    }

    #[test]
    fn test_strip_html_tags() {
        assert_eq!(strip_html_tags("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html_tags("<div><span>Test</span></div>"), "Test");
        assert_eq!(strip_html_tags("Plain text"), "Plain text");
        assert_eq!(
            strip_html_tags("<p>Multi <strong>word</strong> text</p>"),
            "Multi word text"
        );
    }
}
