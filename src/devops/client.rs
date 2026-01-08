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

    pub fn get_work_item_type(
        &self,
        type_name: &str,
    ) -> Result<crate::devops::models::WorkItemType> {
        let url = format!(
            "{}/{}/_apis/wit/workitemtypes/{}?api-version=7.0",
            self.base_url, self.project, type_name
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .context("Failed to fetch work item type definition")?;

        if !response.status().is_success() {
            anyhow::bail!("WorkItemType API error: status {}", response.status());
        }

        let type_def = response
            .json::<crate::devops::models::WorkItemType>()
            .context("Failed to parse WorkItemType")?;

        Ok(type_def)
    }

    pub fn get_work_item(&self, id: u32) -> Result<WorkItem> {
        // GET https://dev.azure.com/{org}/{project}/_apis/wit/workitems/{id}?api-version=7.0
        let url = format!(
            "{}/{}/_apis/wit/workitems/{}?$expand=all&api-version=7.0",
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

    pub fn get_work_items_batch(&self, ids: &[u32]) -> Result<Vec<WorkItem>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Use POST /wit/workitemsbatch per Azure DevOps API spec
        let url = format!(
            "{}/{}/_apis/wit/workitemsbatch?api-version=7.0",
            self.base_url, self.project
        );

        let body = serde_json::json!({
            "ids": ids,
            "$expand": "all"
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .context("Failed to batch fetch work items")?;

        if !response.status().is_success() {
            anyhow::bail!("DevOps Batch API error: status {}", response.status());
        }

        // Response is { "count": N, "value": [ ... ] }
        let json_val = response.json::<serde_json::Value>()?;
        let items_val = json_val
            .get("value")
            .context("Batch response missing 'value' field")?;

        let items: Vec<WorkItem> = serde_json::from_value(items_val.clone())
            .context("Failed to deserialize batch work items")?;

        Ok(items)
    }

    pub fn execute_wiql(&self, query: &str) -> Result<crate::devops::models::WiqlResponse> {
        let url = format!(
            "{}/{}/_apis/wit/wiql?api-version=7.0",
            self.base_url, self.project
        );

        let body = serde_json::json!({ "query": query });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .context("Failed to execute WIQL")?;

        if !response.status().is_success() {
            anyhow::bail!("WIQL API error: status {}", response.status());
        }

        let wiql_resp = response
            .json::<crate::devops::models::WiqlResponse>()
            .context("Failed to parse WiqlResponse")?;

        Ok(wiql_resp)
    }

    pub fn update_work_item(
        &self,
        id: u32,
        operations: Vec<serde_json::Value>,
    ) -> Result<WorkItem> {
        self.update_work_item_with_rev(id, operations, None)
    }

    /// Create a new work item
    pub fn create_work_item(
        &self,
        fields: serde_json::Map<String, serde_json::Value>,
    ) -> Result<WorkItem> {
        // Extract work item type from fields
        let work_item_type = fields
            .get("System.WorkItemType")
            .and_then(|v| v.as_str())
            .unwrap_or("Task");

        let url = format!(
            "{}/{}/_apis/wit/workitems/${}?api-version=7.1",
            self.base_url, self.project, work_item_type
        );

        // Build JSON Patch document for creation
        let mut operations = Vec::new();
        for (key, value) in fields {
            operations.push(serde_json::json!({
                "op": "add",
                "path": format!("/fields/{}", key),
                "value": value
            }));
        }

        let response = self
            .client
            .post(&url)
            .basic_auth("", Some(&self.pat))
            .header("Content-Type", "application/json-patch+json")
            .json(&operations)
            .send()
            .context("Failed to send create work item request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            anyhow::bail!("Create work item failed ({}): {}", status, body);
        }

        response
            .json::<WorkItem>()
            .context("Failed to parse created work item")
    }

    pub fn update_work_item_with_rev(
        &self,
        id: u32,
        operations: Vec<serde_json::Value>,
        expected_rev: Option<u32>,
    ) -> Result<WorkItem> {
        // If expected_rev provided, verify current revision matches (FR1.8 conflict detection)
        if let Some(expected) = expected_rev {
            let current = self.get_work_item(id)?;
            if current.rev != expected {
                anyhow::bail!(
                    "Conflict detected: Work item {} has been modified (expected rev {}, current rev {}). \
                     Fetch latest and retry.",
                    id,
                    expected,
                    current.rev
                );
            }
        }

        let url = format!(
            "{}/{}/_apis/wit/workitems/{}?api-version=7.0",
            self.base_url, self.project, id
        );

        let response = self
            .client
            .patch(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json-patch+json")
            .json(&operations)
            .send()
            .context("Failed to update work item")?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_default();
            anyhow::bail!("Update API error: {}. details: {}", id, error_text);
        }

        let work_item = response
            .json::<WorkItem>()
            .context("Failed to parse updated WorkItem")?;

        Ok(work_item)
    }
}
