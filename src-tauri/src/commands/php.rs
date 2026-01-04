use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use crate::types::{AvailableVersion, InstalledVersion, PhpExtension};
use crate::commands::config::get_harvest_home;

#[tauri::command]
pub async fn get_available_php_versions() -> Result<Vec<AvailableVersion>, String> {
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
pub fn get_installed_php_versions() -> Vec<InstalledVersion> {
    let mut versions = Vec::new();
    let harvest_home = get_harvest_home();

    // Check PHP directory in harvest home
    let php_dir = PathBuf::from(&harvest_home).join("bin").join("php");

    if php_dir.exists() && php_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&php_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");

                    // Check if it's a PHP directory (contains php.exe or php)
                    #[cfg(target_os = "windows")]
                    let php_exe = path.join("php.exe");
                    #[cfg(not(target_os = "windows"))]
                    let php_exe = path.join("php");

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

    // Sort by version descending
    versions.sort_by(|a, b| b.version.cmp(&a.version));
    versions
}

fn extract_php_version(path: &PathBuf, folder_name: &str) -> Option<String> {
    // First try to get version from php.exe -v
    let php_exe = path.join("php.exe");
    if let Ok(output) = Command::new(&php_exe).arg("-v").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            if let Some(version) = parse_php_version_output(&stdout) {
                return Some(version);
            }
        }
    }

    // Fallback: extract from folder name
    parse_php_folder_name(folder_name)
}

/// Parse PHP version from `php -v` output
/// Format: "PHP 8.4.1 (cli) ..."
pub fn parse_php_version_output(output: &str) -> Option<String> {
    output.lines().next().and_then(|line| {
        line.strip_prefix("PHP ")
            .and_then(|rest| rest.split_whitespace().next())
            .map(|v| v.to_string())
    })
}

/// Parse version from PHP folder name like "php-8.4.1-nts-Win32-vs16-x64"
pub fn parse_php_folder_name(folder_name: &str) -> Option<String> {
    let parts: Vec<&str> = folder_name.split('-').collect();
    if parts.len() >= 2 && parts[0] == "php" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

#[tauri::command]
pub async fn install_php_version(app: AppHandle, version: String) -> Result<(), String> {
    let harvest_home = get_harvest_home();
    let install_dir = PathBuf::from(&harvest_home).join("bin").join("php");
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

    let _full_version = info.get("version")
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
    let zip_path_file = temp_dir.join(format!("php-{}.zip", version));

    // Stream download to file
    let mut file = File::create(&zip_path_file)
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
    let zip_file = File::open(&zip_path_file)
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
    fs::remove_file(&zip_path_file).ok();

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
pub async fn uninstall_php_version(version: String) -> Result<(), String> {
    let harvest_home = get_harvest_home();
    let php_dir = PathBuf::from(&harvest_home).join("bin").join("php");

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

#[tauri::command]
pub fn get_php_extensions(php_path: String) -> Result<Vec<PhpExtension>, String> {
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
pub fn toggle_php_extension(php_path: String, extension: String, enable: bool) -> Result<(), String> {
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
