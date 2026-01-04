use std::fs;
use std::path::PathBuf;
use crate::types::AppConfig;

fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("harvest");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("config.json")
}

#[tauri::command]
pub fn get_config() -> AppConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
                // Ensure harvest_home has a value (for configs created before this field existed)
                if config.harvest_home.is_empty() {
                    config.harvest_home = crate::types::get_default_harvest_home();
                }
                return config;
            }
        }
    }
    AppConfig::default()
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<(), String> {
    let config_path = get_config_path();
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

/// Check if this is a first run (harvest home doesn't exist or not configured)
#[tauri::command]
pub fn is_first_run() -> bool {
    let config = get_config();
    if config.first_run_complete {
        return false;
    }

    // Check if harvest home directory exists with proper structure
    let harvest_home = PathBuf::from(&config.harvest_home);
    !harvest_home.join("bin").exists()
}

/// Get the current harvest home path
#[tauri::command]
pub fn get_harvest_home() -> String {
    get_config().harvest_home
}

/// Initialize the harvest home directory structure
#[tauri::command]
pub fn initialize_harvest_home(path: String) -> Result<(), String> {
    let harvest_home = PathBuf::from(&path);

    // Create main directories
    let dirs = ["bin", "etc", "logs", "etc/nginx", "etc/nginx/sites-enabled", "etc/ssl"];

    for dir in dirs {
        let dir_path = harvest_home.join(dir);
        fs::create_dir_all(&dir_path)
            .map_err(|e| format!("Failed to create {}: {}", dir, e))?;
    }

    // Update config with new harvest_home and mark first run complete
    let mut config = get_config();
    config.harvest_home = path;
    config.first_run_complete = true;

    // Set default harvest_paths to harvest_home/www if it doesn't exist
    if config.harvest_paths.is_empty() {
        let www_path = harvest_home.join("www");
        fs::create_dir_all(&www_path)
            .map_err(|e| format!("Failed to create www directory: {}", e))?;
        config.harvest_paths = vec![www_path.to_string_lossy().to_string()];
    }

    save_config(config)?;

    Ok(())
}

/// Validate that a path can be used as harvest home
#[tauri::command]
pub fn validate_harvest_path(path: String) -> Result<bool, String> {
    let harvest_path = PathBuf::from(&path);

    // Check if parent directory exists
    if let Some(parent) = harvest_path.parent() {
        if !parent.exists() {
            return Err(format!("Parent directory does not exist: {}", parent.display()));
        }
    }

    // Check if path already exists and has content (warn but allow)
    if harvest_path.exists() {
        let entries: Vec<_> = fs::read_dir(&harvest_path)
            .map_err(|e| e.to_string())?
            .collect();
        if !entries.is_empty() {
            // Path exists with content - that's okay, we'll merge
            return Ok(true);
        }
    }

    Ok(true)
}
