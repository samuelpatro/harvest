use tauri_app_lib::commands::node::*;
use tauri_app_lib::types::*;

#[test]
fn test_get_node_download_url_major_version() {
    let url = get_node_download_url("22");
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("nodejs.org"));
    assert!(url.contains("22.12.0"));
    assert!(url.ends_with("-win-x64.zip"));
}

#[test]
fn test_get_node_download_url_full_version() {
    let url = get_node_download_url("20.18.1");
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("20.18.1"));
}

#[test]
fn test_get_node_download_url_lts_versions() {
    // Test various LTS versions
    for major in &["18", "20", "22"] {
        let url = get_node_download_url(major);
        assert!(url.is_some(), "Should have URL for Node {}", major);
    }
}

#[test]
fn test_clean_version_string_with_v() {
    assert_eq!(clean_version_string("v20.10.0"), "20.10.0");
}

#[test]
fn test_clean_version_string_without_v() {
    assert_eq!(clean_version_string("20.10.0"), "20.10.0");
}

#[test]
fn test_clean_version_string_with_whitespace() {
    assert_eq!(clean_version_string("  v20.10.0  "), "20.10.0");
}

#[test]
fn test_parse_node_folder_name_standard() {
    let version = parse_node_folder_name("node-v20.10.0-win-x64");
    assert_eq!(version, Some("20.10.0".to_string()));
}

#[test]
fn test_parse_node_folder_name_without_v() {
    let version = parse_node_folder_name("node-20.10.0-win-x64");
    assert_eq!(version, Some("20.10.0".to_string()));
}

#[test]
fn test_parse_node_folder_name_invalid() {
    let version = parse_node_folder_name("not-a-node-folder");
    assert!(version.is_none());
}

#[test]
fn test_parse_node_folder_name_short() {
    let version = parse_node_folder_name("node");
    assert!(version.is_none());
}

#[test]
fn test_package_manager_struct() {
    let pm = PackageManager {
        name: "npm".to_string(),
        command: "npm".to_string(),
        installed: true,
        version: Some("10.2.0".to_string()),
        description: "Node Package Manager".to_string(),
    };
    assert_eq!(pm.name, "npm");
    assert!(pm.installed);
    assert_eq!(pm.version, Some("10.2.0".to_string()));
}

#[test]
fn test_installed_version_struct() {
    let iv = InstalledVersion {
        version: "22.12.0".to_string(),
        path: "/usr/local/bin/node".to_string(),
    };
    assert_eq!(iv.version, "22.12.0");
}

#[test]
fn test_available_version_struct() {
    let av = AvailableVersion {
        version: "22".to_string(),
        full_version: "22.12.0".to_string(),
        lts: true,
        download_url: "https://nodejs.org/dist/v22.12.0/node-v22.12.0-win-x64.zip".to_string(),
    };
    assert!(av.lts);
    assert_eq!(av.version, "22");
}
