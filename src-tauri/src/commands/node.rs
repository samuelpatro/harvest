//! Thin Tauri command wrappers over `harvest_core::node`.

use harvest_core::types::{AvailableVersion, InstalledVersion, PackageManager};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_available_node_versions(force_refresh: Option<bool>) -> Result<Vec<AvailableVersion>, String> {
    harvest_core::node::get_available_node_versions(force_refresh).await
}

#[tauri::command]
pub fn get_installed_node_versions() -> Vec<InstalledVersion> {
    harvest_core::node::get_installed_node_versions()
}

#[tauri::command]
pub async fn install_node_version(app: AppHandle, version: String) -> Result<(), String> {
    let sink = super::make_progress_sink(app);
    harvest_core::node::install_node_version(version, sink).await
}

#[tauri::command]
pub async fn uninstall_node_version(version: String) -> Result<(), String> {
    harvest_core::node::uninstall_node_version(version).await
}

#[tauri::command]
pub fn get_package_managers() -> Vec<PackageManager> {
    harvest_core::node::get_package_managers()
}
