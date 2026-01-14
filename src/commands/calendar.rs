use crate::config::Config;
use crate::graph::auth::GraphAuthenticator;
use crate::graph::client::GraphClient;
use crate::graph::models::{CalendarEvent, DateTimeTimeZone};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use home::home_dir;

/// OAuth login command - initiate device code flow
pub async fn oauth_login(config: &Config) -> Result<()> {
    if config.graph.client_id.is_empty() {
        anyhow::bail!(
            "Graph API client_id not configured. \
             Add it to ~/.ao-no-out7ook/config.toml:\n\n\
             [graph]\n\
             client_id = \"your-application-client-id\"\n"
        );
    }

    let token_cache_path = home_dir()
        .context("Could not find home directory")?
        .join(".ao-no-out7ook")
        .join("tokens.json");

    let auth = GraphAuthenticator::new(config.graph.client_id.clone(), token_cache_path);
    auth.login().await?;

    Ok(())
}

/// OAuth status command - show authentication status
pub async fn oauth_status(config: &Config) -> Result<()> {
    let token_cache_path = home_dir()
        .context("Could not find home directory")?
        .join(".ao-no-out7ook")
        .join("tokens.json");

    if !token_cache_path.exists() {
        println!("❌ Not authenticated. Run 'task oauth login' first.");
        return Ok(());
    }

    let auth = GraphAuthenticator::new(config.graph.client_id.clone(), token_cache_path.clone());

    match auth.get_access_token().await {
        Ok(_) => {
            println!("✓ Authenticated with Microsoft Graph");
            println!("  Token cache: {}", token_cache_path.display());
        }
        Err(e) => {
            println!("❌ Authentication expired or invalid: {}", e);
            println!("  Run 'task oauth login' to re-authenticate.");
        }
    }

    Ok(())
}

/// List calendar events
pub async fn calendar_list(config: &Config, days: u32, work_item: Option<u32>) -> Result<()> {
    let token_cache_path = home_dir()
        .context("Could not find home directory")?
        .join(".ao-no-out7ook")
        .join("tokens.json");

    let auth = GraphAuthenticator::new(config.graph.client_id.clone(), token_cache_path);
    let client = GraphClient::new(auth);

    let start = Utc::now();
    let end = start + Duration::days(days as i64);

    let events = client.list_events(start, end).await?;

    if events.is_empty() {
        println!("No events found in the next {} days.", days);
        return Ok(());
    }

    println!("Calendar Events (next {} days):", days);
    println!(
        "{:<8} {:<50} {:<20} {:<12}",
        "Event ID", "Subject", "Start", "Duration"
    );
    println!("{}", "-".repeat(92));

    for event in &events {
        let event_id = event.id.as_deref().unwrap_or("N/A");
        let subject = &event.subject;

        // Skip if filtering by work_item and this event doesn't match
        if let Some(filter_id) = work_item {
            // Check if event has work_item_id in extended properties
            let has_match = event
                .extended_properties
                .as_ref()
                .and_then(|props| props.iter().find(|p| p.value == filter_id.to_string()))
                .is_some();

            if !has_match {
                continue;
            }
        }

        let start_time = &event.start.date_time;

        // Calculate duration (simplified)
        let duration = "N/A"; // TODO: parse start/end times

        println!(
            "{:<8} {:<50} {:<20} {:<12}",
            if event_id.len() > 8 {
                &event_id[..8]
            } else {
                event_id
            },
            if subject.len() > 48 {
                format!("{}...", &subject[..45])
            } else {
                subject.clone()
            },
            start_time,
            duration
        );
    }

    println!("\nTotal: {} events", events.len());

    Ok(())
}

/// Schedule Focus Block for work item
pub async fn calendar_schedule(
    config: &Config,
    work_item_id: u32,
    start_time: Option<String>,
    duration_mins: u32,
    custom_title: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let token_cache_path = home_dir()
        .context("Could not find home directory")?
        .join(".ao-no-out7ook")
        .join("tokens.json");

    let auth = GraphAuthenticator::new(config.graph.client_id.clone(), token_cache_path);
    let client = GraphClient::new(auth);

    // Get work item title from DevOps
    let pat = config.get_devops_pat()?;
    let devops_client = crate::devops::client::DevOpsClient::new(
        &pat,
        &config.devops.organization,
        &config.devops.project,
    );
    let work_item = devops_client.get_work_item(work_item_id)?;
    let work_item_title = work_item.get_title().unwrap_or("Unknown");

    // Parse start time or use now
    let start = if let Some(time_str) = start_time {
        chrono::DateTime::parse_from_rfc3339(&time_str)
            .context("Invalid start time format. Use ISO 8601: 2026-01-08T14:00:00-07:00")?
            .with_timezone(&Utc)
    } else {
        Utc::now()
    };

    let end = start + Duration::minutes(duration_mins as i64);

    let subject =
        custom_title.unwrap_or_else(|| format!("🎯 Focus: {} - {}", work_item_id, work_item_title));

    let event = CalendarEvent {
        id: None,
        subject: subject.clone(),
        start: DateTimeTimeZone::from_utc(start, "UTC"),
        end: DateTimeTimeZone::from_utc(end, "UTC"),
        body: None,
        categories: vec!["Focus Block".to_string()],
        extended_properties: None, // TODO: Add work_item_id
    };

    if dry_run {
        println!("--- DRY RUN: Calendar Schedule Preview ---");
        println!("  Subject: {}", subject);
        println!("  Start: {}", event.start.date_time);
        println!("  End: {}", event.end.date_time);
        println!("  Duration: {} minutes", duration_mins);
        println!("✓ [DRY RUN] Would create focus block");
    } else {
        let created = client.create_event(event).await?;

        println!("✓ Focus Block scheduled");
        println!("  Event ID: {}", created.id.as_deref().unwrap_or("N/A"));
        println!("  Subject: {}", created.subject);
        println!("  Start: {}", created.start.date_time);
        println!("  End: {}", created.end.date_time);
    }

    Ok(())
}

/// Delete calendar event
pub async fn calendar_delete(config: &Config, event_id: String) -> Result<()> {
    let token_cache_path = home_dir()
        .context("Could not find home directory")?
        .join(".ao-no-out7ook")
        .join("tokens.json");

    let auth = GraphAuthenticator::new(config.graph.client_id.clone(), token_cache_path);
    let client = GraphClient::new(auth);

    client.delete_event(&event_id).await?;
    println!("✓ Event {} deleted", event_id);

    Ok(())
}
