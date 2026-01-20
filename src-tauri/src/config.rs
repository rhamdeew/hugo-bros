// Application configuration management

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub version: String,
    pub last_project_path: Option<String>,
    pub recent_projects: Vec<String>,
    pub ui_language: String,
    pub theme: String,
    pub auto_save_enabled: bool,
    pub auto_save_interval: u32,
    pub editor_font_size: u32,
    pub editor_line_height: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            last_project_path: None,
            recent_projects: Vec::new(),
            ui_language: "en".to_string(),
            theme: "auto".to_string(),
            auto_save_enabled: true,
            auto_save_interval: 30,
            editor_font_size: 16,
            editor_line_height: 1.5,
        }
    }
}

impl AppConfig {
    fn get_config_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or("Failed to get config directory")?;
        let app_config_dir = config_dir.join("hugo-bros");

        // Create directory if it doesn't exist
        if !app_config_dir.exists() {
            fs::create_dir_all(&app_config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        Ok(app_config_dir.join("config.json"))
    }

    pub fn load() -> Result<Self, String> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let config_str = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        serde_json::from_str(&config_str)
            .map_err(|e| format!("Failed to parse config file: {}", e))
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::get_config_path()?;

        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, config_str)
            .map_err(|e| format!("Failed to write config file: {}", e))
    }

    pub fn add_recent_project(&mut self, project_path: String) {
        // Remove if already exists (to move to front)
        self.recent_projects.retain(|p| p != &project_path);

        // Add to front of list
        self.recent_projects.insert(0, project_path.clone());

        // Keep only last 10 projects
        if self.recent_projects.len() > 10 {
            self.recent_projects.truncate(10);
        }

        // Update last_project_path
        self.last_project_path = Some(project_path);
    }
}
