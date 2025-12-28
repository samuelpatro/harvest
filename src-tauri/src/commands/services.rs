// Services module - Phase 2: Service Management
// This module will handle starting/stopping services like nginx, PHP-FPM, MySQL, Redis

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub status: String, // "running", "stopped", "error"
    pub port: u16,
    pub pid: Option<u32>,
}

// TODO: Implement service management commands
// #[tauri::command]
// pub async fn get_services() -> Result<Vec<ServiceInfo>, String> { }

// #[tauri::command]
// pub async fn start_service(name: String) -> Result<(), String> { }

// #[tauri::command]
// pub async fn stop_service(name: String) -> Result<(), String> { }

// #[tauri::command]
// pub async fn restart_service(name: String) -> Result<(), String> { }

// #[tauri::command]
// pub async fn get_service_logs(name: String, lines: u32) -> Result<Vec<String>, String> { }
