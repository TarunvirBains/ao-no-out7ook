# CLI Specification

> **Complete command reference for the DevOps CLI tool (Phase 4).**
>
> **Note:** The CLI binary is named `ano7`.

---

## **Global Options**

All commands support these global flags:

```bash
--help, -h           Show help for command
--version, -V        Show version information
```

---

## **Command Structure**

```
ano7 <COMMAND> [OPTIONS] [ARGS]
```

---

## **Commands**

### **Task Management**

#### `task start <ID>`

Start working on a task. Optionally auto-schedules a Focus Block.

**Arguments:**
- `<ID>` - DevOps Work Item ID

**Options:**
- `--dry-run` - Preview without starting timer
- `--schedule-focus` - Auto-schedule Focus Block in calendar at next available slot

**Examples:**
```bash
ano7 start 12345
ano7 start 12345 --schedule-focus
```

---

#### `task stop`

Stop current task and stop the active timer.

**Options:**
- `--dry-run` - Preview without stopping timer

**Examples:**
```bash
ano7 stop
```

---

#### `task switch <ID>`

Switch to a different task. Stops current task/timer and starts new one.

**Arguments:**
- `<ID>` - New Work Item ID

**Examples:**
```bash
ano7 switch 67890
```

---

#### `task current`

Show currently active task and timer status.

**Examples:**
```bash
ano7 current
```

---

#### `task checkin`

Check in after a Focus Block (interactive). Prompts to Continue, Blocked, or Complete.

**Examples:**
```bash
ano7 checkin
```

---

### **Work Item Operations**

#### `task list [OPTIONS]`

List work items with filtering.

**Options:**
- `--state <STATE>` - Filter by state (e.g. Active)
- `--assigned-to <USER>` - Filter by assignee (email or 'me')
- `--limit <N>` - Limit results (default: 50)

**Examples:**
```bash
ano7 list --state Active
ano7 list --assigned-to me
```

---

#### `task show <ID>`

Show detailed information about a work item.

**Arguments:**
- `<ID>` - Work Item ID

**Examples:**
```bash
ano7 show 12345
```

---

#### `task state <ID> [NEW_STATE]`

Update work item state.

**Arguments:**
- `<ID>` - Work Item ID
- `[NEW_STATE]` - Optional: New state (target)

**Options:**
- `--dry-run` - Preview changes without applying

**Examples:**
```bash
ano7 state 12345 Active
ano7 state 12345 Resolved --dry-run
```

---

### **Markdown Operations (Phase 4)**

#### `task export`

Export work items to Markdown.

**Options:**
- `--ids <IDS>` - Work item IDs to export (comma-separated)
- `--hierarchy` - Export entire hierarchy (parents/children)
- `-o, --output <PATH>` - Output file path

**Examples:**
```bash
ano7 export --ids 123 -o work.md
ano7 export --ids 123 --hierarchy -o epic-tree.md
```

#### `task import <FILE>`

Import work items from Markdown (creates or updates).

**Arguments:**
- `<FILE>` - Input markdown file path

**Options:**
- `--dry-run` - Preview changes without applying
- `--validate` - Validate only, don't import
- `--force` - Force import of completed/closed items (overrides `devops.skip_states`)

**Examples:**
```bash
ano7 import work.md
ano7 import work.md --dry-run
ano7 import work.md --validate
ano7 import work.md --force
```

---

### **7Pace / Time Tracking**

#### `task log-time <ID>`

Manually log time to a work item.

**Arguments:**
- `<ID>` - Work Item ID

**Options:**
- `--hours <HOURS>` - Hours to log (decimal, e.g. 1.5)
- `--comment <TEXT>` - Optional comment
- `--dry-run` - Preview without logging

**Examples:**
```bash
ano7 log-time 12345 --hours 1.5 --comment "Code review"
```

#### `task worklogs`

Show recent worklogs.

**Options:**
- `--days <N>` - Number of days to show (default: 7)

**Examples:**
```bash
ano7 worklogs
ano7 worklogs --days 14
```

---

### **Calendar & OAuth**

#### `task oauth <ACTION>`

Manage Microsoft Graph OAuth authentication.

**Actions:**
- `login` - Authenticate with Microsoft Graph (device code flow)
- `status` - Show current authentication status

**Examples:**
```bash
ano7 oauth login
ano7 oauth status
```

#### `task calendar <ACTION>`

Calendar operations.

**Actions:**
- `list` - List calendar events
  - `--days <N>` - Number of days to show (default: 7)
  - `--work-item <ID>` - Filter by work item ID
- `schedule <ID>` - Schedule Focus Block for work item
  - `--start <ISO8601>` - Start time
  - `--duration <MIN>` - Duration in minutes (default: 45)
  - `--title <TEXT>` - Custom title
- `delete <EVENT_ID>` - Delete calendar event

**Examples:**
```bash
ano7 calendar list
ano7 calendar schedule 12345 --duration 60
ano7 calendar delete "event-id-123"
```

---

### **AI Integration & Documentation**

#### `task doc [TOPIC]`

Output built-in manuals and workflows. This allows AI agents to "read the manual" directly from the binary.

**Arguments:**
- `[TOPIC]` - Topic to display (e.g. `story-breakdown`). If omitted, lists available topics.

**Examples:**
```bash
ano7 doc
ano7 doc story-breakdown
```

---

### **Configuration**

#### `task config <ACTION>`

Manage configuration.

**Actions:**
- `list` - List all configuration values
- `set <KEY> <VALUE>` - Set a configuration value
- `get <KEY>` - Get a specific configuration value

**Examples:**
```bash
ano7 config list
ano7 config set devops.organization "myorg"
ano7 config get work_hours.start
```

---

## **Configuration Reference**

Location: `~/.ao-no-out7ook/config.toml`

```toml
[devops]
organization = "..."
project = "..."
pat = "..." (optional)
skip_states = ["Completed", "Resolved", "Closed", "Removed"] # Phase 4

[graph]
client_id = "..."
tenant_id = "..."

[work_hours]
start = "09:00"
end = "17:00"

[focus_blocks]
duration_minutes = 45
buffer_minutes = 15

[state]
storage_path = "..."
```

---

## **Exit Codes**

```
0   Success
1   General check failure / Error
```

---

## **Command Structure**

```
ano7 <COMMAND> [OPTIONS] [ARGS]
```

---

## **Commands**

### **Task Management**

#### `task start <WORK_ITEM_ID>`

Start working on a task. Creates Focus Block, starts 7Pace timer, sets as current task.

**Arguments:**
- `<WORK_ITEM_ID>` - Azure DevOps work item ID

**Options:**
- `--no-timer` - Skip starting 7Pace timer
- `--no-calendar` - Skip creating Focus Block
- `--duration <MINUTES>` - Override Focus Block duration (default: from config)

**Examples:**
```bash
ano7 start 12345
ano7 start 12345 --no-timer
ano7 start 12345 --duration 30
```

**Output:**
```
✓ Timer started for Task 12345
✓ Focus Block created: 9:15 AM - 10:00 AM
🎯 Currently working on: Implement login feature
```

---

#### `task stop`

Stop current task. Stops 7Pace timer, logs time.

**Options:**
- `--no-log` - Stop timer without logging time

**Examples:**
```bash
ano7 stop
ano7 stop --no-log
```

**Output:**
```
✓ Timer stopped for Task 12345
⏱️  Logged 1h 23m to Task 12345
```

---

#### `task switch <WORK_ITEM_ID>`

Switch to a different task. Stops current timer, starts new one.

**Arguments:**
- `<WORK_ITEM_ID>` - Target work item ID

**Examples:**
```bash
ano7 switch 67890
```

**Output:**
```
✓ Stopped Task 12345 (logged 1h 23m)
✓ Started Task 67890
```

---

#### `task current`

Show currently active task.

**Options:**
- `--json` - Output in JSON format

**Examples:**
```bash
ano7 current
ano7 current --json
```

**Output:**
```
🎯 Task 12345: Implement login feature
⏱️  Timer running: 42 minutes
📅 Focus Block ends at 10:00 AM
📊 State: Active | Priority: 2
```

---

### **Work Item Operations**

#### `task list [OPTIONS]`

List work items with filtering and sorting.

**Options:**
- `--state <STATE>` - Filter by state (Active, New, Resolved, etc.)
- `--assigned-to <USER>` - Filter by assignee (use "me" for current user)
- `--type <TYPE>` - Filter by type (User Story, Task, Bug, etc.)
- `--sort <FIELD>` - Sort by field (urgency, priority, created, updated)
- `--limit <N>` - Limit results (default: 50)
- `--json` - Output in JSON format

**Examples:**
```bash
ano7 list
ano7 list --state Active
ano7 list --assigned-to me --sort urgency
ano7 list --type "User Story" --limit 10
```

**Output:**
```
ID      Title                          State    Priority  Updated
12345   Implement login feature        Active   2         2h ago
67890   Add user profile page          New      1         1d ago
54321   Fix navigation bug             Active   3         3h ago
```

---

#### `task show <WORK_ITEM_ID>`

Show detailed information about a work item.

**Arguments:**
- `<WORK_ITEM_ID>` - Work item ID

**Options:**
- `--json` - Output in JSON format

**Examples:**
```bash
ano7 show 12345
ano7 show 12345 --json
```

**Output:**
```
Task 12345: Implement login feature
Type: User Story
State: Active
Assigned To: John Doe
Priority: 2
Created: 2026-01-05
Updated: 2026-01-07

Description:
As a user, I want to log in to the application so that I can access my profile.

Acceptance Criteria:
- User can enter email and password
- Invalid credentials show error message
- Successful login redirects to dashboard
```

---

#### `task state <WORK_ITEM_ID> [NEW_STATE]`

View or update work item state.

**Arguments:**
- `<WORK_ITEM_ID>` - Work item ID
- `[NEW_STATE]` - Optional: New state to transition to

**Examples:**
```bash
ano7 state 12345               # Show valid transitions
ano7 state 12345 Active        # Update to Active
ano7 state 12345 Resolved      # Update to Resolved
```

**Output (no state provided):**
```
Current State: Active
Valid Transitions:
  - Resolved
  - Removed
```

**Output (state update):**
```
✓ Task 12345 updated: Active → Resolved
```

---

#### `task sync`

Sync local cache with Azure DevOps.

**Options:**
- `--force` - Force full sync, ignore cache timestamps

**Examples:**
```bash
ano7 sync
ano7 sync --force
```

**Output:**
```
Syncing with Azure DevOps...
✓ Updated 23 work items
✓ Fetched 2 new schemas
```

---

### **Calendar Operations**

#### `task checkin <ACTION>`

Respond to Focus Block check-in (typically called by Outlook reminder).

**Arguments:**
- `<ACTION>` - One of: continue, blocked, stop

**Options:**
- `--task-id <ID>` - Specify task ID (default: current task)

**Examples:**
```bash
ano7 checkin continue    # Create next Focus Block
ano7 checkin blocked     # Mark task as blocked, stop timer
ano7 checkin stop        # Stop working, log time
```

**Output:**
```
✓ Next Focus Block created: 10:15 AM - 11:00 AM
```

---

### **Configuration**

#### `task config list`

Show all configuration values.

**Examples:**
```bash
ano7 config list
```

**Output:**
```
devops.organization = "myorg"
devops.project = "MyProject"
work_hours.start = "08:30"
work_hours.end = "17:00"
focus_blocks.duration_minutes = 45
```

---

#### `task config set <KEY> <VALUE>`

Set configuration value.

**Arguments:**
- `<KEY>` - Configuration key (dot-notation)
- `<VALUE>` - Value to set

**Examples:**
```bash
ano7 config set work_hours.start "09:00"
ano7 config set focus_blocks.duration_minutes 30
ano7 config set devops.organization "neworg"
```

**Output:**
```
✓ Configuration updated: work_hours.start = "09:00"
```

---

#### `task config get <KEY>`

Get specific configuration value.

**Arguments:**
- `<KEY>` - Configuration key

**Examples:**
```bash
ano7 config get work_hours.start
```

**Output:**
```
08:30
```

---

### **Authentication**

#### `task auth outlook`

Authenticate with Microsoft Outlook (one-time setup).

**Examples:**
```bash
ano7 auth outlook
```

**Output:**
```
Visit: https://microsoft.com/devicelogin
Enter code: ABC-DEF-GHI

[User authenticates in browser]

✓ Authenticated! Refresh token stored securely.
You won't need to do this again.
```

---

### **AI Agent Integration**

#### `task context [OPTIONS]`

Export current task context for AI assistants.

**Options:**
- `--format <FORMAT>` - Output format: llm, json, markdown (default: llm)
- `--include-children` - Include child work items
- `--include-parent` - Include parent work item

**Examples:**
```bash
ano7 context --format llm
ano7 context --format json --include-children
```

**Output (llm format):**
```
Current Task: #12345 "Implement login feature"
Type: User Story
State: Active
Parent: #10000 "User Authentication Epic"

Description:
As a user, I want to log in...

Child Tasks:
- #12346: Create login form UI
- #12347: Implement authentication logic
```

---

#### `task schema [WORK_ITEM_TYPE]`

Show DevOps schema for work item types.

**Arguments:**
- `[WORK_ITEM_TYPE]` - Optional: Specific type (default: current task's type)

**Examples:**
```bash
ano7 schema
ano7 schema "User Story"
ano7 schema Bug
```

**Output:**
```
Work Item Type: User Story

States:
- New
- Active
- Resolved
- Closed
- Removed

Valid Transitions:
New → [Active, Removed]
Active → [Resolved, Removed]
Resolved → [Closed, Active]
Closed → [Active]

Required Fields:
- System.Title (string)
- System.State (string)
- System.AssignedTo (identity)
```

---

#### `task decompose --input <FILE>`

Bulk create child work items from JSON file (typically AI-generated).

**Options:**
- `--input <FILE>` - Path to JSON file with work item definitions
- `--dry-run` - Preview changes without creating items
- `--parent <ID>` - Parent work item ID

**Examples:**
```bash
ano7 decompose --input tasks.json --parent 12345 --dry-run
ano7 decompose --input tasks.json --parent 12345
```

**Input File Format (tasks.json):**
```json
{
  "tasks": [
    {
      "title": "Create login form UI",
      "type": "Task",
      "description": "Create React component for login form",
      "assignedTo": "john@example.com"
    },
    {
      "title": "Implement auth logic",
      "type": "Task",
      "description": "Add authentication with JWT"
    }
  ]
}
```

**Output:**
```
Preview: Would create 2 child tasks under #12345

✓ Created Task #12346: Create login form UI
✓ Created Task #12347: Implement auth logic

Summary: 2 tasks created successfully
```

---

## **Exit Codes**

```
0   Success
1   General error
2   Configuration error
3   Authentication error
4   API error (DevOps, 7Pace, or Graph)
5   Invalid arguments
6   File I/O error
```

---

## **Configuration File Reference**

Location: `~/.ao-no-out7ook/config.toml`

```toml
[devops]
organization = "myorg"           # Azure DevOps organization
project = "MyProject"            # Project name

[7pace]
# (Uses same PAT as DevOps - no additional config needed)

[outlook]
# (Configured via `task auth outlook` - stored in keyring)

[focus_blocks]
duration_minutes = 45            # Default Focus Block duration
interval_minutes = 15            # Scheduling interval granularity
teams_presence_sync = true       # Set Teams status during Focus

[work_hours]
start = "08:30"                  # Work day start time
end = "17:00"                    # Work day end time
timezone = "America/Los_Angeles" # IANA timezone

[state]
task_expiry_hours = 24          # Clear stale tasks after N hours
```

---

## **State File Reference**

Location: `~/.ao-no-out7ook/state.json`

**Do not manually edit this file while the CLI is running.**

```json
{
  "version": "1.0.0",
  "current_task": {
    "id": 12345,
    "title": "Implement login feature",
    "started_at": "2026-01-07T09:00:00Z",
    "expires_at": "2026-01-08T09:00:00Z",
    "timer_id": "7pace-timer-uuid"
  },
  "last_sync": {
    "devops": "2026-01-07T08:30:00Z",
    "7pace": "2026-01-07T08:30:00Z",
    "calendar": "2026-01-07T08:30:00Z"
  }
}
```

---

## **Environment Variables**

```bash
DEVOPS_PAT              # Override Azure DevOps PAT from config
DEVOPS_CLI_CONFIG       # Override config file location
DEVOPS_CLI_STATE_DIR    # Override state directory (~/.ao-no-out7ook)
RUST_LOG                # Set log level (error, warn, info, debug, trace)
```

**Example:**
```bash
export DEVOPS_PAT="your-pat-here"
export RUST_LOG=debug
ano7 start 12345
```

---

## **Troubleshooting**

### **"Authentication failed" error**

```bash
# Verify PAT is set correctly
ano7 config get devops.pat

# Re-set PAT
ano7 config set devops.pat "new-pat"

# For Outlook auth issues
ano7 auth outlook  # Re-authenticate
```

### **"State file locked" error**

Another CLI instance is running. Wait for it to finish or:
```bash
rm ~/.ao-no-out7ook/state.lock  # Only if no other instance is running
```

### **"Task expired" message**

Task was inactive for more than `task_expiry_hours` (default: 24h). This is expected behavior.

```bash
# Check current task
ano7 current

# Start a new task
ano7 start <ID>
```

---

## **Advanced Usage**

### **Scripting & Automation**

**Check if task is active before running tests:**
```bash
if task current --json | jq -e '.id' > /dev/null; then
  echo "Task is active"
  npm test
else
  echo "No active task - skipping tests"
fi
```

**Auto-start timer on git branch checkout:**
```bash
# .git/hooks/post-checkout
#!/bin/bash
BRANCH=$(git branch --show-current)
if [[ $BRANCH =~ ^task/([0-9]+) ]]; then
  TASK_ID="${BASH_REMATCH[1]}"
  task start "$TASK_ID"
fi
```

### **JSON Output Parsing**

Most commands support `--json` for machine-readable output:

```bash
ano7 list --state Active --json | jq '.[] | select(.priority == 1)'
ano7 current --json | jq -r '.title'
```
