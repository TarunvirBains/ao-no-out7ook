use crate::config::Config;
use crate::devops::client::DevOpsClient;
use anyhow::{Context, Result};
use clap::{Args, Subcommand};

pub fn list(
    config: &Config,
    state: Option<String>,
    assigned_to: Option<String>,
    limit: Option<u32>,
) -> Result<()> {
    let pat = config
        .devops
        .pat
        .as_deref()
        .context("DevOps PAT not set. Run 'task config set devops.pat <PAT>'")?;
    let client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);

    // Build WIQL
    // SELECT [System.Id], [System.Title], [System.State], [Microsoft.VSTS.Common.Priority], [System.ChangedDate]
    // FROM WorkItems
    // WHERE [System.TeamProject] = @project
    // AND [System.State] <> 'Removed'
    // ... filters

    let mut conditions = vec![
        "[System.TeamProject] = @project".to_string(),
        "[System.State] <> 'Removed'".to_string(),
    ];

    if let Some(s) = state {
        conditions.push(format!("[System.State] = '{}'", s));
    }

    if let Some(user) = assigned_to {
        if user == "me" {
            conditions.push("[System.AssignedTo] = @me".to_string());
        } else {
            conditions.push(format!("[System.AssignedTo] = '{}'", user));
        }
    }

    let query = format!(
        "SELECT [System.Id] FROM WorkItems WHERE {} ORDER BY [Microsoft.VSTS.Common.Priority] ASC, [System.ChangedDate] DESC",
        conditions.join(" AND ")
    );

    let wiql_resp = client.execute_wiql(&query)?;

    let ids: Vec<u32> = wiql_resp
        .work_items
        .iter()
        .take(limit.unwrap_or(50) as usize)
        .map(|r| r.id)
        .collect();

    if ids.is_empty() {
        println!("No work items found.");
        return Ok(());
    }

    let items = client.get_work_items_batch(&ids)?;

    println!(
        "{:<8} {:<50} {:<15} {:<5} {:<10}",
        "ID", "Title", "State", "Prio", "Type"
    );
    println!("{}", "-".repeat(90));

    for item in items {
        let id = item.id;
        let title = item.get_title().unwrap_or("No Title");
        let state = item.get_state().unwrap_or("Unknown");
        let type_ = item.get_type().unwrap_or("Unknown");
        // Priority is often a number or string
        let prio = item
            .fields
            .get("Microsoft.VSTS.Common.Priority")
            .map(|v| v.to_string())
            .unwrap_or(" ".to_string());

        // Truncate title
        let title = if title.len() > 48 {
            format!("{}...", &title[0..45])
        } else {
            title.to_string()
        };

        println!(
            "{:<8} {:<50} {:<15} {:<5} {:<10}",
            id, title, state, prio, type_
        );
    }

    Ok(())
}

pub fn show(config: &Config, id: u32) -> Result<()> {
    let pat = config
        .devops
        .pat
        .as_deref()
        .context("DevOps PAT not set. Run 'task config set devops.pat <PAT>'")?;
    let client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);
    let item = client.get_work_item(id)?;

    println!(
        "Task {}: {}",
        item.id,
        item.get_title().unwrap_or("No Title")
    );
    println!("Type: {}", item.get_type().unwrap_or("Unknown"));
    println!("State: {}", item.get_state().unwrap_or("Unknown"));
    println!(
        "Assigned To: {}",
        item.get_assigned_to().unwrap_or("Unassigned")
    );

    // Hierarchy visualization
    // We try to build tree, if fails (e.g. depth limit or whatever), we fallback or just ignore
    // For MVP, depth 1 (immediate children)
    match crate::devops::hierarchy::build_tree(&client, id, 1) {
        Ok(node) => {
            println!("\nHierarchy:");
            crate::devops::hierarchy::print_tree(&node);
        }
        Err(e) => {
            // If error building tree (e.g. child fetch fail), just warn silently or print
            // println!("(Could not build hierarchy: {})", e);
        }
    }

    if let Some(relations) = &item.relations {
        if !relations.is_empty() {
            println!("\nRelations:");
            for rel in relations {
                // Parse URL to get ID? "url": "https://.../workItems/123"
                let target_id = rel.url.split('/').last().unwrap_or("?");
                println!("  - {}: #{}", rel.rel, target_id);
            }
        }
    }

    println!("\nDescription:");
    // Description is often HTML in Azure DevOps.
    // For MVP just print raw or strip tags?
    // Let's print raw for now or try to get System.Description
    if let Some(desc) = item
        .fields
        .get("System.Description")
        .and_then(|v| v.as_str())
    {
        // Simple HTML to text could be done with a regex or crate, but let's just print
        println!("{}", desc);
    } else {
        println!("(No description)");
    }

    Ok(())
}

pub fn state(config: &Config, id: u32, new_state: Option<String>) -> Result<()> {
    let pat = config
        .devops
        .pat
        .as_deref()
        .context("DevOps PAT not set. Run 'task config set devops.pat <PAT>'")?;
    let client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);
    let item = client.get_work_item(id)?;
    let current_state = item.get_state().unwrap_or("Unknown");
    let type_ = item.get_type().context("Work item has no type")?;

    if let Some(target) = new_state {
        // Validate transition
        let type_def = client.get_work_item_type(type_)?;
        let valid_states: Vec<String> = type_def.states.iter().map(|s| s.name.clone()).collect();

        if !valid_states.contains(&target) {
            println!(
                "Invalid state '{}'. Valid states for {}: {:?}",
                target, type_, valid_states
            );
            return Ok(());
        }

        // Update
        // Patch operation
        let patch = serde_json::json!([
            {
                "op": "add",
                "path": "/fields/System.State",
                "value": target
            }
        ]);

        // Convert Value to Vec<Value>... wait patch is array.
        let patch_vec = patch.as_array().unwrap().clone();

        client.update_work_item(id, patch_vec)?;
        println!("✓ Task {} updated: {} -> {}", id, current_state, target);
    } else {
        println!("Current State: {}", current_state);
        let type_def = client.get_work_item_type(type_)?;
        println!("Valid States for {}:", type_);
        for s in type_def.states {
            println!("  - {}", s.name);
        }
    }

    Ok(())
}

pub fn export(config: &Config, id: u32, output: Option<std::path::PathBuf>) -> Result<()> {
    let pat = config
        .devops
        .pat
        .as_deref()
        .context("DevOps PAT not set. Run 'task config set devops.pat <PAT>'")?;
    let client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);

    let item = client.get_work_item(id)?;
    let md = crate::utils::markdown::to_markdown(&item);

    if let Some(path) = output {
        std::fs::write(&path, md).context("Failed to write markdown file")?;
        println!("Exported Task {} to {:?}", id, path);
    } else {
        println!("{}", md);
    }

    Ok(())
}

pub fn import(config: &Config, file: std::path::PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(&file).context("Failed to read markdown file")?;
    let parsed = crate::utils::markdown::from_markdown(&content)?;

    let pat = config
        .devops
        .pat
        .as_deref()
        .context("DevOps PAT not set. Run 'task config set devops.pat <PAT>'")?;
    let client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);

    if let Some(id) = parsed.id {
        // Update existing
        let mut ops = Vec::new();
        for (k, v) in parsed.fields {
            ops.push(serde_json::json!({
                "op": "add",
                "path": format!("/fields/{}", k),
                "value": v
            }));
        }

        if ops.is_empty() {
            println!("No fields to update for Task {}", id);
        } else {
            let patch_vec = ops;
            client.update_work_item(id, patch_vec)?;
            println!("✓ Updated Task {} from markdown", id);
        }

        if !parsed.description.is_empty() {
            let patch = vec![serde_json::json!({
                "op": "add",
                "path": "/fields/System.Description",
                "value": parsed.description
            })];
            client.update_work_item(id, patch)?;
            println!("✓ Updated Description for Task {}", id);
        }
    } else {
        println!("Importing new items (creation) is not yet supported in this version.");
    }

    Ok(())
}
