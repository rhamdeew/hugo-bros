// Hugo integration module
// Handles Hugo project structure, config parsing, and operations

use std::path::PathBuf;
use std::process::{Command, Child, Stdio};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Global state to track running Hugo servers
lazy_static::lazy_static! {
    static ref HUGO_SERVERS: Arc<Mutex<HashMap<String, Child>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub struct HugoProject {
    pub path: PathBuf,
}

impl HugoProject {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn validate(&self) -> Result<bool, String> {
        // Check if Hugo config exists
        if self.find_config_path().is_none() {
            return Err("Hugo config not found (config.* or hugo.*)".to_string());
        }

        // Check if content/ directory exists
        let content_path = self.get_content_dir();
        if !content_path.exists() || !content_path.is_dir() {
            return Err("content/ directory not found".to_string());
        }

        Ok(true)
    }

    pub fn find_config_path(&self) -> Option<PathBuf> {
        let candidates = [
            "hugo.toml",
            "hugo.yaml",
            "hugo.yml",
            "hugo.json",
            "config.toml",
            "config.yaml",
            "config.yml",
            "config.json",
        ];

        for candidate in candidates {
            let path = self.path.join(candidate);
            if path.exists() {
                return Some(path);
            }
        }

        let config_dir = self.path.join("config").join("_default");
        for candidate in candidates {
            let path = config_dir.join(candidate);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    pub fn get_content_dir(&self) -> PathBuf {
        self.path.join("content")
    }

    pub fn get_posts_dir(&self) -> PathBuf {
        let content_dir = self.get_content_dir();
        let posts_dir = content_dir.join("posts");
        if posts_dir.exists() {
            return posts_dir;
        }
        let post_dir = content_dir.join("post");
        if post_dir.exists() {
            return post_dir;
        }
        content_dir
    }

    pub fn get_pages_dir(&self) -> PathBuf {
        self.get_content_dir()
    }

    pub fn get_static_dir(&self) -> PathBuf {
        self.path.join("static")
    }

    /// Run a hugo command (build, clean, deploy, etc.)
    pub fn run_command(&self, args: &[String]) -> Result<CommandOutput, String> {
        let output = Command::new("hugo")
            .args(args)
            .current_dir(&self.path)
            .output()
            .map_err(|e| format!("Failed to execute hugo command: {}", e))?;

        Ok(CommandOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    /// Start hugo server in background
    pub fn start_server(&self) -> Result<String, String> {
        let server_id = self.path.to_string_lossy().to_string();

        // Check if server is already running
        {
            let servers = HUGO_SERVERS.lock().unwrap();
            if servers.contains_key(&server_id) {
                return Err("Server is already running".to_string());
            }
        }

        // Start hugo server
        let child = Command::new("hugo")
            .arg("server")
            .current_dir(&self.path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start hugo server: {}", e))?;

        // Store the child process
        {
            let mut servers = HUGO_SERVERS.lock().unwrap();
            servers.insert(server_id.clone(), child);
        }

        Ok(server_id)
    }

    /// Stop running hugo server
    pub fn stop_server(server_id: &str) -> Result<(), String> {
        let mut servers = HUGO_SERVERS.lock().unwrap();

        if let Some(mut child) = servers.remove(server_id) {
            child.kill()
                .map_err(|e| format!("Failed to kill server process: {}", e))?;
            Ok(())
        } else {
            Err("Server not found".to_string())
        }
    }

    /// Check if server is running
    pub fn is_server_running(&self) -> bool {
        let server_id = self.path.to_string_lossy().to_string();
        let servers = HUGO_SERVERS.lock().unwrap();
        servers.contains_key(&server_id)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CommandOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}
