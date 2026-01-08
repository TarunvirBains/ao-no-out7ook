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
