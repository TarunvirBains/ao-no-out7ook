# API Integration Guide

> **Purpose:** Document the APIs we'll integrate with and identify useful Rust crates to minimize custom implementation.

---

## **Overview**

Our CLI integrates with three services:

| Service | Purpose | Authentication |
|---------|---------|----------------|
| **Azure DevOps** | Work item CRUD, state transitions, hierarchy | PAT (Personal Access Token) |
| **7Pace Timetracker** | Timer start/stop, worklog entries | Same Azure DevOps PAT ✅ |
| **Microsoft Graph** | Calendar events (Outlook) | OAuth2 Device Code Flow |

---

## **1. Azure DevOps REST API**

### **Base URL**
```
https://dev.azure.com/{organization}/{project}/_apis/
```

### **API Version**
Use `api-version=7.0` (latest stable as of 2026)

### **Key Endpoints for Our Use Cases**

#### **Work Items**

| Operation | Method | Endpoint | Purpose |
|-----------|--------|----------|---------|
| Get single work item | `GET` | `/wit/workitems/{id}` | Fetch details for a specific work item |
| Get batch work items | `POST` | `/wit/workitemsbatch` | Fetch multiple work items by IDs |
| Query with WIQL | `POST` | `/wit/wiql` | Query work items using Work Item Query Language |
| Create work item | `POST` | `/{project}/_apis/wit/workitems/${type}` | Create new work item |
| Update work item | `PATCH` | `/wit/workitems/{id}` | Update fields (state, assigned-to, etc.) |
| Get work item types | `GET` | `/wit/workitemtypes` | Fetch available work item types |
| Get work item type | `GET` | `/wit/workitemtypes/{type}` | Get schema for specific type (valid states, transitions) |

**Example: Get Work Item**
```http
GET https://dev.azure.com/{org}/{project}/_apis/wit/workitems/12345?api-version=7.0
Authorization: Basic {base64(":PAT")}
```

**Example: Update State (FR1.13)**
```http
PATCH https://dev.azure.com/{org}/{project}/_apis/wit/workitems/12345?api-version=7.0
Authorization: Basic {base64(":PAT")}
Content-Type: application/json-patch+json

[
  {
    "op": "add",
    "path": "/fields/System.State",
    "value": "Active"
  }
]
```

**Example: Get Valid State Transitions (FR1.14)**
```http
GET https://dev.azure.com/{org}/{project}/_apis/wit/workitemtypes/User%20Story?api-version=7.0
Authorization: Basic {base64(":PAT")}

Response includes:
{
  "states": [...],
  "transitions": {
    "New": ["Active", "Removed"],
    "Active": ["Resolved", "Removed"],
    ...
  }
}
```

### **Authentication**
```rust
use base64::encode;

let pat = "your_personal_access_token";
let auth_header = format!("Basic {}", encode(format!(":{}", pat)));

client.get(url)
    .header("Authorization", auth_header)
    .send()?;
```

### **Useful Rust Crate: `azure_devops_rust_api`**

**Official Microsoft Rust SDK for Azure DevOps**

```toml
[dependencies]
azure_devops_rust_api = "0.19"
azure_identity = "0.21"
azure_core = "0.21"
```

**Features:**
- ✅ Auto-generated from OpenAPI spec
- ✅ Type-safe API wrappers
- ✅ Built on `reqwest` (async)
- ✅ Supports PAT authentication
- ✅ Modular (enable only needed APIs via features)

**Example Usage:**
```rust
use azure_devops_rust_api::wit;
use azure_identity::credential::PersonalAccessTokenCredential;

#[tokio::main]
async fn main() -> Result<()> {
    let org = "myorg";
    let project = "myproject";
    let pat = std::env::var("DEVOPS_PAT")?;
    
    let credential = PersonalAccessTokenCredential::new(pat);
    let client = wit::ClientBuilder::new(credential).build();
    
    // Get work item
    let work_item = client.work_items_client()
        .get(org, 12345)
        .send()
        .await?;
    
    println!("Title: {}", work_item.fields.get("System.Title").unwrap());
    
    Ok(())
}
```

**Pros:**
- Official Microsoft support
- Type-safe
- Well-documented

**Cons:**
- Requires async runtime (tokio)
- Might be overkill for our simple needs

**Alternative: Roll our own with `reqwest`**

For Phase 1 MVP, we might prefer **custom HTTP client with `reqwest`** for simplicity:
```rust
use reqwest::blocking::Client;
use serde_json::Value;

fn get_work_item(client: &Client, org: &str, project: &str, id: u32, pat: &str) -> Result<Value> {
    let url = format!(
        "https://dev.azure.com/{}/{}/_apis/wit/workitems/{}?api-version=7.0",
        org, project, id
    );
    
    let resp = client
        .get(&url)
        .header("Authorization", format!("Basic {}", base64::encode(&format!(":{}", pat))))
        .send()?
        .json()?;
    
    Ok(resp)
}
```

**Recommendation:** Start with **custom `reqwest` client** for MVP, migrate to `azure_devops_rust_api` if complexity grows.

---

## **2. 7Pace Timetracker API**

### **Base URL**
```
https://api.timehub.7pace.com/{organization}/_apis/
```

### **Authentication**
Uses the **same Azure DevOps PAT** with required scopes:
- `Work Items (Read & Write)`
- `User Profile (Read)`
- `Identity (Read)`

### **Key Endpoints**

| Operation | Method | Endpoint | Purpose |
|-----------|--------|----------|---------|
| Start timer | `POST` | `/api/tracking/client/startTracking` | Start timer for work item |
| Stop timer | `POST` | `/api/tracking/client/stopTracking/{reason}` | Stop active timer |
| Get current timer | `GET` | `/api/tracking/client/current/{expand}` | Get active timer state |
| Get worklogs | `GET` | `/worklogs` | Fetch time entries for user |
| Create worklog | `POST` | `/worklogs` | Manually create time entry |
| Update worklog | `PATCH` | `/worklogs` | Update existing time entry |

**Example: Start Timer (FR2.1)**
```http
POST https://api.timehub.7pace.com/{org}/_apis/api/tracking/client/startTracking
Authorization: Basic {base64(":PAT")}
Content-Type: application/json

{
  "workItemId": 12345,
  "comment": "Working on login feature"
}

Response:
{
  "id": "timer-uuid",
  "workItemId": 12345,
  "startedAt": "2026-01-07T17:00:00Z"
}
```

**Example: Stop Timer (FR2.2)**
```http
POST https://api.timehub.7pace.com/{org}/_apis/api/tracking/client/stopTracking/0
Authorization: Basic {base64(":PAT")}

Response:
{
  "worklogId": 789,
  "duration": 3600, // seconds
  "workItemId": 12345
}
```

### **Useful Rust Crates**

**No official 7Pace Rust SDK exists.** Use `reqwest` for HTTP calls.

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "blocking"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Custom Client Example:**
```rust
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct StartTimerRequest {
    #[serde(rename = "workItemId")]
    work_item_id: u32,
    comment: Option<String>,
}

#[derive(Deserialize)]
struct TimerResponse {
    id: String,
    #[serde(rename = "workItemId")]
    work_item_id: u32,
    #[serde(rename = "startedAt")]
    started_at: String,
}

fn start_7pace_timer(
    client: &Client,
    org: &str,
    work_item_id: u32,
    pat: &str,
) -> Result<TimerResponse> {
    let url = format!(
        "https://api.timehub.7pace.com/{}/_apis/api/tracking/client/startTracking",
        org
    );
    
    let body = StartTimerRequest {
        work_item_id,
        comment: None,
    };
    
    let resp = client
        .post(&url)
        .header("Authorization", format!("Basic {}", base64::encode(&format!(":{}", pat))))
        .json(&body)
        .send()?
        .json::<TimerResponse>()?;
    
    Ok(resp)
}
```

---

## **3. Microsoft Graph API (Outlook Calendar)**

### **Base URL**
```
https://graph.microsoft.com/v1.0/
```

### **Authentication**
OAuth2 Device Code Flow (one-time setup, persistent refresh token)

### **Key Endpoints**

| Operation | Method | Endpoint | Purpose |
|-----------|--------|----------|---------|
| List calendar events | `GET` | `/me/calendar/events` | Get user's calendar events |
| Create calendar event | `POST` | `/me/calendar/events` | Create new event (Focus Block) |
| Update calendar event | `PATCH` | `/me/events/{id}` | Update existing event |
| Delete calendar event | `DELETE` | `/me/events/{id}` | Delete event |
| Find meeting times | `POST` | `/me/findMeetingTimes` | Find available slots (for smart scheduling FR3.7) |

**Example: Get Calendar Events (FR3.7 Smart Scheduling - Step 1)**
```http
GET https://graph.microsoft.com/v1.0/me/calendar/events?$filter=start/dateTime ge '2026-01-07T16:00:00' and end/dateTime le '2026-01-07T17:00:00'&$select=subject,start,end
Authorization: Bearer {access_token}

Response:
{
  "value": [
    {
      "subject": "Team Standup",
      "start": {
        "dateTime": "2026-01-07T16:30:00",
        "timeZone": "America/Los_Angeles"
      },
      "end": {
        "dateTime": "2026-01-07T16:45:00",
        "timeZone": "America/Los_Angeles"
      }
    }
  ]
}
```

**Smart Scheduling Workflow (FR3.7):**
1. **GET existing events** in today's work window (8:30am-5:00pm)
2. **Find gaps** between existing events using 15-min intervals
3. **Find next available slot** that fits Focus Block duration (e.g., 45 min)
4. **Create Focus Block** in first available slot

**Example: Create Calendar Event (FR3.7 Focus Block)**
```http
POST https://graph.microsoft.com/v1.0/me/calendar/events
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "subject": "🎯 Focus: Task 12345 - Implement login",
  "start": {
    "dateTime": "2026-01-07T16:15:00",
    "timeZone": "America/Los_Angeles"
  },
  "end": {
    "dateTime": "2026-01-07T17:00:00",
    "timeZone": "America/Los_Angeles"
  },
  "categories": ["7pace-focus"],
  "isReminderOn": true,
  "reminderMinutesBeforeStart": 0,
  "body": {
    "contentType": "text",
    "content": "task://checkin?id=12345"
  }
}
```

**Example: Find Next Available Slot (FR3.7 Smart Scheduling)**
```http
POST https://graph.microsoft.com/v1.0/me/findMeetingTimes
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "attendees": [],
  "timeConstraint": {
    "timeslots": [
      {
        "start": {
          "dateTime": "2026-01-07T16:00:00",
          "timeZone": "America/Los_Angeles"
        },
        "end": {
          "dateTime": "2026-01-07T17:00:00",
          "timeZone": "America/Los_Angeles"
        }
      }
    ]
  },
  "meetingDuration": "PT45M",
  "returnSuggestionReasons": true
}
```

### **Useful Rust Crates**

#### **Option 1: `graph-rs-sdk` (Comprehensive SDK)**

```toml
[dependencies]
graph-rs-sdk = "3.0"
graph-oauth = "3.0"  # Includes device code flow
```

**Features:**
- ✅ Full Microsoft Graph SDK
- ✅ OAuth2 device code flow built-in
- ✅ Type-safe API wrappers
- ✅ Automatic token refresh
- ✅ Async and blocking modes

**Example Usage:**
```rust
use graph_oauth::oauth::{AccessToken, OAuth};
use graph_rs_sdk::GraphClient;

// One-time device code auth
async fn authenticate() -> Result<AccessToken> {
    let mut oauth = OAuth::new();
    oauth
        .client_id("<client-id>")
        .add_scope("Calendars.ReadWrite")
        .add_scope("offline_access");
    
    let response = oauth.request_device_code().send().await?;
    
    println!("Visit: {}", response.verification_uri());
    println!("Code: {}", response.user_code());
    
    let token = oauth.poll_for_device_token(&response).await?;
    Ok(token)
}

// Create calendar event
async fn create_focus_block(access_token: &str) -> Result<()> {
    let client = GraphClient::new(access_token);
    
    let event = serde_json::json!({
        "subject": "🎯 Focus: Task 12345",
        "start": {
            "dateTime": "2026-01-07T16:15:00",
            "timeZone": "America/Los_Angeles"
        },
        "end": {
            "dateTime": "2026-01-07T17:00:00",
            "timeZone": "America/Los_Angeles"
        }
    });
    
    client
        .me()
        .calendar()
        .events()
        .create(&event)
        .send()
        .await?;
    
    Ok(())
}
```

**Pros:**
- Official-ish (community-maintained, well-supported)
- Handles OAuth complexity
- Type-safe

**Cons:**
- Large dependency
- Requires async runtime

#### **Option 2: Minimal `reqwest` + `oauth_device_flows`**

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
oauth_device_flows = "0.1"
serde = "1"
serde_json = "1"
```

**Example:**
```rust
use oauth_device_flows::{DeviceFlow, MicrosoftProvider};

// One-time auth
async fn auth_outlook() -> Result<String> {
    let provider = MicrosoftProvider::new("<client-id>");
    let flow = DeviceFlow::new(provider, vec!["Calendars.ReadWrite".to_string()]);
    
    let code_response = flow.request_code().await?;
    println!("Visit: {}", code_response.verification_uri);
    println!("Code: {}", code_response.user_code);
    
    let token = flow.poll_for_token(code_response).await?;
    Ok(token.refresh_token)
}

// Use reqwest for API calls
async fn create_event(access_token: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let event = serde_json::json!({ /* ... */ });
    
    client
        .post("https://graph.microsoft.com/v1.0/me/calendar/events")
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&event)
        .send()
        .await?;
    
    Ok(())
}
```

**Recommendation:** Use **`graph-rs-sdk`** for Phase 3 (Calendar integration). It handles OAuth complexity well.

---

## **Recommended Rust Crates Summary**

### **Core Dependencies**

```toml
[dependencies]
# CLI Framework
clap = { version = "4", features = ["derive", "env"] }

# HTTP Client (for custom APIs)
reqwest = { version = "0.11", features = ["json", "blocking"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Configuration
config = "0.14"
toml = "0.8"

# File Locking
fs2 = "0.4"

# Time/Date Handling
chrono = "0.4"

# Error Handling
anyhow = "1"
thiserror = "1"

# Base64 encoding (for DevOps PAT auth)
base64 = "0.22"

# Credential Storage
keyring = "2"
```

### **API-Specific Dependencies**

```toml
# Azure DevOps (OPTIONAL - only if we use the SDK)
azure_devops_rust_api = { version = "0.19", optional = true }
azure_identity = { version = "0.21", optional = true }

# Microsoft Graph (for Phase 3)
graph-rs-sdk = { version = "3.0", optional = true }
graph-oauth = { version = "3.0", optional = true }

# OR lightweight OAuth device flow
oauth_device_flows = { version = "0.1", optional = true }
```

### **Phase 1 MVP: Minimal Dependencies**

For Phase 1 (Core CLI + DevOps integration only), use:
- `reqwest` (blocking mode for simplicity)
- `serde`/`serde_json`
- `base64`
- `keyring`
- `clap`
- `fs2`
- `chrono`

**No need for heavy SDKs yet.** Roll our own HTTP clients for DevOps and 7Pace.

---

## **API Integration Strategy**

### **Phase 1: Azure DevOps Only**
- Custom `reqwest` client
- PAT authentication via `Authorization: Basic` header
- Implement:
  - `GET /wit/workitems/{id}` (FR1.2)
  - `PATCH /wit/workitems/{id}` (FR1.13 - update state)
  - `GET /wit/workitemtypes/{type}` (FR1.14 - fetch valid states)
  - `POST /wit/wiql` (FR1.15 - sort by urgency)

### **Phase 2: 7Pace Integration**
- Reuse DevOps PAT
- Custom `reqwest` client
- Implement:
  - `POST /api/tracking/client/startTracking` (FR2.1)
  - `POST /api/tracking/client/stopTracking` (FR2.2)
  - `GET /api/tracking/client/current` (check active timer)

### **Phase 3: Outlook Calendar**
- Use `graph-rs-sdk` or `graph-oauth`
- OAuth2 device code flow (one-time setup)
- Implement:
  - `POST /me/calendar/events` (FR3.7 - Focus Blocks)
  - `POST /me/findMeetingTimes` (FR3.7 - smart scheduling)
  - `PATCH /me/events/{id}` (update events)

---

## **Testing Strategy**

### **Mock APIs for Development**

Use `wiremock` or `httpmock` for testing:

```toml
[dev-dependencies]
wiremock = "0.6"
```

```rust
#[cfg(test)]
mod tests {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    
    #[tokio::test]
    async fn test_get_work_item() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/wit/workitems/12345"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 12345,
                "fields": {
                    "System.Title": "Test Item"
                }
            })))
            .mount(&mock_server)
            .await;
        
        // Test your client against mock_server.uri()
    }
}
```

---
