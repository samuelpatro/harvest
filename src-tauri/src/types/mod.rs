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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.harvest_paths, vec!["C:/laragon/www".to_string()]);
        assert_eq!(config.editor_command, "code");
        assert_eq!(config.terminal_command, "wt -d");
    }

    #[test]
    fn test_app_config_serialization() {
        let config = AppConfig {
            harvest_paths: vec!["/test/path".to_string()],
            editor_command: "vim".to_string(),
            terminal_command: "bash".to_string(),
        };

        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: AppConfig = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.harvest_paths, config.harvest_paths);
        assert_eq!(deserialized.editor_command, config.editor_command);
        assert_eq!(deserialized.terminal_command, config.terminal_command);
    }

    #[test]
    fn test_stack_item_with_version() {
        let item = StackItem {
            name: "vue".to_string(),
            version: Some("3.4.0".to_string()),
        };
        assert_eq!(item.name, "vue");
        assert_eq!(item.version, Some("3.4.0".to_string()));
    }

    #[test]
    fn test_stack_item_without_version() {
        let item = StackItem {
            name: "php".to_string(),
            version: None,
        };
        assert_eq!(item.name, "php");
        assert!(item.version.is_none());
    }

    #[test]
    fn test_site_creation() {
        let site = Site {
            id: "test-site-1".to_string(),
            name: "test-site".to_string(),
            path: "/var/www/test".to_string(),
            domain: "test.test".to_string(),
            php_version: "8.3".to_string(),
            stack: vec![
                StackItem { name: "laravel".to_string(), version: Some("11.0".to_string()) },
            ],
            ssl: true,
        };

        assert_eq!(site.name, "test-site");
        assert_eq!(site.domain, "test.test");
        assert!(site.ssl);
        assert_eq!(site.stack.len(), 1);
    }

    #[test]
    fn test_installed_version() {
        let version = InstalledVersion {
            version: "8.3.1".to_string(),
            path: "/usr/bin/php".to_string(),
        };
        assert_eq!(version.version, "8.3.1");
    }

    #[test]
    fn test_available_version() {
        let version = AvailableVersion {
            version: "8.3".to_string(),
            full_version: "8.3.1".to_string(),
            lts: true,
            download_url: "https://example.com/php.zip".to_string(),
        };
        assert!(version.lts);
        assert_eq!(version.version, "8.3");
    }

    #[test]
    fn test_php_extension() {
        let ext = PhpExtension {
            name: "curl".to_string(),
            enabled: true,
            builtin: false,
        };
        assert!(ext.enabled);
        assert!(!ext.builtin);
    }

    #[test]
    fn test_package_manager() {
        let pm = PackageManager {
            name: "bun".to_string(),
            command: "bun".to_string(),
            installed: true,
            version: Some("1.0.0".to_string()),
            description: "Fast runtime".to_string(),
        };
        assert!(pm.installed);
        assert_eq!(pm.version, Some("1.0.0".to_string()));
    }
}
