use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use crate::commands::config::get_harvest_home;

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

fn get_service_configs(harvest_home: &str) -> Vec<ServiceConfig> {
    vec![
        ServiceConfig {
            id: "nginx".to_string(),
            name: "Nginx".to_string(),
            port: 80,
            exe_name: "nginx.exe".to_string(),
            possible_paths: vec![
                format!("{}/bin/nginx", harvest_home),
            ],
            start_args: vec![
                "-c".to_string(),
                format!("{}/etc/nginx/nginx.conf", harvest_home),
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
                format!("{}/bin/mysql", harvest_home),
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
                format!("{}/bin/redis", harvest_home),
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

        // Check versioned subdirectories (e.g., mysql-8.0.33/, redis-7.2.4/)
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
    let harvest_home = get_harvest_home();
    let configs = get_service_configs(&harvest_home);

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
    let harvest_home = get_harvest_home();
    let configs = get_service_configs(&harvest_home);
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
    let harvest_home = get_harvest_home();
    let configs = get_service_configs(&harvest_home);
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
                let nginx_conf = format!("{}/etc/nginx/nginx.conf", harvest_home);
                let output = Command::new(&exe_path)
                    .args(["-c", &nginx_conf, "-s", "stop"])
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

// ============ Service Installation Commands ============

use std::fs::{self, File};
use std::io::{self, Write};
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use crate::types::{InstallableService, ServiceVersion};

/// Information about available service downloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableService {
    pub id: String,
    pub name: String,
    pub description: String,
    pub versions: Vec<ServiceVersion>,
}

fn get_available_service_downloads() -> Vec<AvailableService> {
    vec![
        AvailableService {
            id: "nginx".to_string(),
            name: "Nginx".to_string(),
            description: "High-performance HTTP server and reverse proxy".to_string(),
            versions: vec![
                ServiceVersion {
                    version: "1.27.3".to_string(),
                    download_url: "https://nginx.org/download/nginx-1.27.3.zip".to_string(),
                    filename: "nginx-1.27.3.zip".to_string(),
                },
                ServiceVersion {
                    version: "1.26.2".to_string(),
                    download_url: "https://nginx.org/download/nginx-1.26.2.zip".to_string(),
                    filename: "nginx-1.26.2.zip".to_string(),
                },
            ],
        },
        AvailableService {
            id: "mysql".to_string(),
            name: "MySQL".to_string(),
            description: "Open-source relational database".to_string(),
            versions: vec![
                ServiceVersion {
                    version: "8.0.40".to_string(),
                    download_url: "https://dev.mysql.com/get/Downloads/MySQL-8.0/mysql-8.0.40-winx64.zip".to_string(),
                    filename: "mysql-8.0.40-winx64.zip".to_string(),
                },
                ServiceVersion {
                    version: "8.4.3".to_string(),
                    download_url: "https://dev.mysql.com/get/Downloads/MySQL-8.4/mysql-8.4.3-winx64.zip".to_string(),
                    filename: "mysql-8.4.3-winx64.zip".to_string(),
                },
            ],
        },
        AvailableService {
            id: "redis".to_string(),
            name: "Redis".to_string(),
            description: "In-memory data store and cache".to_string(),
            versions: vec![
                ServiceVersion {
                    version: "7.4.1".to_string(),
                    download_url: "https://github.com/redis-windows/redis-windows/releases/download/7.4.1/Redis-7.4.1-Windows-x64.zip".to_string(),
                    filename: "Redis-7.4.1-Windows-x64.zip".to_string(),
                },
                ServiceVersion {
                    version: "7.2.6".to_string(),
                    download_url: "https://github.com/redis-windows/redis-windows/releases/download/7.2.6/Redis-7.2.6-Windows-x64.zip".to_string(),
                    filename: "Redis-7.2.6-Windows-x64.zip".to_string(),
                },
            ],
        },
    ]
}

#[tauri::command]
pub fn get_installable_services() -> Vec<InstallableService> {
    let harvest_home = get_harvest_home();
    let available = get_available_service_downloads();

    available.into_iter().map(|service| {
        let service_path = PathBuf::from(&harvest_home).join("bin").join(&service.id);

        // Check if service is installed by looking for the main executable
        let (installed, version) = if service_path.exists() {
            let exe_name = match service.id.as_str() {
                "nginx" => "nginx.exe",
                "mysql" => "bin/mysqld.exe",
                "redis" => "redis-server.exe",
                _ => "",
            };

            let exe_exists = if exe_name.is_empty() {
                false
            } else {
                service_path.join(exe_name).exists()
            };

            if exe_exists {
                // Try to get version from executable
                let version = get_installed_service_version(&service.id, &service_path);
                (true, version)
            } else {
                (false, None)
            }
        } else {
            (false, None)
        };

        InstallableService {
            id: service.id,
            name: service.name,
            description: service.description,
            installed,
            version,
            available_versions: service.versions,
        }
    }).collect()
}

fn get_installed_service_version(service_id: &str, path: &PathBuf) -> Option<String> {
    let (exe_path, version_arg): (PathBuf, &str) = match service_id {
        "nginx" => (path.join("nginx.exe"), "-v"),
        "mysql" => (path.join("bin").join("mysqld.exe"), "--version"),
        "redis" => (path.join("redis-server.exe"), "--version"),
        _ => return None,
    };

    if !exe_path.exists() {
        return None;
    }

    let output = Command::new(&exe_path)
        .arg(version_arg)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    extract_version_from_string(&combined)
}

#[tauri::command]
pub async fn install_service(app: AppHandle, service_id: String, version: String) -> Result<(), String> {
    let harvest_home = get_harvest_home();
    let available = get_available_service_downloads();

    let service = available.iter()
        .find(|s| s.id == service_id)
        .ok_or_else(|| format!("Service '{}' not found", service_id))?;

    let service_version = service.versions.iter()
        .find(|v| v.version == version)
        .ok_or_else(|| format!("Version '{}' not found for {}", version, service_id))?;

    let install_dir = PathBuf::from(&harvest_home).join("bin").join(&service_id);
    fs::create_dir_all(&install_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Emit progress event
    let _ = app.emit("install-progress", serde_json::json!({
        "type": "service",
        "service": service_id,
        "version": version,
        "status": "downloading",
        "progress": 0
    }));

    // Download the file
    let client = reqwest::Client::builder()
        .user_agent("Harvest/1.0")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&service_version.download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let temp_dir = std::env::temp_dir();
    let zip_path = temp_dir.join(&service_version.filename);

    // Stream download to file
    let mut file = File::create(&zip_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write: {}", e))?;

        downloaded += chunk.len() as u64;
        let progress = if total_size > 0 {
            (downloaded as f64 / total_size as f64 * 100.0) as u32
        } else {
            0
        };

        let _ = app.emit("install-progress", serde_json::json!({
            "type": "service",
            "service": service_id,
            "version": version,
            "status": "downloading",
            "progress": progress
        }));
    }

    let _ = app.emit("install-progress", serde_json::json!({
        "type": "service",
        "service": service_id,
        "version": version,
        "status": "extracting",
        "progress": 100
    }));

    // Extract the zip file
    let zip_file = File::open(&zip_path)
        .map_err(|e| format!("Failed to open zip: {}", e))?;

    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| format!("Failed to read zip: {}", e))?;

    // For services, extract directly to install_dir (handling nested folders)
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;

        // Remove the top-level directory from the path
        let file_path = file.name();
        let stripped_path = if let Some(pos) = file_path.find('/') {
            &file_path[pos + 1..]
        } else {
            file_path
        };

        if stripped_path.is_empty() {
            continue;
        }

        let outpath = install_dir.join(stripped_path);

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut outfile = File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }

    // Clean up temp file
    fs::remove_file(&zip_path).ok();

    // Post-installation setup for specific services
    post_install_setup(&service_id, &install_dir, &harvest_home)?;

    let _ = app.emit("install-progress", serde_json::json!({
        "type": "service",
        "service": service_id,
        "version": version,
        "status": "complete",
        "progress": 100
    }));

    Ok(())
}

fn post_install_setup(service_id: &str, install_dir: &PathBuf, harvest_home: &str) -> Result<(), String> {
    match service_id {
        "nginx" => {
            // Create nginx config directory structure
            let etc_nginx = PathBuf::from(harvest_home).join("etc").join("nginx");
            fs::create_dir_all(etc_nginx.join("sites-enabled")).ok();
            fs::create_dir_all(etc_nginx.join("snippets")).ok();

            // Create a basic nginx.conf if it doesn't exist
            let nginx_conf = etc_nginx.join("nginx.conf");
            if !nginx_conf.exists() {
                let default_config = format!(r#"worker_processes  auto;

events {{
    worker_connections  1024;
}}

http {{
    include       mime.types;
    default_type  application/octet-stream;
    sendfile        on;
    keepalive_timeout  65;

    # PHP upstream
    upstream php_upstream {{
        server 127.0.0.1:9000;
    }}

    # FastCGI snippets
    include {}/etc/nginx/snippets/*.conf;

    # Virtual hosts
    include {}/etc/nginx/sites-enabled/*.conf;

    # Default server
    server {{
        listen       80;
        server_name  localhost;
        root         {}/www;

        location / {{
            index  index.html index.htm index.php;
        }}
    }}
}}
"#, harvest_home.replace('\\', "/"), harvest_home.replace('\\', "/"), harvest_home.replace('\\', "/"));
                fs::write(&nginx_conf, default_config).ok();
            }

            // Create fastcgi snippet
            let fastcgi_snippet = etc_nginx.join("snippets").join("fastcgi-php.conf");
            if !fastcgi_snippet.exists() {
                let fastcgi_config = r#"fastcgi_split_path_info ^(.+\.php)(/.+)$;
fastcgi_index index.php;
fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
include fastcgi_params;
"#;
                fs::write(&fastcgi_snippet, fastcgi_config).ok();
            }

            // Copy mime.types if not exists
            let mime_types_src = install_dir.join("conf").join("mime.types");
            let mime_types_dst = etc_nginx.join("mime.types");
            if mime_types_src.exists() && !mime_types_dst.exists() {
                fs::copy(&mime_types_src, &mime_types_dst).ok();
            }

            // Copy fastcgi_params if not exists
            let fastcgi_params_src = install_dir.join("conf").join("fastcgi_params");
            let fastcgi_params_dst = etc_nginx.join("fastcgi_params");
            if fastcgi_params_src.exists() && !fastcgi_params_dst.exists() {
                fs::copy(&fastcgi_params_src, &fastcgi_params_dst).ok();
            }
        },
        "mysql" => {
            // Create data directory if needed
            let data_dir = install_dir.join("data");
            if !data_dir.exists() {
                fs::create_dir_all(&data_dir).ok();

                // Initialize MySQL data directory
                let mysqld = install_dir.join("bin").join("mysqld.exe");
                if mysqld.exists() {
                    let _ = Command::new(&mysqld)
                        .args(["--initialize-insecure", &format!("--datadir={}", data_dir.to_string_lossy())])
                        .output();
                }
            }
        },
        "redis" => {
            // Redis doesn't need much setup, but create logs dir
            let logs_dir = PathBuf::from(harvest_home).join("logs");
            fs::create_dir_all(&logs_dir).ok();
        },
        _ => {}
    }

    Ok(())
}

#[tauri::command]
pub async fn uninstall_service(service_id: String) -> Result<(), String> {
    let harvest_home = get_harvest_home();
    let install_dir = PathBuf::from(&harvest_home).join("bin").join(&service_id);

    if !install_dir.exists() {
        return Err(format!("Service '{}' is not installed", service_id));
    }

    // Stop the service if running
    let _ = stop_service(service_id.clone()).await;

    // Wait a moment for process to stop
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Remove the directory
    fs::remove_dir_all(&install_dir)
        .map_err(|e| format!("Failed to remove service directory: {}", e))?;

    Ok(())
}
