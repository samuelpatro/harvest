//! Thin Tauri command wrappers over `harvest_core::php`.

use harvest_core::types::{AvailableVersion, InstalledVersion, PhpExtension};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_available_php_versions(force_refresh: Option<bool>) -> Result<Vec<AvailableVersion>, String> {
    harvest_core::php::get_available_php_versions(force_refresh).await
}

#[tauri::command]
pub fn get_installed_php_versions() -> Vec<InstalledVersion> {
    harvest_core::php::get_installed_php_versions()
}

#[tauri::command]
pub async fn install_php_version(app: AppHandle, version: String) -> Result<(), String> {
    let sink = super::make_progress_sink(app);
    harvest_core::php::install_php_version(version, sink).await
}

#[tauri::command]
pub async fn uninstall_php_version(version: String) -> Result<(), String> {
    harvest_core::php::uninstall_php_version(version).await
}

#[tauri::command]
pub fn get_php_extensions(php_path: String) -> Result<Vec<PhpExtension>, String> {
    harvest_core::php::get_php_extensions(php_path)
}

#[tauri::command]
pub fn toggle_php_extension(php_path: String, extension: String, enable: bool) -> Result<(), String> {
    harvest_core::php::toggle_php_extension(php_path, extension, enable)
}
