// Tauri commands for frontend-backend communication

use crate::hexo::HexoProject;
use crate::markdown::{Draft, ImageInfo, Page, Post};
use std::fs;
use std::path::{Path, PathBuf};
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
        .set_title("Select Hexo Project Folder")
        .blocking_pick_folder();

    if let Some(path) = folder_path {
        // Convert FilePath to PathBuf
        let path_buf = PathBuf::from(path.to_string());
        let path_string = path_buf.to_string_lossy().to_string();

        // Validate it's a Hexo project
        let project = HexoProject::new(path_buf);
        match project.validate() {
            Ok(_) => Ok(path_string),
            Err(e) => {
                // Show error dialog to user
                app.dialog()
                    .message(format!("Invalid Hexo project: {}", e))
                    .kind(MessageDialogKind::Error)
                    .title("Invalid Project")
                    .blocking_show();
                Err(format!("Invalid Hexo project: {}", e))
            }
        }
    } else {
        Err("No folder selected".to_string())
    }
}

#[command]
pub fn get_project_config(project_path: String) -> Result<HexoConfig, String> {
    let config_path = Path::new(&project_path).join("_config.yml");

    if !config_path.exists() {
        return Err("_config.yml not found".to_string());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    // Parse YAML config
    let config: HexoConfig = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(config)
}

// ====================
// Posts Commands
// ====================

#[command]
pub fn list_posts(project_path: String) -> Result<Vec<Post>, String> {
    let project = HexoProject::new(PathBuf::from(&project_path));
    let posts_dir = project.get_posts_dir();

    if !posts_dir.exists() {
        return Ok(Vec::new());
    }

    let mut posts = Vec::new();

    for entry in walkdir::WalkDir::new(&posts_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            match Post::from_file(path, Path::new(&project_path)) {
                Ok(post) => posts.push(post),
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
pub fn create_post(project_path: String, title: String) -> Result<Post, String> {
    let project = HexoProject::new(PathBuf::from(&project_path));
    let posts_dir = project.get_posts_dir();

    // Create posts directory if it doesn't exist
    fs::create_dir_all(&posts_dir)
        .map_err(|e| format!("Failed to create posts directory: {}", e))?;

    // Generate filename from title (transliterate to ASCII)
    let filename = sanitize_filename(&title);
    let file_path = posts_dir.join(format!("{}.md", filename));

    // Get current time in ISO 8601 format
    let now = chrono::Local::now();
    let date_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    // Create default frontmatter
    let frontmatter = crate::markdown::Frontmatter {
        title: title.clone(),
        date: date_str,
        tags: Vec::new(),
        categories: Vec::new(),
        permalink: None,
        list_image: None,
        list_image_alt: None,
        main_image: None,
        main_image_alt: None,
        custom_fields: Default::default(),
    };

    // Create markdown content
    let frontmatter_yaml = serde_yaml::to_string(&frontmatter)
        .map_err(|e| format!("Failed to serialize frontmatter: {}", e))?;

    let content = format!("---\n{}---\n\n", frontmatter_yaml);

    // Write file
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to create post: {}", e))?;

    // Read back as Post
    Post::from_file(&file_path, Path::new(&project_path))
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

// ====================
// Pages Commands
// ====================

#[command]
pub fn list_pages(project_path: String) -> Result<Vec<Page>, String> {
    let project = HexoProject::new(PathBuf::from(&project_path));
    let pages_dir = project.get_pages_dir();

    if !pages_dir.exists() {
        return Ok(Vec::new());
    }

    let mut pages = Vec::new();

    // Look for index.md files in subdirectories of source/
    for entry in walkdir::WalkDir::new(&pages_dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.file_name().and_then(|s| s.to_str()) == Some("index.md") {
            // Skip _posts and _drafts
            if let Some(parent) = path.parent() {
                if let Some(folder_name) = parent.file_name() {
                    if folder_name == "_posts" || folder_name == "_drafts" {
                        continue;
                    }
                }
            }

            match Page::from_file(path, Path::new(&project_path)) {
                Ok(page) => pages.push(page),
                Err(e) => eprintln!("Failed to parse page: {}", e),
            }
        }
    }

    pages.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    Ok(pages)
}

// ====================
// Drafts Commands
// ====================

#[command]
pub fn list_drafts(project_path: String) -> Result<Vec<Draft>, String> {
    let project = HexoProject::new(PathBuf::from(&project_path));
    let drafts_dir = project.path.join("source").join("_drafts");

    if !drafts_dir.exists() {
        return Ok(Vec::new());
    }

    let mut drafts = Vec::new();

    for entry in walkdir::WalkDir::new(&drafts_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            match Draft::from_file(path, Path::new(&project_path)) {
                Ok(draft) => drafts.push(draft),
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
    let project = HexoProject::new(PathBuf::from(&project_path));
    let images_dir = project.get_images_dir();

    if !images_dir.exists() {
        return Ok(Vec::new());
    }

    let mut images = Vec::new();

    for entry in walkdir::WalkDir::new(&images_dir)
        .max_depth(10) // Allow subdirectories in images
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "ico") {
                    match create_image_info(path, &images_dir, Path::new(&project_path)) {
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
pub fn copy_image_to_project(
    project_path: String,
    source_path: String,
) -> Result<String, String> {
    let project = HexoProject::new(PathBuf::from(&project_path));
    let images_dir = project.get_images_dir();

    // Create images directory if it doesn't exist
    fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;

    let source = Path::new(&source_path);
    let filename = source
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid source filename")?;

    let dest_path = images_dir.join(filename);

    // Handle duplicate filenames
    let final_dest = if dest_path.exists() {
        let timestamp = chrono::Utc::now().timestamp();
        let stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = source.extension().and_then(|s| s.to_str()).unwrap_or("");
        images_dir.join(format!("{}_{}.{}", stem, timestamp, ext))
    } else {
        dest_path
    };

    fs::copy(&source, &final_dest)
        .map_err(|e| format!("Failed to copy image: {}", e))?;

    // Return URL path for markdown
    let relative_path = final_dest
        .strip_prefix(&project_path)
        .ok()
        .and_then(|p| p.to_str())
        .ok_or("Failed to get relative path")?;

    Ok(format!("/{}", relative_path.replace('\\', "/")))
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
    images_dir: &Path,
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
        .strip_prefix(images_dir)
        .ok()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string();

    let full_path = image_path
        .to_string_lossy()
        .to_string();

    let url = format!("/images/{}", path.replace('\\', "/"));

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
pub struct HexoConfig {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub language: String,
    pub url: String,
    #[serde(flatten)]
    pub other: serde_json::Value,
}
