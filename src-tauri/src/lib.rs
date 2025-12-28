use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledVersion {
    pub version: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableVersion {
    pub version: String,
    pub full_version: String,
    pub lts: bool,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub harvest_paths: Vec<String>,
    pub editor_command: String,
    pub terminal_command: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            harvest_paths: vec!["C:/laragon/www".to_string()],
            editor_command: "code".to_string(),
            terminal_command: "wt -d".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackItem {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub id: String,
    pub name: String,
    pub path: String,
    pub domain: String,
    pub php_version: String,
    pub stack: Vec<StackItem>,
    pub ssl: bool,
}

fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("harvest");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("config.json")
}

#[tauri::command]
fn get_config() -> AppConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    AppConfig::default()
}

#[tauri::command]
fn save_config(config: AppConfig) -> Result<(), String> {
    let config_path = get_config_path();
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn get_version_from_value(val: Option<&serde_json::Value>) -> Option<String> {
    val.and_then(|v| v.as_str())
        .map(|s| s.trim_start_matches('^').trim_start_matches('~').to_string())
}

fn detect_stack(path: &PathBuf) -> Vec<StackItem> {
    let mut stack = Vec::new();

    // Check composer.json for PHP framework versions
    let composer_file = path.join("composer.json");
    if composer_file.exists() {
        if let Ok(content) = fs::read_to_string(&composer_file) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(require) = json.get("require") {
                    // Check for OctoberCMS
                    let october_indicators = [
                        path.join("modules").join("system"),
                        path.join("modules").join("backend"),
                    ];
                    let has_october = october_indicators.iter().filter(|p| p.exists()).count() >= 2;

                    if has_october {
                        let version = get_version_from_value(require.get("october/rain"));
                        stack.push(StackItem { name: "octobercms".to_string(), version });
                    } else if require.get("laravel/framework").is_some() {
                        let version = get_version_from_value(require.get("laravel/framework"));
                        stack.push(StackItem { name: "laravel".to_string(), version });
                    } else {
                        stack.push(StackItem { name: "php".to_string(), version: None });
                    }
                }
            }
        }
    } else if path.join("index.php").exists() {
        stack.push(StackItem { name: "php".to_string(), version: None });
    }

    // Check package.json for frontend stack
    let package_json = path.join("package.json");
    if package_json.exists() {
        if let Ok(content) = fs::read_to_string(&package_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let deps = json.get("dependencies").cloned().unwrap_or_default();
                let dev_deps = json.get("devDependencies").cloned().unwrap_or_default();

                // Check Vue
                if let Some(ver) = deps.get("vue").or(dev_deps.get("vue")) {
                    stack.push(StackItem {
                        name: "vue".to_string(),
                        version: get_version_from_value(Some(ver)),
                    });
                }

                // Check React
                if let Some(ver) = deps.get("react").or(dev_deps.get("react")) {
                    stack.push(StackItem {
                        name: "react".to_string(),
                        version: get_version_from_value(Some(ver)),
                    });
                }

                // Check Vite
                if let Some(ver) = dev_deps.get("vite") {
                    stack.push(StackItem {
                        name: "vite".to_string(),
                        version: get_version_from_value(Some(ver)),
                    });
                }

                // Check Tailwind
                if let Some(ver) = dev_deps.get("tailwindcss") {
                    stack.push(StackItem {
                        name: "tailwind".to_string(),
                        version: get_version_from_value(Some(ver)),
                    });
                }

                // Check TypeScript
                if let Some(ver) = dev_deps.get("typescript") {
                    stack.push(StackItem {
                        name: "typescript".to_string(),
                        version: get_version_from_value(Some(ver)),
                    });
                }
            }
        }
    }

    // Check for package manager
    if path.join("bun.lockb").exists() {
        stack.push(StackItem { name: "bun".to_string(), version: None });
    } else if path.join("pnpm-lock.yaml").exists() {
        stack.push(StackItem { name: "pnpm".to_string(), version: None });
    } else if path.join("yarn.lock").exists() {
        stack.push(StackItem { name: "yarn".to_string(), version: None });
    }

    stack
}

fn detect_php_version(path: &PathBuf) -> String {
    // Try to read .php-version file
    let php_version_file = path.join(".php-version");
    if php_version_file.exists() {
        if let Ok(version) = fs::read_to_string(&php_version_file) {
            return version.trim().to_string();
        }
    }

    // Try composer.json
    let composer_file = path.join("composer.json");
    if composer_file.exists() {
        if let Ok(content) = fs::read_to_string(&composer_file) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(require) = json.get("require") {
                    if let Some(php) = require.get("php") {
                        if let Some(constraint) = php.as_str() {
                            // Extract version from constraint like "^8.1" or ">=8.2"
                            let version: String = constraint
                                .chars()
                                .filter(|c| c.is_ascii_digit() || *c == '.')
                                .collect();
                            if !version.is_empty() {
                                return version;
                            }
                        }
                    }
                }
            }
        }
    }

    "8.4".to_string()
}

#[tauri::command]
fn scan_sites(directories: Vec<String>) -> Result<Vec<Site>, String> {
    let mut sites = Vec::new();

    for directory in directories {
        let path = PathBuf::from(&directory);

        if !path.exists() || !path.is_dir() {
            continue; // Skip invalid paths
        }

        let entries = match fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();

            if !entry_path.is_dir() {
                continue;
            }

            let name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Skip hidden directories and common non-site directories
            if name.starts_with('.') || name == "node_modules" || name == "vendor" {
                continue;
            }

            // Check if it looks like a web project
            let is_web_project = entry_path.join("index.php").exists()
                || entry_path.join("public").join("index.php").exists()
                || entry_path.join("artisan").exists()
                || entry_path.join("composer.json").exists()
                || entry_path.join("package.json").exists()
                || entry_path.join("index.html").exists();

            if !is_web_project {
                continue;
            }

            let stack = detect_stack(&entry_path);
            let php_version = detect_php_version(&entry_path);
            let domain = format!("{}.test", name);

            sites.push(Site {
                id: format!("{}-{}", name, entry_path.to_string_lossy().replace(['/', '\\', ':'], "-")),
                name: name.clone(),
                path: entry_path.to_string_lossy().to_string(),
                domain,
                php_version,
                stack,
                ssl: true,
            });
        }
    }

    // Sort by name
    sites.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(sites)
}

#[tauri::command]
async fn open_in_editor(path: String, editor: String) -> Result<(), String> {
    std::process::Command::new(&editor)
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open editor: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn open_in_terminal(path: String, terminal: String) -> Result<(), String> {
    // Parse terminal command (e.g., "wt -d" becomes ["wt", "-d"])
    let parts: Vec<&str> = terminal.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Invalid terminal command".to_string());
    }

    let mut cmd = std::process::Command::new(parts[0]);
    for part in &parts[1..] {
        cmd.arg(part);
    }
    cmd.arg(&path);

    cmd.spawn()
        .map_err(|e| format!("Failed to open terminal: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn open_in_browser(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open browser: {}", e))?;
    Ok(())
}

// PHP Version Management

#[derive(Debug, Deserialize)]
struct NodeDistVersion {
    version: String,
    lts: serde_json::Value, // Can be false or string like "Jod"
}

#[tauri::command]
async fn get_available_php_versions() -> Result<Vec<AvailableVersion>, String> {
    let client = reqwest::Client::new();

    // Fetch active/supported versions from php.net
    let active_response = client
        .get("https://www.php.net/releases/active")
        .header("User-Agent", "Harvest/0.1")
        .header("Accept", "application/json")
        .send()
        .await;

    let mut active_versions: std::collections::HashSet<String> = std::collections::HashSet::new();

    if let Ok(resp) = active_response {
        if let Ok(text) = resp.text().await {
            if let Ok(active_data) = serde_json::from_str::<serde_json::Value>(&text) {
                // Structure is {"8": {"8.1": {...}, "8.2": {...}}}
                if let Some(obj) = active_data.as_object() {
                    for (_major, minor_versions) in obj {
                        if let Some(minors) = minor_versions.as_object() {
                            for (version_key, _) in minors {
                                active_versions.insert(version_key.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Fetch Windows builds
    let response = client
        .get("https://windows.php.net/downloads/releases/releases.json")
        .header("User-Agent", "Harvest/0.1")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch PHP versions: {}", e))?;

    let text = response.text().await
        .map_err(|e| format!("Failed to read PHP versions response: {}", e))?;

    let json_text = text.trim_start_matches('\u{feff}');

    let releases: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(json_text)
        .map_err(|e| format!("Failed to parse PHP versions JSON: {}", e))?;

    let mut versions: Vec<AvailableVersion> = Vec::new();

    for (major_minor, info) in releases {
        let full_version = match info.get("version").and_then(|v| v.as_str()) {
            Some(v) => v.to_string(),
            None => continue,
        };

        let build_keys = ["nts-vs17-x64", "nts-vs16-x64", "nts-vc15-x64"];
        let mut zip_path: Option<String> = None;

        for key in build_keys {
            if let Some(build) = info.get(key) {
                if let Some(zip) = build.get("zip") {
                    if let Some(path) = zip.get("path").and_then(|p| p.as_str()) {
                        zip_path = Some(path.to_string());
                        break;
                    }
                }
            }
        }

        let path = match zip_path {
            Some(p) => p,
            None => continue,
        };

        let download_url = format!("https://windows.php.net/downloads/releases/{}", path);
        let is_supported = active_versions.contains(&major_minor);

        versions.push(AvailableVersion {
            version: major_minor,
            full_version,
            lts: is_supported,
            download_url,
        });
    }

    // Sort by version descending
    versions.sort_by(|a, b| {
        let a_parts: Vec<u32> = a.version.split('.').filter_map(|s| s.parse().ok()).collect();
        let b_parts: Vec<u32> = b.version.split('.').filter_map(|s| s.parse().ok()).collect();
        b_parts.cmp(&a_parts)
    });

    Ok(versions)
}

#[tauri::command]
async fn get_available_node_versions() -> Result<Vec<AvailableVersion>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://nodejs.org/dist/index.json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Node versions: {}", e))?;

    let dist_versions: Vec<NodeDistVersion> = response.json().await
        .map_err(|e| format!("Failed to parse Node versions: {}", e))?;

    let mut versions: Vec<AvailableVersion> = Vec::new();
    let mut seen_major: std::collections::HashSet<String> = std::collections::HashSet::new();

    for dv in dist_versions {
        let full_version = dv.version.trim_start_matches('v').to_string();
        let parts: Vec<&str> = full_version.split('.').collect();

        if parts.is_empty() {
            continue;
        }

        let major = parts[0].to_string();

        // Only include major versions 16 and above, one per major version
        if let Ok(major_num) = major.parse::<u32>() {
            if major_num >= 16 && !seen_major.contains(&major) {
                seen_major.insert(major.clone());

                let is_lts = match &dv.lts {
                    serde_json::Value::Bool(false) => false,
                    _ => true,
                };

                let download_url = format!(
                    "https://nodejs.org/dist/v{}/node-v{}-win-x64.zip",
                    full_version, full_version
                );

                versions.push(AvailableVersion {
                    version: major,
                    full_version,
                    lts: is_lts,
                    download_url,
                });
            }
        }
    }

    // Sort by version descending
    versions.sort_by(|a, b| {
        let a_num: u32 = a.version.parse().unwrap_or(0);
        let b_num: u32 = b.version.parse().unwrap_or(0);
        b_num.cmp(&a_num)
    });

    Ok(versions)
}

#[tauri::command]
fn get_installed_php_versions() -> Vec<InstalledVersion> {
    let mut versions = Vec::new();

    // Check common Laragon PHP paths
    let php_dirs = vec![
        PathBuf::from("C:/laragon/bin/php"),
        PathBuf::from("C:/php"),
    ];

    for php_dir in php_dirs {
        if php_dir.exists() && php_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&php_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");

                        // Check if it's a PHP directory (contains php.exe)
                        let php_exe = path.join("php.exe");
                        if php_exe.exists() {
                            // Try to extract version from folder name or run php -v
                            let version = extract_php_version(&path, name);
                            if let Some(ver) = version {
                                versions.push(InstalledVersion {
                                    version: ver,
                                    path: path.to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by version descending
    versions.sort_by(|a, b| b.version.cmp(&a.version));
    versions
}

fn extract_php_version(path: &PathBuf, folder_name: &str) -> Option<String> {
    // First try to get version from php.exe -v
    let php_exe = path.join("php.exe");
    if let Ok(output) = Command::new(&php_exe).arg("-v").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            // Parse "PHP 8.4.1 (cli) ..." format
            if let Some(line) = stdout.lines().next() {
                if let Some(ver) = line.strip_prefix("PHP ") {
                    if let Some(ver) = ver.split_whitespace().next() {
                        return Some(ver.to_string());
                    }
                }
            }
        }
    }

    // Fallback: extract from folder name like "php-8.4.1-nts-Win32-vs16-x64"
    let parts: Vec<&str> = folder_name.split('-').collect();
    if parts.len() >= 2 && parts[0] == "php" {
        return Some(parts[1].to_string());
    }

    None
}

fn get_php_download_url(version: &str) -> Option<String> {
    // PHP download URLs from windows.php.net
    // Format: https://windows.php.net/downloads/releases/php-8.4.1-nts-Win32-vs17-x64.zip
    // For older versions, they're in archives
    let (base_url, vs_version) = match version {
        v if v.starts_with("8.4") || v.starts_with("8.5") => {
            ("https://windows.php.net/downloads/releases", "vs17")
        }
        v if v.starts_with("8.3") || v.starts_with("8.2") || v.starts_with("8.1") => {
            ("https://windows.php.net/downloads/releases", "vs16")
        }
        v if v.starts_with("8.0") => {
            ("https://windows.php.net/downloads/releases/archives", "vs16")
        }
        v if v.starts_with("7.4") => {
            ("https://windows.php.net/downloads/releases/archives", "vc15")
        }
        _ => return None,
    };

    // Map short version to full version for download
    let full_version = match version {
        "8.5" => "8.5.0",
        "8.4" => "8.4.2",
        "8.3" => "8.3.15",
        "8.2" => "8.2.27",
        "8.1" => "8.1.31",
        "8.0" => "8.0.30",
        "7.4" => "7.4.33",
        v => v,
    };

    Some(format!(
        "{}/php-{}-nts-Win32-{}-x64.zip",
        base_url, full_version, vs_version
    ))
}

#[tauri::command]
async fn install_php_version(app: AppHandle, version: String) -> Result<(), String> {
    let install_dir = PathBuf::from("C:/laragon/bin/php");
    fs::create_dir_all(&install_dir).map_err(|e| format!("Failed to create directory: {}", e))?;

    // Emit progress event
    let _ = app.emit("install-progress", serde_json::json!({
        "type": "php",
        "version": version,
        "status": "fetching",
        "progress": 0
    }));

    // Fetch releases.json to get the correct download URL
    let client = reqwest::Client::new();
    let releases_response = client
        .get("https://windows.php.net/downloads/releases/releases.json")
        .header("User-Agent", "Harvest/0.1")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    let releases_text = releases_response.text().await
        .map_err(|e| format!("Failed to read releases: {}", e))?;

    let releases: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(&releases_text)
        .map_err(|e| format!("Failed to parse releases: {}", e))?;

    let info = releases.get(&version)
        .ok_or_else(|| format!("PHP version {} not found in releases", version))?;

    let full_version = info.get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing version in release info".to_string())?
        .to_string();

    // Find the nts-x64 download URL
    let build_keys = ["nts-vs17-x64", "nts-vs16-x64", "nts-vc15-x64"];
    let mut zip_path: Option<String> = None;

    for key in build_keys {
        if let Some(build) = info.get(key) {
            if let Some(zip) = build.get("zip") {
                if let Some(path) = zip.get("path").and_then(|p| p.as_str()) {
                    zip_path = Some(path.to_string());
                    break;
                }
            }
        }
    }

    let path = zip_path.ok_or_else(|| format!("No nts-x64 build found for PHP {}", version))?;
    let download_url = format!("https://windows.php.net/downloads/releases/{}", path);

    let _ = app.emit("install-progress", serde_json::json!({
        "type": "php",
        "version": version,
        "status": "downloading",
        "progress": 0
    }));

    // Download the file
    let response = client
        .get(&download_url)
        .header("User-Agent", "Harvest/0.1")
        .send()
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let temp_dir = std::env::temp_dir();
    let zip_path = temp_dir.join(format!("php-{}.zip", version));

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
            "type": "php",
            "version": version,
            "status": "downloading",
            "progress": progress
        }));
    }

    let _ = app.emit("install-progress", serde_json::json!({
        "type": "php",
        "version": version,
        "status": "extracting",
        "progress": 100
    }));

    // Extract the zip file
    let zip_file = File::open(&zip_path)
        .map_err(|e| format!("Failed to open zip: {}", e))?;

    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| format!("Failed to read zip: {}", e))?;

    // Create target directory using the zip filename (without .zip extension)
    let zip_filename = path.trim_end_matches(".zip");
    let target_dir = install_dir.join(zip_filename);
    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    // Extract files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;

        let outpath = target_dir.join(file.name());

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

    // Copy php.ini-development to php.ini if it doesn't exist
    let ini_dev = target_dir.join("php.ini-development");
    let ini_target = target_dir.join("php.ini");
    if ini_dev.exists() && !ini_target.exists() {
        fs::copy(&ini_dev, &ini_target).ok();
    }

    let _ = app.emit("install-progress", serde_json::json!({
        "type": "php",
        "version": version,
        "status": "complete",
        "progress": 100
    }));

    Ok(())
}

#[tauri::command]
async fn uninstall_php_version(version: String) -> Result<(), String> {
    let php_dir = PathBuf::from("C:/laragon/bin/php");

    // Find the directory matching this version
    if let Ok(entries) = fs::read_dir(&php_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Check if folder name contains the version
                if name.contains(&version) || name.starts_with(&format!("php-{}", version)) {
                    fs::remove_dir_all(&path)
                        .map_err(|e| format!("Failed to remove directory: {}", e))?;
                    return Ok(());
                }
            }
        }
    }

    Err(format!("PHP {} not found in {}", version, php_dir.display()))
}

// PHP Extensions Management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpExtension {
    pub name: String,
    pub enabled: bool,
    pub builtin: bool,
}

#[tauri::command]
fn get_php_extensions(php_path: String) -> Result<Vec<PhpExtension>, String> {
    let php_dir = PathBuf::from(&php_path);
    let ext_dir = php_dir.join("ext");
    let ini_path = php_dir.join("php.ini");

    if !ext_dir.exists() {
        return Err("Extensions directory not found".to_string());
    }

    // Read php.ini to find enabled extensions
    let ini_content = fs::read_to_string(&ini_path).unwrap_or_default();
    let enabled_extensions: std::collections::HashSet<String> = ini_content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with(';') && (trimmed.starts_with("extension=") || trimmed.starts_with("zend_extension="))
        })
        .filter_map(|line| {
            let trimmed = line.trim();
            let ext_part = if trimmed.starts_with("extension=") {
                trimmed.strip_prefix("extension=")
            } else {
                trimmed.strip_prefix("zend_extension=")
            };
            ext_part.map(|e| {
                e.trim()
                    .trim_matches('"')
                    .trim_end_matches(".dll")
                    .trim_start_matches("php_")
                    .to_string()
            })
        })
        .collect();

    // Scan ext directory for available extensions
    let mut extensions: Vec<PhpExtension> = Vec::new();

    if let Ok(entries) = fs::read_dir(&ext_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("php_") && filename.ends_with(".dll") {
                    let ext_name = filename
                        .trim_start_matches("php_")
                        .trim_end_matches(".dll")
                        .to_string();

                    let is_enabled = enabled_extensions.contains(&ext_name);
                    let is_builtin = ["date", "pcre", "reflection", "spl", "standard"].contains(&ext_name.as_str());

                    extensions.push(PhpExtension {
                        name: ext_name,
                        enabled: is_enabled,
                        builtin: is_builtin,
                    });
                }
            }
        }
    }

    // Sort alphabetically
    extensions.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(extensions)
}

#[tauri::command]
fn toggle_php_extension(php_path: String, extension: String, enable: bool) -> Result<(), String> {
    let php_dir = PathBuf::from(&php_path);
    let ini_path = php_dir.join("php.ini");

    if !ini_path.exists() {
        // Try to create from php.ini-development
        let dev_ini = php_dir.join("php.ini-development");
        if dev_ini.exists() {
            fs::copy(&dev_ini, &ini_path)
                .map_err(|e| format!("Failed to create php.ini: {}", e))?;
        } else {
            return Err("php.ini not found".to_string());
        }
    }

    let content = fs::read_to_string(&ini_path)
        .map_err(|e| format!("Failed to read php.ini: {}", e))?;

    let ext_line = format!("extension={}", extension);
    let ext_line_dll = format!("extension=php_{}.dll", extension);
    let commented_line = format!(";extension={}", extension);
    let commented_line_dll = format!(";extension=php_{}.dll", extension);

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut found = false;

    for line in &mut lines {
        let trimmed = line.trim();

        // Check if this line is about our extension
        if trimmed == ext_line || trimmed == ext_line_dll ||
           trimmed == commented_line || trimmed == commented_line_dll ||
           trimmed == format!(";extension=php_{}.dll", extension) {
            found = true;
            if enable {
                *line = format!("extension={}", extension);
            } else {
                *line = format!(";extension={}", extension);
            }
            break;
        }
    }

    // If not found and we want to enable, add it
    if !found && enable {
        // Find the extensions section or add at end
        let mut insert_idx = lines.len();
        for (i, line) in lines.iter().enumerate() {
            if line.trim().starts_with("[") && line.contains("extension") {
                insert_idx = i + 1;
                break;
            }
        }
        lines.insert(insert_idx, format!("extension={}", extension));
    }

    let new_content = lines.join("\n");
    fs::write(&ini_path, new_content)
        .map_err(|e| format!("Failed to write php.ini: {}", e))?;

    Ok(())
}

// Node Version Management
#[tauri::command]
fn get_installed_node_versions() -> Vec<InstalledVersion> {
    let mut versions = Vec::new();

    // Check common Node paths
    let node_dirs = vec![
        PathBuf::from("C:/laragon/bin/nodejs"),
        PathBuf::from("C:/Program Files/nodejs"),
    ];

    for node_dir in node_dirs {
        if node_dir.exists() && node_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&node_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");

                        // Check if it's a Node directory (contains node.exe)
                        let node_exe = path.join("node.exe");
                        if node_exe.exists() {
                            let version = extract_node_version(&path, name);
                            if let Some(ver) = version {
                                versions.push(InstalledVersion {
                                    version: ver,
                                    path: path.to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Also check if node is directly in a path (not versioned subdirectories)
    for node_dir in &[PathBuf::from("C:/Program Files/nodejs")] {
        let node_exe = node_dir.join("node.exe");
        if node_exe.exists() {
            if let Ok(output) = Command::new(&node_exe).arg("-v").output() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    let ver = stdout.trim().trim_start_matches('v');
                    if !ver.is_empty() && !versions.iter().any(|v| v.version == ver) {
                        versions.push(InstalledVersion {
                            version: ver.to_string(),
                            path: node_dir.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    // Sort by version descending
    versions.sort_by(|a, b| b.version.cmp(&a.version));
    versions
}

fn extract_node_version(path: &PathBuf, folder_name: &str) -> Option<String> {
    // First try to get version from node.exe -v
    let node_exe = path.join("node.exe");
    if let Ok(output) = Command::new(&node_exe).arg("-v").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            let ver = stdout.trim().trim_start_matches('v');
            if !ver.is_empty() {
                return Some(ver.to_string());
            }
        }
    }

    // Fallback: extract from folder name like "node-v20.10.0-win-x64"
    let parts: Vec<&str> = folder_name.split('-').collect();
    if parts.len() >= 2 && parts[0] == "node" {
        return Some(parts[1].trim_start_matches('v').to_string());
    }

    None
}

fn get_node_download_url(version: &str) -> Option<String> {
    // Node.js download URLs from nodejs.org
    // Format: https://nodejs.org/dist/v20.10.0/node-v20.10.0-win-x64.zip
    let full_version = match version {
        "23" => "23.5.0",
        "22" => "22.12.0",
        "21" => "21.7.3",
        "20" => "20.18.1",
        "18" => "18.20.5",
        "16" => "16.20.2",
        v => v,
    };

    Some(format!(
        "https://nodejs.org/dist/v{}/node-v{}-win-x64.zip",
        full_version, full_version
    ))
}

#[tauri::command]
async fn install_node_version(app: AppHandle, version: String) -> Result<(), String> {
    let download_url = get_node_download_url(&version)
        .ok_or_else(|| format!("Unknown Node.js version: {}", version))?;

    let install_dir = PathBuf::from("C:/laragon/bin/nodejs");
    fs::create_dir_all(&install_dir).map_err(|e| format!("Failed to create directory: {}", e))?;

    // Emit progress event
    let _ = app.emit("install-progress", serde_json::json!({
        "type": "node",
        "version": version,
        "status": "downloading",
        "progress": 0
    }));

    // Download the file
    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let temp_dir = std::env::temp_dir();
    let zip_path = temp_dir.join(format!("node-{}.zip", version));

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
            "type": "node",
            "version": version,
            "status": "downloading",
            "progress": progress
        }));
    }

    let _ = app.emit("install-progress", serde_json::json!({
        "type": "node",
        "version": version,
        "status": "extracting",
        "progress": 100
    }));

    // Extract the zip file
    let zip_file = File::open(&zip_path)
        .map_err(|e| format!("Failed to open zip: {}", e))?;

    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| format!("Failed to read zip: {}", e))?;

    // Extract to install directory (Node zip has a root folder like "node-v20.10.0-win-x64")
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;

        // Get the path, keeping the root directory structure
        let outpath = install_dir.join(file.name());

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

    let _ = app.emit("install-progress", serde_json::json!({
        "type": "node",
        "version": version,
        "status": "complete",
        "progress": 100
    }));

    Ok(())
}

#[tauri::command]
async fn uninstall_node_version(version: String) -> Result<(), String> {
    let node_dir = PathBuf::from("C:/laragon/bin/nodejs");

    // Find the directory matching this version
    if let Ok(entries) = fs::read_dir(&node_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Check if folder name contains the version
                if name.contains(&format!("v{}", version)) || name.contains(&format!("-{}", version)) {
                    fs::remove_dir_all(&path)
                        .map_err(|e| format!("Failed to remove directory: {}", e))?;
                    return Ok(());
                }
            }
        }
    }

    Err(format!("Node.js {} not found in {}", version, node_dir.display()))
}

// Package Manager structs and commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManager {
    pub name: String,
    pub command: String,
    pub installed: bool,
    pub version: Option<String>,
    pub description: String,
}

#[tauri::command]
fn get_package_managers() -> Vec<PackageManager> {
    let managers = vec![
        ("npm", "npm", "Node Package Manager - comes with Node.js"),
        ("yarn", "yarn", "Fast, reliable dependency management"),
        ("pnpm", "pnpm", "Fast, disk space efficient package manager"),
        ("bun", "bun", "All-in-one JavaScript runtime & toolkit"),
    ];

    managers.iter().map(|(name, cmd, desc)| {
        let (installed, version) = check_package_manager_version(cmd);
        PackageManager {
            name: name.to_string(),
            command: cmd.to_string(),
            installed,
            version,
            description: desc.to_string(),
        }
    }).collect()
}

fn check_package_manager_version(command: &str) -> (bool, Option<String>) {
    use std::process::Command;

    // On Windows, use cmd /c to run commands (handles .cmd files like npm.cmd)
    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(["/c", command, "-v"])
        .output();

    #[cfg(not(target_os = "windows"))]
    let result = Command::new(command)
        .arg("-v")
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            // Clean up version string (remove 'v' prefix if present)
            let clean_version = version.trim_start_matches('v').to_string();
            (true, Some(clean_version))
        }
        _ => (false, None)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            scan_sites,
            open_in_editor,
            open_in_terminal,
            open_in_browser,
            get_installed_php_versions,
            get_available_php_versions,
            install_php_version,
            uninstall_php_version,
            get_php_extensions,
            toggle_php_extension,
            get_installed_node_versions,
            get_available_node_versions,
            install_node_version,
            uninstall_node_version,
            get_package_managers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
