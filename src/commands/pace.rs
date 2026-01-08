use crate::config::Config;
use crate::pace::client::PaceClient;
use crate::pace::duration::format_duration;
use anyhow::{Context, Result};
use chrono::Utc;

/// FR2.5: Manually log time to a work item
pub fn log_time(
    config: &Config,
    work_item_id: u32,
    hours: f32,
    comment: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let pat = config
        .devops
        .pat
        .as_deref()
        .context("DevOps PAT not set. Run 'task config set devops.pat <PAT>'")?;
    let pace_client = PaceClient::new(pat, &config.devops.organization);

    let duration_secs = (hours * 3600.0) as u32;

    if dry_run {
        let formatted = format_duration(duration_secs);
        println!(
            "[DRY-RUN] Would log {:.2}h ({}) to Task {}",
            hours, formatted, work_item_id
        );
        if let Some(ref c) = comment {
            println!("[DRY-RUN] Comment: {}", c);
        }
    } else {
        let worklog = pace_client.create_worklog(work_item_id, duration_secs, comment)?;
        let formatted = format_duration(worklog.duration);
        println!(
            "✓ Logged {} to Task {} (Worklog ID: {})",
            formatted, work_item_id, worklog.id
        );
    }

    Ok(())
}

/// FR2.6: Fetch and display worklogs for reconciliation
pub fn worklogs(config: &Config, days: u32) -> Result<()> {
    let pat = config
        .devops
        .pat
        .as_deref()
        .context("DevOps PAT not set. Run 'task config set devops.pat <PAT>'")?;
    let pace_client = PaceClient::new(pat, &config.devops.organization);

    let end = Utc::now();
    let start = end - chrono::Duration::days(days as i64);

    let logs = pace_client.get_worklogs(start, end)?;

    if logs.is_empty() {
        println!("No worklogs found in the last {} days.", days);
        return Ok(());
    }

    println!("Worklogs (last {} days):", days);
    println!(
        "{:<8} {:<50} {:<12} {:<20}",
        "Task ID", "Comment", "Duration", "Date"
    );
    println!("{}", "-".repeat(92));

    for log in &logs {
        let duration_str = format_duration(log.duration);
        let comment_str = log.comment.as_deref().unwrap_or("(no comment)");
        let comment_display = if comment_str.len() > 48 {
            format!("{}...", &comment_str[0..45])
        } else {
            comment_str.to_string()
        };
        let date_str = log.timestamp.format("%Y-%m-%d %H:%M");

        println!(
            "{:<8} {:<50} {:<12} {:<20}",
            log.work_item_id, comment_display, duration_str, date_str
        );
    }

    // Summary
    let total_secs: u32 = logs.iter().map(|l| l.duration).sum();
    let total_str = format_duration(total_secs);
    println!("\nTotal: {} ({} entries)", total_str, logs.len());

    Ok(())
}
