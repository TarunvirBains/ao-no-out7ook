use crate::devops::models::WorkItem;
use anyhow::{Context, Result};
use base64::prelude::*;
use reqwest::blocking::Client;

pub struct DevOpsClient {
    client: Client,
    base_url: String, // https://dev.azure.com/{org}
    project: String,
    pat: String,
}

impl DevOpsClient {
    pub fn new(pat: &str, org: &str, project: &str) -> Self {
        let base_url = format!("https://dev.azure.com/{}", org);
        Self {
            client: Client::new(),
            base_url,
            project: project.to_string(),
            pat: pat.to_string(),
        }
    }

    /// Helper for testing to override base URL (e.g. wiremock)
    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.trim_end_matches('/').to_string();
        self
    }

    fn auth_header(&self) -> String {
        let val = format!(":{}", self.pat);
        format!("Basic {}", BASE64_STANDARD.encode(val))
    }

    pub fn get_work_item(&self, id: u32) -> Result<WorkItem> {
        // GET https://dev.azure.com/{org}/{project}/_apis/wit/workitems/{id}?api-version=7.0
        let url = format!(
            "{}/{}/_apis/wit/workitems/{}?api-version=7.0",
            self.base_url, self.project, id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .context("Failed to send request to DevOps REST API")?;

        if !response.status().is_success() {
            anyhow::bail!("DevOps API error: status {}", response.status());
        }

        let work_item = response
            .json::<WorkItem>()
            .context("Failed to parse WorkItem JSON response")?;

        Ok(work_item)
    }
}
