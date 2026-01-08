# Roadmap-Oriented Functional Requirements – DevOps / 7pace / Calendar / Markdown App

> **Goal:** Build a robust, hierarchy-aware, cross-platform tool with CLI-core first, optional frontends, calendar scheduling, and 7pace integration.

---

## **Phase 1: Core CLI & Data Foundation (Mandatory)**

**Goal:** Establish the CLI core, canonical JSON data format, and basic DevOps integration.

1. **FR1.1:** Use **JSON as the canonical internal format** for all operations.
2. **FR1.2:** List and search work items by ID, title, state, assigned-to, or tags.
3. **FR1.3:** Display key metadata: ID, title, description, state, assigned-to, tags, planned effort, logged time.
4. **FR1.4:** Maintain hierarchy (User Story → Feature → Epic) and visualize in CLI/tree output.
5. **FR1.5:** Parse Markdown to JSON and validate required fields, types, and hierarchy.
6. **FR1.6:** Reject bad or incomplete formatting with clear CLI error messages.
7. **FR1.7:** Support round-trip conversion: JSON ↔ Markdown with validation.
8. **FR1.8:** Handle conflicts if work items have been updated in DevOps since last sync.
9. **FR1.9:** Export/import work items via Markdown or JSON.
10. **FR1.10:** Provide dry-run mode for safe preview.
11. **FR1.11:** Maintain local state of "current active task" to allow quick resumption and context awareness.
12. **FR1.12:** Switch current active task via `task switch <ID>` command.
13. **FR1.13:** Update work item fields (state, assigned-to, priority, etc.) via CLI and sync to DevOps.
14. **FR1.14:** Fetch DevOps work item type schemas to get valid state transitions (e.g., New → Active → Resolved → Closed).
15. **FR1.15:** Sort tasks by "urgency" (DevOps priority, deadlines, etc.) when listing.

**Outcome:** CLI core can manage work items reliably and maintain structured data.

---

## **Phase 2: 7pace Timer & Time Logging**

**Goal:** Enable robust time tracking tied to work items and hierarchy.

1. **FR2.1:** Start 7pace timer programmatically (replicates "Start Working" button).
2. **FR2.2:** Stop active timers via 7pace API.
3. **FR2.3:** Automatically stop conflicting timers if only one is allowed per user.
4. **FR2.4:** Compute duration from calendar events or timer entries.
5. **FR2.5:** Post time entries with optional comments, respecting hierarchy aggregation.
6. **FR2.6:** Fetch existing time entries for reconciliation.
7. **FR2.7:** Dry-run mode for timer operations.
8. **FR2.8:** Graceful handling of failed API calls (retry/error logging).

**Outcome:** CLI can log work item time fully automatically and safely.

---

## **Phase 3: Calendar Integration & Scheduling**

**Goal:** Sync work items with calendar blocks, enabling optional automation of 7pace timers.

1. **FR3.1:** Display scheduled blocks per work item (weekly/daily).
2. **FR3.2:** Create calendar events with start, end, recurrence, and category/tag.
3. **FR3.3:** Maintain mapping between calendar events and work items.
4. **FR3.4:** Update calendar events if work items or hierarchy change.
5. **FR3.5:** Visualize conflicts (overlaps, parent-child dependencies).
6. **FR3.6:** Optional start/stop 7pace timers based on calendar blocks.
7. **FR3.7 (Smart Focus Blocks):** When starting a task, find the next available calendar slot using 15-minute interval granularity (:00, :15, :30, :45). Schedule a Focus Block for configurable duration (e.g., 45 min, 1 hour) that:
   - Starts at next available interval within configured work hours
   - Respects existing calendar events (truncates if conflict detected)
   - Rolls over to next work day if insufficient time remains today
8. **FR3.8 (Check-in):** Interactive prompt after Focus Block ends: "Continue", "Blocked", or "Stop".
9. **FR3.9 (Notifications):** Use Outlook's native calendar reminders for Focus Block notifications. Calendar events include custom actions (URL scheme or add-in) that trigger `task checkin --continue|--blocked|--stop` commands. No persistent daemon required.
10. **FR3.10 (Teams Presence):** Optional integration to sync Microsoft Teams presence status during Focus Blocks (e.g., set to "Do Not Disturb" or "Focusing"). Toggleable via config flag.

**Outcome:** CLI can plan, visualize, and optionally automate work items based on calendar.

---

## **Phase 4: Markdown / Human-Editable Layer**

**Goal:** Allow developers to edit work items in a Git-friendly, human-readable format.

1. **FR4.1:** Generate Markdown for selected work items with hierarchy and key fields.
   - Hierarchical format using headers (# Epic, ## Feature, ### Story, #### Task)
   - Metadata as inline bullets (State, Assigned, Priority, Parent, Effort, Tags)
   - HTML tag stripping for clean descriptions
2. **FR4.2:** Parse Markdown back to JSON with validation.
   - Support creating new work items (ID=#0 or omitted)
   - **State Filtering:** Import skips Completed/Resolved/Closed/Removed items by default
   - `--force` flag to override and import all items
   - Export includes ALL items (full state snapshot)
3. **FR4.3:** Highlight errors, missing fields, or malformed hierarchy.
   - **Hierarchy Validation:** Enforce parent-child relationships:
     * Epic: No parent required (standalone OK)
     * Feature: Must have Epic parent
     * User Story: Must have Feature or Epic parent
     * Task/Bug: Must have User Story or Feature parent
   - Validate parent exists in DevOps before import
   - **Error reporting format:**
     * Line number where error occurred
     * Actual line content from markdown file
     * Clear error message explaining the problem
     * Suggested fix when applicable
   - **Example error output:**
     ```
     ❌ Line 12: ### User Story: Add login (#100)
         Error: User Story must have a parent (Feature or Epic)
         Suggestion: Add **Parent:** #<ID> to the metadata line
     ```
4. **FR4.4:** Versioning support via Git or local history.
5. **FR4.5:** Optional: update calendar blocks based on Markdown edits.

**Outcome:** Developers can safely edit work items outside DevOps with CLI validation.

---

## **Phase 5: AI & Agent Integration**

**Goal:** Empower AI agents (Copilot, Windsurf, etc.) to act as intelligent drivers of the DevOps process using the CLI as their "hands".

1. **FR5.1 (Context Export):** `task context --format llm` command to dump current task, parent, and siblings in token-optimized format for AI reasoning.
2. **FR5.2 (Decomposition):** Support bulk child-item creation JSON input to allow Agents to "explode" a story into tasks in one operation.
3. **FR5.3 (Schema Reflection):** CLI can output its own schema/capabilities so an Agent knows *how* to interact with it dynamically.
4. **FR5.4 (Safe-Mode Diffs):** "Dry-run" outputs a precise diff of what *would* change in DevOps, allowing the Agent to ask "User, do you approve these 5 new sub-tasks?" before committing.

**Outcome:** Users gain convenience, visual hierarchy, and calendar integration without sacrificing CLI robustness.

---

## **Phase 6: Configuration, Security & Distribution**

**Goal:** Enable secure, cross-platform operation and open source distribution.

1. **FR6.1:** Store credentials securely (avoid plain text).
2. **FR6.2:** Use personal access tokens (PAT) where possible.
3. **FR6.3:** Configurable paths, projects, calendars, mappings, work hours (start/end time), Focus Block duration, and scheduling interval granularity (e.g., 15 min, 25 min for Pomodoro).
4. **FR6.4:** Optional multi-user support for shared calendars or DevOps projects.
5. **FR6.5:** MIT/BSD license for open source distribution; premium frontends optional.

**Outcome:** CLI and frontends are secure, configurable, and ready for open source and potential monetization.

---

## **Phase 7: Automation / Scheduling**

**Goal:** Support fully automated workflows for time tracking and syncing.

1. **FR7.1:** CLI exposes commands for start/stop timers, sync Markdown ↔ DevOps, schedule calendar blocks, bulk import/export.
2. **FR7.2:** Dry-run mode for safe automation.
3. **FR7.3:** Cross-platform support (Windows/macOS/Linux).

**Outcome:** Users can run scheduled automation (cron/Task Scheduler) reliably.

---

## **Phase 8: Optional Frontends (GUI & Editor Plugins)**

**Goal:** Provide user-friendly interfaces while relying on CLI core.

1. **FR8.1:** Frontends may include VS Code plugin, Dioxus/Tauri GUI, Vim/Emacs/JetBrains plugins.
2. **FR8.2:** All frontends call CLI core for operations.
3. **FR8.3:** Display unsynced changes, conflicts, time logged vs planned, hierarchy visualization.

**Outcome:** Users gain convenience, visual hierarchy, and calendar integration without sacrificing CLI robustness.

---

## **Usefulness Analysis**

### **✅ What Makes This Useful**

1. **Closes the "Context-Switching Tax"**  
   - Developers lose 10-15 minutes every time they check DevOps, update timers, or block time.  
   - **FR1.11 (Statefulness)** + **FR3.7 (Pomodoro)** eliminates this entirely: `task start 12345` does *everything*.

2. **7pace is Clunky for Keyboard Users**  
   - The web UI is click-heavy. Developers want: `task start 12345` → timer starts, calendar blocks, done.  
   - **FR2.1-2.8** make time tracking *invisible*.

3. **AI-Augmented Workflows Are the Future**  
   - **Phase 5**. Modern developers use AI copilots—this CLI lets the AI *drive* DevOps.  
   - Example: "Break this feature into frontend/backend tasks" → AI generates JSON → `task decompose --input ai_tasks.json` → Done.

4. **Markdown = Git-Friendly**  
   - **Phase 4** allows treating work items like code: branch, edit, PR, merge.  
   - Perfect for distributed teams who hate web UIs.

5. **Pomodoro + Check-ins**  
   - **FR3.8** is brilliant: If you hit a blocker, it asks "Continue/Blocked/Stop". If "Blocked", it can auto-update the task state in DevOps.  
   - This creates *real-time* visibility for managers without Slack interruptions.

### **⚠️ Gaps/Considerations**

1. **~~Missing: Notifications~~ ✅ RESOLVED**  
   - **FR3.9** now specifies Outlook's native calendar reminders with custom action handlers.

2. **~~Missing: Offline Mode~~ ❌ NOT NEEDED**  
   - This CLI requires cloud connectivity by design (7pace, DevOps, Outlook APIs are all cloud-based).  
   - Offline operation doesn't make sense for a synchronization tool.

3. **~~Missing: Teams Integration~~ ✅ RESOLVED**  
   - **FR3.10** now provides optional Teams presence status sync (toggleable).  
   - Note: Teams calendar = Outlook calendar (same Exchange backend), so calendar integration already works for both.

4. **Phase 5 Dependency on Phase 1**  
   - AI integration is powerful, but only if Phase 1 is *rock-solid*.  
   - **Risk Mitigation:** Strict JSON schema validation (FR1.5-1.7) is critical.

