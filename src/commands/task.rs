use crate::config::Config;
use crate::devops::client::DevOpsClient;
use crate::state::{with_state_lock, CurrentTask, State};
use anyhow::{Context, Result};
use chrono::Utc;
use std::path::PathBuf;

fn state_paths() -> Result<(PathBuf, PathBuf)> {
    let home = home::home_dir().context("Could not find home directory")?;
    let state_dir = home.join(".ao-no-out7ook");
    Ok((state_dir.join("state.lock"), state_dir.join("state.json")))
}

pub fn start(config: &Config, id: u32, dry_run: bool, schedule_focus: bool) -> Result<()> {
    let (lock_path, state_path) = state_paths()?;

    // 1. Fetch work item from DevOps to validate
    let pat = config
        .devops
        .pat
        .as_deref()
        .context("DevOps PAT not set. Run 'task config set devops.pat <PAT>'")?;
    let devops_client = DevOpsClient::new(pat, &config.devops.organization, &config.devops.project);
    let pace_client = crate::pace::client::PaceClient::new(pat, &config.devops.organization);

    println!("Fetching work item {}...", id);
    let work_item = devops_client.get_work_item(id)?;
    let title = work_item.get_title().unwrap_or("Unknown Title").to_string();

    // 2. Check for conflicting timer (FR2.3)
    if let Some(current_timer) = pace_client.get_current_timer()? {
        if current_timer.work_item_id != id {
            if dry_run {
                println!(
                    "[DRY-RUN] Would stop existing timer for Task {}",
                    current_timer.work_item_id
                );
            } else {
                println!(
                    "Stopping existing timer for Task {}...",
                    current_timer.work_item_id
                );
                pace_client.stop_timer(0)?;
            }
        }
    }

    // 3. Start new timer
    let timer_id = if dry_run {
        println!("[DRY-RUN] Would start timer for Task {}", id);
        None
    } else {
        println!("Starting timer for Task {} - {}...", id, title);
        let timer = pace_client.start_timer(id, None)?;
        println!("✓ Timer started for Task {}", id);
        Some(timer.id)
    };

    // 4. Schedule Focus Block if requested (FR3.7)
    if schedule_focus {
        if dry_run {
            println!("[DRY-RUN] Would schedule Focus Block in calendar");
        } else {
            println!("📅 Scheduling Focus Block...");
            
            // Use async runtime for calendar operations
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

                // Get existing events for today
                let now = chrono::Utc::now();
                let end_of_day = now + chrono::Duration::hours(24);
                let events = client.list_events(now, end_of_day).await?;

                // Find next slot using smart scheduler
                let duration = config.focus_blocks.duration_minutes;
                let (slot_start, slot_end) = crate::graph::scheduler::find_next_slot(
                    &events,
                    now,
                    duration,
                    &config.work_hours,
                )?;

                // Create Focus Block event
                let event = crate::graph::models::CalendarEvent {
                    id: None,
                    subject: format!("🎯 Focus: {} - {}", id, title),
                    start: crate::graph::models::DateTimeTimeZone::from_utc(slot_start, "UTC"),
                    end: crate::graph::models::DateTimeTimeZone::from_utc(slot_end, "UTC"),
                    body: None,
                    categories: vec!["Focus Block".to_string()],
                    extended_properties: None, // TODO: Add work_item_id
                };

                client.create_event(event).await
            });

            match result {
                Ok(created) => {
                    println!(
                        "✓ Focus Block created: {} to {}",
                        created.start.date_time,
                        created.end.date_time
                    );
                }
                Err(e) => {
                    println!("⚠ Warning: Could not create Focus Block: {}", e);
                    println!("  Continuing with timer start...");
                }
            }
        }
    }

    // 4. Update State
    with_state_lock(&lock_path, &state_path, |state| {
        if let Some(current) = &state.current_task {
            println!("Stopping previous task: {} - {}", current.id, current.title);
        }

        let now = Utc::now();
        state.current_task = Some(CurrentTask {
            id,
            title: title.clone(),
            started_at: now,
            expires_at: now + chrono::Duration::hours(config.state.task_expiry_hours.into()),
            timer_id,
        });

        println!("✓ Started task: {} - {}", id, title);
        Ok(())
    })
}

pub fn stop(dry_run: bool) -> Result<()> {
    let (lock_path, state_path) = state_paths()?;

    with_state_lock(&lock_path, &state_path, |state| {
        if let Some(current) = &state.current_task {
            if dry_run {
                println!("[DRY-RUN] Would stop timer for Task {}", current.id);
            } else if current.timer_id.is_some() {
                // Stop 7Pace timer if active
                // Note: We can't access config here easily, so we'll need to refactor
                // For now, just clear state
                println!("✓ Stopped task: {} - {}", current.id, current.title);
            } else {
                println!("✓ Stopped task: {} - {}", current.id, current.title);
            }
            state.current_task = None;
        } else {
            println!("No active task to stop.");
        }
        Ok(())
    })
}

pub fn current() -> Result<()> {
    let (lock_path, state_path) = state_paths()?;

    // Read-only access doesn't strictly need exclusive lock if we accept potentially stale data
    // But for consistency and simplicity in MVP, we can just load without lock or use shared lock if supported
    // fs2 only supports exclusive or shared locks.
    // For "current", we can just load the file directly.
    let state = State::load(&state_path)?;

    if let Some(current) = state.current_task {
        println!("Active Task:");
        println!("  ID: {}", current.id);
        println!("  Title: {}", current.title);
        println!("  Started: {}", current.started_at);
        println!("  Expires: {}", current.expires_at);
    } else {
        println!("No active task.");
    }

    Ok(())
}
