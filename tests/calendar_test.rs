use ao_no_out7ook::config::{Config, DevOpsConfig, GraphConfig};
use ao_no_out7ook::graph::models::{CalendarEvent, DateTimeTimeZone, ItemBody};
use chrono::Utc;

fn create_test_config() -> Config {
    let mut config = Config::default();
    config.devops = DevOpsConfig {
        pat: Some("test-pat".to_string()),
        organization: "test-org".to_string(),
        project: "test-project".to_string(),
        skip_states: vec![],
    };
    config.graph = GraphConfig {
        client_id: "test-client-id".to_string(),
        tenant_id: "common".to_string(),
    };
    config
}

#[tokio::test]
#[ignore] // Requires actual Graph API or extensive mocking
async fn test_calendar_schedule_creates_event() {
    let _config = create_test_config();

    // This test would require:
    // 1. Mocking GraphClient
    // 2. Mocking DevOpsClient for work item fetch
    // 3. Verifying event creation

    // For now, we document the expected behavior
    assert!(true); // Placeholder
}

#[tokio::test]
#[ignore] // Requires Graph API mocking
async fn test_calendar_list_filters_by_work_item() {
    let _config = create_test_config();

    // This test would verify filtering logic
    // Currently requires extensive mocking setup
    assert!(true); // Placeholder
}

#[test]
fn test_calendar_event_model_serialization() {
    let start = Utc::now();
    let end = start + chrono::Duration::minutes(45);

    let event = CalendarEvent {
        id: Some("test-id".to_string()),
        subject: "Test Event".to_string(),
        start: DateTimeTimeZone::from_utc(start, "UTC"),
        end: DateTimeTimeZone::from_utc(end, "UTC"),
        body: Some(ItemBody {
            content_type: "text".to_string(),
            content: "Test body".to_string(),
        }),
        categories: vec!["Focus Block".to_string()],
        extended_properties: None,
    };

    // Verify serialization works
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("Test Event"));
}

#[test]
fn test_calendar_event_deserialize() {
    let json = r#"{
        "id": "event-123",
        "subject": "Meeting",
        "start": {
            "dateTime": "2026-01-01T10:00:00",
            "timeZone": "UTC"
        },
        "end": {
            "dateTime": "2026-01-01T11:00:00",
            "timeZone": "UTC"
        },
        "categories": ["Meeting"]
    }"#;

    let result: Result<CalendarEvent, _> = serde_json::from_str(json);
    assert!(result.is_ok());

    let event = result.unwrap();
    assert_eq!(event.id, Some("event-123".to_string()));
    assert_eq!(event.subject, "Meeting");
}

#[test]
fn test_datetime_timezone_formatting() {
    let now = Utc::now();
    let dt_tz = DateTimeTimeZone::from_utc(now, "America/Los_Angeles");

    assert_eq!(dt_tz.time_zone, "America/Los_Angeles");
    assert!(!dt_tz.date_time.is_empty());
}

#[test]
fn test_calendar_event_minimal_fields() {
    let start = Utc::now();
    let end = start + chrono::Duration::hours(1);

    let event = CalendarEvent {
        id: None,
        subject: "Minimal Event".to_string(),
        start: DateTimeTimeZone::from_utc(start, "UTC"),
        end: DateTimeTimeZone::from_utc(end, "UTC"),
        body: None,
        categories: vec![],
        extended_properties: None,
    };

    // Should serialize without errors
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("Minimal Event"));
}
