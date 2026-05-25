//! Thin Tauri command wrappers over `harvest_core::services`.

use harvest_core::services::ServiceInfo;
use harvest_core::types::InstallableService;
use tauri::AppHandle;

#[tauri::command]
pub fn get_services() -> Vec<ServiceInfo> {
    harvest_core::services::get_services()
}

#[tauri::command]
pub async fn start_service(id: String) -> Result<(), String> {
    harvest_core::services::start_service(id).await
}

#[tauri::command]
pub async fn stop_service(id: String) -> Result<(), String> {
    harvest_core::services::stop_service(id).await
}

#[tauri::command]
pub async fn restart_service(id: String) -> Result<(), String> {
    harvest_core::services::restart_service(id).await
}

#[tauri::command]
pub fn get_installable_services() -> Vec<InstallableService> {
    harvest_core::services::get_installable_services()
}

#[tauri::command]
pub async fn install_service(app: AppHandle, service_id: String, version: String) -> Result<(), String> {
    let sink = super::make_progress_sink(app);
    harvest_core::services::install_service(service_id, version, sink).await
}

#[tauri::command]
pub async fn uninstall_service(service_id: String) -> Result<(), String> {
    harvest_core::services::uninstall_service(service_id).await
}
