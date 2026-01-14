use crate::config::Config;
use crate::devops::client::DevOpsClient;
use anyhow::{Context, Result};

pub fn list(
    config: &Config,
    state: Option<String>,
    assigned_to: Option<String>,
    search: Option<String>,
    tags: Option<String>,
    limit: Option<u32>,
) -> Result<()> {
    let pat = config
        .devops
        .pat
        .as_deref()
        .context("DevOps PAT not set. Run 'task config set devops.pat <PAT>'")?;
    let mut client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);
    if let Some(url) = &config.devops.api_url {
        client = client.with_base_url(url);
    }

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

    // FR1.2: Search by title
    if let Some(term) = search {
        // Escape single quotes for SQL injection prevention
        let escaped = term.replace("'", "''");
        conditions.push(format!("[System.Title] CONTAINS '{}'", escaped));
    }

    // FR1.2: Filter by tags
    if let Some(tag) = tags {
        let escaped = tag.replace("'", "''");
        conditions.push(format!("[System.Tags] CONTAINS '{}'", escaped));
    }

    // FR1.15: Default sort by priority then changed date
    let order_clause = "ORDER BY [Microsoft.VSTS.Common.Priority] ASC, [System.ChangedDate] DESC";

    let query = format!(
        "SELECT [System.Id] FROM WorkItems WHERE {} {}",
        conditions.join(" AND "),
        order_clause
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
        let prio = item
            .fields
            .get("Microsoft.VSTS.Common.Priority")
            .map(|v| v.to_string())
            .unwrap_or(" ".to_string());

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

// Helper function for testing custom sort (will be used when we add --sort flag to CLI)
#[allow(dead_code)]
pub fn list_with_sort(
    config: &Config,
    state: Option<String>,
    assigned_to: Option<String>,
    search: Option<String>,
    tags: Option<String>,
    sort_by: &str,
    limit: Option<u32>,
) -> Result<()> {
    let pat = config.devops.pat.as_deref().context("DevOps PAT not set")?;
    let mut client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);
    if let Some(url) = &config.devops.api_url {
        client = client.with_base_url(url);
    }

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

    if let Some(term) = search {
        let escaped = term.replace("'", "''");
        conditions.push(format!("[System.Title] CONTAINS '{}'", escaped));
    }

    if let Some(tag) = tags {
        let escaped = tag.replace("'", "''");
        conditions.push(format!("[System.Tags] CONTAINS '{}'", escaped));
    }

    // FR1.15: Configurable sorting
    let order_clause = match sort_by {
        "priority" => "ORDER BY [Microsoft.VSTS.Common.Priority] ASC",
        "changed" => "ORDER BY [System.ChangedDate] DESC",
        "created" => "ORDER BY [System.CreatedDate] DESC",
        "title" => "ORDER BY [System.Title] ASC",
        _ => "ORDER BY [Microsoft.VSTS.Common.Priority] ASC, [System.ChangedDate] DESC",
    };

    let query = format!(
        "SELECT [System.Id] FROM WorkItems WHERE {} {}",
        conditions.join(" AND "),
        order_clause
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
        let prio = item
            .fields
            .get("Microsoft.VSTS.Common.Priority")
            .map(|v| v.to_string())
            .unwrap_or(" ".to_string());

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

    match crate::devops::hierarchy::build_tree(&client, id, 1) {
        Ok(node) => {
            println!("\nHierarchy:");
            crate::devops::hierarchy::print_tree(&node);
        }
        Err(_e) => {
            // Silently skip if hierarchy can't be built
        }
    }

    if let Some(relations) = &item.relations
        && !relations.is_empty()
    {
        println!("\nRelations:");
        for rel in relations {
            let target_id = rel.url.split('/').next_back().unwrap_or("?");
            println!("  - {}: #{}", rel.rel, target_id);
        }
    }

    println!("\nDescription:");
    if let Some(desc) = item
        .fields
        .get("System.Description")
        .and_then(|v| v.as_str())
    {
        println!("{}", desc);
    } else {
        println!("(No description)");
    }

    Ok(())
}

pub fn state(config: &Config, id: u32, new_state: Option<String>, dry_run: bool) -> Result<()> {
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
        let type_def = client.get_work_item_type(type_)?;
        let valid_states: Vec<String> = type_def.states.iter().map(|s| s.name.clone()).collect();

        if !valid_states.contains(&target) {
            println!(
                "Invalid state '{}'. Valid states for {}: {:?}",
                target, type_, valid_states
            );
            return Ok(());
        }

        let patch = serde_json::json!([
            {
                "op": "add",
                "path": "/fields/System.State",
                "value": target
            }
        ]);

        let patch_vec = patch.as_array().unwrap().clone();

        if dry_run {
            println!(
                "[DRY-RUN] Would update Task {} from {} to {}",
                id, current_state, target
            );
            println!(
                "[DRY-RUN] Patch operations: {}",
                serde_json::to_string_pretty(&patch)?
            );
        } else {
            client.update_work_item_with_rev(id, patch_vec, Some(item.rev))?;
            println!("✓ Task {} updated: {} -> {}", id, current_state, target);
        }
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

pub fn import(_config: &Config, _file: std::path::PathBuf, _dry_run: bool) -> Result<()> {
    anyhow::bail!(
        "Import command temporarily disabled during Phase 4 refactor. Use 'task export' for now."
    )
}

/// FR1.13: Update work item fields (assigned-to, priority, tags)
pub fn update(
    config: &Config,
    id: u32,
    assigned_to: Option<String>,
    priority: Option<u32>,
    tags: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let pat = config
        .devops
        .pat
        .as_deref()
        .context("DevOps PAT not set. Run 'task config set devops.pat <PAT>'")?;
    let mut client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);
    if let Some(url) = &config.devops.api_url {
        client = client.with_base_url(url);
    }

    // Fetch current work item to get rev
    let item = client.get_work_item(id)?;

    // Build JSON Patch operations
    let mut operations = Vec::new();

    if let Some(ref user) = assigned_to {
        operations.push(serde_json::json!({
            "op": "add",
            "path": "/fields/System.AssignedTo",
            "value": user
        }));
    }

    if let Some(p) = priority {
        // Validate priority range
        if !(1..=4).contains(&p) {
            anyhow::bail!("Priority must be between 1 and 4 (inclusive). Got: {}", p);
        }
        operations.push(serde_json::json!({
            "op": "add",
            "path": "/fields/Microsoft.VSTS.Common.Priority",
            "value": p
        }));
    }

    if let Some(ref tags_input) = tags {
        // Convert comma-separated to semicolon-separated (DevOps format)
        let formatted_tags = tags_input
            .split(',')
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .join("; ");
        operations.push(serde_json::json!({
            "op": "add",
            "path": "/fields/System.Tags",
            "value": formatted_tags
        }));
    }

    if operations.is_empty() {
        println!("No fields to update. Specify --assigned-to, --priority, or --tags");
        return Ok(());
    }

    if dry_run {
        println!("[DRY-RUN] Would update Task {} with:", id);
        println!("{}", serde_json::to_string_pretty(&operations)?);
        return Ok(());
    }

    client.update_work_item_with_rev(id, operations, Some(item.rev))?;

    println!("✓ Task {} updated successfully", id);
    if let Some(user) = assigned_to {
        println!("  - Assigned To: {}", user);
    }
    if let Some(p) = priority {
        println!("  - Priority: {}", p);
    }
    if let Some(t) = tags {
        println!("  - Tags: {}", t);
    }

    Ok(())
}
