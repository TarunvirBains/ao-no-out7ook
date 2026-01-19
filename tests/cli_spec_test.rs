use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_list_json_contract() {
    // 1. Start Mock Server
    let mock_server = MockServer::start().await;

    // 2. Setup Mock Responses
    // WIQL Query Response
    Mock::given(method("POST"))
        .and(path("/test_proj/_apis/wit/wiql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "queryType": "flat",
            "workItems": [
                { "id": 101, "url": "http://mock/101" },
                { "id": 102, "url": "http://mock/102" }
            ]
        })))
        .mount(&mock_server)
        .await;

    // Work Items Batch Response
    // Note: The path includes generic batch or specific IDs?
    // Client calls POST workitemsbatch
    Mock::given(method("POST"))
        .and(path("/test_proj/_apis/wit/workitemsbatch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 2,
            "value": [
                {
                    "id": 101,
                    "rev": 1,
                    "fields": {
                        "System.Title": "Implement Login",
                        "System.State": "Active",
                        "System.WorkItemType": "Task",
                        "System.AssignedTo": { "displayName": "Alice" },
                        "Microsoft.VSTS.Common.Priority": 2
                    },
                    "url": "http://mock/101"
                },
                {
                    "id": 102,
                    "rev": 1,
                    "fields": {
                        "System.Title": "Fix CSS",
                        "System.State": "New",
                        "System.WorkItemType": "Bug",
                        "System.AssignedTo": { "displayName": "Bob" },
                        "Microsoft.VSTS.Common.Priority": 1
                    },
                    "url": "http://mock/102"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    // 3. Prepare Environment
    let temp_home = tempfile::tempdir().unwrap();
    let config_dir = temp_home.path().join(".ao-no-out7ook");
    fs::create_dir_all(&config_dir).unwrap();

    let config_content = format!(
        r#"
[devops]
organization = "test_org"
project = "test_proj"
api_url = "{}"
pat = "dummy_pat"
use_keyring = false

[graph]
client_id = "dummy_client"
"#,
        mock_server.uri()
    );

    fs::write(config_dir.join("config.toml"), config_content).unwrap();

    // 4. Run CLI Command
    let mut cmd = Command::cargo_bin("ano7").unwrap();
    cmd.env("HOME", temp_home.path())
        .arg("list")
        .arg("--format")
        .arg("json");

    // 5. Assert Output
    let assert = cmd.assert();
    let assert = assert.success();
    let output = assert.get_output();
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();

    // 6. Verify JSON Structure
    let items: Vec<Value> =
        serde_json::from_str(&stdout).expect("Output should be valid JSON array");

    assert_eq!(items.len(), 2, "Should return 2 items");

    let item1 = &items[0];
    assert_eq!(item1["id"], 101);
    assert_eq!(item1["fields"]["System.Title"], "Implement Login");
    assert_eq!(item1["fields"]["System.State"], "Active");
    assert_eq!(item1["fields"]["System.AssignedTo"]["displayName"], "Alice");

    let item2 = &items[1];
    assert_eq!(item2["id"], 102);
    assert_eq!(item2["fields"]["System.Title"], "Fix CSS");
}

#[tokio::test]
async fn test_task_lifecycle_json() {
    let mock_server = MockServer::start().await;

    // Mock GET Work Item 101 for Start
    Mock::given(method("GET"))
        .and(path("/test_proj/_apis/wit/workitems/101"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 101,
            "rev": 1,
            "fields": {
                "System.Title": "Task 101",
                "System.State": "Active",
                "System.WorkItemType": "Task"
            },
            "url": "http://mock/101"
        })))
        .mount(&mock_server)
        .await;

    // Mock 7Pace Current Timer (Return null = no active timer)
    Mock::given(method("GET"))
        .and(path("/_apis/api/tracking/client/current"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::Value::Null))
        .mount(&mock_server)
        .await;

    // Mock 7Pace Start Timer
    Mock::given(method("POST"))
        .and(path("/_apis/api/tracking/client/startTracking"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
             "id": "timer_123",
             "startedAt": "2023-01-01T12:00:00Z",
             "workItemId": 101
        })))
        .mount(&mock_server)
        .await;

    // Mock 7Pace Stop Timer
    Mock::given(method("POST"))
        .and(path("/_apis/api/tracking/client/stopTracking/0")) // Assuming reason 0
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
             "worklogId": 999,
             "duration": 3600,
             "workItemId": 101
        })))
        .mount(&mock_server)
        .await;

    // Config Setup
    let temp_home = tempfile::tempdir().unwrap();
    let config_dir = temp_home.path().join(".ao-no-out7ook");
    fs::create_dir_all(&config_dir).unwrap();
    let config_content = format!(
        r#"
[devops]
organization = "test_org"
project = "test_proj"
api_url = "{}"
pace_api_url = "{}"
pat = "dummy"
use_keyring = false
[graph]
client_id = "dummy"
"#,
        mock_server.uri(),
        mock_server.uri()
    );
    fs::write(config_dir.join("config.toml"), config_content).unwrap();

    // 1. Start Task
    let mut cmd_start = Command::cargo_bin("ano7").unwrap();
    let assert_start = cmd_start
        .env("HOME", temp_home.path())
        .args(&["start", "101", "--format", "json"])
        .assert()
        .success();
    let out_start = assert_start.get_output();
    let json_start: Value = serde_json::from_slice(&out_start.stdout).unwrap();

    assert_eq!(json_start["id"], 101);
    assert_eq!(json_start["title"], "Task 101");
    assert!(json_start["started_at"].is_string());

    // 2. Stop Task
    let mut cmd_stop = Command::cargo_bin("ano7").unwrap();
    let assert_stop = cmd_stop
        .env("HOME", temp_home.path())
        .args(&["stop", "--format", "json"])
        .assert()
        .success();
    let out_stop = assert_stop.get_output();
    let json_stop: Value = serde_json::from_slice(&out_stop.stdout).unwrap();

    assert_eq!(json_stop["id"], 101);
    assert_eq!(json_stop["status"], "stopped");
}
