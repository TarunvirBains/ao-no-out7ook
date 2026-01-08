use ano7::devops::client::DevOpsClient;
use ano7::devops::models::WorkItem;
use tokio;
use wiremock::matchers::{header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_work_item() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test_proj/_apis/wit/workitems/12345"))
        .and(header_exists("Authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 12345,
            "rev": 1,
            "fields": {
                "System.Title": "Mocked Task",
                "System.State": "Active",
                "System.AssignedTo": { "displayName": "Tester" },
                "System.WorkItemType": "Task"
            },
            "url": "http://mock/..."
        })))
        .mount(&mock_server)
        .await;

    // Use mock server base URL and move into blocking task
    let uri = mock_server.uri();

    let result = tokio::task::spawn_blocking(move || {
        let client = DevOpsClient::new("test_pat", "test_org", "test_proj");
        let client = client.with_base_url(&uri);

        client.get_work_item(12345)
    })
    .await
    .expect("Task failed");

    let work_item = result.expect("Failed to fetch work item");

    assert_eq!(work_item.id, 12345);
    assert_eq!(work_item.get_title(), Some("Mocked Task"));
    assert_eq!(work_item.get_state(), Some("Active"));
}
