use crate::pace::models::{
    CreateWorklogRequest, StartTimerRequest, StopTimerResponse, Timer, Worklog,
};
use anyhow::{Context, Result};
use base64::prelude::*;
use chrono::{DateTime, Utc};
use reqwest::blocking::Client;

pub struct PaceClient {
    client: Client,
    base_url: String,
    organization: String,
    pat: String,
}

impl PaceClient {
    pub fn new(pat: &str, organization: &str) -> Self {
        let base_url = format!("https://api.timehub.7pace.com/{}", organization);
        Self {
            client: Client::new(),
            base_url,
            organization: organization.to_string(),
            pat: pat.to_string(),
        }
    }

    /// Helper for testing to override base URL
    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.trim_end_matches('/').to_string();
        self
    }

    fn auth_header(&self) -> String {
        let val = format!(":{}", self.pat);
        format!("Basic {}", BASE64_STANDARD.encode(val))
    }

    /// FR2.1: Start timer for a work item
    pub fn start_timer(&self, work_item_id: u32, comment: Option<String>) -> Result<Timer> {
        let url = format!("{}/_apis/api/tracking/client/startTracking", self.base_url);

        let request_body = StartTimerRequest {
            work_item_id,
            comment,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .context("Failed to start timer")?;

        if !response.status().is_success() {
            anyhow::bail!("7Pace start timer API error: status {}", response.status());
        }

        let timer = response
            .json::<Timer>()
            .context("Failed to parse Timer response")?;

        Ok(timer)
    }

    /// FR2.2: Stop active timer
    pub fn stop_timer(&self, reason: u8) -> Result<StopTimerResponse> {
        let url = format!(
            "{}/_apis/api/tracking/client/stopTracking/{}",
            self.base_url, reason
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .send()
            .context("Failed to stop timer")?;

        if !response.status().is_success() {
            anyhow::bail!("7Pace stop timer API error: status {}", response.status());
        }

        let stop_response = response
            .json::<StopTimerResponse>()
            .context("Failed to parse StopTimerResponse")?;

        Ok(stop_response)
    }

    /// FR2.3: Get current active timer
    pub fn get_current_timer(&self) -> Result<Option<Timer>> {
        let url = format!("{}/_apis/api/tracking/client/current", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .context("Failed to get current timer")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "7Pace get current timer API error: status {}",
                response.status()
            );
        }

        // API returns null if no timer active
        let timer_opt = response
            .json::<Option<Timer>>()
            .context("Failed to parse current timer response")?;

        Ok(timer_opt)
    }

    /// FR2.5: Create manual worklog entry
    pub fn create_worklog(
        &self,
        work_item_id: u32,
        duration_secs: u32,
        comment: Option<String>,
    ) -> Result<Worklog> {
        let url = format!("{}/_apis/worklogs", self.base_url);

        let request_body = CreateWorklogRequest {
            work_item_id,
            duration: duration_secs,
            timestamp: Utc::now(),
            comment,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .context("Failed to create worklog")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "7Pace create worklog API error: status {}",
                response.status()
            );
        }

        let worklog = response
            .json::<Worklog>()
            .context("Failed to parse Worklog response")?;

        Ok(worklog)
    }

    /// FR2.6: Fetch worklogs for reconciliation
    pub fn get_worklogs(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<Worklog>> {
        let url = format!(
            "{}/_apis/worklogs?startDate={}&endDate={}",
            self.base_url,
            start_date.to_rfc3339(),
            end_date.to_rfc3339()
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .context("Failed to fetch worklogs")?;

        if !response.status().is_success() {
            anyhow::bail!("7Pace get worklogs API error: status {}", response.status());
        }

        let worklogs = response
            .json::<Vec<Worklog>>()
            .context("Failed to parse worklogs response")?;

        Ok(worklogs)
    }
}
