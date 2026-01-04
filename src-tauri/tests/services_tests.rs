use tauri_app_lib::commands::services::*;

#[test]
fn test_extract_version_nginx() {
    let output = "nginx version: nginx/1.24.0";
    let version = extract_version_from_string(output);
    assert_eq!(version, Some("1.24.0".to_string()));
}

#[test]
fn test_extract_version_mysql() {
    let output = "mysqld  Ver 8.0.30 for Win64 on x86_64 (MySQL Community Server - GPL)";
    let version = extract_version_from_string(output);
    assert_eq!(version, Some("8.0.30".to_string()));
}

#[test]
fn test_extract_version_redis() {
    let output = "Redis server v=7.2.4 sha=00000000:0 malloc=jemalloc-5.2.1 bits=64";
    let version = extract_version_from_string(output);
    assert_eq!(version, Some("7.2.4".to_string()));
}

#[test]
fn test_extract_version_simple() {
    let output = "version 1.0.0";
    let version = extract_version_from_string(output);
    assert_eq!(version, Some("1.0.0".to_string()));
}

#[test]
fn test_extract_version_with_prefix() {
    let output = "v2.5.1 release";
    let version = extract_version_from_string(output);
    assert_eq!(version, Some("2.5.1".to_string()));
}

#[test]
fn test_extract_version_no_version() {
    let output = "no version here";
    let version = extract_version_from_string(output);
    assert!(version.is_none());
}

#[test]
fn test_extract_version_single_number() {
    let output = "version 5";
    let version = extract_version_from_string(output);
    assert!(version.is_none()); // Need at least one dot
}

#[test]
fn test_extract_version_two_parts() {
    let output = "PHP 8.3";
    let version = extract_version_from_string(output);
    assert_eq!(version, Some("8.3".to_string()));
}

#[test]
fn test_extract_version_four_parts() {
    let output = "Build 1.2.3.4";
    let version = extract_version_from_string(output);
    // Should stop at third dot or handle gracefully
    assert!(version.is_some());
}
