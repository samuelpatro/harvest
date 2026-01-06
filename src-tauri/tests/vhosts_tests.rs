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

// Note: generate_nginx_config is private and requires HarvestPaths,
// so we test the public API (VhostConfig struct) instead

#[test]
fn test_vhost_config_path_handling() {
    // Test that VhostConfig can hold Windows paths
    let config = VhostConfig {
        site_name: "winsite".to_string(),
        domain: "winsite.test".to_string(),
        root_path: "C:\\harvest\\www\\winsite".to_string(),
        php_version: Some("8.3".to_string()),
        ssl: true,
    };
    assert_eq!(config.root_path, "C:\\harvest\\www\\winsite");
    assert_eq!(config.php_version, Some("8.3".to_string()));
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
    // Test with empty line in the middle (not at the end)
    let content = "127.0.0.1 localhost\n\n127.0.0.1 other.test\n127.0.0.1 mysite.test";
    let filtered = filter_hosts_content(content, "mysite.test");

    assert!(filtered.contains("localhost"));
    assert!(filtered.contains("other.test"));
    // Empty lines in the middle are preserved
    let lines: Vec<&str> = filtered.lines().collect();
    assert!(lines.iter().any(|l| l.is_empty()));
}

// Note: get_nginx_config_path is private and requires HarvestPaths,
// so we test get_nginx_config_filename which is the public version

#[test]
fn test_nginx_config_filename_format() {
    let filename = get_nginx_config_filename("testsite");
    assert_eq!(filename, "auto.testsite.test.conf");
    assert!(filename.starts_with("auto."));
    assert!(filename.ends_with(".test.conf"));
}
