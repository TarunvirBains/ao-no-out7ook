use anyhow::Result;
use crate::graph::models::CalendarEvent;
use chrono::{DateTime, Utc};

pub struct GraphClient;

impl GraphClient {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn list_events(&self, _start: DateTime<Utc>, _end: DateTime<Utc>) -> Result<Vec<CalendarEvent>> {
        todo!("List calendar events to be implemented")
    }
    
    pub async fn create_event(&self, _event: CalendarEvent) -> Result<CalendarEvent> {
        todo!("Create calendar event to be implemented")
    }
    
    pub async fn update_event(&self, _event_id: &str, _event: CalendarEvent) -> Result<CalendarEvent> {
        todo!("Update calendar event to be implemented")
    }
    
    pub async fn delete_event(&self, _event_id: &str) -> Result<()> {
        todo!("Delete calendar event to be implemented")
    }
}
