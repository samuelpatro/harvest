use serde::{Deserialize, Serialize};

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
    pub harvest_home: String,
    pub harvest_paths: Vec<String>,
    pub editor_command: String,
    pub terminal_command: String,
    #[serde(default)]
    pub first_run_complete: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            harvest_home: get_default_harvest_home(),
            harvest_paths: vec![],
            editor_command: "code".to_string(),
            terminal_command: "wt -d".to_string(),
            first_run_complete: false,
        }
    }
}

/// Get the default harvest home path based on platform
pub fn get_default_harvest_home() -> String {
    #[cfg(target_os = "windows")]
    {
        "C:/harvest".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        "/usr/local/harvest".to_string()
    }
    #[cfg(target_os = "linux")]
    {
        "/opt/harvest".to_string()
    }
}

/// Information about an installable service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallableService {
    pub id: String,
    pub name: String,
    pub description: String,
    pub installed: bool,
    pub version: Option<String>,
    pub available_versions: Vec<ServiceVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceVersion {
    pub version: String,
    pub download_url: String,
    pub filename: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpExtension {
    pub name: String,
    pub enabled: bool,
    pub builtin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManager {
    pub name: String,
    pub command: String,
    pub installed: bool,
    pub version: Option<String>,
    pub description: String,
}
