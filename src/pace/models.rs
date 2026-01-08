use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Timer response from 7Pace API when starting tracking
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Timer {
    pub id: String,
    #[serde(rename = "workItemId")]
    pub work_item_id: u32,
    #[serde(rename = "startedAt")]
    pub started_at: DateTime<Utc>,
    pub comment: Option<String>,
}

/// Worklog entry (time log) from 7Pace
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Worklog {
    pub id: u32,
    #[serde(rename = "workItemId")]
    pub work_item_id: u32,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub duration: u32, // seconds
    pub timestamp: DateTime<Utc>,
    pub comment: Option<String>,
}

/// Request body for starting a timer
#[derive(Debug, Serialize)]
pub struct StartTimerRequest {
    #[serde(rename = "workItemId")]
    pub work_item_id: u32,
    pub comment: Option<String>,
}

/// Response from stopping a timer
#[derive(Debug, Deserialize)]
pub struct StopTimerResponse {
    #[serde(rename = "worklogId")]
    pub worklog_id: u32,
    pub duration: u32, // seconds
    #[serde(rename = "workItemId")]
    pub work_item_id: u32,
}

/// Request body for creating a manual worklog
#[derive(Debug, Serialize)]
pub struct CreateWorklogRequest {
    #[serde(rename = "workItemId")]
    pub work_item_id: u32,
    pub duration: u32, // seconds
    pub timestamp: DateTime<Utc>,
    pub comment: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_timer() {
        let json = json!({
            "id": "timer-abc-123",
            "workItemId": 456,
            "startedAt": "2026-01-07T18:00:00Z",
            "comment": "Working on feature"
        });

        let timer: Timer = serde_json::from_value(json).unwrap();
        assert_eq!(timer.id, "timer-abc-123");
        assert_eq!(timer.work_item_id, 456);
        assert_eq!(timer.comment, Some("Working on feature".to_string()));
    }

    #[test]
    fn test_deserialize_stop_response() {
        let json = json!({
            "worklogId": 789,
            "duration": 3600,
            "workItemId": 456
        });

        let response: StopTimerResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.worklog_id, 789);
        assert_eq!(response.duration, 3600);
        assert_eq!(response.work_item_id, 456);
    }

    #[test]
    fn test_serialize_start_request() {
        let request = StartTimerRequest {
            work_item_id: 123,
            comment: Some("Test comment".to_string()),
        };

        let json = serde_json::to_value(request).unwrap();
        assert_eq!(json["workItemId"], 123);
        assert_eq!(json["comment"], "Test comment");
    }
}
