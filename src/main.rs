use anyhow::Result;
use ao_no_out7ook::commands;
use ao_no_out7ook::config;
use ao_no_out7ook::state::State;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "task")]
#[command(about = "SevenPace & Outlook Integration for DevOps")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start working on a task
    Start {
        #[arg(help = "DevOps Work Item ID")]
        id: u32,
        #[arg(long, help = "Preview without starting timer")]
        dry_run: bool,
        #[arg(long, help = "Auto-schedule Focus Block in calendar")]
        schedule_focus: bool,
    },
    /// Stop current task
    Stop {
        #[arg(long, help = "Preview without stopping timer")]
        dry_run: bool,
    },
    /// Switch to a new task
    Switch {
        #[arg(help = "New Work Item ID")]
        id: u32,
    },
    /// Show current task status
    Current,
    /// Check in after Focus Block (Continue/Blocked/Complete)
    Checkin,
    /// List configuration
    Config(ConfigArgs),

    /// List work items
    List {
        #[arg(long, help = "Filter by state (e.g. Active)")]
        state: Option<String>,
        #[arg(long, help = "Filter by assignee (email or 'me')")]
        assigned_to: Option<String>,
        #[arg(long, help = "Limit results", default_value = "50")]
        limit: u32,
    },

    /// Show work item details
    Show {
        #[arg(help = "Work Item ID")]
        id: u32,
    },

    /// Update work item state
    State {
        #[arg(help = "Work Item ID")]
        id: u32,
        #[arg(help = "New state (target)")]
        new_state: Option<String>,
        #[arg(long, help = "Preview changes without applying")]
        dry_run: bool,
    },
    /// Export work item to Markdown
    Export {
        #[arg(help = "Work Item ID")]
        id: u32,
        #[arg(long, help = "Output file path")]
        output: Option<std::path::PathBuf>,
    },

    /// Import work item from Markdown
    Import {
        #[arg(help = "Input file path")]
        file: std::path::PathBuf,
        #[arg(long, help = "Preview changes without applying")]
        dry_run: bool,
    },

    /// Manually log time to a work item
    LogTime {
        #[arg(help = "Work Item ID")]
        id: u32,
        #[arg(long, help = "Hours to log (decimal, e.g. 1.5)")]
        hours: f32,
        #[arg(long, help = "Optional comment")]
        comment: Option<String>,
        #[arg(long, help = "Preview without logging")]
        dry_run: bool,
    },

    /// Show recent worklogs
    Worklogs {
        #[arg(long, default_value = "7", help = "Number of days to show")]
        days: u32,
    },

    /// OAuth authentication for Microsoft Graph
    Oauth(OauthArgs),

    /// Calendar operations
    Calendar(CalendarArgs),
}

#[derive(Args)]
struct OauthArgs {
    #[command(subcommand)]
    action: OauthAction,
}

#[derive(Subcommand)]
enum OauthAction {
    /// Authenticate with Microsoft Graph (device code flow)
    Login,
    /// Show current authentication status
    Status,
}

#[derive(Args)]
struct CalendarArgs {
    #[command(subcommand)]
    action: CalendarAction,
}

#[derive(Subcommand)]
enum CalendarAction {
    /// List calendar events
    List {
        #[arg(long, default_value = "7", help = "Number of days to show")]
        days: u32,
        #[arg(long, help = "Filter by work item ID")]
        work_item: Option<u32>,
    },
    /// Schedule Focus Block for work item
    Schedule {
        #[arg(help = "Work Item ID")]
        id: u32,
        #[arg(long, help = "Start time (ISO 8601, e.g., 2026-01-08T14:00:00)")]
        start: Option<String>,
        #[arg(long, default_value = "45", help = "Duration in minutes")]
        duration: u32,
        #[arg(long, help = "Custom title (defaults to work item title)")]
        title: Option<String>,
    },
    /// Delete calendar event
    Delete {
        #[arg(help = "Event ID")]
        event_id: String,
    },
}

#[derive(Parser)]
struct ConfigArgs {
    #[command(subcommand)]
    action: ConfigAction,
}

#[derive(Subcommand)]
enum ConfigAction {
    List,
    Set { key: String, value: String },
    Get { key: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure state dir exists
    let config = config::load().unwrap_or_else(|_| {
        // Initial load might fail if file missing, that's okay for now
        // In real app, we'd prompt setup
        println!("Warning: No config found. Run 'task config set ...'");
        config::Config::default()
    });

    match &cli.command {
        Commands::Start {
            id,
            dry_run,
            schedule_focus,
        } => {
            commands::task::start(&config, *id, *dry_run, *schedule_focus)?;
        }
        Commands::Stop { dry_run } => {
            commands::task::stop(*dry_run)?;
        }
        Commands::Switch { id } => {
            commands::task::stop(false)?;
            // Switch doesn't auto-schedule Focus Block
            commands::task::start(&config, *id, false, false)?;
        }
        Commands::Current => {
            commands::task::current()?;
        }
        Commands::Checkin => {
            commands::checkin::checkin(&config)?;
        }
        Commands::Config(args) => match &args.action {
            ConfigAction::List => commands::config::list(&config)?,
            ConfigAction::Set { key, value } => commands::config::set(key, value)?,
            ConfigAction::Get { key } => commands::config::get(key, &config)?,
        },
        Commands::List {
            state,
            assigned_to,
            limit,
        } => {
            commands::devops::list(&config, state.clone(), assigned_to.clone(), Some(*limit))?;
        }
        Commands::Show { id } => {
            commands::devops::show(&config, *id)?;
        }
        Commands::State {
            id,
            new_state,
            dry_run,
        } => {
            commands::devops::state(&config, *id, new_state.clone(), *dry_run)?;
        }
        Commands::Export { id, output } => {
            commands::devops::export(&config, *id, output.clone())?;
        }
        Commands::Import { file, dry_run } => {
            commands::devops::import(&config, file.clone(), *dry_run)?;
        }
        Commands::LogTime {
            id,
            hours,
            comment,
            dry_run,
        } => {
            commands::pace::log_time(&config, *id, *hours, comment.clone(), *dry_run)?;
        }
        Commands::Worklogs { days } => {
            commands::pace::worklogs(&config, *days)?;
        }
        Commands::Oauth(oauth_args) => match &oauth_args.action {
            OauthAction::Login => {
                tokio::runtime::Runtime::new()?
                    .block_on(commands::calendar::oauth_login(&config))?;
            }
            OauthAction::Status => {
                tokio::runtime::Runtime::new()?
                    .block_on(commands::calendar::oauth_status(&config))?;
            }
        },
        Commands::Calendar(calendar_args) => match &calendar_args.action {
            CalendarAction::List { days, work_item } => {
                tokio::runtime::Runtime::new()?.block_on(commands::calendar::calendar_list(
                    &config, *days, *work_item,
                ))?;
            }
            CalendarAction::Schedule {
                id,
                start,
                duration,
                title,
            } => {
                tokio::runtime::Runtime::new()?.block_on(commands::calendar::calendar_schedule(
                    &config,
                    *id,
                    start.clone(),
                    *duration,
                    title.clone(),
                ))?;
            }
            CalendarAction::Delete { event_id } => {
                tokio::runtime::Runtime::new()?.block_on(commands::calendar::calendar_delete(
                    &config,
                    event_id.clone(),
                ))?;
            }
        },
    }

    Ok(())
}
