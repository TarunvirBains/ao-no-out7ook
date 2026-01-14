use ao_no_out7ook::graph::models::{CalendarEvent, DateTimeTimeZone, ItemBody};
use chrono::Utc;

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
    assert!(json.contains("Focus Block"));
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
    assert_eq!(event.categories, vec!["Meeting"]);
}

#[test]
fn test_datetime_timezone_formatting() {
    let now = Utc::now();
    let dt_tz = DateTimeTimeZone::from_utc(now, "America/Los_Angeles");

    assert_eq!(dt_tz.time_zone, "America/Los_Angeles");
    assert!(!dt_tz.date_time.is_empty());
    // Date format should be ISO 8601
    assert!(dt_tz.date_time.contains("T"));
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

#[test]
fn test_calendar_event_with_html_body() {
    let start = Utc::now();
    let end = start + chrono::Duration::minutes(30);

    let event = CalendarEvent {
        id: Some("html-event".to_string()),
        subject: "HTML Event".to_string(),
        start: DateTimeTimeZone::from_utc(start, "UTC"),
        end: DateTimeTimeZone::from_utc(end, "UTC"),
        body: Some(ItemBody {
            content_type: "html".to_string(),
            content: "<p>This is <strong>HTML</strong> content</p>".to_string(),
        }),
        categories: vec!["Work".to_string()],
        extended_properties: None,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("html"));
    assert!(json.contains("<p>"));
}

#[test]
fn test_calendar_event_multiple_categories() {
    let start = Utc::now();
    let end = start + chrono::Duration::hours(2);

    let event = CalendarEvent {
        id: None,
        subject: "Multi-Category Event".to_string(),
        start: DateTimeTimeZone::from_utc(start, "UTC"),
        end: DateTimeTimeZone::from_utc(end, "UTC"),
        body: None,
        categories: vec![
            "Focus Block".to_string(),
            "Deep Work".to_string(),
            "Priority".to_string(),
        ],
        extended_properties: None,
    };

    assert_eq!(event.categories.len(), 3);
    assert!(event.categories.contains(&"Deep Work".to_string()));
}
