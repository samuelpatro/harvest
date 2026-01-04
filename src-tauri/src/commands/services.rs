use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub status: String, // "running", "stopped", "not_installed"
    pub port: u16,
    pub pid: Option<u32>,
    pub path: Option<String>,
    pub description: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub exe_name: String,
    pub possible_paths: Vec<String>,
    pub start_args: Vec<String>,
    pub stop_method: String, // "taskkill", "signal", "service"
    pub description: String,
    pub version_arg: Option<String>, // argument to get version, e.g. "-v" or "--version"
}

// Harvest home directory - all services live here
const HARVEST_HOME: &str = "C:/harvest";

fn get_service_configs() -> Vec<ServiceConfig> {
    vec![
        ServiceConfig {
            id: "nginx".to_string(),
            name: "Nginx".to_string(),
            port: 80,
            exe_name: "nginx.exe".to_string(),
            possible_paths: vec![
                format!("{}/bin/nginx", HARVEST_HOME),
            ],
            start_args: vec![
                "-c".to_string(),
                format!("{}/etc/nginx/nginx.conf", HARVEST_HOME),
            ],
            stop_method: "signal".to_string(),
            description: "High-performance HTTP server and reverse proxy".to_string(),
            version_arg: Some("-v".to_string()),
        },
        ServiceConfig {
            id: "mysql".to_string(),
            name: "MySQL".to_string(),
            port: 3306,
            exe_name: "mysqld.exe".to_string(),
            possible_paths: vec![
                format!("{}/bin/mysql", HARVEST_HOME),
            ],
            start_args: vec![],
            stop_method: "service".to_string(),
            description: "Open-source relational database".to_string(),
            version_arg: Some("--version".to_string()),
        },
        ServiceConfig {
            id: "redis".to_string(),
            name: "Redis".to_string(),
            port: 6379,
            exe_name: "redis-server.exe".to_string(),
            possible_paths: vec![
                format!("{}/bin/redis", HARVEST_HOME),
            ],
            start_args: vec![],
            stop_method: "taskkill".to_string(),
            description: "In-memory data store and cache".to_string(),
            version_arg: Some("--version".to_string()),
        },
    ]
}

fn find_executable(config: &ServiceConfig) -> Option<PathBuf> {
    for base_path in &config.possible_paths {
        let base = PathBuf::from(base_path);

        // Check direct path
        let direct = base.join(&config.exe_name);
        if direct.exists() {
            return Some(direct);
        }

        // Check in bin subdirectory
        let bin_path = base.join("bin").join(&config.exe_name);
        if bin_path.exists() {
            return Some(bin_path);
        }

        // For Laragon, check versioned subdirectories
        if base.exists() && base.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Check in versioned folder
                        let exe_path = path.join(&config.exe_name);
                        if exe_path.exists() {
                            return Some(exe_path);
                        }
                        // Check in bin subfolder
                        let bin_exe = path.join("bin").join(&config.exe_name);
                        if bin_exe.exists() {
                            return Some(bin_exe);
                        }
                    }
                }
            }
        }
    }
    None
}

fn is_process_running(exe_name: &str) -> Option<u32> {
    let output = Command::new("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {}", exe_name), "/FO", "CSV", "/NH"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse CSV output: "nginx.exe","1234","Console","1","12,345 K"
    for line in stdout.lines() {
        if line.contains(exe_name) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let pid_str = parts[1].trim_matches('"');
                if let Ok(pid) = pid_str.parse::<u32>() {
                    return Some(pid);
                }
            }
        }
    }
    None
}

fn is_port_in_use(port: u16) -> bool {
    let output = Command::new("netstat")
        .args(["-an"])
        .output()
        .ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let port_str = format!(":{}", port);
        for line in stdout.lines() {
            if line.contains(&port_str) && line.contains("LISTENING") {
                return true;
            }
        }
    }
    false
}

fn get_service_version(exe_path: &PathBuf, version_arg: &Option<String>) -> Option<String> {
    let arg = version_arg.as_ref()?;

    let output = Command::new(exe_path)
        .arg(arg)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Try to extract version number from output
    for line in combined.lines() {
        let line = line.to_lowercase();
        // Look for version patterns like "nginx/1.24.0" or "Ver 8.0.30" or "redis-server v=7.0.0"
        if line.contains("version") || line.contains("ver ") || line.contains("/") || line.contains("v=") {
            // Extract version number using regex-like matching
            let version = extract_version_from_string(&combined);
            if version.is_some() {
                return version;
            }
        }
    }

    // Fallback: try to find any version-like pattern
    extract_version_from_string(&combined)
}

/// Extract version number from a string containing version info
/// Looks for patterns like "1.24.0", "8.0.30", etc.
pub fn extract_version_from_string(s: &str) -> Option<String> {
    // Look for patterns like "1.24.0", "8.0.30", etc.
    let mut version = String::new();
    let mut in_version = false;
    let mut dot_count = 0;

    for c in s.chars() {
        if c.is_ascii_digit() {
            in_version = true;
            version.push(c);
        } else if c == '.' && in_version && dot_count < 3 {
            version.push(c);
            dot_count += 1;
        } else if in_version && version.len() >= 3 && dot_count >= 1 {
            // We have a valid version, stop
            break;
        } else if in_version {
            // Reset if we hit a non-version character before getting a valid version
            if version.len() < 3 || dot_count < 1 {
                version.clear();
                dot_count = 0;
                in_version = false;
            } else {
                break;
            }
        }
    }

    if version.len() >= 3 && dot_count >= 1 {
        // Clean up trailing dots
        Some(version.trim_end_matches('.').to_string())
    } else {
        None
    }
}

#[tauri::command]
pub fn get_services() -> Vec<ServiceInfo> {
    let configs = get_service_configs();

    configs.iter().map(|config| {
        let exe_path = find_executable(config);
        let pid = is_process_running(&config.exe_name);

        // Get version if executable exists
        let version = exe_path.as_ref()
            .and_then(|path| get_service_version(path, &config.version_arg));

        // Only check if process is running, not port (port might be used by another service)
        let status = if exe_path.is_none() {
            "not_installed".to_string()
        } else if pid.is_some() {
            "running".to_string()
        } else {
            "stopped".to_string()
        };

        ServiceInfo {
            id: config.id.clone(),
            name: config.name.clone(),
            status,
            port: config.port,
            pid,
            path: exe_path.map(|p| p.to_string_lossy().to_string()),
            description: config.description.clone(),
            version,
        }
    }).collect()
}

#[tauri::command]
pub async fn start_service(id: String) -> Result<(), String> {
    let configs = get_service_configs();
    let config = configs.iter()
        .find(|c| c.id == id)
        .ok_or_else(|| format!("Service '{}' not found", id))?;

    let exe_path = find_executable(config)
        .ok_or_else(|| format!("{} is not installed", config.name))?;

    // Check if already running
    if is_process_running(&config.exe_name).is_some() {
        return Err(format!("{} is already running", config.name));
    }

    // Get the working directory (parent of executable)
    let working_dir = exe_path.parent()
        .ok_or_else(|| "Could not determine working directory".to_string())?;

    // Start the service
    let mut cmd = Command::new(&exe_path);
    cmd.current_dir(working_dir);

    for arg in &config.start_args {
        cmd.arg(arg);
    }

    // For services that don't detach, we need to spawn them differently
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        const DETACHED_PROCESS: u32 = 0x00000008;
        cmd.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
    }

    cmd.spawn()
        .map_err(|e| format!("Failed to start {}: {}", config.name, e))?;

    // Wait a moment and verify it started
    std::thread::sleep(std::time::Duration::from_millis(500));

    if is_process_running(&config.exe_name).is_some() || is_port_in_use(config.port) {
        Ok(())
    } else {
        Err(format!("{} failed to start", config.name))
    }
}

#[tauri::command]
pub async fn stop_service(id: String) -> Result<(), String> {
    let configs = get_service_configs();
    let config = configs.iter()
        .find(|c| c.id == id)
        .ok_or_else(|| format!("Service '{}' not found", id))?;

    let pid = is_process_running(&config.exe_name);

    if pid.is_none() {
        return Err(format!("{} is not running", config.name));
    }

    match config.stop_method.as_str() {
        "signal" => {
            // For nginx, use nginx -s stop with config path
            if let Some(exe_path) = find_executable(config) {
                let output = Command::new(&exe_path)
                    .args(["-c", &format!("{}/etc/nginx/nginx.conf", HARVEST_HOME), "-s", "stop"])
                    .output()
                    .map_err(|e| format!("Failed to stop {}: {}", config.name, e))?;

                // If graceful stop fails, force kill
                if !output.status.success() {
                    Command::new("taskkill")
                        .args(["/F", "/IM", &config.exe_name])
                        .output()
                        .map_err(|e| format!("Failed to force stop {}: {}", config.name, e))?;
                }
            }
        },
        "service" => {
            // For MySQL, try mysqladmin shutdown first
            if config.id == "mysql" {
                let _ = Command::new("cmd")
                    .args(["/c", "mysqladmin", "-u", "root", "shutdown"])
                    .output();
            }
            // Fallback to taskkill
            if is_process_running(&config.exe_name).is_some() {
                Command::new("taskkill")
                    .args(["/F", "/IM", &config.exe_name])
                    .output()
                    .map_err(|e| format!("Failed to stop {}: {}", config.name, e))?;
            }
        },
        _ => {
            // Default: taskkill
            Command::new("taskkill")
                .args(["/F", "/IM", &config.exe_name])
                .output()
                .map_err(|e| format!("Failed to stop {}: {}", config.name, e))?;
        }
    }

    // Wait and verify
    std::thread::sleep(std::time::Duration::from_millis(500));

    if is_process_running(&config.exe_name).is_none() {
        Ok(())
    } else {
        Err(format!("{} did not stop cleanly", config.name))
    }
}

#[tauri::command]
pub async fn restart_service(id: String) -> Result<(), String> {
    // Stop if running
    let _ = stop_service(id.clone()).await;

    // Wait a moment
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Start
    start_service(id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_info_creation() {
        let info = ServiceInfo {
            id: "nginx".to_string(),
            name: "Nginx".to_string(),
            status: "running".to_string(),
            port: 80,
            pid: Some(1234),
            path: Some("/usr/sbin/nginx".to_string()),
            description: "HTTP server".to_string(),
            version: Some("1.24.0".to_string()),
        };
        assert_eq!(info.id, "nginx");
        assert_eq!(info.status, "running");
        assert!(info.pid.is_some());
    }

    #[test]
    fn test_service_config_creation() {
        let config = ServiceConfig {
            id: "redis".to_string(),
            name: "Redis".to_string(),
            port: 6379,
            exe_name: "redis-server".to_string(),
            possible_paths: vec!["/usr/bin".to_string()],
            start_args: vec![],
            stop_method: "signal".to_string(),
            description: "In-memory database".to_string(),
            version_arg: Some("--version".to_string()),
        };
        assert_eq!(config.port, 6379);
        assert!(config.version_arg.is_some());
    }

    #[test]
    fn test_extract_version_nginx() {
        let output = "nginx version: nginx/1.24.0";
        let version = extract_version_from_string(output);
        assert_eq!(version, Some("1.24.0".to_string()));
    }

    #[test]
    fn test_extract_version_mysql() {
        let output = "mysqld  Ver 8.0.30 for Win64 on x86_64 (MySQL Community Server - GPL)";
        let version = extract_version_from_string(output);
        assert_eq!(version, Some("8.0.30".to_string()));
    }

    #[test]
    fn test_extract_version_redis() {
        let output = "Redis server v=7.2.4 sha=00000000:0 malloc=jemalloc-5.2.1 bits=64";
        let version = extract_version_from_string(output);
        assert_eq!(version, Some("7.2.4".to_string()));
    }

    #[test]
    fn test_extract_version_simple() {
        let output = "version 1.0.0";
        let version = extract_version_from_string(output);
        assert_eq!(version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_extract_version_with_prefix() {
        let output = "v2.5.1 release";
        let version = extract_version_from_string(output);
        assert_eq!(version, Some("2.5.1".to_string()));
    }

    #[test]
    fn test_extract_version_no_version() {
        let output = "no version here";
        let version = extract_version_from_string(output);
        assert!(version.is_none());
    }

    #[test]
    fn test_extract_version_single_number() {
        let output = "version 5";
        let version = extract_version_from_string(output);
        assert!(version.is_none()); // Need at least one dot
    }

    #[test]
    fn test_extract_version_two_parts() {
        let output = "PHP 8.3";
        let version = extract_version_from_string(output);
        assert_eq!(version, Some("8.3".to_string()));
    }

    #[test]
    fn test_extract_version_four_parts() {
        let output = "Build 1.2.3.4";
        let version = extract_version_from_string(output);
        // Should stop at third dot or handle gracefully
        assert!(version.is_some());
    }

    #[test]
    fn test_get_service_configs_returns_all() {
        let configs = get_service_configs();
        assert_eq!(configs.len(), 3); // nginx, mysql, redis

        let ids: Vec<&str> = configs.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"nginx"));
        assert!(ids.contains(&"mysql"));
        assert!(ids.contains(&"redis"));
    }

    #[test]
    fn test_get_service_configs_nginx() {
        let configs = get_service_configs();
        let nginx = configs.iter().find(|c| c.id == "nginx").unwrap();

        assert_eq!(nginx.name, "Nginx");
        assert_eq!(nginx.port, 80);
        assert_eq!(nginx.exe_name, "nginx.exe");
        assert_eq!(nginx.stop_method, "signal");
    }

    #[test]
    fn test_get_service_configs_mysql() {
        let configs = get_service_configs();
        let mysql = configs.iter().find(|c| c.id == "mysql").unwrap();

        assert_eq!(mysql.name, "MySQL");
        assert_eq!(mysql.port, 3306);
        assert_eq!(mysql.exe_name, "mysqld.exe");
        assert_eq!(mysql.stop_method, "service");
    }

    #[test]
    fn test_get_service_configs_redis() {
        let configs = get_service_configs();
        let redis = configs.iter().find(|c| c.id == "redis").unwrap();

        assert_eq!(redis.name, "Redis");
        assert_eq!(redis.port, 6379);
        assert_eq!(redis.exe_name, "redis-server.exe");
        assert_eq!(redis.stop_method, "taskkill");
    }
}
