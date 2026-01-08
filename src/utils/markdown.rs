use crate::devops::models::WorkItem;
use anyhow::Result;

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
    if let Some(tags) = item.get_tags()
        && !tags.is_empty() {
            metadata.push(format!("**Tags:** {}", tags.join(", ")));
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

/// Validation error with line content and suggestions (FR4.3)
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub line: usize,
    pub line_content: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,   // Blocks import
    Warning, // Allows import but shows warning
}

/// Validate markdown structure with hierarchy checks (FR4.3)
pub fn validate_markdown_structure(content: &str) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();
    let _lines: Vec<&str> = content.lines().collect();

    // Parse items first
    let items = from_markdown(content)?;

    // Validate each item
    for (idx, item) in items.iter().enumerate() {
        // Find the line number for this item (approximate)
        let line_num = idx + 1; // Simple approximation
        let line_content = format!(
            "{} {}: {} (#{})",
            get_header_prefix(&item.work_item_type),
            item.work_item_type,
            item.title,
            item.id.unwrap_or(0)
        );

        // Required field: State
        if !item.fields.contains_key("System.State") {
            errors.push(ValidationError {
                line: line_num,
                line_content: line_content.clone(),
                message: format!("{} is missing required field: State", item.work_item_type),
                suggestion: Some("Add **State:** <value> to the metadata line".to_string()),
                severity: Severity::Error,
            });
        }

        // Hierarchy validation
        match item.work_item_type.as_str() {
            "Feature" => {
                if item.parent_id.is_none() {
                    errors.push(ValidationError {
                        line: line_num,
                        line_content: line_content.clone(),
                        message: "Feature must have an Epic parent".to_string(),
                        suggestion: Some(
                            "Add **Parent:** #<epic_id> to the metadata line".to_string(),
                        ),
                        severity: Severity::Error,
                    });
                }
            }
            "User Story" => {
                if item.parent_id.is_none() {
                    errors.push(ValidationError {
                        line: line_num,
                        line_content: line_content.clone(),
                        message: "User Story must have a Feature or Epic parent".to_string(),
                        suggestion: Some(
                            "Add **Parent:** #<feature_id> to the metadata line".to_string(),
                        ),
                        severity: Severity::Error,
                    });
                }
            }
            "Task" | "Bug" => {
                if item.parent_id.is_none() {
                    errors.push(ValidationError {
                        line: line_num,
                        line_content: line_content.clone(),
                        message: format!(
                            "{} must have a User Story or Feature parent",
                            item.work_item_type
                        ),
                        suggestion: Some(
                            "Add **Parent:** #<story_id> to the metadata line".to_string(),
                        ),
                        severity: Severity::Error,
                    });
                }
            }
            "Epic" => {
                // Epic can be standalone
            }
            _ => {
                errors.push(ValidationError {
                    line: line_num,
                    line_content,
                    message: format!("Unknown work item type: {}", item.work_item_type),
                    suggestion: Some("Use Epic, Feature, User Story, Task, or Bug".to_string()),
                    severity: Severity::Warning,
                });
            }
        }
    }

    Ok(errors)
}

fn get_header_prefix(work_item_type: &str) -> &'static str {
    match work_item_type {
        "Epic" => "#",
        "Feature" => "##",
        "User Story" => "###",
        "Task" | "Bug" => "####",
        _ => "###",
    }
}

/// Display validation errors in user-friendly format
pub fn display_validation_errors(errors: &[ValidationError]) {
    for error in errors {
        match error.severity {
            Severity::Error => println!("❌ Line {}: {}", error.line, error.line_content),
            Severity::Warning => println!("⚠  Line {}: {}", error.line, error.line_content),
        }
        println!("    Error: {}", error.message);
        if let Some(suggestion) = &error.suggestion {
            println!("    Suggestion: {}", suggestion);
        }
        println!();
    }
}

/// Enhanced parsed work item (FR4.2)
#[derive(Debug, Clone)]
pub struct ParsedWorkItem {
    pub id: Option<u32>,
    pub work_item_type: String,
    pub title: String,
    pub fields: std::collections::HashMap<String, String>,
    pub parent_id: Option<u32>,
    pub description: String,
}

/// Parse hierarchical markdown back to work items (FR4.2)
pub fn from_markdown(content: &str) -> Result<Vec<ParsedWorkItem>> {
    let mut items = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Check for work item header (# Epic, ## Feature, ### Story, #### Task)
        if line.starts_with('#') {
            let (item, consumed) = parse_work_item(&lines[i..], i + 1)?;
            items.push(item);
            i += consumed;
        } else {
            i += 1;
        }
    }

    Ok(items)
}

fn parse_work_item(lines: &[&str], _start_line: usize) -> Result<(ParsedWorkItem, usize)> {
    let header_line = lines[0];

    // Parse header: "## Feature: Title (#123)"
    let (header_level, rest) = parse_header(header_line)?;
    let work_item_type = determine_type_from_header(header_level, rest)?;

    // Extract title and ID from "Feature: Title (#123)"
    let (title, id) = parse_title_and_id(rest)?;

    // Parse metadata line if present
    let mut fields = std::collections::HashMap::new();
    let mut parent_id = None;
    let mut description = String::new();
    let mut consumed = 1;

    if lines.len() > 1 {
        let metadata_line = lines[1].trim();
        if metadata_line.contains("**") {
            // Parse metadata: "**State:** Active | **Parent:** #123"
            parse_metadata(metadata_line, &mut fields, &mut parent_id)?;
            consumed += 1;

            // Collect description (lines after metadata until next header or separator)
            let mut desc_lines = Vec::new();
            for j in consumed..lines.len() {
                let line = lines[j].trim();
                if line.starts_with('#') || line.starts_with("---") {
                    break;
                }
                if !line.is_empty() {
                    desc_lines.push(line);
                }
                consumed += 1;
            }
            if !desc_lines.is_empty() {
                description = desc_lines.join("\n");
            }
        }
    }

    Ok((
        ParsedWorkItem {
            id,
            work_item_type,
            title,
            fields,
            parent_id,
            description,
        },
        consumed,
    ))
}

fn parse_header(line: &str) -> Result<(usize, &str)> {
    let level = line.chars().take_while(|&c| c == '#').count();
    if level == 0 {
        anyhow::bail!("Not a header line");
    }
    let rest = line[level..].trim();
    Ok((level, rest))
}

fn determine_type_from_header(level: usize, content: &str) -> Result<String> {
    // Try to extract type from content first (e.g., "Epic: Title")
    if let Some(colon_pos) = content.find(':') {
        let type_str = content[..colon_pos].trim();
        return Ok(type_str.to_string());
    }

    // Fallback to header level
    let type_name = match level {
        1 => "Epic",
        2 => "Feature",
        3 => "User Story",
        4 => "Task",
        _ => anyhow::bail!("Invalid header level: {}", level),
    };
    Ok(type_name.to_string())
}

fn parse_title_and_id(content: &str) -> Result<(String, Option<u32>)> {
    // Extract from "Epic: Title (#123)" or "Title (#123)"
    let without_type = if let Some(colon_pos) = content.find(':') {
        content[colon_pos + 1..].trim()
    } else {
        content
    };

    // Extract ID from "(#123)"
    let id = if let Some(start) = without_type.rfind("(#") {
        if let Some(end) = without_type[start..].find(')') {
            let id_str = &without_type[start + 2..start + end];
            id_str.parse().ok()
        } else {
            None
        }
    } else {
        None
    };

    // Extract title (everything before "(#...")
    let title = if let Some(paren_pos) = without_type.rfind("(#") {
        without_type[..paren_pos].trim()
    } else {
        without_type.trim()
    };

    Ok((title.to_string(), id))
}

fn parse_metadata(
    line: &str,
    fields: &mut std::collections::HashMap<String, String>,
    parent_id: &mut Option<u32>,
) -> Result<()> {
    // Split by "| " to get individual metadata items
    let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();

    for part in parts {
        if part.is_empty() {
            continue;
        }

        // Parse "**Key:** Value"
        if let Some(start) = part.find("**")
            && let Some(end) = part[start + 2..].find("**") {
                let key = part[start + 2..start + 2 + end].trim();
                let value = part[start + 2 + end + 2..].trim_start_matches(':').trim();

                match key {
                    "State" => {
                        fields.insert("System.State".to_string(), value.to_string());
                    }
                    "Assigned" => {
                        fields.insert("System.AssignedTo".to_string(), value.to_string());
                    }
                    "Priority" => {
                        fields.insert(
                            "Microsoft.VSTS.Common.Priority".to_string(),
                            value.to_string(),
                        );
                    }
                    "Effort" => {
                        let effort_val = value.trim_end_matches('h');
                        fields.insert(
                            "Microsoft.VSTS.Scheduling.Effort".to_string(),
                            effort_val.to_string(),
                        );
                    }
                    "Tags" => {
                        fields.insert("System.Tags".to_string(), value.replace(", ", ";"));
                    }
                    "Parent" => {
                        if let Some(id_str) = value.strip_prefix('#') {
                            *parent_id = id_str.parse().ok();
                        }
                    }
                    _ => {
                        // Store unknown fields as-is
                        fields.insert(key.to_string(), value.to_string());
                    }
                }
            }
    }

    Ok(())
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
