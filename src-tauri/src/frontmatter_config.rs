// Frontmatter configuration loading for custom fields

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrontmatterConfig {
    pub version: String,
    pub preview_image_field: Option<String>,
    #[serde(default)]
    pub custom_fields: Vec<FrontmatterField>,
    #[serde(default)]
    pub field_groups: Vec<FrontmatterFieldGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrontmatterField {
    pub name: String,
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub field_type: String,
    pub description: Option<String>,
    pub ui: Option<FrontmatterFieldUi>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrontmatterFieldUi {
    pub placeholder: Option<String>,
    pub rows: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrontmatterFieldGroup {
    pub name: String,
    pub label: Option<String>,
    pub fields: Vec<String>,
    pub collapsed: Option<bool>,
}

impl Default for FrontmatterConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            preview_image_field: None,
            custom_fields: Vec::new(),
            field_groups: Vec::new(),
        }
    }
}

pub fn load_frontmatter_config(project_path: &Path) -> Result<FrontmatterConfig, String> {
    let config_path = project_path
        .join(".hex-tool")
        .join("frontmatter-config.json");

    if !config_path.exists() {
        return Ok(FrontmatterConfig::default());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read frontmatter config: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse frontmatter config: {}", e))
}
