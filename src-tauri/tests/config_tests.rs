use tauri_app_lib::commands::config::*;

// ============ Path Normalization Tests ============

#[test]
fn test_normalize_path_forward_slashes() {
    let path = "C:/harvest/bin/php/8.3";
    let normalized = normalize_path_for_comparison(path);
    assert_eq!(normalized, "c:\\harvest\\bin\\php\\8.3");
}

#[test]
fn test_normalize_path_backslashes() {
    let path = "C:\\harvest\\bin\\php\\8.3";
    let normalized = normalize_path_for_comparison(path);
    assert_eq!(normalized, "c:\\harvest\\bin\\php\\8.3");
}

#[test]
fn test_normalize_path_trailing_slash() {
    let path = "C:\\harvest\\bin\\php\\8.3\\";
    let normalized = normalize_path_for_comparison(path);
    assert_eq!(normalized, "c:\\harvest\\bin\\php\\8.3");
}

#[test]
fn test_normalize_path_mixed_slashes() {
    let path = "C:/harvest\\bin/php\\8.3/";
    let normalized = normalize_path_for_comparison(path);
    assert_eq!(normalized, "c:\\harvest\\bin\\php\\8.3");
}

#[test]
fn test_normalize_path_lowercase() {
    let path = "C:\\HARVEST\\Bin\\PHP\\8.3";
    let normalized = normalize_path_for_comparison(path);
    assert_eq!(normalized, "c:\\harvest\\bin\\php\\8.3");
}

#[test]
fn test_normalize_path_with_spaces() {
    let path = "  C:\\harvest\\bin  ";
    let normalized = normalize_path_for_comparison(path);
    assert_eq!(normalized, "c:\\harvest\\bin");
}

// ============ Path Exists in PATH Variable Tests ============

#[test]
fn test_path_exists_in_pathvar_exact_match() {
    let path = "C:\\harvest\\bin\\php\\8.3";
    let path_var = "C:\\Windows\\System32;C:\\harvest\\bin\\php\\8.3;C:\\Program Files";
    assert!(path_exists_in_pathvar(path, path_var, ';'));
}

#[test]
fn test_path_exists_in_pathvar_case_insensitive() {
    let path = "c:\\harvest\\bin\\php\\8.3";
    let path_var = "C:\\Windows\\System32;C:\\HARVEST\\BIN\\PHP\\8.3;C:\\Program Files";
    assert!(path_exists_in_pathvar(path, path_var, ';'));
}

#[test]
fn test_path_exists_in_pathvar_with_trailing_slash() {
    let path = "C:\\harvest\\bin\\php\\8.3\\";
    let path_var = "C:\\Windows\\System32;C:\\harvest\\bin\\php\\8.3;C:\\Program Files";
    assert!(path_exists_in_pathvar(path, path_var, ';'));
}

#[test]
fn test_path_exists_in_pathvar_not_found() {
    let path = "C:\\harvest\\bin\\php\\8.2";
    let path_var = "C:\\Windows\\System32;C:\\harvest\\bin\\php\\8.3;C:\\Program Files";
    assert!(!path_exists_in_pathvar(path, path_var, ';'));
}

#[test]
fn test_path_exists_in_pathvar_empty_pathvar() {
    let path = "C:\\harvest\\bin\\php\\8.3";
    let path_var = "";
    assert!(!path_exists_in_pathvar(path, path_var, ';'));
}

#[test]
fn test_path_exists_in_pathvar_first_entry() {
    let path = "C:\\harvest\\bin\\php\\8.3";
    let path_var = "C:\\harvest\\bin\\php\\8.3;C:\\Windows\\System32";
    assert!(path_exists_in_pathvar(path, path_var, ';'));
}

#[test]
fn test_path_exists_in_pathvar_last_entry() {
    let path = "C:\\harvest\\bin\\php\\8.3";
    let path_var = "C:\\Windows\\System32;C:\\harvest\\bin\\php\\8.3";
    assert!(path_exists_in_pathvar(path, path_var, ';'));
}

#[test]
fn test_path_exists_in_pathvar_forward_slashes() {
    let path = "C:/harvest/bin/php/8.3";
    let path_var = "C:\\Windows\\System32;C:\\harvest\\bin\\php\\8.3;C:\\Program Files";
    assert!(path_exists_in_pathvar(path, path_var, ';'));
}

#[test]
fn test_path_exists_in_pathvar_unix_style() {
    let path = "/usr/local/bin";
    let path_var = "/usr/bin:/usr/local/bin:/home/user/bin";
    assert!(path_exists_in_pathvar(path, path_var, ':'));
}

// ============ Filter Path from PATH Variable Tests ============

#[test]
fn test_filter_path_from_pathvar_middle() {
    let path = "C:\\harvest\\bin\\php\\8.3";
    let path_var = "C:\\Windows\\System32;C:\\harvest\\bin\\php\\8.3;C:\\Program Files";
    let result = filter_path_from_pathvar(path, path_var, ';');
    assert_eq!(result, "C:\\Windows\\System32;C:\\Program Files");
}

#[test]
fn test_filter_path_from_pathvar_first() {
    let path = "C:\\harvest\\bin\\php\\8.3";
    let path_var = "C:\\harvest\\bin\\php\\8.3;C:\\Windows\\System32;C:\\Program Files";
    let result = filter_path_from_pathvar(path, path_var, ';');
    assert_eq!(result, "C:\\Windows\\System32;C:\\Program Files");
}

#[test]
fn test_filter_path_from_pathvar_last() {
    let path = "C:\\harvest\\bin\\php\\8.3";
    let path_var = "C:\\Windows\\System32;C:\\Program Files;C:\\harvest\\bin\\php\\8.3";
    let result = filter_path_from_pathvar(path, path_var, ';');
    assert_eq!(result, "C:\\Windows\\System32;C:\\Program Files");
}

#[test]
fn test_filter_path_from_pathvar_case_insensitive() {
    let path = "c:\\harvest\\bin\\php\\8.3";
    let path_var = "C:\\Windows\\System32;C:\\HARVEST\\BIN\\PHP\\8.3;C:\\Program Files";
    let result = filter_path_from_pathvar(path, path_var, ';');
    assert_eq!(result, "C:\\Windows\\System32;C:\\Program Files");
}

#[test]
fn test_filter_path_from_pathvar_with_trailing_slash() {
    let path = "C:\\harvest\\bin\\php\\8.3\\";
    let path_var = "C:\\Windows\\System32;C:\\harvest\\bin\\php\\8.3;C:\\Program Files";
    let result = filter_path_from_pathvar(path, path_var, ';');
    assert_eq!(result, "C:\\Windows\\System32;C:\\Program Files");
}

#[test]
fn test_filter_path_from_pathvar_not_found() {
    let path = "C:\\harvest\\bin\\php\\8.2";
    let path_var = "C:\\Windows\\System32;C:\\harvest\\bin\\php\\8.3;C:\\Program Files";
    let result = filter_path_from_pathvar(path, path_var, ';');
    assert_eq!(result, "C:\\Windows\\System32;C:\\harvest\\bin\\php\\8.3;C:\\Program Files");
}

#[test]
fn test_filter_path_from_pathvar_only_entry() {
    let path = "C:\\harvest\\bin\\php\\8.3";
    let path_var = "C:\\harvest\\bin\\php\\8.3";
    let result = filter_path_from_pathvar(path, path_var, ';');
    assert_eq!(result, "");
}

#[test]
fn test_filter_path_from_pathvar_empty() {
    let path = "C:\\harvest\\bin\\php\\8.3";
    let path_var = "";
    let result = filter_path_from_pathvar(path, path_var, ';');
    assert_eq!(result, "");
}

#[test]
fn test_filter_path_from_pathvar_unix_style() {
    let path = "/usr/local/bin";
    let path_var = "/usr/bin:/usr/local/bin:/home/user/bin";
    let result = filter_path_from_pathvar(path, path_var, ':');
    assert_eq!(result, "/usr/bin:/home/user/bin");
}

// ============ Non-Windows Platform Tests ============

#[cfg(not(target_os = "windows"))]
mod non_windows_tests {
    use super::*;

    #[test]
    fn test_add_to_path_returns_error_on_non_windows() {
        let result = add_to_path("some/path".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("shell profile"));
    }

    #[test]
    fn test_remove_from_path_returns_error_on_non_windows() {
        let result = remove_from_path("some/path".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("shell profile"));
    }

    #[test]
    fn test_is_in_path_checks_env_var() {
        // This tests the actual environment variable on the current system
        // PATH should exist on any Unix system
        let result = is_in_path("/definitely/not/a/real/path/12345".to_string());
        assert!(!result);
    }
}
