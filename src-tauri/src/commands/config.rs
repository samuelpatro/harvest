//! Thin Tauri command wrappers over `harvest_core::config`.

use harvest_core::types::AppConfig;

#[tauri::command]
pub fn get_config() -> AppConfig {
    harvest_core::config::get_config()
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<(), String> {
    harvest_core::config::save_config(config)
}

#[tauri::command]
pub fn is_first_run() -> bool {
    harvest_core::config::is_first_run()
}

#[tauri::command]
pub fn get_harvest_home() -> String {
    harvest_core::config::get_harvest_home()
}

#[tauri::command]
pub fn initialize_harvest_home(path: String) -> Result<(), String> {
    harvest_core::config::initialize_harvest_home(path)
}

#[tauri::command]
pub fn validate_harvest_path(path: String) -> Result<bool, String> {
    harvest_core::config::validate_harvest_path(path)
}

#[tauri::command]
pub fn is_in_path(path: String) -> bool {
    harvest_core::config::is_in_path(path)
}

#[tauri::command]
pub fn add_to_path(path: String) -> Result<(), String> {
    harvest_core::config::add_to_path(path)
}

#[tauri::command]
pub fn remove_from_path(path: String) -> Result<(), String> {
    harvest_core::config::remove_from_path(path)
}

#[tauri::command]
pub fn get_default_php_path() -> Option<String> {
    harvest_core::config::get_default_php_path()
}

#[tauri::command]
pub fn get_default_node_path() -> Option<String> {
    harvest_core::config::get_default_node_path()
}
