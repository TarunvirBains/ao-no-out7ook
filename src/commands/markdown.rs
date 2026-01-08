use crate::config::Config;
use crate::devops::client::DevOpsClient;
use crate::utils::markdown::{
    display_validation_errors, from_markdown, to_markdown, validate_markdown_structure, Severity,
};
use anyhow::{Context, Result};
use std::path::Path;

/// Export work items to markdown (FR4.1)
/// Exports ALL items including completed (full state snapshot)
pub fn export(config: &Config, ids: Vec<u32>, hierarchy: bool, output: &Path) -> Result<()> {
    let pat = config
        .devops
        .pat
        .as_deref()
        .context("DevOps PAT not set. Run 'task config set devops.pat <PAT>'")?;
    let client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);

    // Fetch work items
    let items: Vec<_> = if hierarchy {
        // TODO: Use get_hierarchy_items when available
        ids.iter()
            .map(|id| client.get_work_item(*id))
            .collect::<Result<Vec<_>>>()?
    } else {
        ids.iter()
            .map(|id| client.get_work_item(*id))
            .collect::<Result<Vec<_>>>()?
    };

    // Generate markdown using to_markdown
    let markdown = if hierarchy {
        // For hierarchy, we want to maintain structure
        items.iter().map(to_markdown).collect::<Vec<_>>().join("\n")
    } else {
        items
            .iter()
            .map(to_markdown)
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    };

    std::fs::write(output, markdown)?;
    println!("✓ Exported {} items to {}", items.len(), output.display());
    Ok(())
}

/// Import work items from markdown (FR4.2, FR4.3)
/// Skips completed/resolved/closed items by default
pub fn import(
    config: &Config,
    file: &Path,
    dry_run: bool,
    validate_only: bool,
    force: bool,
) -> Result<()> {
    let markdown = std::fs::read_to_string(file)?;

    // FR4.3: Validation
    let validation_errors = validate_markdown_structure(&markdown)?;
    if !validation_errors.is_empty() {
        println!("Validation results:");
        display_validation_errors(&validation_errors);

        let has_errors = validation_errors
            .iter()
            .any(|e| e.severity == Severity::Error);
        if has_errors {
            anyhow::bail!("Cannot proceed with validation errors");
        }
    }

    if validate_only {
        println!("✓ Markdown is valid");
        return Ok(());
    }

    // Parse work items
    let items = from_markdown(&markdown)?;

    // Filter out closed items unless forced
    let filtered_items: Vec<_> = if force {
        items
    } else {
        items
            .into_iter()
            .filter(|item| {
                let state = item
                    .fields
                    .get("System.State")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let is_closed = matches!(
                    state.to_lowercase().as_str(),
                    "completed" | "resolved" | "closed" | "removed"
                );
                if is_closed {
                    println!(
                        "⊘ Skipping closed item: {} #{} (use --force to import)",
                        item.work_item_type,
                        item.id.unwrap_or(0)
                    );
                }
                !is_closed
            })
            .collect()
    };

    if dry_run {
        println!("[DRY-RUN] Would import {} items:", filtered_items.len());
        for item in &filtered_items {
            println!(
                "  - {} #{}: {}",
                item.work_item_type,
                item.id.unwrap_or(0),
                item.title
            );
        }
        return Ok(());
    }

    // Import to DevOps
    let pat = config.devops.pat.as_deref().context("DevOps PAT not set")?;
    let client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);

    for item in filtered_items {
        if let Some(id) = item.id {
            // Update existing work item
            println!("Updating {} #{}...", item.work_item_type, id);

            // Build patch operations
            let mut operations = Vec::new();
            for (key, val) in &item.fields {
                operations.push(serde_json::json!({
                    "op": "add",
                    "path": format!("/fields/{}", key),
                    "value": val
                }));
            }

            if !item.description.is_empty() {
                operations.push(serde_json::json!({
                    "op": "add",
                    "path": "/fields/System.Description",
                    "value": item.description
                }));
            }

            client.update_work_item(id, operations)?;
            println!("✓ Updated #{}", id);
        } else {
            // Create new work item
            println!("Creating new {} '{}'...", item.work_item_type, item.title);

            let mut fields = serde_json::Map::new();
            fields.insert(
                "System.WorkItemType".to_string(),
                serde_json::json!(item.work_item_type),
            );
            fields.insert("System.Title".to_string(), serde_json::json!(item.title));

            for (key, val) in &item.fields {
                fields.insert(key.clone(), serde_json::json!(val));
            }

            if !item.description.is_empty() {
                fields.insert(
                    "System.Description".to_string(),
                    serde_json::json!(item.description),
                );
            }

            let new_item = client.create_work_item(fields)?;
            println!("✓ Created #{}", new_item.id);
        }
    }

    Ok(())
}
