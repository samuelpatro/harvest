//! Behavior-locking tests for the pure helpers extracted into harvest-core.
//! These avoid filesystem/network/process access so they run anywhere.

use harvest_core::{config, node, php, services, sites, vhosts};

#[test]
fn php_version_constraint_keeps_digits_and_dots() {
    assert_eq!(sites::parse_php_version_constraint("^8.1"), Some("8.1".into()));
    assert_eq!(sites::parse_php_version_constraint(">=7.4"), Some("7.4".into()));
    assert_eq!(sites::parse_php_version_constraint("8.2.*"), Some("8.2.".into()));
    assert_eq!(sites::parse_php_version_constraint("abc"), None);
    assert_eq!(sites::parse_php_version_constraint(""), None);
}

#[test]
fn site_id_and_domain_generation() {
    assert_eq!(sites::generate_domain("mysite"), "mysite.test");
    assert_eq!(sites::generate_site_id("foo", "C:/x/foo"), "foo-C--x-foo");
}

#[test]
fn version_from_json_value_strips_range_prefixes() {
    let caret = serde_json::json!("^1.2.3");
    let tilde = serde_json::json!("~4.5");
    let num = serde_json::json!(10);
    assert_eq!(sites::get_version_from_value(Some(&caret)), Some("1.2.3".into()));
    assert_eq!(sites::get_version_from_value(Some(&tilde)), Some("4.5".into()));
    assert_eq!(sites::get_version_from_value(Some(&num)), None);
    assert_eq!(sites::get_version_from_value(None), None);
}

#[test]
fn path_normalization_and_pathvar_ops() {
    assert_eq!(config::normalize_path_for_comparison("C:/Foo/Bar/"), "c:\\foo\\bar");
    assert!(config::path_exists_in_pathvar("C:/foo", "X;C:\\Foo;Y", ';'));
    assert!(!config::path_exists_in_pathvar("C:/nope", "X;C:\\Foo;Y", ';'));
    assert_eq!(
        config::filter_path_from_pathvar("C:/foo", "A;C:\\Foo;B", ';'),
        "A;B"
    );
}

#[test]
fn service_version_extraction() {
    assert_eq!(
        services::extract_version_from_string("nginx version: nginx/1.24.0"),
        Some("1.24.0".into())
    );
    assert_eq!(
        services::extract_version_from_string("Ver 8.0.30 for Win64"),
        Some("8.0.30".into())
    );
    assert_eq!(services::extract_version_from_string("no version here"), None);
}

#[test]
fn php_version_parsing() {
    assert_eq!(
        php::parse_php_version_output("PHP 8.4.1 (cli) (built: Dec 2024)"),
        Some("8.4.1".into())
    );
    assert_eq!(
        php::parse_php_folder_name("php-8.4.1-nts-Win32-vs16-x64"),
        Some("8.4.1".into())
    );
    assert_eq!(php::parse_php_folder_name("notphp"), None);
}

#[test]
fn node_helpers() {
    assert_eq!(node::clean_version_string("v20.10.0"), "20.10.0");
    assert_eq!(
        node::parse_node_folder_name("node-v20.10.0-win-x64"),
        Some("20.10.0".into())
    );
    let url = node::get_node_download_url("20").unwrap();
    assert!(url.contains("20.18.1"), "got {url}");
    assert!(url.contains("node-v20.18.1-win-x64.zip"), "got {url}");
}

#[test]
fn vhost_hosts_and_config_helpers() {
    assert_eq!(vhosts::generate_hosts_entry("foo.test"), "127.0.0.1 foo.test");
    assert_eq!(vhosts::get_nginx_config_filename("foo"), "auto.foo.test.conf");
    assert!(vhosts::check_hosts_entry_in_content(
        "# comment\n127.0.0.1 foo.test",
        "foo.test"
    ));
    assert!(!vhosts::check_hosts_entry_in_content("# foo.test", "foo.test"));
    assert_eq!(
        vhosts::filter_hosts_content("127.0.0.1 foo.test\n127.0.0.1 bar.test", "foo.test"),
        "127.0.0.1 bar.test"
    );
}
