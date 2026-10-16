//! Tests for the version grouping/compare logic shared by the PHP/Node screens.

use harvest_core::types::{AvailableVersion, InstalledVersion};
use harvest_core::versions::{compare_versions, group_versions};
use std::cmp::Ordering;

fn avail(version: &str, full: &str, lts: bool) -> AvailableVersion {
    AvailableVersion {
        version: version.into(),
        full_version: full.into(),
        lts,
        download_url: String::new(),
    }
}

fn inst(version: &str, path: &str) -> InstalledVersion {
    InstalledVersion {
        version: version.into(),
        path: path.into(),
    }
}

#[test]
fn compare_is_numeric_not_lexical() {
    assert_eq!(compare_versions("8.4", "8.10"), Ordering::Less);
    assert_eq!(compare_versions("8.4.2", "8.4.1"), Ordering::Greater);
    assert_eq!(compare_versions("8.4", "8.4"), Ordering::Equal);
    assert_eq!(compare_versions("9", "10"), Ordering::Less);
}

#[test]
fn php_group_marks_installed_default_and_update() {
    let available = vec![avail("8.4", "8.4.2", true), avail("8.3", "8.3.5", true)];
    let installed = vec![inst("8.4.1", "C:/harvest/bin/php/php-8.4.1-x64")];
    let default = Some("C:/harvest/bin/php/php-8.4.1-x64");

    let groups = group_versions(&installed, &available, default, false);
    assert_eq!(groups.len(), 2);

    // Sorted newest-first.
    assert_eq!(groups[0].id, "8.4");
    assert_eq!(groups[1].id, "8.3");

    let g84 = &groups[0];
    assert!(g84.installed);
    assert_eq!(g84.label, "8.4.1");
    assert!(g84.is_default);
    assert!(g84.update_available, "8.4.2 is newer than installed 8.4.1");

    let g83 = &groups[1];
    assert!(!g83.installed);
    assert_eq!(g83.label, "8.3.5");
    assert!(!g83.is_default);
    assert!(!g83.update_available);
}

#[test]
fn node_groups_by_major() {
    let available = vec![avail("22", "22.12.0", true), avail("20", "20.18.1", true)];
    let installed = vec![inst("22.12.0", "C:/harvest/bin/nodejs/node-v22.12.0-win-x64")];
    let default = Some("C:/harvest/bin/nodejs/node-v22.12.0-win-x64");

    let groups = group_versions(&installed, &available, default, true);
    assert_eq!(groups[0].id, "22");
    assert!(groups[0].installed);
    assert!(groups[0].is_default);
    assert!(!groups[0].update_available, "installed equals latest");
    assert!(!groups[1].installed);
}

#[test]
fn installed_without_available_is_kept_as_eol_group() {
    let available = vec![avail("8.4", "8.4.2", true)];
    let installed = vec![inst("7.4.33", "C:/harvest/bin/php/php-7.4.33-x64")];

    let groups = group_versions(&installed, &available, None, false);
    let ids: Vec<&str> = groups.iter().map(|g| g.id.as_str()).collect();
    assert!(ids.contains(&"8.4"));
    assert!(ids.contains(&"7.4"));

    let eol = groups.iter().find(|g| g.id == "7.4").unwrap();
    assert!(eol.installed);
    assert!(!eol.lts);
}

#[test]
fn default_detection_normalizes_separators_and_case() {
    let available = vec![avail("8.4", "8.4.1", false)];
    // Installed path uses backslashes + different case than the default arg.
    let installed = vec![inst("8.4.1", "C:\\Harvest\\bin\\php\\PHP-8.4.1")];
    let default = Some("c:/harvest/bin/php/php-8.4.1");

    let groups = group_versions(&installed, &available, default, false);
    assert!(groups[0].is_default);
}

#[test]
fn groups_sort_numerically_descending() {
    let available = vec![avail("8.9", "8.9.0", false), avail("8.10", "8.10.0", false)];
    let groups = group_versions(&[], &available, None, false);
    assert_eq!(groups[0].id, "8.10");
    assert_eq!(groups[1].id, "8.9");
}
