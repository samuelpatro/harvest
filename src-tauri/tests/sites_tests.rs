use tauri_app_lib::commands::sites::*;

#[test]
fn test_get_version_from_value_with_caret() {
    let value = serde_json::json!("^3.4.0");
    let result = get_version_from_value(Some(&value));
    assert_eq!(result, Some("3.4.0".to_string()));
}

#[test]
fn test_get_version_from_value_with_tilde() {
    let value = serde_json::json!("~2.0.0");
    let result = get_version_from_value(Some(&value));
    assert_eq!(result, Some("2.0.0".to_string()));
}

#[test]
fn test_get_version_from_value_plain() {
    let value = serde_json::json!("1.0.0");
    let result = get_version_from_value(Some(&value));
    assert_eq!(result, Some("1.0.0".to_string()));
}

#[test]
fn test_get_version_from_value_none() {
    let result = get_version_from_value(None);
    assert!(result.is_none());
}

#[test]
fn test_get_version_from_value_non_string() {
    let value = serde_json::json!(123);
    let result = get_version_from_value(Some(&value));
    assert!(result.is_none());
}

#[test]
fn test_parse_php_version_constraint_caret() {
    let result = parse_php_version_constraint("^8.1");
    assert_eq!(result, Some("8.1".to_string()));
}

#[test]
fn test_parse_php_version_constraint_gte() {
    let result = parse_php_version_constraint(">=8.2");
    assert_eq!(result, Some("8.2".to_string()));
}

#[test]
fn test_parse_php_version_constraint_range() {
    let result = parse_php_version_constraint(">=8.1 <9.0");
    assert_eq!(result, Some("8.19.0".to_string())); // Note: concatenates all digits
}

#[test]
fn test_parse_php_version_constraint_exact() {
    let result = parse_php_version_constraint("8.3.1");
    assert_eq!(result, Some("8.3.1".to_string()));
}

#[test]
fn test_parse_php_version_constraint_empty() {
    let result = parse_php_version_constraint("*");
    assert!(result.is_none());
}

#[test]
fn test_generate_site_id_unix_path() {
    let id = generate_site_id("mysite", "/var/www/mysite");
    assert_eq!(id, "mysite--var-www-mysite");
}

#[test]
fn test_generate_site_id_windows_path() {
    let id = generate_site_id("mysite", "C:\\harvest\\www\\mysite");
    // C:\ becomes C-- (colon->dash, backslash->dash)
    assert_eq!(id, "mysite-C--harvest-www-mysite");
}

#[test]
fn test_generate_domain() {
    let domain = generate_domain("my-project");
    assert_eq!(domain, "my-project.test");
}

#[test]
fn test_generate_domain_simple() {
    let domain = generate_domain("blog");
    assert_eq!(domain, "blog.test");
}

#[test]
fn test_scan_sites_empty_directories() {
    let result = scan_sites(vec![]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_scan_sites_nonexistent_directory() {
    let result = scan_sites(vec!["/nonexistent/path/12345".to_string()]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}
