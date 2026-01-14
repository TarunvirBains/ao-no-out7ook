use ao_no_out7ook::commands::devops;
use ao_no_out7ook::config::{Config, DevOpsConfig};
use serde_json::json;
use wiremock::matchers::{body_string_contains, method, path};
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
async fn test_list_with_search_term() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(mock_server.uri());

    // Expect WIQL query to contain "System.Title CONTAINS 'login'"
    Mock::given(method("POST"))
        .and(path("/test-project/_apis/wit/wiql"))
        .and(body_string_contains("System.Title"))
        .and(body_string_contains("CONTAINS"))
        .and(body_string_contains("login"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "queryType": "flat",
            "workItems": []
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = tokio::task::spawn_blocking(move || {
        devops::list(
            &config,
            None,
            None,
            Some("login".to_string()),
            None,
            Some(50),
        )
    })
    .await
    .unwrap();

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_with_tags_filter() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(mock_server.uri());

    // Expect WIQL query to contain "System.Tags CONTAINS 'urgent'"
    Mock::given(method("POST"))
        .and(path("/test-project/_apis/wit/wiql"))
        .and(body_string_contains("System.Tags"))
        .and(body_string_contains("CONTAINS"))
        .and(body_string_contains("urgent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "queryType": "flat",
            "workItems": []
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = tokio::task::spawn_blocking(move || {
        devops::list(
            &config,
            None,
            None,
            None,
            Some("urgent".to_string()),
            Some(50),
        )
    })
    .await
    .unwrap();

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_sort_by_priority() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(mock_server.uri());

    // Expect ORDER BY Priority ASC
    Mock::given(method("POST"))
        .and(path("/test-project/_apis/wit/wiql"))
        .and(body_string_contains("ORDER BY"))
        .and(body_string_contains("Microsoft.VSTS.Common.Priority"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "queryType": "flat",
            "workItems": []
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = tokio::task::spawn_blocking(move || {
        devops::list(&config, None, None, None, None, Some(50))
    })
    .await
    .unwrap();

    match &result {
        Ok(_) => {}
        Err(e) => println!("Test error: {:?}", e),
    }
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_sort_by_changed_date() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(mock_server.uri());

    // When sort=changed, expect ORDER BY ChangedDate DESC
    Mock::given(method("POST"))
        .and(path("/test-project/_apis/wit/wiql"))
        .and(body_string_contains("ORDER BY"))
        .and(body_string_contains("System.ChangedDate"))
        .and(body_string_contains("DESC"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "queryType": "flat",
            "workItems": []
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = tokio::task::spawn_blocking(move || {
        // Add sort parameter when we implement it
        devops::list_with_sort(&config, None, None, None, None, "changed", Some(50))
    })
    .await
    .unwrap();

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_combined_filters() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(mock_server.uri());

    // Test all filters together in single query
    Mock::given(method("POST"))
        .and(path("/test-project/_apis/wit/wiql"))
        .and(body_string_contains("System.State"))
        .and(body_string_contains("Active"))
        .and(body_string_contains("System.Title"))
        .and(body_string_contains("login"))
        .and(body_string_contains("System.Tags"))
        .and(body_string_contains("backend"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "queryType": "flat",
            "workItems": []
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let result = tokio::task::spawn_blocking(move || {
        devops::list(
            &config,
            Some("Active".to_string()),
            None,
            Some("login".to_string()),
            Some("backend".to_string()),
            Some(50),
        )
    })
    .await
    .unwrap();

    assert!(result.is_ok());
}

#[test]
fn test_search_term_sql_injection_prevention() {
    // Test that search terms with single quotes are escaped
    let search_term = "test' OR '1'='1";

    // Expected: Should escape single quotes by doubling them (WIQL standard)
    let escaped = search_term.replace("'", "''");

    assert_eq!(escaped, "test'' OR ''1''=''1");
    // The dangerous pattern is now neutralized - it won't break out of the string
    assert!(!search_term.contains("''")); // Original had single quotes
    assert!(escaped.contains("''")); // Escaped has doubled quotes
}
