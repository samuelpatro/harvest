use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use futures_util::StreamExt;
use serde::Deserialize;
use tauri::{AppHandle, Emitter};
use crate::types::{AvailableVersion, InstalledVersion, PackageManager};
use crate::commands::config::get_harvest_home;

#[derive(Debug, Deserialize)]
struct NodeDistVersion {
    version: String,
    lts: serde_json::Value, // Can be false or string like "Jod"
}

#[tauri::command]
pub async fn get_available_node_versions() -> Result<Vec<AvailableVersion>, String> {
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
pub fn get_installed_node_versions() -> Vec<InstalledVersion> {
    let mut versions = Vec::new();
    let harvest_home = get_harvest_home();

    // Check Node.js directory in harvest home
    let node_dir = PathBuf::from(&harvest_home).join("bin").join("nodejs");

    if node_dir.exists() && node_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&node_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");

                    // Check if it's a Node directory (contains node.exe or node)
                    #[cfg(target_os = "windows")]
                    let node_exe = path.join("node.exe");
                    #[cfg(not(target_os = "windows"))]
                    let node_exe = path.join("node");

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

/// Get the download URL for a Node.js version
pub fn get_node_download_url(version: &str) -> Option<String> {
    // Node.js download URLs from nodejs.org
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
pub async fn install_node_version(app: AppHandle, version: String) -> Result<(), String> {
    let download_url = get_node_download_url(&version)
        .ok_or_else(|| format!("Unknown Node.js version: {}", version))?;

    let harvest_home = get_harvest_home();
    let install_dir = PathBuf::from(&harvest_home).join("bin").join("nodejs");
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

    // Extract to install directory
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;

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
pub async fn uninstall_node_version(version: String) -> Result<(), String> {
    let harvest_home = get_harvest_home();
    let node_dir = PathBuf::from(&harvest_home).join("bin").join("nodejs");

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

// Package Manager Commands

#[tauri::command]
pub fn get_package_managers() -> Vec<PackageManager> {
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

/// Clean a version string by removing 'v' prefix
pub fn clean_version_string(version: &str) -> String {
    version.trim().trim_start_matches('v').to_string()
}

/// Parse a Node.js folder name to extract version
pub fn parse_node_folder_name(folder_name: &str) -> Option<String> {
    let parts: Vec<&str> = folder_name.split('-').collect();
    if parts.len() >= 2 && parts[0] == "node" {
        Some(parts[1].trim_start_matches('v').to_string())
    } else {
        None
    }
}
