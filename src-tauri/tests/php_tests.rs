use tauri_app_lib::commands::php::*;
use tauri_app_lib::types::*;

#[test]
fn test_parse_php_version_output_standard() {
    let output = "PHP 8.4.1 (cli) (built: Dec 18 2024 12:00:00) (NTS Visual C++ 2019 x64)
Copyright (c) The PHP Group
Zend Engine v4.4.1, Copyright (c) Zend Technologies";
    let version = parse_php_version_output(output);
    assert_eq!(version, Some("8.4.1".to_string()));
}

#[test]
fn test_parse_php_version_output_simple() {
    let output = "PHP 8.3.0 (cli)";
    let version = parse_php_version_output(output);
    assert_eq!(version, Some("8.3.0".to_string()));
}

#[test]
fn test_parse_php_version_output_empty() {
    let output = "";
    let version = parse_php_version_output(output);
    assert!(version.is_none());
}

#[test]
fn test_parse_php_version_output_invalid() {
    let output = "Not PHP output";
    let version = parse_php_version_output(output);
    assert!(version.is_none());
}

#[test]
fn test_parse_php_folder_name_standard() {
    let version = parse_php_folder_name("php-8.4.1-nts-Win32-vs16-x64");
    assert_eq!(version, Some("8.4.1".to_string()));
}

#[test]
fn test_parse_php_folder_name_simple() {
    let version = parse_php_folder_name("php-8.3");
    assert_eq!(version, Some("8.3".to_string()));
}

#[test]
fn test_parse_php_folder_name_invalid() {
    let version = parse_php_folder_name("not-a-php-folder");
    assert!(version.is_none());
}

#[test]
fn test_parse_php_folder_name_short() {
    let version = parse_php_folder_name("php");
    assert!(version.is_none());
}

#[test]
fn test_installed_version_struct() {
    let v = InstalledVersion {
        version: "8.3.1".to_string(),
        path: "C:/harvest/bin/php/php-8.3.1".to_string(),
    };
    assert_eq!(v.version, "8.3.1");
}

#[test]
fn test_available_version_struct() {
    let v = AvailableVersion {
        version: "8.4".to_string(),
        full_version: "8.4.1".to_string(),
        lts: true,
        download_url: "https://windows.php.net/downloads/releases/php-8.4.1-nts-Win32-vs17-x64.zip"
            .to_string(),
    };
    assert!(v.lts);
    assert_eq!(v.version, "8.4");
}

#[test]
fn test_php_extension_struct() {
    let ext = PhpExtension {
        name: "curl".to_string(),
        enabled: true,
        builtin: false,
    };
    assert_eq!(ext.name, "curl");
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
}
