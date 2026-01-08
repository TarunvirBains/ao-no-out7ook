use crate::config::Config;
use crate::devops::client::DevOpsClient;
use crate::devops::models::WorkItem;
use crate::state::State;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct DecomposeInput {
    pub parent_id: u32,
    pub tasks: Vec<DecomposeTask>,
}

#[derive(Serialize, Deserialize)]
pub struct DecomposeTask {
    pub title: String,
    pub description: Option<String>,
    pub effort: Option<f32>,
    pub work_item_type: Option<String>, // e.g. "Task"
}

pub fn agent_context(config: &Config, format: &str) -> Result<()> {
    if format != "llm" {
        anyhow::bail!("Only 'llm' format is currently supported");
    }

    let (lock_path, state_path) = match state_paths() {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Failed to determine state paths: {}", e);
            return Ok(());
        }
    };

    // We don't necessarily need lock to read for context dump, but safer.
    // However, for speed, just loading state.json is fine.
    let state = State::load(&state_path)?;
    let current_task_id = match state.current_task.as_ref() {
        Some(task) => task.id,
        None => {
            println!("No active task.");
            return Ok(());
        }
    };

    let pat = config.devops.pat.as_deref().unwrap_or("");
    let client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);
    let work_item = client.get_work_item(current_task_id)?;

    println!("Current Task:");
    print_compact(&work_item);

    if let Some(parent_id) = work_item.get_parent_id() {
        println!("\nParent:");
        let parent = client.get_work_item(parent_id)?;
        print_compact(&parent);

        println!("\nSiblings:");
        if let Some(relations) = parent.relations {
            let siblings: Vec<_> = relations
                .iter()
                .filter(|r| r.rel == "System.LinkTypes.Hierarchy-Forward")
                .filter_map(|r| parse_id_from_url(&r.url))
                .filter(|&id| id != current_task_id)
                .collect();

            if siblings.is_empty() {
                println!("(None)");
            } else {
                for sibling_id in siblings {
                    // Fetch sibling details. In future, use batch API or WIQL for perf.
                    // For now, simple fetch is acceptable for typical <10 siblings.
                    if let Ok(sibling) = client.get_work_item(sibling_id) {
                        print_compact(&sibling);
                    }
                }
            }
        }
    } else {
        println!("\nParent: (None)");
    }

    Ok(())
}

fn state_paths() -> Result<(PathBuf, PathBuf)> {
    let home = home::home_dir().context("Could not find home directory")?;
    // We stay consistent with .ao_no_out7ook directory if we migrated, but strictly speaking
    // the previous code used ".ao-no-out7ook". I'll stick to that for compatibility
    // since I haven't migrated the config folder yet.
    // Actually, I renamed the binary but not the config folder in code.
    // Ideally I should rename the folder too, but that's a data migration.
    // Providing legacy folder logic:
    let state_dir = home.join(".ao-no-out7ook");
    Ok((state_dir.join("state.lock"), state_dir.join("state.json")))
}

pub fn agent_decompose(config: &Config, input_path: PathBuf, dry_run: bool) -> Result<()> {
    let content = fs::read_to_string(&input_path)
        .with_context(|| format!("Failed to read input file: {:?}", input_path))?;

    let input: DecomposeInput =
        serde_json::from_str(&content).context("Failed to parse decomposition JSON")?;

    let pat = config.devops.pat.as_deref().unwrap_or("");
    let client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);

    // Validate parent
    let parent = client
        .get_work_item(input.parent_id)
        .context("Parent work item not found")?;

    println!(
        "Decomposing under Parent: #{} {}",
        parent.id,
        parent.get_title().unwrap_or("?")
    );

    for task in input.tasks {
        let wi_type = task.work_item_type.as_deref().unwrap_or("Task");
        println!(
            "{} Creating '{}': {}",
            if dry_run { "[DRY-RUN]" } else { "[CREATE]" },
            wi_type,
            task.title
        );

        if !dry_run {
            // Build fields map
            let mut fields = serde_json::Map::new();
            fields.insert(
                "System.Title".to_string(),
                serde_json::Value::String(task.title.clone()),
            );
            fields.insert(
                "System.WorkItemType".to_string(),
                serde_json::Value::String(wi_type.to_string()),
            );

            if let Some(desc) = &task.description {
                fields.insert(
                    "System.Description".to_string(),
                    serde_json::Value::String(desc.clone()),
                );
            }
            if let Some(effort) = task.effort {
                fields.insert(
                    "Microsoft.VSTS.Scheduling.Effort".to_string(),
                    serde_json::json!(effort),
                );
            }

            match client.create_work_item(fields) {
                Ok(new_wi) => {
                    println!("  -> Created #{}", new_wi.id);
                    // Link to parent
                    let parent_url = &parent.url;
                    let link_op = serde_json::json!({
                        "op": "add",
                        "path": "/relations/-",
                        "value": {
                            "rel": "System.LinkTypes.Hierarchy-Reverse",
                            "url": parent_url,
                             "attributes": {
                                "comment": "Created via ao_no_out7ook decompose"
                            }
                        }
                    });
                    // Using update_work_item which takes Vec<Value> (operations)
                    if let Err(e) = client.update_work_item(new_wi.id, vec![link_op]) {
                        eprintln!("  -> Failed to link parent: {}", e);
                    }
                }
                Err(e) => eprintln!("  -> Failed: {}", e),
            }
        }
    }

    Ok(())
}

fn print_compact(wi: &WorkItem) {
    println!(
        "- #{} {} [{}] ({})",
        wi.id,
        wi.get_title().unwrap_or("?"),
        wi.get_state().unwrap_or("?"),
        wi.get_type().unwrap_or("?")
    );
}

fn parse_id_from_url(url: &str) -> Option<u32> {
    url.split('/').next_back()?.parse().ok()
}
