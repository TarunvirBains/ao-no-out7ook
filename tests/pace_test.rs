use ao_no_out7ook::pace::client::PaceClient;
use ao_no_out7ook::pace::models::{StopTimerResponse, Timer, Worklog};
use chrono::Utc;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_start_timer_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/_apis/api/tracking/client/startTracking"))
        .and(header("Authorization", "Basic OlRFU1RfUEFU"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "timer-abc-123",
            "workItemId": 456,
            "startedAt": "2026-01-07T18:00:00Z",
            "comment": "Working on feature"
        })))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let timer = tokio::task::spawn_blocking(move || {
        let client = PaceClient::new("TEST_PAT", "test-org").with_base_url(&uri);
        client.start_timer(456, Some("Working on feature".to_string()))
    })
    .await
    .unwrap()
    .unwrap();

    assert_eq!(timer.id, "timer-abc-123");
    assert_eq!(timer.work_item_id, 456);
    assert_eq!(timer.comment, Some("Working on feature".to_string()));
}

#[tokio::test]
async fn test_stop_timer_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/_apis/api/tracking/client/stopTracking/0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "worklogId": 789,
            "duration": 3600,
            "workItemId": 456
        })))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let response = tokio::task::spawn_blocking(move || {
        let client = PaceClient::new("TEST_PAT", "test-org").with_base_url(&uri);
        client.stop_timer(0)
    })
    .await
    .unwrap()
    .unwrap();

    assert_eq!(response.worklog_id, 789);
    assert_eq!(response.duration, 3600);
    assert_eq!(response.work_item_id, 456);
}

#[tokio::test]
async fn test_get_current_timer_active() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/_apis/api/tracking/client/current"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "timer-current",
            "workItemId": 123,
            "startedAt": "2026-01-07T17:00:00Z",
            "comment": null
        })))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let timer = tokio::task::spawn_blocking(move || {
        let client = PaceClient::new("TEST_PAT", "test-org").with_base_url(&uri);
        client.get_current_timer()
    })
    .await
    .unwrap()
    .unwrap();

    assert!(timer.is_some());
    let timer = timer.unwrap();
    assert_eq!(timer.id, "timer-current");
    assert_eq!(timer.work_item_id, 123);
}

#[tokio::test]
async fn test_get_current_timer_none() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/_apis/api/tracking/client/current"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(null)))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let timer = tokio::task::spawn_blocking(move || {
        let client = PaceClient::new("TEST_PAT", "test-org").with_base_url(&uri);
        client.get_current_timer()
    })
    .await
    .unwrap()
    .unwrap();

    assert!(timer.is_none());
}

#[tokio::test]
async fn test_create_worklog_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/_apis/worklogs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 999,
            "workItemId": 123,
            "userId": "user-123",
            "duration": 7200,
            "timestamp": "2026-01-07T18:00:00Z",
            "comment": "Manual entry"
        })))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let worklog = tokio::task::spawn_blocking(move || {
        let client = PaceClient::new("TEST_PAT", "test-org").with_base_url(&uri);
        client.create_worklog(123, 7200, Some("Manual entry".to_string()))
    })
    .await
    .unwrap()
    .unwrap();

    assert_eq!(worklog.id, 999);
    assert_eq!(worklog.work_item_id, 123);
    assert_eq!(worklog.duration, 7200);
}

#[tokio::test]
async fn test_get_worklogs_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/_apis/worklogs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": 1,
                "workItemId": 100,
                "userId": "user-1",
                "duration": 1800,
                "timestamp": "2026-01-07T10:00:00Z",
                "comment": "Morning work"
            },
            {
                "id": 2,
                "workItemId": 101,
                "userId": "user-1",
                "duration": 3600,
                "timestamp": "2026-01-07T14:00:00Z",
                "comment": null
            }
        ])))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let worklogs = tokio::task::spawn_blocking(move || {
        let client = PaceClient::new("TEST_PAT", "test-org").with_base_url(&uri);
        let start = Utc::now() - chrono::Duration::days(7);
        let end = Utc::now();
        client.get_worklogs(start, end)
    })
    .await
    .unwrap()
    .unwrap();

    assert_eq!(worklogs.len(), 2);
    assert_eq!(worklogs[0].id, 1);
    assert_eq!(worklogs[0].duration, 1800);
    assert_eq!(worklogs[1].id, 2);
    assert_eq!(worklogs[1].duration, 3600);
}
