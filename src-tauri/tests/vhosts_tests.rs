use tauri_app_lib::commands::vhosts::*;

#[test]
fn test_vhost_config_creation() {
    let config = VhostConfig {
        site_name: "mysite".to_string(),
        domain: "mysite.test".to_string(),
        root_path: "/var/www/mysite".to_string(),
        php_version: Some("8.3".to_string()),
        ssl: true,
    };
    assert_eq!(config.site_name, "mysite");
    assert_eq!(config.domain, "mysite.test");
    assert!(config.ssl);
}

#[test]
fn test_vhost_status_linked() {
    let status = VhostStatus {
        site_name: "mysite".to_string(),
        domain: "mysite.test".to_string(),
        nginx_config_exists: true,
        hosts_entry_exists: true,
        is_linked: true,
    };
    assert!(status.is_linked);
}

#[test]
fn test_vhost_status_unlinked() {
    let status = VhostStatus {
        site_name: "mysite".to_string(),
        domain: "mysite.test".to_string(),
        nginx_config_exists: false,
        hosts_entry_exists: false,
        is_linked: false,
    };
    assert!(!status.is_linked);
}

#[test]
fn test_generate_nginx_config_basic() {
    let config = VhostConfig {
        site_name: "testsite".to_string(),
        domain: "testsite.test".to_string(),
        root_path: "/var/www/testsite".to_string(),
        php_version: None,
        ssl: true,
    };
    let nginx_config = generate_nginx_config(&config);

    assert!(nginx_config.contains("server_name testsite.test"));
    assert!(nginx_config.contains("listen 80;"));
    assert!(nginx_config.contains("listen 8443 ssl;"));
    assert!(nginx_config.contains("root \"/var/www/testsite\""));
    assert!(nginx_config.contains("ssl_certificate"));
}

#[test]
fn test_generate_nginx_config_windows_path() {
    let config = VhostConfig {
        site_name: "winsite".to_string(),
        domain: "winsite.test".to_string(),
        root_path: "C:\\laragon\\www\\winsite".to_string(),
        php_version: None,
        ssl: true,
    };
    let nginx_config = generate_nginx_config(&config);

    // Backslashes should be converted to forward slashes
    assert!(nginx_config.contains("C:/laragon/www/winsite"));
    assert!(!nginx_config.contains("\\"));
}

#[test]
fn test_get_nginx_config_filename() {
    let filename = get_nginx_config_filename("myproject");
    assert_eq!(filename, "auto.myproject.test.conf");
}

#[test]
fn test_get_nginx_config_filename_with_dashes() {
    let filename = get_nginx_config_filename("my-awesome-project");
    assert_eq!(filename, "auto.my-awesome-project.test.conf");
}

#[test]
fn test_check_hosts_entry_in_content_found() {
    let content = "127.0.0.1 localhost\n127.0.0.1 mysite.test\n::1 localhost";
    assert!(check_hosts_entry_in_content(content, "mysite.test"));
}

#[test]
fn test_check_hosts_entry_in_content_not_found() {
    let content = "127.0.0.1 localhost\n127.0.0.1 other.test\n::1 localhost";
    assert!(!check_hosts_entry_in_content(content, "mysite.test"));
}

#[test]
fn test_check_hosts_entry_in_content_commented() {
    let content = "127.0.0.1 localhost\n# 127.0.0.1 mysite.test\n::1 localhost";
    assert!(!check_hosts_entry_in_content(content, "mysite.test"));
}

#[test]
fn test_check_hosts_entry_in_content_empty() {
    let content = "";
    assert!(!check_hosts_entry_in_content(content, "mysite.test"));
}

#[test]
fn test_generate_hosts_entry() {
    let entry = generate_hosts_entry("mysite.test");
    assert_eq!(entry, "127.0.0.1 mysite.test");
}

#[test]
fn test_filter_hosts_content_removes_domain() {
    let content = "127.0.0.1 localhost\n127.0.0.1 mysite.test\n127.0.0.1 other.test";
    let filtered = filter_hosts_content(content, "mysite.test");

    assert!(filtered.contains("localhost"));
    assert!(filtered.contains("other.test"));
    assert!(!filtered.contains("mysite.test"));
}

#[test]
fn test_filter_hosts_content_preserves_comments() {
    let content = "# This is a comment\n127.0.0.1 localhost\n127.0.0.1 mysite.test";
    let filtered = filter_hosts_content(content, "mysite.test");

    assert!(filtered.contains("# This is a comment"));
    assert!(filtered.contains("localhost"));
    assert!(!filtered.contains("mysite.test"));
}

#[test]
fn test_filter_hosts_content_preserves_empty_lines() {
    let content = "127.0.0.1 localhost\n\n127.0.0.1 mysite.test";
    let filtered = filter_hosts_content(content, "mysite.test");

    assert!(filtered.contains("localhost"));
    // Empty lines are preserved
    let lines: Vec<&str> = filtered.lines().collect();
    assert!(lines.iter().any(|l| l.is_empty()));
}

#[test]
fn test_get_nginx_config_path_format() {
    let path = get_nginx_config_path("testsite");
    let path_str = path.to_string_lossy();

    assert!(path_str.ends_with("auto.testsite.test.conf"));
    assert!(path_str.contains("sites-enabled"));
}
