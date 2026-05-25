//! Thin Tauri command wrappers over `harvest_core::sites`.

use harvest_core::types::Site;

#[tauri::command]
pub fn scan_sites(directories: Vec<String>) -> Result<Vec<Site>, String> {
    harvest_core::sites::scan_sites(directories)
}

#[tauri::command]
pub async fn open_in_editor(path: String, editor: String) -> Result<(), String> {
    harvest_core::sites::open_in_editor(path, editor).await
}

#[tauri::command]
pub async fn open_in_terminal(path: String, terminal: String) -> Result<(), String> {
    harvest_core::sites::open_in_terminal(path, terminal).await
}

#[tauri::command]
pub async fn open_in_browser(url: String) -> Result<(), String> {
    harvest_core::sites::open_in_browser(url).await
}
