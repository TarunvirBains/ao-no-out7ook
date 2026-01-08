# ao-no-out7ook (Azure + 7Pace + Outlook Integration)

A powerful Rust-based CLI for seamless integration between Azure DevOps, 7Pace Timetracker, and Microsoft Outlook Calendar. Designed for developers who want to minimize context switching and maximize productivity.


[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange.svg)](https://www.rust-lang.org/)

---

## **Features**

### **🎯 Stateful Task Management**
- Track current active task locally
- Switch between tasks seamlessly
- Sort tasks by DevOps priority/urgency
- Update work item states from CLI

### **⏱️ Automated Time Tracking (7Pace)**
- Start/stop timers programmatically
- Automatic timer management tied to tasks
- No more "forgot to start the timer" moments

### **📅 Smart Calendar Scheduling (Outlook)**
- Create "Focus Blocks" in your calendar automatically
- Smart scheduling finds next available time slot (15-min intervals)
- Respects existing meetings and work hours
- Pomodoro-style workflow with check-ins

### **🤖 AI Agent Integration**
- Export task context for AI coding assistants
- Bulk task decomposition via JSON input
- Schema reflection for dynamic agent interaction

---

## **Quick Start**

### **Prerequisites**
- Rust 1.92 or later
- Azure DevOps account with PAT (Personal Access Token)
- 7Pace Timetracker enabled on your DevOps org
- Microsoft 365 account (for Outlook calendar integration)

### **Installation**

**From Source:**
```bash
git clone https://github.com/yourusername/devops-cli.git
cd devops-cli
cargo build --release
./target/release/task --version
```

**From Crates.io (when published):**
```bash
cargo install devops-cli
```

### **First-Time Setup**

1. **Configure Azure DevOps:**
```bash
task config set devops.organization "myorg"
task config set devops.project "MyProject"
task config set devops.pat "your-pat-here"
```

2. **Authenticate Outlook (one-time):**
```bash
task auth outlook
# Follow the prompts to authenticate via device code flow
```

3. **Configure Work Hours:**
```bash
task config set work_hours.start "08:30"
task config set work_hours.end "17:00"
task config set work_hours.timezone "America/Los_Angeles"
```

---

## **Basic Usage**

### **Start Working on a Task**
```bash
# Start task, create Focus Block, start 7Pace timer
task start 12345

# Output:
# ✓ Timer started for Task 12345
# ✓ Focus Block created: 9:15 AM - 10:00 AM
# 🎯 Currently working on: Implement login feature
```

### **Check Current Task**
```bash
task current

# Output:
# 🎯 Task 12345: Implement login feature
# ⏱️  Timer running: 32 minutes
# 📅 Focus Block ends at 10:00 AM
```

### **Switch Tasks**
```bash
task switch 67890
# Stops timer on 12345, starts timer on 67890
```

### **List Work Items**
```bash
task list --state Active
task list --assigned-to me --sort urgency
```

### **Update Work Item State**
```bash
task state 12345 Active   # Move to Active
task state 12345          # Show valid transitions
```

---

## **Advanced Features**

### **Pomodoro Check-ins**
After your Focus Block ends, Outlook will prompt:
- **Continue** → Creates next Focus Block
- **Blocked** → Updates task state in DevOps, stops timer
- **Stop** → Stops timer, logs time

### **AI Agent Integration**
```bash
# Export context for AI assistant
task context --format llm

# Bulk create sub-tasks from AI-generated JSON
task decompose --input ai_tasks.json

# Show DevOps schema for current work item type
task schema
```

---

## **Configuration**

Configuration is stored in `~/.devops-cli/config.toml`:

```toml
[devops]
organization = "myorg"
project = "MyProject"

[work_hours]
start = "08:30"
end = "17:00"
timezone = "America/Los_Angeles"

[focus_blocks]
duration_minutes = 45
interval_minutes = 15
teams_presence_sync = true

[state]
task_expiry_hours = 24  # Clear stale tasks after this period
```

Credentials are stored securely in your OS keyring (Keychain/Credential Manager/Secret Service).

---

## **Architecture**

- **Single Binary** - No external dependencies, just compile and run
- **File-Based State** - All state stored in `~/.devops-cli/state.json` with file locking
- **API-Driven** - Integrates with Azure DevOps, 7Pace, and Microsoft Graph APIs
- **Cross-Platform** - Works on Linux, macOS, and Windows

For detailed architecture documentation, see [ARCHITECTURE.md](ARCHITECTURE.md).

---

## **Development**

### **Build from Source**
```bash
cargo build
cargo test
cargo run -- help
```

### **Run Tests**
```bash
cargo test
cargo test --all-features
```

### **Generate Documentation**
```bash
cargo doc --open
```

---

## **Project Documentation**

- **[Functional Requirements](docs/REQUIREMENTS.md)** - What the system does
- **[Architecture](docs/ARCHITECTURE.md)** - System design and structure
- **[API Integration](docs/API_INTEGRATION.md)** - External API documentation
- **[CLI Specification](docs/CLI_SPEC.md)** - Complete command reference

---

## **Roadmap**

- [x] **Phase 1:** Core CLI & DevOps integration
- [x] **Phase 2:** 7Pace timer integration
- [ ] **Phase 3:** Outlook calendar & smart scheduling
- [ ] **Phase 4:** Markdown import/export
- [ ] **Phase 5:** GUI frontends (VS Code plugin, Tauri app)
- [ ] **Phase 6:** Security hardening & distribution
- [ ] **Phase 7:** Automation & scheduled workflows
- [ ] **Phase 8:** AI agent integration

See [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md) for details.

---

## **Contributing**

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass (`cargo test`)
5. Submit a pull request

---

## **License**

MIT License - see [LICENSE](LICENSE) for details.

---

## **Acknowledgments**

- Built with [Rust](https://www.rust-lang.org/)
- Uses [reqwest](https://github.com/seanmonstar/reqwest) for HTTP
- Calendar integration via [Microsoft Graph API](https://docs.microsoft.com/en-us/graph/)
- Time tracking via [7Pace Timetracker](https://www.7pace.com/)

---

## **Support**

- **Documentation:** See docs in this repository
- **Issues:** [GitHub Issues](https://github.com/yourusername/devops-cli/issues)
- **Discussions:** [GitHub Discussions](https://github.com/yourusername/devops-cli/discussions)
