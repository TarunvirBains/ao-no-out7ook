use anyhow::Result;
use ao_no_out7ook::OutputFormat;
use ao_no_out7ook::commands;
use ao_no_out7ook::config;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ao_no_out7ook")]
#[command(about = "SevenPace & Outlook Integration for DevOps")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start working on a task
    ///
    /// Creates a Focus Block in Outlook, starts a 7Pace timer, and sets the task as current.
    /// Useful for establishing context before beginning work.
    Start {
        #[arg(help = "DevOps Work Item ID (e.g., 12345)")]
        id: u32,
        #[arg(
            long,
            help = "Preview actions without starting timer or creating calendar event"
        )]
        dry_run: bool,
        #[arg(
            long,
            help = "Auto-schedule a Focus Block in the calendar for immediate work"
        )]
        schedule_focus: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Stop current task
    Stop {
        #[arg(long, help = "Preview without stopping timer")]
        dry_run: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Switch to a new task
    Switch {
        #[arg(help = "New Work Item ID")]
        id: u32,
    },
    /// Show current task status
    Current,
    /// Check in after Focus Block (Continue/Blocked/Complete)
    ///
    /// Interactive command to update task status after a focus session.
    /// Agents: Use 'task state' or 'task stop' for non-interactive updates instead.
    Checkin {
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// List configuration
    Config(ConfigArgs),

    /// List work items
    List {
        #[arg(long, help = "Filter by state (e.g. Active)")]
        state: Option<String>,
        #[arg(long, help = "Filter by assignee (email or 'me')")]
        assigned_to: Option<String>,
        #[arg(long, help = "Search by title text")]
        search: Option<String>,
        #[arg(long, help = "Filter by tag")]
        tags: Option<String>,
        #[arg(long, help = "Limit results", default_value = "50")]
        limit: u32,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Show work item details
    Show {
        #[arg(help = "Work Item ID")]
        id: u32,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
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

    /// Update work item fields (FR1.13)
    ///
    /// Update assigned-to, priority, or tags in a single operation.
    /// Multiple fields can be updated simultaneously with a single API call.
    Update {
        #[arg(help = "Work Item ID")]
        id: u32,
        #[arg(long, help = "Assign to user (email or 'me')")]
        assigned_to: Option<String>,
        #[arg(long, help = "Set priority (1-4)")]
        priority: Option<u32>,
        #[arg(long, help = "Set tags (comma-separated)")]
        tags: Option<String>,
        #[arg(long, help = "Preview changes without applying")]
        dry_run: bool,
    },

    /// Export work items to Markdown (Phase 4)
    ///
    /// Exports one or more work items to a hierarchical Markdown format.
    /// Use --hierarchy to include all children (features, stories, tasks).
    /// This is the preferred format for AI agents to read and reason about work scope.
    Export {
        #[arg(
            long,
            help = "Work item IDs to export (comma-separated)",
            value_delimiter = ','
        )]
        ids: Vec<u32>,
        #[arg(long, help = "Export entire hierarchy (parents and children)")]
        hierarchy: bool,
        #[arg(short, long, help = "Output file path")]
        output: std::path::PathBuf,
        #[arg(long, help = "Preview export without writing file")]
        dry_run: bool,
    },

    /// Import work items from Markdown (Phase 4)
    ///
    /// Parses Markdown and updates or creates work items in DevOps.
    /// To CREATE a new item, use ID #0 or omit the ID in the markdown header.
    /// To UPDATE, ensure the ID matches an existing work item.
    Import {
        #[arg(help = "Input markdown file path")]
        file: std::path::PathBuf,
        #[arg(long, help = "Preview changes (diff) without applying to DevOps")]
        dry_run: bool,
        #[arg(long, help = "Validate markdown structure only, do not contact DevOps")]
        validate: bool,
        #[arg(
            long,
            help = "Force import of completed/closed items (overrides skip_states config)"
        )]
        force: bool,
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

    /// Documentation and AI Workflows
    ///
    /// Outputs built-in guides and standard operating procedures (SOPs) for AI agents.
    /// Use this to learn how to combine commands for complex workflows.
    Doc {
        #[arg(help = "Topic to read (e.g., 'story-breakdown', 'list')")]
        topic: Option<String>,
    },

    /// Export current task context for AI Agents
    Context {
        #[arg(long, default_value = "llm", help = "Format (currently only 'llm')")]
        format: String,
    },

    /// Decompose a User Story into tasks via JSON input
    Decompose {
        #[arg(long, help = "Input JSON file path")]
        input: std::path::PathBuf,
        #[arg(long, help = "Preview changes without creating items")]
        dry_run: bool,
    },
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
    Status {
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
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
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
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
        #[arg(long, help = "Preview event without creating")]
        dry_run: bool,
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
    /// List all configuration values
    List,
    /// Set a configuration value
    Set {
        #[arg(help = "Config key (e.g. devops.pat, devops.organization, devops.skip_states)")]
        key: String,
        #[arg(help = "Value to set")]
        value: String,
    },
    /// Get a specific configuration value
    Get {
        #[arg(help = "Config key")]
        key: String,
    },
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
            format,
        } => {
            commands::task::start(&config, *id, *dry_run, *schedule_focus, *format)?;
        }
        Commands::Stop { dry_run, format } => {
            commands::task::stop(&config, *dry_run, *format)?;
        }
        Commands::Switch { id } => {
            commands::task::stop(&config, false, OutputFormat::Text)?;
            // Switch doesn't auto-schedule Focus Block
            commands::task::start(&config, *id, false, false, OutputFormat::Text)?;
        }
        Commands::Current => {
            commands::task::current(&config)?;
        }
        Commands::Checkin { format } => {
            commands::checkin::checkin(&config, *format)?;
        }
        Commands::Config(args) => match &args.action {
            ConfigAction::List => commands::config::list(&config)?,
            ConfigAction::Set { key, value } => commands::config::set(key, value)?,
            ConfigAction::Get { key } => commands::config::get(key, &config)?,
        },
        Commands::List {
            state,
            assigned_to,
            search,
            tags,
            limit,
            format,
        } => {
            commands::devops::list(
                &config,
                state.clone(),
                assigned_to.clone(),
                search.clone(),
                tags.clone(),
                Some(*limit),
                *format,
            )?;
        }
        Commands::Show { id, format } => {
            commands::devops::show(&config, *id, *format)?;
        }
        Commands::State {
            id,
            new_state,
            dry_run,
        } => {
            commands::devops::state(&config, *id, new_state.clone(), *dry_run)?;
        }
        Commands::Update {
            id,
            assigned_to,
            priority,
            tags,
            dry_run,
        } => {
            commands::devops::update(
                &config,
                *id,
                assigned_to.clone(),
                *priority,
                tags.clone(),
                *dry_run,
            )?;
        }
        Commands::Export {
            ids,
            hierarchy,
            output,
            dry_run,
        } => {
            commands::markdown::export(&config, ids.clone(), *hierarchy, output, *dry_run)?;
        }
        Commands::Import {
            file,
            dry_run,
            validate,
            force,
        } => {
            commands::markdown::import(&config, file, *dry_run, *validate, *force)?;
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
            OauthAction::Status { format } => {
                tokio::runtime::Runtime::new()?
                    .block_on(commands::calendar::oauth_status(&config, *format))?;
            }
        },
        Commands::Calendar(calendar_args) => match &calendar_args.action {
            CalendarAction::List {
                days,
                work_item,
                format,
            } => {
                tokio::runtime::Runtime::new()?.block_on(commands::calendar::calendar_list(
                    &config, *days, *work_item, *format,
                ))?;
            }
            CalendarAction::Schedule {
                id,
                start,
                duration,
                title,
                dry_run,
            } => {
                tokio::runtime::Runtime::new()?.block_on(commands::calendar::calendar_schedule(
                    &config,
                    *id,
                    start.clone(),
                    *duration,
                    title.clone(),
                    *dry_run,
                ))?;
            }
            CalendarAction::Delete { event_id } => {
                tokio::runtime::Runtime::new()?.block_on(commands::calendar::calendar_delete(
                    &config,
                    event_id.clone(),
                ))?;
            }
        },
        Commands::Doc { topic } => match topic.as_deref() {
            Some("story-breakdown") => {
                println!("{}", include_str!("../.agent/workflows/breakdown_story.md"));
            }
            Some("list") => {
                println!("Available documentation topics:");
                println!("- story-breakdown: SOP for decomposing User Stories into Tasks");
            }
            _ => {
                println!("Available documentation topics:");
                println!("- story-breakdown: SOP for decomposing User Stories into Tasks");
                println!("\nUsage: ao_no_out7ook doc <TOPIC>");
            }
        },
        Commands::Context { format } => {
            commands::agent::agent_context(&config, format)?;
        }
        Commands::Decompose { input, dry_run } => {
            commands::agent::agent_decompose(&config, input.clone(), *dry_run)?;
        }
    }

    Ok(())
}
