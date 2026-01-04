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
