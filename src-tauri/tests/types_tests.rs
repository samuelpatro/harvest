use tauri_app_lib::types::*;

// ============ AppConfig Tests ============

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();
    // Default harvest_paths is empty, not a hardcoded path
    assert!(config.harvest_paths.is_empty());
    assert_eq!(config.editor_command, "code");
    assert_eq!(config.terminal_command, "wt -d");
    // Check harvest_home is set to platform default
    assert!(!config.harvest_home.is_empty());
    // first_run_complete defaults to false
    assert!(!config.first_run_complete);
}

#[test]
fn test_app_config_serialization() {
    let config = AppConfig {
        harvest_home: "/opt/harvest".to_string(),
        harvest_paths: vec!["/test/path".to_string()],
        editor_command: "vim".to_string(),
        terminal_command: "bash".to_string(),
        first_run_complete: true,
    };

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: AppConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.harvest_home, config.harvest_home);
    assert_eq!(deserialized.harvest_paths, config.harvest_paths);
    assert_eq!(deserialized.editor_command, config.editor_command);
    assert_eq!(deserialized.terminal_command, config.terminal_command);
    assert_eq!(deserialized.first_run_complete, config.first_run_complete);
}

#[test]
fn test_app_config_first_run_default() {
    // When deserializing JSON without first_run_complete, it should default to false
    let json = r#"{"harvest_home":"/opt/harvest","harvest_paths":[],"editor_command":"code","terminal_command":"wt -d"}"#;
    let config: AppConfig = serde_json::from_str(json).expect("Should deserialize");
    assert!(!config.first_run_complete);
}

// ============ get_default_harvest_home Tests ============

#[test]
fn test_get_default_harvest_home() {
    let home = get_default_harvest_home();
    assert!(!home.is_empty());

    #[cfg(target_os = "windows")]
    assert_eq!(home, "C:/harvest");

    #[cfg(target_os = "macos")]
    assert_eq!(home, "/usr/local/harvest");

    #[cfg(target_os = "linux")]
    assert_eq!(home, "/opt/harvest");
}

// ============ StackItem Tests ============

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
fn test_stack_item_serialization() {
    let item = StackItem {
        name: "react".to_string(),
        version: Some("18.2.0".to_string()),
    };

    let json = serde_json::to_string(&item).expect("Should serialize");
    let deserialized: StackItem = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.name, item.name);
    assert_eq!(deserialized.version, item.version);
}

// ============ Site Tests ============

#[test]
fn test_site_creation() {
    let site = Site {
        id: "test-site-1".to_string(),
        name: "test-site".to_string(),
        path: "/var/www/test".to_string(),
        domain: "test.test".to_string(),
        php_version: "8.3".to_string(),
        stack: vec![StackItem {
            name: "laravel".to_string(),
            version: Some("11.0".to_string()),
        }],
        ssl: true,
    };

    assert_eq!(site.name, "test-site");
    assert_eq!(site.domain, "test.test");
    assert!(site.ssl);
    assert_eq!(site.stack.len(), 1);
}

#[test]
fn test_site_with_multiple_stack_items() {
    let site = Site {
        id: "fullstack-1".to_string(),
        name: "fullstack".to_string(),
        path: "/var/www/fullstack".to_string(),
        domain: "fullstack.test".to_string(),
        php_version: "8.4".to_string(),
        stack: vec![
            StackItem { name: "laravel".to_string(), version: Some("11.0".to_string()) },
            StackItem { name: "vue".to_string(), version: Some("3.4.0".to_string()) },
            StackItem { name: "tailwind".to_string(), version: Some("3.4.0".to_string()) },
            StackItem { name: "vite".to_string(), version: None },
        ],
        ssl: false,
    };

    assert_eq!(site.stack.len(), 4);
    assert!(!site.ssl);
}

// ============ InstalledVersion Tests ============

#[test]
fn test_installed_version() {
    let version = InstalledVersion {
        version: "8.3.1".to_string(),
        path: "/usr/bin/php".to_string(),
    };
    assert_eq!(version.version, "8.3.1");
    assert_eq!(version.path, "/usr/bin/php");
}

#[test]
fn test_installed_version_windows_path() {
    let version = InstalledVersion {
        version: "8.4.1".to_string(),
        path: "C:\\harvest\\bin\\php\\php-8.4.1".to_string(),
    };
    assert!(version.path.contains("harvest"));
}

// ============ AvailableVersion Tests ============

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
fn test_available_version_non_lts() {
    let version = AvailableVersion {
        version: "8.4".to_string(),
        full_version: "8.4.0".to_string(),
        lts: false,
        download_url: "https://windows.php.net/downloads/releases/php-8.4.0-nts-Win32-vs17-x64.zip".to_string(),
    };
    assert!(!version.lts);
    assert!(version.download_url.contains("windows.php.net"));
}

// ============ PhpExtension Tests ============

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
fn test_php_extension_builtin() {
    let ext = PhpExtension {
        name: "standard".to_string(),
        enabled: true,
        builtin: true,
    };
    assert!(ext.builtin);
    assert!(ext.enabled);
}

#[test]
fn test_php_extension_disabled() {
    let ext = PhpExtension {
        name: "xdebug".to_string(),
        enabled: false,
        builtin: false,
    };
    assert!(!ext.enabled);
}

// ============ PackageManager Tests ============

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

#[test]
fn test_package_manager_not_installed() {
    let pm = PackageManager {
        name: "pnpm".to_string(),
        command: "pnpm".to_string(),
        installed: false,
        version: None,
        description: "Fast, disk space efficient package manager".to_string(),
    };
    assert!(!pm.installed);
    assert!(pm.version.is_none());
}

// ============ InstallableService Tests ============

#[test]
fn test_installable_service_basic() {
    let service = InstallableService {
        id: "nginx".to_string(),
        name: "nginx".to_string(),
        description: "High-performance HTTP server".to_string(),
        installed: true,
        version: Some("1.24.0".to_string()),
        available_versions: vec![],
    };
    assert_eq!(service.id, "nginx");
    assert!(service.installed);
    assert_eq!(service.version, Some("1.24.0".to_string()));
}

#[test]
fn test_installable_service_not_installed() {
    let service = InstallableService {
        id: "redis".to_string(),
        name: "Redis".to_string(),
        description: "In-memory data structure store".to_string(),
        installed: false,
        version: None,
        available_versions: vec![
            ServiceVersion {
                version: "7.2.4".to_string(),
                download_url: "https://github.com/redis/redis/releases/download/7.2.4/redis.zip".to_string(),
                filename: "redis-7.2.4.zip".to_string(),
            },
        ],
    };
    assert!(!service.installed);
    assert!(service.version.is_none());
    assert_eq!(service.available_versions.len(), 1);
}

#[test]
fn test_installable_service_with_multiple_versions() {
    let service = InstallableService {
        id: "mysql".to_string(),
        name: "MySQL".to_string(),
        description: "Relational database server".to_string(),
        installed: false,
        version: None,
        available_versions: vec![
            ServiceVersion {
                version: "8.0.35".to_string(),
                download_url: "https://dev.mysql.com/downloads/mysql/8.0.35.zip".to_string(),
                filename: "mysql-8.0.35.zip".to_string(),
            },
            ServiceVersion {
                version: "8.0.34".to_string(),
                download_url: "https://dev.mysql.com/downloads/mysql/8.0.34.zip".to_string(),
                filename: "mysql-8.0.34.zip".to_string(),
            },
        ],
    };
    assert_eq!(service.available_versions.len(), 2);
    assert_eq!(service.available_versions[0].version, "8.0.35");
}

// ============ ServiceVersion Tests ============

#[test]
fn test_service_version() {
    let sv = ServiceVersion {
        version: "1.24.0".to_string(),
        download_url: "https://nginx.org/download/nginx-1.24.0.zip".to_string(),
        filename: "nginx-1.24.0.zip".to_string(),
    };
    assert_eq!(sv.version, "1.24.0");
    assert!(sv.download_url.contains("nginx.org"));
    assert!(sv.filename.ends_with(".zip"));
}

#[test]
fn test_service_version_serialization() {
    let sv = ServiceVersion {
        version: "7.2.4".to_string(),
        download_url: "https://github.com/redis/redis/releases/7.2.4.zip".to_string(),
        filename: "redis-7.2.4-win64.zip".to_string(),
    };

    let json = serde_json::to_string(&sv).expect("Should serialize");
    let deserialized: ServiceVersion = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.version, sv.version);
    assert_eq!(deserialized.download_url, sv.download_url);
    assert_eq!(deserialized.filename, sv.filename);
}
