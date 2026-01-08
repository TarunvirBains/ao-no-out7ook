use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkItem {
    pub id: u32,
    pub rev: u32,
    pub fields: HashMap<String, Value>,
    #[serde(default)]
    pub relations: Option<Vec<WorkItemRelation>>,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkItemRelation {
    pub rel: String,
    pub url: String,
    pub attributes: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WiqlResponse {
    #[serde(rename = "queryType")]
    pub query_type: String,
    #[serde(rename = "workItems")]
    pub work_items: Vec<WorkItemReference>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkItemReference {
    pub id: u32,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkItemUpdate {
    pub id: u32,
    pub rev: u32,
    pub fields: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkItemType {
    pub name: String,
    pub states: Vec<WorkItemStateColor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkItemStateColor {
    pub name: String,
    pub color: String,
    pub category: String,
}

// Helper to access common fields easily
impl WorkItem {
    pub fn get_title(&self) -> Option<&str> {
        self.fields.get("System.Title").and_then(|v| v.as_str())
    }

    pub fn get_state(&self) -> Option<&str> {
        self.fields.get("System.State").and_then(|v| v.as_str())
    }

    pub fn get_assigned_to(&self) -> Option<&str> {
        self.fields
            .get("System.AssignedTo")
            .and_then(|v| v.get("displayName"))
            .and_then(|v| v.as_str())
    }

    pub fn get_type(&self) -> Option<&str> {
        self.fields
            .get("System.WorkItemType")
            .and_then(|v| v.as_str())
    }

    pub fn get_work_item_type(&self) -> Option<String> {
        self.get_type().map(|s| s.to_string())
    }

    pub fn get_parent_id(&self) -> Option<u32> {
        self.relations
            .as_ref()?
            .iter()
            .find(|r| r.rel == "System.LinkTypes.Hierarchy-Reverse")
            .and_then(|r| r.url.split('/').last())
            .and_then(|id_str| id_str.parse().ok())
    }

    pub fn get_tags(&self) -> Option<Vec<String>> {
        self.fields
            .get("System.Tags")
            .and_then(|v| v.as_str())
            .map(|tags_str| {
                tags_str
                    .split(';')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
    }

    pub fn get_description(&self) -> Option<&str> {
        self.fields
            .get("System.Description")
            .and_then(|v| v.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_work_item() {
        let json_data = json!({
            "id": 12345,
            "rev": 1,
            "fields": {
                "System.Title": "Implement login",
                "System.State": "Active",
                "System.AssignedTo": {
                    "displayName": "John Doe",
                    "id": "uuid"
                },
                "System.WorkItemType": "User Story"
            },
            "url": "https://dev.azure.com/..."
        });

        let work_item: WorkItem = serde_json::from_value(json_data).expect("Failed to parse");

        assert_eq!(work_item.id, 12345);
        assert_eq!(work_item.get_title(), Some("Implement login"));
        assert_eq!(work_item.get_state(), Some("Active"));
        assert_eq!(work_item.get_assigned_to(), Some("John Doe"));
        assert_eq!(work_item.get_type(), Some("User Story"));
    }

    #[test]
    fn test_deserialize_wiql_response() {
        let json_data = json!({
            "queryType": "flat",
            "workItems": [
                {"id": 1, "url": "http://..."},
                {"id": 2, "url": "http://..."}
            ]
        });

        let response: WiqlResponse =
            serde_json::from_value(json_data).expect("Failed to parse WIQL");
        assert_eq!(response.work_items.len(), 2);
        assert_eq!(response.work_items[0].id, 1);
    }
}
