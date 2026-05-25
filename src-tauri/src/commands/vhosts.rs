//! Thin Tauri command wrappers over `harvest_core::vhosts`.

use harvest_core::vhosts::VhostStatus;

#[tauri::command]
pub fn get_vhost_status(site_name: String) -> VhostStatus {
    harvest_core::vhosts::get_vhost_status(site_name)
}

#[tauri::command]
pub fn get_all_vhost_statuses(site_names: Vec<String>) -> Vec<VhostStatus> {
    harvest_core::vhosts::get_all_vhost_statuses(site_names)
}

#[tauri::command]
pub async fn link_site(site_name: String, root_path: String) -> Result<String, String> {
    harvest_core::vhosts::link_site(site_name, root_path).await
}

#[tauri::command]
pub async fn unlink_site(site_name: String) -> Result<String, String> {
    harvest_core::vhosts::unlink_site(site_name).await
}

#[tauri::command]
pub fn check_nginx_config() -> Result<String, String> {
    harvest_core::vhosts::check_nginx_config()
}
