// Markdown and frontmatter parsing

use crate::files;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Frontmatter {
  pub title: String,
  pub date: String,
  #[serde(default)]
  pub tags: Vec<String>,
  #[serde(default)]
  pub categories: Vec<String>,
  pub updated: Option<String>,
  pub comments: Option<bool>,
  pub layout: Option<String>,
  pub permalink: Option<String>,
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub draft: Option<bool>,
  #[serde(default)]
  pub custom_fields: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
struct FrontmatterYaml {
    pub title: String,
    pub date: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permalink: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    #[serde(flatten)]
    #[serde(default)]
    pub custom_fields: HashMap<String, serde_yaml::Value>,
}

impl From<FrontmatterYaml> for Frontmatter {
    fn from(frontmatter: FrontmatterYaml) -> Self {
        Self {
            title: frontmatter.title,
            date: frontmatter.date,
            tags: frontmatter.tags,
            categories: frontmatter.categories,
            updated: frontmatter.updated,
            comments: frontmatter.comments,
            layout: frontmatter.layout,
            permalink: frontmatter.permalink,
            description: frontmatter.description,
            draft: frontmatter.draft,
            custom_fields: frontmatter.custom_fields,
        }
    }
}

impl From<Frontmatter> for FrontmatterYaml {
    fn from(frontmatter: Frontmatter) -> Self {
        Self {
            title: frontmatter.title,
            date: frontmatter.date,
            tags: frontmatter.tags,
            categories: frontmatter.categories,
            updated: frontmatter.updated,
            comments: frontmatter.comments,
            layout: frontmatter.layout,
            permalink: frontmatter.permalink,
            description: frontmatter.description,
            draft: frontmatter.draft,
            custom_fields: frontmatter.custom_fields,
        }
    }
}

pub fn frontmatter_to_yaml(frontmatter: &Frontmatter) -> Result<String, String> {
    serde_yaml::to_string(&FrontmatterYaml::from(frontmatter.clone()))
        .map_err(|e| format!("Failed to serialize frontmatter: {}", e))
}

#[derive(Debug)]
pub struct MarkdownDocument {
    pub frontmatter: Frontmatter,
    pub content: String,
}

impl MarkdownDocument {
    pub fn parse(raw: &str) -> Result<(Self, bool), String> {
        // Standard format: ---\nfrontmatter\n---\ncontent
        if raw.starts_with("---") {
            let parts: Vec<&str> = raw.splitn(3, "---").collect();
            if parts.len() >= 3 {
                let frontmatter_str = parts[1].trim();
                if let Ok(frontmatter) = serde_yaml::from_str::<FrontmatterYaml>(frontmatter_str) {
                    let content = parts[2].trim().to_string();
                    return Ok((Self { frontmatter: frontmatter.into(), content }, false));
                }
            }
        }

        // TOML frontmatter: +++\nfrontmatter\n+++\ncontent
        if raw.starts_with("+++") {
            let parts: Vec<&str> = raw.splitn(3, "+++").collect();
            if parts.len() >= 3 {
                let frontmatter_str = parts[1].trim();
                if let Ok(toml_value) = toml::from_str::<toml::Value>(frontmatter_str) {
                    if let Ok(json_value) = serde_json::to_value(toml_value) {
                        if let Ok(frontmatter) = serde_json::from_value::<FrontmatterYaml>(json_value) {
                            let content = parts[2].trim().to_string();
                            return Ok((Self { frontmatter: frontmatter.into(), content }, false));
                        }
                    }
                }
            }
        }

        // JSON frontmatter: { ... }\ncontent
        if raw.trim_start().starts_with('{') {
            if let Some((frontmatter_str, content)) = split_json_frontmatter(raw) {
                if let Ok(frontmatter) = serde_yaml::from_str::<FrontmatterYaml>(&frontmatter_str) {
                    return Ok((Self { frontmatter: frontmatter.into(), content }, false));
                }
            }
        }

        // Alternative format: frontmatter\n---\ncontent (without opening ---)
        // This is used by some Hugo content workflows
        if let Some(separator_pos) = raw.find("\n---") {
            let frontmatter_str = &raw[..separator_pos].trim();
            // Check if this looks like YAML frontmatter (contains "title:" or "date:")
            if frontmatter_str.contains("title:") || frontmatter_str.contains("date:") {
                if let Ok(frontmatter) = serde_yaml::from_str::<FrontmatterYaml>(frontmatter_str) {
                    // Content is everything after the ---
                    let content_start = separator_pos + 4; // "\n---".len()
                    let content = if content_start < raw.len() {
                        raw[content_start..].trim().to_string()
                    } else {
                        String::new()
                    };
                    return Ok((Self { frontmatter: frontmatter.into(), content }, false));
                }
            }
        }

        // No valid frontmatter found - create default
        let frontmatter = Frontmatter {
            title: "Untitled Post".to_string(),
            date: "".to_string(),
            tags: Vec::new(),
            categories: Vec::new(),
            updated: None,
            comments: None,
            layout: None,
            permalink: None,
            description: None,
            draft: None,
            custom_fields: HashMap::new(),
        };

        Ok((Self {
            frontmatter,
            content: raw.to_string(),
        }, true))
    }

}

fn split_json_frontmatter(raw: &str) -> Option<(String, String)> {
    let mut depth = 0usize;
    let mut end_idx = None;

    for (idx, ch) in raw.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = Some(idx);
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    end_idx.map(|idx| {
        let frontmatter_str = raw[..=idx].trim().to_string();
        let content = raw[idx + 1..].trim().to_string();
        (frontmatter_str, content)
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub id: String,
    pub title: String,
    pub date: String,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub file_path: String,
    pub created_at: i64,
    pub modified_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub id: String,
    pub title: String,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub file_path: String,
    pub created_at: i64,
    pub modified_at: i64,
}

impl Page {
    pub fn to_markdown(&self) -> Result<String, String> {
        let frontmatter_yaml = frontmatter_to_yaml(&self.frontmatter)?;

        Ok(format!("---\n{}---\n\n{}", frontmatter_yaml, self.content))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub id: String,
    pub title: String,
    pub content: String,
    pub frontmatter: Frontmatter,
    pub file_path: String,
    pub created_at: i64,
    pub modified_at: i64,
}

impl Draft {
    pub fn to_markdown(&self) -> Result<String, String> {
        let frontmatter_yaml = frontmatter_to_yaml(&self.frontmatter)?;

        Ok(format!("---\n{}---\n\n{}", frontmatter_yaml, self.content))
    }
}

// Helper function to extract title from markdown content
fn extract_title_from_markdown(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        // Look for H1 heading: # Title
        if trimmed.starts_with("# ") {
            let title = trimmed.trim_start_matches('#').trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub filename: String,
    pub path: String,
    pub full_path: String,
    pub url: String,
    pub size: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub created_at: i64,
}

impl Post {
    pub fn from_file(file_path: &Path, project_path: &Path) -> Result<Self, String> {
        let content = files::read_file(file_path)?;

        let (mut doc, had_no_frontmatter) = MarkdownDocument::parse(&content)?;

        // Get file metadata
        let metadata = fs::metadata(file_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;

        let created_at = metadata
            .created()
            .ok()
            .or(metadata.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d: std::time::Duration| d.as_secs() as i64)
            .unwrap_or(0);

        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d: std::time::Duration| d.as_secs() as i64)
            .unwrap_or(0);

        // If title is "Untitled Post", try to extract from content or filename
        if doc.frontmatter.title == "Untitled Post" {
            // Try to extract title from first H1 heading
            if let Some(title) = extract_title_from_markdown(&doc.content) {
                doc.frontmatter.title = title;
            } else {
                // Use filename without extension
                let filename = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled Post");
                doc.frontmatter.title = filename.to_string();
            }
        }

        // Only use file modified time as date if post had no frontmatter or date is empty
        if had_no_frontmatter || doc.frontmatter.date.is_empty() {
            let file_date = chrono::DateTime::<chrono::Local>::from(
                metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            );
            doc.frontmatter.date = file_date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        }

        // Generate ID (relative path from source/_posts/)
        let id = file_path
            .strip_prefix(project_path)
            .ok()
            .and_then(|p| p.to_str())
            .unwrap_or_else(|| file_path.to_str().unwrap_or(""))
            .to_string();

        Ok(Self {
            id,
            title: doc.frontmatter.title.clone(),
            date: doc.frontmatter.date.clone(),
            content: doc.content,
            frontmatter: doc.frontmatter,
            file_path: file_path.to_string_lossy().to_string(),
            created_at,
            modified_at,
        })
    }

    pub fn to_markdown(&self) -> Result<String, String> {
        let frontmatter_yaml = frontmatter_to_yaml(&self.frontmatter)?;

        Ok(format!("---\n{}---\n\n{}", frontmatter_yaml, self.content))
    }
}

#[cfg(test)]
mod tests {
    use super::MarkdownDocument;

    #[test]
    fn parse_standard_frontmatter() {
        let raw = "---\ntitle: \"Hello\"\ndate: \"2024-01-01 10:00:00\"\n---\nBody";
        let (doc, had_no_frontmatter) = MarkdownDocument::parse(raw).expect("parse failed");

        assert!(!had_no_frontmatter);
        assert_eq!(doc.frontmatter.title, "Hello");
        assert_eq!(doc.frontmatter.date, "2024-01-01 10:00:00");
        assert_eq!(doc.content, "Body");
    }

    #[test]
    fn parse_alternative_frontmatter() {
        let raw = "title: \"Alt\"\ndate: \"2024-01-02\"\n---\nAlt body";
        let (doc, had_no_frontmatter) = MarkdownDocument::parse(raw).expect("parse failed");

        assert!(!had_no_frontmatter);
        assert_eq!(doc.frontmatter.title, "Alt");
        assert_eq!(doc.frontmatter.date, "2024-01-02");
        assert_eq!(doc.content, "Alt body");
    }

    #[test]
    fn parse_without_frontmatter_defaults() {
        let raw = "Just text";
        let (doc, had_no_frontmatter) = MarkdownDocument::parse(raw).expect("parse failed");

        assert!(had_no_frontmatter);
        assert_eq!(doc.frontmatter.title, "Untitled Post");
        assert_eq!(doc.content, "Just text");
    }
}
