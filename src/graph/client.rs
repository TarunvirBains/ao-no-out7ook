use crate::graph::auth::GraphAuthenticator;
use crate::graph::models::{CalendarEvent, EventsResponse};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::sync::Arc;

pub struct GraphClient {
    client: Client,
    auth: Arc<GraphAuthenticator>,
}

impl GraphClient {
    pub fn new(auth: GraphAuthenticator) -> Self {
        Self {
            client: Client::new(),
            auth: Arc::new(auth),
        }
    }

    async fn auth_header(&self) -> Result<String> {
        let token = self.auth.get_access_token().await?;
        Ok(format!("Bearer {}", token))
    }

    /// FR3.1: List calendar events in time range
    pub async fn list_events(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>> {
        let url = format!(
            "https://graph.microsoft.com/v1.0/me/calendar/events?\
             $filter=start/dateTime ge '{}' and end/dateTime le '{}'&\
             $select=id,subject,start,end,categories,singleValueExtendedProperties",
            start.to_rfc3339(),
            end.to_rfc3339()
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header().await?)
            .send()
            .await
            .context("Failed to list calendar events")?;

        if !response.status().is_success() {
            anyhow::bail!("Graph API error: status {}", response.status());
        }

        let events_response: EventsResponse = response
            .json()
            .await
            .context("Failed to parse events response")?;

        Ok(events_response.value)
    }

    /// FR3.2: Create calendar event (Focus Block)
    pub async fn create_event(&self, event: CalendarEvent) -> Result<CalendarEvent> {
        let url = "https://graph.microsoft.com/v1.0/me/calendar/events";

        let response = self
            .client
            .post(url)
            .header("Authorization", self.auth_header().await?)
            .header("Content-Type", "application/json")
            .json(&event)
            .send()
            .await
            .context("Failed to create calendar event")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Graph API create event error: Status: {}, Body: {}",
                status,
                body
            );
        }

        let created: CalendarEvent = response
            .json()
            .await
            .context("Failed to parse created event")?;

        Ok(created)
    }

    /// FR3.4: Update calendar event
    pub async fn update_event(
        &self,
        event_id: &str,
        event: CalendarEvent,
    ) -> Result<CalendarEvent> {
        let url = format!("https://graph.microsoft.com/v1.0/me/events/{}", event_id);

        let response = self
            .client
            .patch(&url)
            .header("Authorization", self.auth_header().await?)
            .header("Content-Type", "application/json")
            .json(&event)
            .send()
            .await
            .context("Failed to update calendar event")?;

        if !response.status().is_success() {
            anyhow::bail!("Graph API update event error: status {}", response.status());
        }

        let updated: CalendarEvent = response
            .json()
            .await
            .context("Failed to parse updated event")?;

        Ok(updated)
    }

    /// Delete calendar event
    pub async fn delete_event(&self, event_id: &str) -> Result<()> {
        let url = format!("https://graph.microsoft.com/v1.0/me/events/{}", event_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", self.auth_header().await?)
            .send()
            .await
            .context("Failed to delete calendar event")?;

        if !response.status().is_success() {
            anyhow::bail!("Graph API delete event error: status {}", response.status());
        }

        Ok(())
    }
}
