use std::fs;
use std::path::PathBuf;
use crate::types::{Site, StackItem};

/// Extract version string from a serde_json Value, stripping common prefixes like ^ and ~
pub fn get_version_from_value(val: Option<&serde_json::Value>) -> Option<String> {
    val.and_then(|v| v.as_str())
        .map(|s| s.trim_start_matches('^').trim_start_matches('~').to_string())
}

/// Parse PHP version constraint and extract the numeric version
pub fn parse_php_version_constraint(constraint: &str) -> Option<String> {
    let version: String = constraint
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

/// Generate a site ID from name and path
pub fn generate_site_id(name: &str, path: &str) -> String {
    format!("{}-{}", name, path.replace(['/', '\\', ':'], "-"))
}

/// Generate a domain from site name
pub fn generate_domain(name: &str) -> String {
    format!("{}.test", name)
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
                            if let Some(version) = parse_php_version_constraint(constraint) {
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
pub fn scan_sites(directories: Vec<String>) -> Result<Vec<Site>, String> {
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
pub async fn open_in_editor(path: String, editor: String) -> Result<(), String> {
    std::process::Command::new(&editor)
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open editor: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn open_in_terminal(path: String, terminal: String) -> Result<(), String> {
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
pub async fn open_in_browser(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open browser: {}", e))?;
    Ok(())
}
