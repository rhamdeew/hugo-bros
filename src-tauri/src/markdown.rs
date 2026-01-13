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
    pub permalink: Option<String>,
    pub list_image: Option<String>,
    pub list_image_alt: Option<String>,
    pub main_image: Option<String>,
    pub main_image_alt: Option<String>,
    #[serde(flatten)]
    pub custom_fields: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug)]
pub struct MarkdownDocument {
    pub frontmatter: Frontmatter,
    pub content: String,
}

impl MarkdownDocument {
    pub fn parse(raw: &str) -> Result<(Self, bool), String> {
        // Check if the document starts with frontmatter delimiter
        if !raw.starts_with("---") {
            // No frontmatter found - create default frontmatter
            let frontmatter = Frontmatter {
                title: "Untitled Post".to_string(),
                date: "".to_string(), // Empty date - will be filled with file time
                tags: Vec::new(),
                categories: Vec::new(),
                permalink: None,
                list_image: None,
                list_image_alt: None,
                main_image: None,
                main_image_alt: None,
                custom_fields: HashMap::new(),
            };

            return Ok((Self {
                frontmatter,
                content: raw.to_string(),
            }, true)); // true = had no frontmatter
        }

        // Find the end of frontmatter
        let parts: Vec<&str> = raw.splitn(3, "---").collect();
        if parts.len() < 3 {
            // Invalid frontmatter format - treat entire content as markdown
            let frontmatter = Frontmatter {
                title: "Untitled Post".to_string(),
                date: "".to_string(), // Empty date - will be filled with file time
                tags: Vec::new(),
                categories: Vec::new(),
                permalink: None,
                list_image: None,
                list_image_alt: None,
                main_image: None,
                main_image_alt: None,
                custom_fields: HashMap::new(),
            };

            return Ok((Self {
                frontmatter,
                content: raw.to_string(),
            }, true)); // true = had no frontmatter
        }

        // Parse YAML frontmatter
        let frontmatter_str = parts[1].trim();
        let frontmatter: Frontmatter = serde_yaml::from_str(frontmatter_str)
            .map_err(|e| format!("Failed to parse frontmatter: {}", e))?;

        // Content is everything after the second ---
        let content = parts[2].trim().to_string();

        Ok((Self {
            frontmatter,
            content,
        }, false)) // false = had frontmatter
    }

    pub fn to_string(&self) -> Result<String, String> {
        let frontmatter_yaml = serde_yaml::to_string(&self.frontmatter)
            .map_err(|e| format!("Failed to serialize frontmatter: {}", e))?;

        Ok(format!("---\n{}---\n\n{}", frontmatter_yaml, self.content))
    }
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
        let frontmatter_yaml = serde_yaml::to_string(&self.frontmatter)
            .map_err(|e| format!("Failed to serialize frontmatter: {}", e))?;

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
        let frontmatter_yaml = serde_yaml::to_string(&self.frontmatter)
            .map_err(|e| format!("Failed to serialize frontmatter: {}", e))?;

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
            doc.frontmatter.date = file_date.format("%Y-%m-%d %H:%M:%S").to_string();
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
        let frontmatter_yaml = serde_yaml::to_string(&self.frontmatter)
            .map_err(|e| format!("Failed to serialize frontmatter: {}", e))?;

        Ok(format!("---\n{}---\n\n{}", frontmatter_yaml, self.content))
    }
}
