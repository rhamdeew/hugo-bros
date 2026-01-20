// Frontmatter configuration loading for custom fields

use crate::hugo::HugoProject;
use crate::markdown::MarkdownDocument;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrontmatterConfig {
    pub version: String,
    pub preview_image_field: Option<String>,
    #[serde(default)]
    pub is_default: bool,
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
            is_default: true,
            custom_fields: Vec::new(),
            field_groups: Vec::new(),
        }
    }
}

pub fn load_frontmatter_config(project_path: &Path) -> Result<FrontmatterConfig, String> {
    let config_path = project_path
        .join(".hugo-bros")
        .join("frontmatter-config.json");

    if !config_path.exists() {
        return Ok(FrontmatterConfig::default());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read frontmatter config: {}", e))?;

    let mut config: FrontmatterConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse frontmatter config: {}", e))?;
    config.is_default = false;
    Ok(config)
}

pub fn generate_frontmatter_config(project_path: &Path) -> Result<FrontmatterConfig, String> {
    let project = HugoProject::new(project_path.to_path_buf());
    let posts_dir = project.get_posts_dir();

    let mut stats: HashMap<String, FieldStats> = HashMap::new();

    if posts_dir.exists() {
        for entry in WalkDir::new(&posts_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                let content = fs::read_to_string(path)
                    .map_err(|e| format!("Failed to read post {:?}: {}", path, e))?;
                if let Ok((doc, _)) = MarkdownDocument::parse(&content) {
                    for (key, value) in doc.frontmatter.custom_fields {
                        let entry = stats.entry(key).or_default();
                        entry.total += 1;
                        if let Some(field_type) = infer_value_type(&value) {
                            *entry.type_counts.entry(field_type).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    let mut custom_fields: Vec<FrontmatterField> = stats
        .iter()
        .map(|(name, stat)| FrontmatterField {
            name: name.clone(),
            label: Some(format_label(name)),
            field_type: stat.preferred_type(),
            description: None,
            ui: None,
        })
        .collect();
    custom_fields.sort_by(|a, b| a.name.cmp(&b.name));

    let image_fields: Vec<String> = custom_fields
        .iter()
        .filter(|field| field.field_type == "image")
        .map(|field| field.name.clone())
        .collect();

    let preview_image_field = pick_preview_image_field(&stats, &image_fields);

    let field_groups = if image_fields.is_empty() {
        Vec::new()
    } else {
        let mut fields = Vec::new();
        for image_field in &image_fields {
            fields.push(image_field.clone());
            let snake_alt = format!("{}_alt", image_field);
            let camel_alt = format!("{}Alt", image_field);
            if stats.contains_key(&snake_alt) {
                fields.push(snake_alt);
            } else if stats.contains_key(&camel_alt) {
                fields.push(camel_alt);
            }
        }
        vec![FrontmatterFieldGroup {
            name: "images".to_string(),
            label: Some("Images".to_string()),
            fields,
            collapsed: Some(false),
        }]
    };

    Ok(FrontmatterConfig {
        version: "1.0".to_string(),
        preview_image_field,
        is_default: false,
        custom_fields,
        field_groups,
    })
}

#[derive(Default)]
struct FieldStats {
    total: u32,
    type_counts: HashMap<String, u32>,
}

impl FieldStats {
    fn preferred_type(&self) -> String {
        if self.type_counts.is_empty() {
            return "string".to_string();
        }
        let mut sorted: Vec<(&String, &u32)> = self.type_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        sorted
            .first()
            .map(|(name, _)| (*name).clone())
            .unwrap_or_else(|| "string".to_string())
    }
}

fn infer_value_type(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::Null => None,
        serde_yaml::Value::Bool(_) => Some("boolean".to_string()),
        serde_yaml::Value::Number(_) => Some("number".to_string()),
        serde_yaml::Value::Sequence(_) => Some("array".to_string()),
        serde_yaml::Value::Mapping(_) => Some("object".to_string()),
        serde_yaml::Value::Tagged(tagged) => infer_value_type(&tagged.value),
        serde_yaml::Value::String(value) => {
            if value.trim().is_empty() {
                return None;
            }
            if looks_like_image(value) {
                return Some("image".to_string());
            }
            if looks_like_datetime(value) {
                return Some("datetime".to_string());
            }
            if looks_like_date(value) {
                return Some("date".to_string());
            }
            if value.contains('\n') {
                return Some("text".to_string());
            }
            Some("string".to_string())
        }
    }
}

fn looks_like_image(value: &str) -> bool {
    let lowercase = value.to_lowercase();
    let has_image_path = lowercase.contains("/images/");
    let has_ext = lowercase.ends_with(".png")
        || lowercase.ends_with(".jpg")
        || lowercase.ends_with(".jpeg")
        || lowercase.ends_with(".gif")
        || lowercase.ends_with(".webp")
        || lowercase.ends_with(".svg");
    has_image_path || has_ext
}

fn looks_like_datetime(value: &str) -> bool {
    if DateTime::parse_from_rfc3339(value).is_ok() {
        return true;
    }
    chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").is_ok()
        || chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M").is_ok()
}

fn looks_like_date(value: &str) -> bool {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
}

fn format_label(name: &str) -> String {
    let mut label = String::new();
    let mut prev_is_lower = false;
    for ch in name.chars() {
        if ch == '_' || ch == '-' {
            label.push(' ');
            prev_is_lower = false;
            continue;
        }
        if prev_is_lower && ch.is_uppercase() {
            label.push(' ');
        }
        label.push(ch);
        prev_is_lower = ch.is_lowercase();
    }
    label
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn pick_preview_image_field(
    stats: &HashMap<String, FieldStats>,
    image_fields: &[String],
) -> Option<String> {
    if image_fields.is_empty() {
        return None;
    }

    let preferred_names = ["list_image", "cover", "thumbnail", "main_image", "banner", "image"];
    for candidate in preferred_names {
        if image_fields.iter().any(|name| name == candidate) {
            return Some(candidate.to_string());
        }
    }

    let mut best: Option<(String, u32)> = None;
    for field in image_fields {
        let count = stats
            .get(field)
            .map(|stat| stat.total)
            .unwrap_or(0);
        match &best {
            Some((_, best_count)) if *best_count >= count => {}
            _ => best = Some((field.clone(), count)),
        }
    }

    best.map(|(name, _)| name)
}
