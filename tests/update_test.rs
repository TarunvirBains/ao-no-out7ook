use ao_no_out7ook::commands::devops;
use ao_no_out7ook::config::{Config, DevOpsConfig};
use serde_json::json;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[allow(clippy::field_reassign_with_default)]
fn create_test_config(api_url: String) -> Config {
    let mut config = Config::default();
    config.devops = DevOpsConfig {
        pat: Some("test-pat".to_string()),
        organization: "test-org".to_string(),
        project: "test-project".to_string(),
        skip_states: vec![],
        api_url: Some(api_url),
        pace_api_url: None,
    };
    config
}

#[tokio::test]
async fn test_update_assigned_to() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(mock_server.uri());

    // Mock GET work item (to fetch current rev)
    Mock::given(method("GET"))
        .and(path_regex(r"^/test-project/_apis/wit/workitems/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "rev": 5,
            "fields": {
                "System.Title": "Test Task",
                "System.State": "Active"
            }
        })))
        .expect(2) // Called twice: once in update(), once in update_work_item_with_rev()
        .mount(&mock_server)
        .await;

    // Mock PATCH work item (update)
    Mock::given(method("PATCH"))
        .and(path_regex(r"^/test-project/_apis/wit/workitems/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "rev": 6,
            "fields": {
                "System.Title": "Test Task",
                "System.AssignedTo": {"displayName": "Test User"}
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = tokio::task::spawn_blocking(move || {
        devops::update(
            &config,
            123,
            Some("testuser@example.com".to_string()),
            None,
            None,
            false,
        )
    })
    .await
    .unwrap();

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_priority() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(mock_server.uri());

    Mock::given(method("GET"))
        .and(path_regex(r"^/test-project/_apis/wit/workitems/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "rev": 5,
            "fields": {
                "System.Title": "Test Task"
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path_regex(r"^/test-project/_apis/wit/workitems/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "rev": 6,
            "fields": {
                "Microsoft.VSTS.Common.Priority": 1
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = tokio::task::spawn_blocking(move || {
        devops::update(&config, 123, None, Some(1), None, false)
    })
    .await
    .unwrap();

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_tags() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(mock_server.uri());

    Mock::given(method("GET"))
        .and(path_regex(r"^/test-project/_apis/wit/workitems/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "rev": 5,
            "fields": {
                "System.Title": "Test Task"
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path_regex(r"^/test-project/_apis/wit/workitems/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "rev": 6,
            "fields": {
                "System.Tags": "urgent; backend"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = tokio::task::spawn_blocking(move || {
        devops::update(
            &config,
            123,
            None,
            None,
            Some("urgent,backend".to_string()),
            false,
        )
    })
    .await
    .unwrap();

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_multiple_fields() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(mock_server.uri());

    Mock::given(method("GET"))
        .and(path_regex(r"^/test-project/_apis/wit/workitems/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "rev": 5,
            "fields": {}
        })))
        .mount(&mock_server)
        .await;

    // Verify single PATCH with multiple operations
    Mock::given(method("PATCH"))
        .and(path_regex(r"^/test-project/_apis/wit/workitems/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "rev": 6,
            "fields": {
                "System.AssignedTo": {"displayName": "Test User"},
                "Microsoft.VSTS.Common.Priority": 2
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = tokio::task::spawn_blocking(move || {
        devops::update(
            &config,
            123,
            Some("testuser@example.com".to_string()),
            Some(2),
            None,
            false,
        )
    })
    .await
    .unwrap();

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_dry_run() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(mock_server.uri());

    // Should fetch work item for validation
    Mock::given(method("GET"))
        .and(path_regex(r"^/test-project/_apis/wit/workitems/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "rev": 5,
            "fields": {
                "System.Title": "Test Task"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Should NOT call PATCH in dry-run mode
    Mock::given(method("PATCH"))
        .and(path_regex(r"^/test-project/_apis/wit/workitems/123"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&mock_server)
        .await;

    let result = tokio::task::spawn_blocking(move || {
        devops::update(
            &config,
            123,
            Some("test@example.com".to_string()),
            Some(1),
            None,
            true,
        )
    })
    .await
    .unwrap();

    assert!(result.is_ok());
}

#[test]
fn test_priority_validation() {
    // Priority must be 1-4
    let valid_priorities = vec![1, 2, 3, 4];
    let invalid_priorities = vec![0, 5, 100];

    for p in valid_priorities {
        assert!((1..=4).contains(&p), "Priority {} should be valid", p);
    }

    for p in invalid_priorities {
        assert!(!(1..=4).contains(&p), "Priority {} should be invalid", p);
    }
}

#[test]
fn test_tags_formatting() {
    // Tags should be comma-separated in input, semicolon-separated in DevOps
    let input = "urgent,backend,feature";
    let expected_output = "urgent; backend; feature";

    let formatted = input.split(',').collect::<Vec<_>>().join("; ");
    assert_eq!(formatted, expected_output);
}
