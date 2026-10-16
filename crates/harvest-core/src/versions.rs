//! Pure grouping logic shared by the PHP and Node version screens. Kept here
//! (not in the UI layer) so it can be unit-tested without Slint.

use std::cmp::Ordering;
use std::collections::HashSet;

use crate::config::normalize_path_for_comparison;
use crate::types::{AvailableVersion, InstalledVersion};

/// One row in the versions list: an available major(.minor) group, plus whether
/// a matching version is installed / default / updatable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionGroup {
    pub id: String,    // PHP: "8.4"; Node: "22"
    pub label: String, // installed version if present, else latest available
    pub path: String,
    pub installed: bool,
    pub is_default: bool,
    pub lts: bool,
    pub update_available: bool,
}

/// Numeric, component-wise version comparison ("8.4" < "8.10").
pub fn compare_versions(a: &str, b: &str) -> Ordering {
    let pa: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let pb: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
    pa.cmp(&pb)
}

/// Group installed + available versions into UI rows. `node` selects major-only
/// grouping (Node) vs major.minor grouping (PHP). `default_path` is the version
/// directory currently on PATH. Result is sorted newest-first.
pub fn group_versions(
    installed: &[InstalledVersion],
    available: &[AvailableVersion],
    default_path: Option<&str>,
    node: bool,
) -> Vec<VersionGroup> {
    let def = default_path.map(normalize_path_for_comparison);
    let is_default = |path: &str| -> bool {
        def.as_deref()
            .is_some_and(|d| normalize_path_for_comparison(path) == d)
    };
    let group_of = |version: &str| -> String {
        if node {
            version.split('.').next().unwrap_or(version).to_string()
        } else {
            version.split('.').take(2).collect::<Vec<_>>().join(".")
        }
    };

    let mut groups = Vec::new();
    let mut seen = HashSet::new();

    for av in available {
        seen.insert(av.version.clone());
        let inst = installed.iter().find(|i| group_of(&i.version) == av.version);
        let group = match inst {
            Some(i) => VersionGroup {
                id: av.version.clone(),
                label: i.version.clone(),
                path: i.path.clone(),
                installed: true,
                is_default: is_default(&i.path),
                lts: av.lts,
                update_available: compare_versions(&av.full_version, &i.version) == Ordering::Greater,
            },
            None => VersionGroup {
                id: av.version.clone(),
                label: av.full_version.clone(),
                path: String::new(),
                installed: false,
                is_default: false,
                lts: av.lts,
                update_available: false,
            },
        };
        groups.push(group);
    }

    // Installed versions with no matching available entry (e.g. EOL).
    for i in installed {
        let gid = group_of(&i.version);
        if seen.insert(gid.clone()) {
            groups.push(VersionGroup {
                id: gid,
                label: i.version.clone(),
                path: i.path.clone(),
                installed: true,
                is_default: is_default(&i.path),
                lts: false,
                update_available: false,
            });
        }
    }

    groups.sort_by(|a, b| compare_versions(&b.id, &a.id));
    groups
}
