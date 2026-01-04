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

// ============ PATH Management Commands ============

/// Normalize a path for comparison (Windows-style with backslashes, lowercase)
pub fn normalize_path_for_comparison(path: &str) -> String {
    path.replace('/', "\\").to_lowercase().trim().trim_end_matches('\\').to_string()
}

/// Check if a path exists in a PATH-style string (semicolon-separated on Windows)
pub fn path_exists_in_pathvar(path: &str, path_var: &str, separator: char) -> bool {
    let normalized = normalize_path_for_comparison(path);
    path_var.split(separator).any(|p| {
        let p_normalized = normalize_path_for_comparison(p);
        p_normalized == normalized
    })
}

/// Filter out a path from a PATH-style string
pub fn filter_path_from_pathvar(path_to_remove: &str, path_var: &str, separator: char) -> String {
    let normalized = normalize_path_for_comparison(path_to_remove);
    path_var
        .split(separator)
        .filter(|p| {
            let p_normalized = normalize_path_for_comparison(p);
            p_normalized != normalized
        })
        .collect::<Vec<&str>>()
        .join(&separator.to_string())
}

/// Check if a specific path is in the user's PATH environment variable
#[tauri::command]
pub fn is_in_path(path: String) -> bool {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(env) = hkcu.open_subkey("Environment") {
            if let Ok(current_path) = env.get_value::<String, _>("Path") {
                let normalized_path = path.replace('/', "\\").to_lowercase();
                return current_path
                    .split(';')
                    .any(|p| p.to_lowercase().trim() == normalized_path ||
                         p.to_lowercase().trim().trim_end_matches('\\') == normalized_path.trim_end_matches('\\'));
            }
        }
        false
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(env_path) = std::env::var("PATH") {
            return env_path.split(':').any(|p| p == path);
        }
        false
    }
}

/// Add a path to the user's PATH environment variable
#[tauri::command]
pub fn add_to_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
            .map_err(|e| format!("Failed to open registry: {}", e))?;

        let current_path: String = env.get_value("Path").unwrap_or_default();

        // Normalize the path (use backslashes on Windows)
        let normalized_path = path.replace('/', "\\");

        // Check if already in PATH
        let paths: Vec<&str> = current_path.split(';').collect();
        let already_exists = paths.iter().any(|p| {
            p.to_lowercase().trim() == normalized_path.to_lowercase() ||
            p.to_lowercase().trim().trim_end_matches('\\') == normalized_path.to_lowercase().trim_end_matches('\\')
        });

        if already_exists {
            return Ok(()); // Already in PATH
        }

        // Add the new path
        let new_path = if current_path.is_empty() {
            normalized_path
        } else {
            format!("{};{}", current_path.trim_end_matches(';'), normalized_path)
        };

        env.set_value("Path", &new_path)
            .map_err(|e| format!("Failed to set PATH: {}", e))?;

        // Broadcast WM_SETTINGCHANGE to notify other applications
        broadcast_environment_change();

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On Unix systems, we'd need to modify shell profile files
        // This is more complex and varies by shell
        Err("PATH modification on this platform requires editing shell profile files manually.".to_string())
    }
}

/// Remove a path from the user's PATH environment variable
#[tauri::command]
pub fn remove_from_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
            .map_err(|e| format!("Failed to open registry: {}", e))?;

        let current_path: String = env.get_value("Path").unwrap_or_default();
        let normalized_path = path.replace('/', "\\").to_lowercase();

        // Filter out the path we want to remove
        let new_paths: Vec<&str> = current_path
            .split(';')
            .filter(|p| {
                let p_lower = p.to_lowercase();
                p_lower.trim() != normalized_path &&
                p_lower.trim().trim_end_matches('\\') != normalized_path.trim_end_matches('\\')
            })
            .collect();

        let new_path = new_paths.join(";");

        env.set_value("Path", &new_path)
            .map_err(|e| format!("Failed to set PATH: {}", e))?;

        // Broadcast WM_SETTINGCHANGE to notify other applications
        broadcast_environment_change();

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("PATH modification on this platform requires editing shell profile files manually.".to_string())
    }
}

/// Get the current default PHP path (the one in PATH)
#[tauri::command]
pub fn get_default_php_path() -> Option<String> {
    let harvest_home = get_harvest_home();
    let php_dir = PathBuf::from(&harvest_home).join("bin").join("php");

    if !php_dir.exists() {
        return None;
    }

    // Check which PHP version directory is in PATH
    if let Ok(entries) = fs::read_dir(&php_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let path_str = path.to_string_lossy().to_string();
                if is_in_path(path_str.clone()) {
                    return Some(path_str);
                }
            }
        }
    }

    None
}

/// Get the current default Node.js path (the one in PATH)
#[tauri::command]
pub fn get_default_node_path() -> Option<String> {
    let harvest_home = get_harvest_home();
    let node_dir = PathBuf::from(&harvest_home).join("bin").join("nodejs");

    if !node_dir.exists() {
        return None;
    }

    // Check which Node.js version directory is in PATH
    if let Ok(entries) = fs::read_dir(&node_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let path_str = path.to_string_lossy().to_string();
                if is_in_path(path_str.clone()) {
                    return Some(path_str);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn broadcast_environment_change() {
    use std::ffi::CString;
    use std::ptr;

    // Use Windows API to broadcast WM_SETTINGCHANGE
    #[link(name = "user32")]
    extern "system" {
        fn SendMessageTimeoutA(
            hwnd: *mut std::ffi::c_void,
            msg: u32,
            wparam: usize,
            lparam: *const i8,
            flags: u32,
            timeout: u32,
            result: *mut usize,
        ) -> isize;
    }

    const HWND_BROADCAST: *mut std::ffi::c_void = 0xffff as *mut std::ffi::c_void;
    const WM_SETTINGCHANGE: u32 = 0x001A;
    const SMTO_ABORTIFHUNG: u32 = 0x0002;

    if let Ok(env) = CString::new("Environment") {
        unsafe {
            let mut result: usize = 0;
            SendMessageTimeoutA(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                env.as_ptr(),
                SMTO_ABORTIFHUNG,
                5000,
                &mut result,
            );
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn broadcast_environment_change() {
    // No-op on non-Windows platforms
}
