use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Microsoft Graph calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub subject: String,
    pub start: DateTimeTimeZone,
    pub end: DateTimeTimeZone,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<ItemBody>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(
        rename = "singleValueExtendedProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub extended_properties: Option<Vec<ExtendedProperty>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateTimeTimeZone {
    #[serde(rename = "dateTime")]
    pub date_time: String, // ISO 8601 format
    #[serde(rename = "timeZone")]
    pub time_zone: String,
}

impl DateTimeTimeZone {
    pub fn from_utc(dt: DateTime<Utc>, tz: &str) -> Self {
        Self {
            date_time: dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
            time_zone: tz.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemBody {
    #[serde(rename = "contentType")]
    pub content_type: String, // "text" or "html"
    pub content: String,
}

/// Extended property for storing work_item_id in calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedProperty {
    pub id: String,
    pub value: String,
}

/// Response from Graph API list events
#[derive(Debug, Deserialize)]
pub struct EventsResponse {
    pub value: Vec<CalendarEvent>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_calendar_event() {
        let json = serde_json::json!({
            "id": "evt-123",
            "subject": "Focus Block",
            "start": {
                "dateTime": "2026-01-08T09:00:00",
                "timeZone": "America/Los_Angeles"
            },
            "end": {
                "dateTime": "2026-01-08T10:00:00",
                "timeZone": "America/Los_Angeles"
            },
            "categories": ["Focus"],
        });

        let event: CalendarEvent = serde_json::from_value(json).unwrap();
        assert_eq!(event.subject, "Focus Block");
        assert_eq!(event.start.time_zone, "America/Los_Angeles");
    }

    #[test]
    fn test_serialize_calendar_event() {
        let event = CalendarEvent {
            id: None,
            subject: "Test Event".to_string(),
            start: DateTimeTimeZone {
                date_time: "2026-01-08T09:00:00".to_string(),
                time_zone: "UTC".to_string(),
            },
            end: DateTimeTimeZone {
                date_time: "2026-01-08T10:00:00".to_string(),
                time_zone: "UTC".to_string(),
            },
            body: None,
            categories: vec![],
            extended_properties: None,
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["subject"], "Test Event");
        assert!(json.get("id").is_none()); // Should be skipped
    }
}
