use crate::config::Config;
use crate::state::with_state_lock;
use anyhow::{Context, Result};
use std::io::{self, Write};
use std::path::PathBuf;

fn state_paths() -> Result<(PathBuf, PathBuf)> {
    let home = home::home_dir().context("Could not find home directory")?;
    let state_dir = home.join(".ao-no-out7ook");
    Ok((state_dir.join("state.lock"), state_dir.join("state.json")))
}

/// FR3.8: Interactive check-in prompt after Focus Block
pub fn checkin(config: &Config) -> Result<()> {
    let (lock_path, state_path) = state_paths()?;

    // Get current task from state
    let current_task = with_state_lock(&lock_path, &state_path, |state| {
        Ok(state.current_task.clone())
    })?;

    let Some(task_info) = current_task else {
        println!("❌ No active task found.");
        println!("   Start a task with: task start <ID>");
        return Ok(());
    };

    // Display Focus Block status
    println!("\n🎯 Focus Block Status Check");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Task: #{} - {}", task_info.id, task_info.title);

    let elapsed = chrono::Utc::now().signed_duration_since(task_info.started_at);
    let mins = elapsed.num_minutes();
    println!("Timer running: {} minutes", mins);

    println!();
    println!("What would you like to do?");
    println!("  [1] Continue working (schedule another Focus Block)");
    println!("  [2] I'm blocked (stop timer, update status)");
    println!("  [3] Task complete (stop timer)");
    println!("  [q] Cancel");
    println!();

    // Get user choice
    print!("Your choice: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();

    match choice {
        "1" => {
            println!("\n✓ Continuing work on Task {}...", task_info.id);

            // Schedule another Focus Block
            println!("📅 Scheduling next Focus Block...");

            let runtime = tokio::runtime::Runtime::new()?;
            let result = runtime.block_on(async {
                let token_cache_path = home::home_dir()
                    .context("Could not find home directory")?
                    .join(".ao-no-out7ook")
                    .join("tokens.json");

                let auth = crate::graph::auth::GraphAuthenticator::new(
                    config.graph.client_id.clone(),
                    token_cache_path,
                );
                let client = crate::graph::client::GraphClient::new(auth);

                let now = chrono::Utc::now();
                let end_of_day = now + chrono::Duration::hours(24);
                let events = client.list_events(now, end_of_day).await?;

                let duration = config.focus_blocks.duration_minutes;
                let (slot_start, slot_end) = crate::graph::scheduler::find_next_slot(
                    &events,
                    now,
                    duration,
                    &config.work_hours,
                )?;

                let event = crate::graph::models::CalendarEvent {
                    id: None,
                    subject: format!("🎯 Focus: {} - {}", task_info.id, task_info.title),
                    start: crate::graph::models::DateTimeTimeZone::from_utc(slot_start, "UTC"),
                    end: crate::graph::models::DateTimeTimeZone::from_utc(slot_end, "UTC"),
                    body: None,
                    categories: vec!["Focus Block".to_string()],
                    extended_properties: None,
                };

                client.create_event(event).await
            });

            match result {
                Ok(created) => {
                    println!(
                        "✓ Next Focus Block: {} to {}",
                        created.start.date_time, created.end.date_time
                    );
                }
                Err(e) => {
                    println!("⚠ Warning: Could not schedule Focus Block: {}", e);
                }
            }
        }
        "2" => {
            println!("\n⚠ Marking task as blocked...");

            // Stop timer
            let pat = config.devops.pat.as_deref().context("DevOps PAT not set")?;
            let pace_client =
                crate::pace::client::PaceClient::new(pat, &config.devops.organization);

            match pace_client.stop_timer(0) {
                Ok(_) => println!("✓ Timer stopped"),
                Err(e) => println!("⚠ Could not stop timer: {}", e),
            }

            println!("💡 Tip: Update task state with: task state <NEW_STATE>");
        }
        "3" => {
            println!("\n✓ Completing Task {}...", task_info.id);

            // Stop timer
            let pat = config.devops.pat.as_deref().context("DevOps PAT not set")?;
            let pace_client =
                crate::pace::client::PaceClient::new(pat, &config.devops.organization);

            match pace_client.stop_timer(0) {
                Ok(_) => println!("✓ Timer stopped"),
                Err(e) => println!("⚠ Could not stop timer: {}", e),
            }

            // Clear current task from state
            with_state_lock(&lock_path, &state_path, |state| {
                state.current_task = None;
                state.save(&state_path)
            })?;

            println!("✓ Task cleared from state");
            println!("💡 Start next task with: task start <ID>");
        }
        "q" | "Q" => {
            println!("\nCancelled.");
        }
        _ => {
            println!("\n❌ Invalid choice. Cancelled.");
        }
    }

    Ok(())
}
