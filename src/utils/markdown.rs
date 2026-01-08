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

pub fn to_markdown(item: &WorkItem) -> String {
    let title = item.get_title().unwrap_or("");
    let state = item.get_state().unwrap_or("");
    let assigned = item.get_assigned_to().unwrap_or("");
    let id = item.id;

    // Frontmatter
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("id: {}\n", id));
    out.push_str(&format!("title: {}\n", title));
    out.push_str(&format!("state: {}\n", state));
    out.push_str(&format!("assigned_to: {}\n", assigned));
    // Could add relations here too
    out.push_str("---\n\n");

    // Body (Description)
    if let Some(desc) = item
        .fields
        .get("System.Description")
        .and_then(|v| v.as_str())
    {
        out.push_str(desc);
    }

    out
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
