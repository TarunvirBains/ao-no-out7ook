# Architecture: DevOps/7pace/Outlook CLI

> **Design Philosophy:** Single standalone binary, zero external dependencies, file-based state, simple to deploy and use.

---

## **Core Principles**

1. **Single Binary** - Compile to one executable, no runtime dependencies
2. **File-Based State** - No databases, Redis, or daemons required
3. **API-Driven** - All integrations via REST APIs (Azure DevOps, 7pace, Microsoft Graph)
4. **Structured Storage** - JSON as canonical format for all data
5. **Concurrent-Safe** - File locking for multi-process safety

---

## **Technology Stack**

### **Language & Tooling**
- **Rust** (stable channel)
- **Cargo** for build/dependency management

### **Key Dependencies**
```toml
[dependencies]
# CLI Framework
clap = { version = "4", features = ["derive", "env"] }

# HTTP Client
reqwest = { version = "0.11", features = ["json", "blocking"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Configuration
config = "0.14"
toml = "0.8"

# File Locking
fs2 = "0.4"  # Cross-platform file locking

# Time/Date Handling
chrono = "0.4"

# Error Handling
anyhow = "1"
thiserror = "1"

# Calendar/iCal (if needed for Outlook format parsing)
ical = "0.10"
```

---

## **File-Based State Architecture**

### **State Directory Structure**
```
~/.ao-no-out7ook/
├── config.toml              # User configuration
├── state.json               # Current active task, last sync times
├── state.lock               # Lock file for concurrent access
├── cache/
│   ├── work_items.json      # Cached work items from DevOps
│   ├── schemas.json         # DevOps work item type schemas
│   └── time_entries.json    # Cached 7pace entries
└── logs/
    └── cli.log              # Optional debug logs
```

### **State File Format (`state.json`)**
```json
{
  "version": "1.0.0",
  "current_task": {
    "id": 12345,
    "title": "Implement login feature",
    "started_at": "2026-01-07T17:00:00Z",
    "expires_at": "2026-01-08T17:00:00Z",
    "timer_id": "7pace-timer-uuid"
  },
  "last_sync": {
    "devops": "2026-01-07T16:30:00Z",
    "7pace": "2026-01-07T16:30:00Z",
    "calendar": "2026-01-07T16:30:00Z"
  },
  "work_hours": {
    "start": "08:30",
    "end": "17:00"
  }
}
```

**State Expiry Logic:**
- `expires_at` defaults to 24 hours from `started_at` (configurable)
- On any CLI command, check if `current_task.expires_at < now()`
- If expired:
  - Clear `current_task` to `null`
  - Stop any running 7pace timers
  - Log: "Task 12345 expired after 24 hours of inactivity"
- User can configure expiry duration in `config.toml`

### **Config File Format (`config.toml`)**
```toml
[devops]
organization = "myorg"
project = "MyProject"
pat = "encrypted_or_keyring_reference"

[7pace]
api_key = "encrypted_or_keyring_reference"
base_url = "https://api.7pace.com"

[outlook]
tenant_id = "..."
client_id = "..."
# Use OAuth device flow or delegated auth

[focus_blocks]
duration_minutes = 45
interval_minutes = 15
teams_presence_sync = true

[work_hours]
start = "08:30"
end = "17:00"
timezone = "America/Los_Angeles"

[state]
task_expiry_hours = 24  # Clear current task after this many hours of inactivity
```

### **File Locking Strategy**

```rust
use fs2::FileExt;
use std::fs::OpenOptions;

fn with_state_lock<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut State) -> Result<R>,
{
    let lock_path = state_dir().join("state.lock");
    let lock_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&lock_path)?;
    
    // Exclusive lock (blocks other processes)
    lock_file.lock_exclusive()?;
    
    let mut state = State::load()?;
    let result = f(&mut state)?;
    state.save()?;
    
    lock_file.unlock()?;
    Ok(result)
}
```

---

## **Module Architecture**

```
src/
├── main.rs              # CLI entry point, arg parsing
├── lib.rs               # Core library exports
├── state/
│   ├── mod.rs           # State management
│   ├── lock.rs          # File locking utilities
│   └── config.rs        # Config loading/validation
├── api/
│   ├── mod.rs
│   ├── devops.rs        # Azure DevOps API client
│   ├── sevenpace.rs     # 7pace API client
│   └── outlook.rs       # Microsoft Graph API client
├── models/
│   ├── mod.rs
│   ├── work_item.rs     # WorkItem struct
│   ├── time_entry.rs    # TimeEntry struct
│   └── calendar.rs      # CalendarEvent struct
├── commands/
│   ├── mod.rs
│   ├── start.rs         # `task start <id>`
│   ├── switch.rs        # `task switch <id>`
│   ├── state_update.rs  # `task state <id> <state>`
│   ├── checkin.rs       # `task checkin --continue|--blocked|--stop`
│   ├── list.rs          # `task list`
│   └── sync.rs          # `task sync`
├── scheduler/
│   ├── mod.rs
│   ├── focus_blocks.rs  # Focus Block scheduling logic
│   └── calendar.rs      # Calendar slot finding
└── utils/
    ├── mod.rs
    ├── auth.rs          # OAuth helpers
    └── logging.rs       # Logging setup
```

---

## **Key Design Decisions**

### **1. Authentication Strategy**

#### **Service-Specific Auth:**

| Service | Auth Method | Storage | Expiry Handling |
|---------|-------------|---------|-----------------|
| **Azure DevOps** | Personal Access Token (PAT) | OS Keyring | User manually renews (up to 1 year) |
| **7Pace Timetracker** | **Same Azure DevOps PAT** ✅ | OS Keyring (shared with DevOps) | Same as DevOps PAT |
| **Microsoft Graph (Outlook)** | OAuth2 Device Code Flow | Refresh token in OS Keyring | Auto-refresh access tokens |

> **Key Finding:** 7Pace Timetracker uses your **Azure DevOps PAT** for authentication (with scopes: Work Items Read/Write, User Profile Read, Identity Read). This means you only need **two credentials total**: DevOps PAT + Outlook OAuth.

#### **Credential Security Architecture:**

**OS Keyring Storage (Cross-Platform):**

Credentials are stored in the operating system's secure credential storage, **not in plain text files**:

- **macOS:** Keychain (`security` command)
- **Windows:** Credential Manager (Windows Credential Vault)
- **Linux:** Secret Service API (gnome-keyring or KWallet)

**Security Properties:**
- ✅ Encrypted at rest by the OS
- ✅ Access controlled by user login session
- ✅ Requires user authentication (password/biometric) on first access
- ✅ Cannot be read by other users on the system
- ✅ Survives reboots (persistent storage)

**Implementation with `keyring` crate:**
```rust
use keyring::Entry;

// Store Azure DevOps PAT (one-time setup)
fn store_devops_pat(pat: &str) -> Result<()> {
    let entry = Entry::new("ao-no-out7ook", "azure_devops_pat")?;
    entry.set_password(pat)?;  // Stored encrypted in OS keyring
    Ok(())
}

// Retrieve PAT (every API call)
fn get_devops_pat() -> Result<String> {
    let entry = Entry::new("ao-no-out7ook", "azure_devops_pat")?;
    let pat = entry.get_password()?;  // OS handles decryption
    Ok(pat)
}

// Store Outlook refresh token
fn store_outlook_token(refresh_token: &str) -> Result<()> {
    let entry = Entry::new("ao-no-out7ook", "outlook_refresh_token")?;
    entry.set_password(refresh_token)?;
    Ok(())
}
```

**Fallback for Headless Systems:**
If OS keyring is unavailable (e.g., headless Linux server), fall back to encrypted file:
```rust
// ~/.ao-no-out7ook/credentials.enc (AES-256 encrypted)
// User provides passphrase on CLI startup
// Use `age` crate for encryption
```

#### **Implementation Details:**

**Azure DevOps PAT:**
```rust
// User provides PAT via config or CLI
task config set devops.pat "your-pat-here"

// Or environment variable
export DEVOPS_PAT="your-pat-here"

// API call with PAT
let client = reqwest::blocking::Client::new();
client.get(url)
    .header("Authorization", format!("Basic {}", base64_encode(&format!(":{}", pat))))
    .send()?;
```

**Microsoft Graph OAuth2 Device Code Flow (ONE-TIME SETUP):**

> **Important:** Users only authenticate **once** during initial setup. The refresh token is stored persistently and automatically refreshes access tokens in the background. Re-authentication only needed if refresh token expires (~90 days) or is revoked.

```rust
// ONE-TIME setup (only run once)
task auth outlook

// CLI output:
// "Visit https://microsoft.com/devicelogin"
// "Enter code: ABC-DEF-GHI"
// [User authenticates in browser]
// "✓ Authenticated! Refresh token stored securely."

// Implementation:
use oauth2::{DeviceCodeFlow, TokenResponse};

async fn authenticate_outlook() -> Result<TokenResponse> {
    let client = /* Azure AD app config */;
    let device_auth = client.exchange_device_code()?;
    
    println!("Visit: {}", device_auth.verification_uri());
    println!("Code: {}", device_auth.user_code());
    
    // Poll for token (automatic with oauth2 crate)
    let token = device_auth.poll_token()?;
    
    // Store refresh token PERSISTENTLY in keyring
    store_refresh_token(&token.refresh_token())?;
    Ok(token)
}

// Every subsequent CLI command uses this (NO user interaction)
fn get_outlook_client() -> Result<Client> {
    let refresh_token = load_refresh_token()?;  // Load from keyring
    let access_token = refresh_if_needed(refresh_token)?;  // Auto-refresh if expired
    Ok(Client::new(access_token))
}
```

**User Experience:**
```bash
# First time ever
$ task start 12345
Error: Outlook not authenticated. Run: task auth outlook

$ task auth outlook
Visit: https://microsoft.com/devicelogin
Enter code: ABC-DEF
[User enters code in browser]
✓ Authenticated! You won't need to do this again.

# All subsequent commands work seamlessly
$ task start 12345   # ✓ Just works, no login needed
$ task list          # ✓ Just works
$ task current       # ✓ Just works

# 90 days later, refresh token expires
$ task start 67890
Warning: Outlook token expired. Run: task auth outlook
```

**Credential Storage Options:**

**Option A: System Keyring (Recommended for security)**
```toml
[dependencies]
keyring = "2"
```

```rust
use keyring::Entry;

// Store PAT
let entry = Entry::new("ao-no-out7ook", "azure_devops_pat")?;
entry.set_password(&pat)?;

// Retrieve PAT
let pat = entry.get_password()?;
```

**Option B: Encrypted File (Fallback for headless systems)**
```rust
// Encrypt credentials with user-provided passphrase
// Store in ~/.ao-no-out7ook/credentials.enc
// Prompt for passphrase on CLI startup if needed
```

**Recommendation:** Use keyring by default, fall back to encrypted file if keyring unavailable.

### **2. API Rate Limiting**

Cache aggressively to minimize API calls:
- DevOps work items: Cache for 5 minutes
- 7pace time entries: Cache for 1 minute
- Calendar events: Cache for 2 minutes
- Schemas: Cache for 24 hours (rarely change)

```rust
struct CachedData<T> {
    data: T,
    cached_at: DateTime<Utc>,
    ttl: Duration,
}
```

### **3. Concurrent Safety**

Use file locking for:
- ✅ `state.json` updates (current task, timers)
- ❌ Read-only cache files (no locking needed, stale reads OK)

### **4. Error Recovery**

If state file is corrupted:
1. Rename to `state.json.backup`
2. Create fresh state file
3. Log warning to user
4. Continue execution (graceful degradation)

---

## **CLI Command Structure**

```bash
# Core task management
task start <id>              # Start task, create Focus Block, start 7pace timer
task switch <id>             # Switch to different task
task stop                    # Stop current task and timer
task current                 # Show current task

# Work item operations
task list [--state Active]   # List work items
task show <id>               # Show work item details
task state <id> [new-state]  # View/update work item state
task sync                    # Sync with DevOps

# Time tracking
task checkin --continue|--blocked|--stop  # Respond to Focus Block notification

# Schema inspection (for AI agents)
task context --format llm    # Export current context for AI
task schema                  # Show DevOps schema for current work item type
task decompose --input <file>  # Bulk create sub-tasks

# Config
task config list             # Show current config
task config set <key> <value>  # Update config
```

---

## **Build & Distribution**

### **Single Binary Compilation**
```bash
# Release build with optimizations
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin  # Apple Silicon
```

### **Static Linking (for portability)**
```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
strip = true        # Strip symbols
codegen-units = 1   # Better optimization
```

### **Distribution**
- GitHub Releases with binaries for Linux/macOS/Windows
- Optional: Homebrew tap, Cargo crates.io, Chocolatey (Windows)

---

## **Security Considerations**

1. **Credential Storage**
   - Never log PATs/tokens
   - Use keyring or encrypted files
   - Prompt for re-auth if tokens expire

2. **File Permissions**
   - Set `state.json` and `config.toml` to `0600` (owner read/write only)
   - Use `std::fs::set_permissions` on Unix

3. **API Security**
   - Use HTTPS for all API calls
   - Validate TLS certificates
   - Implement token refresh for OAuth

---

## **Phase 1 MVP Scope**

For the initial implementation, focus on:

✅ **Must Have:**
- File-based state with locking
- DevOps API integration (list, show, update state)
- State management (current task, switch)
- Basic config file loading
- `task start`, `task stop`, `task list`, `task state`

❌ **Defer to Later:**
- 7pace integration (Phase 2)
- Calendar/Focus Blocks (Phase 3)
- Markdown export (Phase 4)
- AI integration (Phase 8)

---

## **Testing Strategy**

```rust
#[cfg(test)]
mod tests {
    // Unit tests for each module
    
    #[test]
    fn test_state_concurrent_access() {
        // Spawn multiple threads/processes
        // Verify file locking works correctly
    }
    
    #[test]
    fn test_devops_api_mock() {
        // Use wiremock to test API client
    }
}
```

Integration tests with mock APIs using `wiremock` or `httptest`.

---
