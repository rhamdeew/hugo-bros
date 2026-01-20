// Tauri commands for frontend-backend communication

use crate::hugo::HugoProject;
use crate::markdown::{Draft, ImageInfo, Page, Post};
use crate::frontmatter_config::{
    generate_frontmatter_config, load_frontmatter_config, FrontmatterConfig,
};
use std::fs;
use std::path::{Component, Path, PathBuf};
use tauri::command;
use tauri::AppHandle;

// ====================
// Project Commands
// ====================

#[command]
pub async fn select_project_folder(app: AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

    // Open folder picker dialog
    let folder_path = app.dialog()
        .file()
        .set_title("Select Hugo Project Folder")
        .blocking_pick_folder();

    if let Some(path) = folder_path {
        // Convert FilePath to PathBuf
        let path_buf = PathBuf::from(path.to_string());
        let path_string = path_buf.to_string_lossy().to_string();

        // Validate it's a Hugo project
        let project = HugoProject::new(path_buf);
        match project.validate() {
            Ok(_) => {
                // Add to recent projects
                let mut config = crate::config::AppConfig::load()
                    .unwrap_or_default();
                config.add_recent_project(path_string.clone());
                let _ = config.save(); // Ignore save errors

                Ok(path_string)
            },
            Err(e) => {
                // Show error dialog to user
                app.dialog()
                    .message(format!("Invalid Hugo project: {}", e))
                    .kind(MessageDialogKind::Error)
                    .title("Invalid Project")
                    .blocking_show();
                Err(format!("Invalid Hugo project: {}", e))
            }
        }
    } else {
        Err("No folder selected".to_string())
    }
}

#[command]
pub fn get_project_config(project_path: String) -> Result<HugoConfig, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    let config_path = project
        .find_config_path()
        .ok_or("Hugo config not found (config.* or hugo.*)".to_string())?;

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let config_value = parse_hugo_config(&config_path, &content)?;
    Ok(HugoConfig::from_value(config_value))
}

#[command]
pub fn get_frontmatter_config(project_path: String) -> Result<FrontmatterConfig, String> {
    load_frontmatter_config(Path::new(&project_path))
}

#[command]
pub fn generate_frontmatter_config_command(project_path: String) -> Result<FrontmatterConfig, String> {
    let config_path = Path::new(&project_path)
        .join(".hugo-bros")
        .join("frontmatter-config.json");

    if config_path.exists() {
        return Err("frontmatter-config.json already exists".to_string());
    }

    let config = generate_frontmatter_config(Path::new(&project_path))?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .hugo-bros directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize frontmatter config: {}", e))?;
    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write frontmatter config: {}", e))?;

    Ok(config)
}

// ====================
// Posts Commands
// ====================

#[command]
pub fn list_posts(project_path: String) -> Result<Vec<Post>, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    let posts_dir = project.get_posts_dir();
    let drafts_dir = project.get_content_dir().join("drafts");

    if !posts_dir.exists() {
        return Ok(Vec::new());
    }

    let mut posts = Vec::new();

    for entry in walkdir::WalkDir::new(&posts_dir)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            if path.file_name().and_then(|s| s.to_str()) == Some("_index.md") {
                continue;
            }
            if drafts_dir.exists() && path.starts_with(&drafts_dir) {
                continue;
            }
            match Post::from_file(path, Path::new(&project_path)) {
                Ok(post) => {
                    if post.frontmatter.draft.unwrap_or(false) {
                        continue;
                    }
                    posts.push(post);
                },
                Err(e) => eprintln!("Failed to parse post {:?}: {}", path, e),
            }
        }
    }

    // Sort by date (newest first)
    posts.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    Ok(posts)
}

#[command]
pub fn get_post(project_path: String, post_id: String) -> Result<Post, String> {
    let file_path = Path::new(&project_path).join(&post_id);

    if !file_path.exists() {
        return Err("Post not found".to_string());
    }

    Post::from_file(&file_path, Path::new(&project_path))
}

#[command]
pub fn save_post(_project_path: String, post: Post) -> Result<(), String> {
    let file_path = Path::new(&post.file_path);

    let markdown = post.to_markdown()?;

    fs::write(file_path, markdown)
        .map_err(|e| format!("Failed to save post: {}", e))?;

    Ok(())
}

#[command]
pub fn get_page(project_path: String, page_id: String) -> Result<Page, String> {
    let file_path = Path::new(&project_path).join(&page_id);

    if !file_path.exists() {
        return Err("Page not found".to_string());
    }

    Page::from_file(&file_path, Path::new(&project_path))
}

#[command]
pub fn save_page(_project_path: String, page: Page) -> Result<(), String> {
    let file_path = Path::new(&page.file_path);

    let markdown = page.to_markdown()?;

    fs::write(file_path, markdown)
        .map_err(|e| format!("Failed to save page: {}", e))?;

    Ok(())
}

#[command]
pub fn create_post(project_path: String, title: String) -> Result<Post, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    let posts_dir = project.get_posts_dir();

    // Create posts directory if it doesn't exist
    fs::create_dir_all(&posts_dir)
        .map_err(|e| format!("Failed to create posts directory: {}", e))?;

    // Generate filename from title (transliterate to ASCII)
    let filename = sanitize_filename(&title);
    let file_path = posts_dir.join(format!("{}.md", filename));

    // Get current time in ISO 8601 format
    let now = chrono::Local::now();
    let date_str = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    // Create default frontmatter
    let frontmatter = crate::markdown::Frontmatter {
        title: title.clone(),
        date: date_str,
        tags: Vec::new(),
        categories: Vec::new(),
        updated: None,
        comments: None,
        layout: None,
        description: None,
        permalink: None,
        draft: None,
        custom_fields: Default::default(),
    };

    // Create markdown content
    let frontmatter_yaml = crate::markdown::frontmatter_to_yaml(&frontmatter)?;

    let content = format!("---\n{}---\n\n", frontmatter_yaml);

    // Write file
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to create post: {}", e))?;

    // Read back as Post
    Post::from_file(&file_path, Path::new(&project_path))
}

#[command]
pub fn get_draft(project_path: String, draft_id: String) -> Result<Draft, String> {
    let file_path = Path::new(&project_path).join(&draft_id);

    if !file_path.exists() {
        return Err("Draft not found".to_string());
    }

    Draft::from_file(&file_path, Path::new(&project_path))
}

#[command]
pub fn save_draft(_project_path: String, draft: Draft) -> Result<(), String> {
    let file_path = Path::new(&draft.file_path);

    let markdown = draft.to_markdown()?;

    fs::write(file_path, markdown)
        .map_err(|e| format!("Failed to save draft: {}", e))?;

    Ok(())
}

#[command]
pub fn delete_post(project_path: String, post_id: String) -> Result<(), String> {
    let file_path = Path::new(&project_path).join(&post_id);

    if !file_path.exists() {
        return Err("Post not found".to_string());
    }

    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete post: {}", e))?;

    Ok(())
}

#[command]
pub fn delete_page(project_path: String, page_id: String) -> Result<(), String> {
    let file_path = Path::new(&project_path).join(&page_id);

    if !file_path.exists() {
        return Err("Page not found".to_string());
    }

    if let Some(parent) = file_path.parent() {
        if parent.file_name().and_then(|s| s.to_str()) == Some("content") {
            return Err("Refusing to delete content root".to_string());
        }
        if parent.ends_with("posts") || parent.ends_with("drafts") {
            return Err("Invalid page path".to_string());
        }
        if matches!(
            file_path.file_name().and_then(|s| s.to_str()),
            Some("index.md") | Some("_index.md")
        ) {
            fs::remove_file(&file_path)
                .map_err(|e| format!("Failed to delete page: {}", e))?;
            if fs::read_dir(parent).map(|mut i| i.next().is_none()).unwrap_or(false) {
                let _ = fs::remove_dir(parent);
            }
            return Ok(());
        }
    }

    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete page: {}", e))?;

    Ok(())
}

// ====================
// Pages Commands
// ====================

#[command]
pub fn create_page(project_path: String, title: String) -> Result<Page, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    let pages_dir = project.get_pages_dir();

    fs::create_dir_all(&pages_dir)
        .map_err(|e| format!("Failed to create pages directory: {}", e))?;

    let mut folder_name = sanitize_filename(&title);
    if folder_name.is_empty() {
        folder_name = "page".to_string();
    }

    let mut page_dir = pages_dir.join(&folder_name);
    if page_dir.exists() {
        let timestamp = chrono::Utc::now().timestamp();
        page_dir = pages_dir.join(format!("{}_{}", folder_name, timestamp));
    }

    fs::create_dir_all(&page_dir)
        .map_err(|e| format!("Failed to create page directory: {}", e))?;

    let file_path = page_dir.join("index.md");

    let now = chrono::Local::now();
    let date_str = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let frontmatter = crate::markdown::Frontmatter {
        title: title.clone(),
        date: date_str,
        tags: Vec::new(),
        categories: Vec::new(),
        updated: None,
        comments: None,
        layout: None,
        description: None,
        permalink: None,
        draft: None,
        custom_fields: Default::default(),
    };

    let frontmatter_yaml = crate::markdown::frontmatter_to_yaml(&frontmatter)?;

    let content = format!("---\n{}---\n\n", frontmatter_yaml);

    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to create page: {}", e))?;

    Page::from_file(&file_path, Path::new(&project_path))
}

#[command]
pub fn list_pages(project_path: String) -> Result<Vec<Page>, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    let pages_dir = project.get_pages_dir();
    let posts_dir = project.get_posts_dir();
    let drafts_dir = project.get_content_dir().join("drafts");
    let should_skip_posts = posts_dir != pages_dir;

    if !pages_dir.exists() {
        return Ok(Vec::new());
    }

    let mut pages = Vec::new();

    // Look for index.md/_index.md files and standalone pages in content/
    for entry in walkdir::WalkDir::new(&pages_dir)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        if (should_skip_posts && path.starts_with(&posts_dir)) || path.starts_with(&drafts_dir) {
            continue;
        }
        let filename = path.file_name().and_then(|s| s.to_str());
        let is_index = matches!(filename, Some("index.md") | Some("_index.md"));
        let is_root_page = path.parent() == Some(pages_dir.as_path());
        if !is_index && !is_root_page {
            continue;
        }

        match Page::from_file(path, Path::new(&project_path)) {
            Ok(page) => {
                if page.frontmatter.draft.unwrap_or(false) {
                    continue;
                }
                pages.push(page);
            },
            Err(e) => eprintln!("Failed to parse page: {}", e),
        }
    }

    pages.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    Ok(pages)
}

// ====================
// Drafts Commands
// ====================

#[command]
pub fn create_draft(project_path: String, title: String) -> Result<Draft, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    let drafts_dir = project.get_content_dir().join("drafts");

    fs::create_dir_all(&drafts_dir)
        .map_err(|e| format!("Failed to create drafts directory: {}", e))?;

    let mut filename = sanitize_filename(&title);
    if filename.is_empty() {
        filename = "draft".to_string();
    }
    let file_path = drafts_dir.join(format!("{}.md", filename));

    let final_path = if file_path.exists() {
        let timestamp = chrono::Utc::now().timestamp();
        drafts_dir.join(format!("{}_{}.md", filename, timestamp))
    } else {
        file_path
    };

    let now = chrono::Local::now();
    let date_str = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let frontmatter = crate::markdown::Frontmatter {
        title: title.clone(),
        date: date_str,
        tags: Vec::new(),
        categories: Vec::new(),
        updated: None,
        comments: None,
        layout: None,
        description: None,
        permalink: None,
        draft: Some(true),
        custom_fields: Default::default(),
    };

    let frontmatter_yaml = crate::markdown::frontmatter_to_yaml(&frontmatter)?;

    let content = format!("---\n{}---\n\n", frontmatter_yaml);

    fs::write(&final_path, content)
        .map_err(|e| format!("Failed to create draft: {}", e))?;

    Draft::from_file(&final_path, Path::new(&project_path))
}

#[command]
pub fn delete_draft(project_path: String, draft_id: String) -> Result<(), String> {
    let file_path = Path::new(&project_path).join(&draft_id);

    if !file_path.exists() {
        return Err("Draft not found".to_string());
    }

    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete draft: {}", e))?;

    Ok(())
}

#[command]
pub fn list_drafts(project_path: String) -> Result<Vec<Draft>, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    let content_dir = project.get_content_dir();
    let drafts_dir = content_dir.join("drafts");

    if !content_dir.exists() {
        return Ok(Vec::new());
    }

    let mut drafts = Vec::new();

    for entry in walkdir::WalkDir::new(&content_dir)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let is_draft_path = drafts_dir.exists() && path.starts_with(&drafts_dir);
            match Draft::from_file(path, Path::new(&project_path)) {
                Ok(draft) => {
                    if draft.frontmatter.draft.unwrap_or(false) || is_draft_path {
                        drafts.push(draft);
                    }
                },
                Err(e) => eprintln!("Failed to parse draft: {}", e),
            }
        }
    }

    drafts.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    Ok(drafts)
}

// ====================
// Images Commands
// ====================

#[command]
pub fn list_images(project_path: String) -> Result<Vec<ImageInfo>, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    let static_dir = project.get_static_dir();

    if !static_dir.exists() {
        return Ok(Vec::new());
    }

    let mut images = Vec::new();

    for entry in walkdir::WalkDir::new(&static_dir)
        .max_depth(10) // Allow subdirectories in images
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "ico") {
                    match create_image_info(path, &static_dir, Path::new(&project_path)) {
                        Ok(img) => images.push(img),
                        Err(e) => eprintln!("Failed to read image {:?}: {}", path, e),
                    }
                }
            }
        }
    }

    images.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(images)
}

#[command]
pub fn list_static_entries(
    project_path: String,
    dir: Option<String>,
) -> Result<Vec<StaticEntry>, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    let static_dir = project.get_static_dir();

    if !static_dir.exists() {
        return Ok(Vec::new());
    }

    let relative_dir = dir.unwrap_or_default();
    let relative_path = validate_relative_path(&relative_dir)?;
    let target_dir = static_dir.join(&relative_path);

    if !target_dir.exists() {
        return Err("Directory not found".to_string());
    }
    if !target_dir.is_dir() {
        return Err("Not a directory".to_string());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&target_dir).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let name = entry
            .file_name()
            .to_string_lossy()
            .to_string();

        if path.is_dir() {
            let (created_at, modified_at) = file_times(&path)?;
            let relative_path = path
                .strip_prefix(&static_dir)
                .ok()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();
            entries.push(StaticEntry {
                name,
                path: relative_path.replace('\\', "/"),
                kind: "dir".to_string(),
                size: 0,
                created_at,
                modified_at,
                url: None,
                full_path: path.to_string_lossy().to_string(),
            });
            continue;
        }

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if !matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "ico") {
                    continue;
                }
            } else {
                continue;
            }

            let (created_at, modified_at) = file_times(&path)?;
            let size = fs::metadata(&path)
                .map(|m| m.len())
                .unwrap_or(0);
            let relative_path = path
                .strip_prefix(&static_dir)
                .ok()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();
            let url = format!("/{}", relative_path.replace('\\', "/"));
            entries.push(StaticEntry {
                name,
                path: relative_path.replace('\\', "/"),
                kind: "file".to_string(),
                size,
                created_at,
                modified_at,
                url: Some(url),
                full_path: path.to_string_lossy().to_string(),
            });
        }
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

#[command]
pub fn create_static_folder(
    project_path: String,
    parent_dir: Option<String>,
    name: String,
) -> Result<String, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    let static_dir = project.get_static_dir();

    if !static_dir.exists() {
        fs::create_dir_all(&static_dir)
            .map_err(|e| format!("Failed to create static directory: {}", e))?;
    }

    let trimmed_name = name.trim();
    validate_folder_name(trimmed_name)?;
    let relative_parent = validate_relative_path(parent_dir.as_deref().unwrap_or(""))?;
    let target_dir = static_dir.join(&relative_parent).join(trimmed_name);

    if target_dir.exists() {
        return Err("Folder already exists".to_string());
    }

    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create folder: {}", e))?;

    let relative_path = target_dir
        .strip_prefix(&static_dir)
        .ok()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string();

    Ok(relative_path.replace('\\', "/"))
}

#[command]
pub fn delete_static_entry(project_path: String, relative_path: String) -> Result<(), String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    let static_dir = project.get_static_dir();
    if relative_path.trim().is_empty() {
        return Err("Refusing to delete static root".to_string());
    }
    let relative = validate_relative_path(&relative_path)?;
    let target_path = static_dir.join(&relative);

    if !target_path.exists() {
        return Err("Entry not found".to_string());
    }

    if target_path.is_dir() {
        fs::remove_dir_all(&target_path)
            .map_err(|e| format!("Failed to delete folder: {}", e))?;
    } else {
        fs::remove_file(&target_path)
            .map_err(|e| format!("Failed to delete file: {}", e))?;
    }

    Ok(())
}

#[command]
pub fn copy_image_to_project(
    project_path: String,
    source_path: String,
    target_dir: Option<String>,
) -> Result<String, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    let static_dir = project.get_static_dir();
    let target_dir = target_dir.unwrap_or_default();
    let relative_target = validate_relative_path(&target_dir)?;
    let dest_dir = if target_dir.is_empty() {
        static_dir.clone()
    } else {
        static_dir.join(relative_target)
    };

    // Create images directory if it doesn't exist
    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    let source = Path::new(&source_path);
    let filename = source
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid source filename")?;
    let sanitized_filename = sanitize_image_filename(filename);

    let dest_path = dest_dir.join(&sanitized_filename);

    // Handle duplicate filenames
    let final_dest = if dest_path.exists() {
        let timestamp = chrono::Utc::now().timestamp();
        let stem = Path::new(&sanitized_filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let ext = source.extension().and_then(|s| s.to_str()).unwrap_or("");
        dest_dir.join(format!("{}_{}.{}", stem, timestamp, ext))
    } else {
        dest_path
    };

    fs::copy(source, &final_dest)
        .map_err(|e| format!("Failed to copy image: {}", e))?;

    // Return URL path for markdown
    let relative_path = final_dest
        .strip_prefix(&static_dir)
        .ok()
        .and_then(|p| p.to_str())
        .ok_or("Failed to get relative path")?;

    Ok(format!("/{}", relative_path.replace('\\', "/")))
}

fn sanitize_image_filename(filename: &str) -> String {
    let path = Path::new(filename);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let mut sanitized = String::with_capacity(stem.len());
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            sanitized.push(ch);
        } else if ch.is_whitespace() {
            sanitized.push('-');
        }
    }

    if sanitized.is_empty() {
        sanitized.push_str("image");
    }

    if ext.is_empty() {
        sanitized
    } else {
        format!("{}.{}", sanitized, ext)
    }
}

fn validate_relative_path(relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty() {
        return Ok(PathBuf::new());
    }

    let path = Path::new(relative);
    if path.is_absolute() {
        return Err("Path must be relative".to_string());
    }

    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err("Path must not contain parent or root segments".to_string());
            }
        }
    }

    Ok(path.to_path_buf())
}

fn validate_folder_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Folder name is required".to_string());
    }
    if name == "." || name == ".." {
        return Err("Invalid folder name".to_string());
    }
    if name.contains('/') || name.contains('\\') {
        return Err("Folder name must not contain path separators".to_string());
    }
    Ok(())
}

fn file_times(path: &Path) -> Result<(i64, i64), String> {
    let metadata = fs::metadata(path)
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

    Ok((created_at, modified_at))
}

#[command]
pub fn delete_image(project_path: String, image_path: String) -> Result<(), String> {
    let file_path = Path::new(&project_path).join(&image_path);

    if !file_path.exists() {
        return Err("Image not found".to_string());
    }

    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete image: {}", e))?;

    Ok(())
}

// ====================
// App Config Commands
// ====================

#[command]
pub fn get_app_config() -> Result<crate::config::AppConfig, String> {
    crate::config::AppConfig::load()
}

#[command]
pub fn save_app_config(config: crate::config::AppConfig) -> Result<(), String> {
    config.save()
}

// ====================
// Helper Functions
// ====================

fn sanitize_filename(title: &str) -> String {
    use regex::Regex;

    // Transliterate Russian to Latin (basic)
    let transliterated = transliterate_russian(title);

    // Convert to lowercase, replace spaces with hyphens
    let result = transliterated
        .to_lowercase()
        .replace(&[' ', '_', '+'][..], "-");

    // Remove special characters except alphanumerics and hyphens
    let re = Regex::new(r"[^a-z0-9-]").unwrap();
    let cleaned = re.replace_all(&result, "");

    // Remove consecutive hyphens
    let re = Regex::new(r"-+").unwrap();
    let deduped = re.replace_all(&cleaned, "-");

    // Trim hyphens from start and end
    deduped.trim_matches('-').to_string()
}

fn transliterate_russian(text: &str) -> String {
    let mapping = [
        ('а', "a"), ('б', "b"), ('в', "v"), ('г', "g"), ('д', "d"), ('е', "e"),
        ('ё', "yo"), ('ж', "zh"), ('з', "z"), ('и', "i"), ('й', "y"), ('к', "k"),
        ('л', "l"), ('м', "m"), ('н', "n"), ('о', "o"), ('п', "p"), ('р', "r"),
        ('с', "s"), ('т', "t"), ('у', "u"), ('ф', "f"), ('х', "h"), ('ц', "ts"),
        ('ч', "ch"), ('ш', "sh"), ('щ', "shch"), ('ъ', ""), ('ы', "y"), ('ь', ""),
        ('э', "e"), ('ю', "yu"), ('я', "ya"),
        ('А', "A"), ('Б', "B"), ('В', "V"), ('Г', "G"), ('Д', "D"), ('Е', "E"),
        ('Ё', "Yo"), ('Ж', "Zh"), ('З', "Z"), ('И', "I"), ('Й', "Y"), ('К', "K"),
        ('Л', "L"), ('М', "M"), ('Н', "N"), ('О', "O"), ('П', "P"), ('Р', "R"),
        ('С', "S"), ('Т', "T"), ('У', "U"), ('Ф', "F"), ('Х', "H"), ('Ц', "Ts"),
        ('Ч', "Ch"), ('Ш', "Sh"), ('Щ', "Shch"), ('Ъ', ""), ('Ы', "Y"), ('Ь', ""),
        ('Э', "E"), ('Ю', "Yu"), ('Я', "Ya"),
    ];

    let mut result = text.to_string();

    for (from, to) in &mapping {
        result = result.replace(*from, to);
    }

    result
}

fn create_image_info(
    image_path: &Path,
    static_dir: &Path,
    _project_path: &Path,
) -> Result<ImageInfo, String> {
    let metadata = fs::metadata(image_path)
        .map_err(|e| format!("Failed to get image metadata: {}", e))?;

    let filename = image_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let path = image_path
        .strip_prefix(static_dir)
        .ok()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string();

    let full_path = image_path
        .to_string_lossy()
        .to_string();

    let url = format!("/{}", path.replace('\\', "/"));

    let created_at = metadata
        .created()
        .ok()
        .or(metadata.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d: std::time::Duration| d.as_secs() as i64)
        .unwrap_or(0);

    // Get image dimensions if possible
    let (width, height) = get_image_dimensions(image_path);

    Ok(ImageInfo {
        filename,
        path,
        full_path,
        url,
        size: metadata.len(),
        width,
        height,
        created_at,
    })
}

fn get_image_dimensions(_path: &Path) -> (Option<u32>, Option<u32>) {
    // For now, return None. Can be implemented with image crate later
    (None, None)
}

// ====================
// Page & Draft Implementations
// ====================

impl Page {
    pub fn from_file(file_path: &Path, project_path: &Path) -> Result<Self, String> {
        let content = crate::files::read_file(file_path)?;
        let (doc, _) = crate::markdown::MarkdownDocument::parse(&content)?;

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

        let id = file_path
            .strip_prefix(project_path)
            .ok()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();

        Ok(Self {
            id,
            title: doc.frontmatter.title.clone(),
            content: doc.content,
            frontmatter: doc.frontmatter,
            file_path: file_path.to_string_lossy().to_string(),
            created_at,
            modified_at,
        })
    }
}

impl Draft {
    pub fn from_file(file_path: &Path, project_path: &Path) -> Result<Self, String> {
        let content = crate::files::read_file(file_path)?;
        let (doc, _) = crate::markdown::MarkdownDocument::parse(&content)?;

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

        let id = file_path
            .strip_prefix(project_path)
            .ok()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();

        Ok(Self {
            id,
            title: doc.frontmatter.title.clone(),
            content: doc.content,
            frontmatter: doc.frontmatter,
            file_path: file_path.to_string_lossy().to_string(),
            created_at,
            modified_at,
        })
    }
}

// ====================
// Data Types
// ====================

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticEntry {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub size: u64,
    pub created_at: i64,
    pub modified_at: i64,
    pub url: Option<String>,
    pub full_path: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HugoConfig {
    pub title: Option<String>,
    pub base_url: Option<String>,
    pub language_code: Option<String>,
    pub default_content_language: Option<String>,
    pub theme: Option<String>,
    pub raw: serde_json::Value,
}

impl HugoConfig {
    pub fn from_value(raw: serde_json::Value) -> Self {
        let title = extract_string(&raw, &["title"]);
        let base_url = extract_string(&raw, &["baseURL", "baseUrl", "base_url"]);
        let language_code = extract_string(&raw, &["languageCode", "language_code"]);
        let default_content_language =
            extract_string(&raw, &["defaultContentLanguage", "default_content_language"]);
        let theme = extract_string(&raw, &["theme"]);

        Self {
            title,
            base_url,
            language_code,
            default_content_language,
            theme,
            raw,
        }
    }
}

// ====================
// Hugo Commands
// ====================

#[command]
pub fn run_hugo_command(
    project_path: String,
    args: Vec<String>,
) -> Result<crate::hugo::CommandOutput, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    project.run_command(&args)
}

#[command]
pub fn start_hugo_server(project_path: String) -> Result<String, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    project.start_server()
}

#[command]
pub fn stop_hugo_server(server_id: String) -> Result<(), String> {
    HugoProject::stop_server(&server_id)
}

#[command]
pub fn is_hugo_server_running(project_path: String) -> Result<bool, String> {
    let project = HugoProject::new(PathBuf::from(&project_path));
    Ok(project.is_server_running())
}

fn parse_hugo_config(path: &Path, content: &str) -> Result<serde_json::Value, String> {
    match path.extension().and_then(|s| s.to_str()) {
        Some("toml") => {
            let value: toml::Value = toml::from_str(content)
                .map_err(|e| format!("Failed to parse TOML config: {}", e))?;
            serde_json::to_value(value)
                .map_err(|e| format!("Failed to convert TOML config: {}", e))
        }
        Some("yml") | Some("yaml") => {
            serde_yaml::from_str(content)
                .map_err(|e| format!("Failed to parse YAML config: {}", e))
        }
        Some("json") => {
            serde_json::from_str(content)
                .map_err(|e| format!("Failed to parse JSON config: {}", e))
        }
        _ => Err("Unsupported Hugo config format".to_string()),
    }
}

fn extract_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(found) = value.get(*key) {
            if let Some(text) = found.as_str() {
                return Some(text.to_string());
            }
        }
    }
    None
}
