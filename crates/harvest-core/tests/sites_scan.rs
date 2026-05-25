//! Integration test for scan_sites + stack/version detection against a temp tree.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use harvest_core::sites::scan_sites;

struct TempTree(PathBuf);

impl TempTree {
    fn new(tag: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("harvest-test-{tag}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn project(&self, name: &str) -> PathBuf {
        let p = self.0.join(name);
        fs::create_dir_all(&p).unwrap();
        p
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, name: &str, contents: &str) {
    fs::write(path.join(name), contents).unwrap();
}

#[test]
fn scans_and_detects_projects() {
    let tree = TempTree::new("scan");

    // Laravel project with a PHP constraint.
    let laravel = tree.project("laravel-app");
    write(&laravel, "composer.json", r#"{"require":{"laravel/framework":"^11.0","php":"^8.2"}}"#);
    write(&laravel, "artisan", "#!/usr/bin/env php");

    // Vue + Vite project using bun.
    let vue = tree.project("vue-app");
    write(&vue, "package.json", r#"{"dependencies":{"vue":"^3.4.0"},"devDependencies":{"vite":"^5.0.0"}}"#);
    write(&vue, "bun.lockb", "");

    // Plain PHP project.
    let plain = tree.project("plain");
    write(&plain, "index.php", "<?php echo 'hi';");

    // Things that must be ignored.
    tree.project(".hidden");
    tree.project("node_modules");
    tree.project("not-a-project"); // no web files

    let sites = scan_sites(vec![tree.0.to_string_lossy().to_string()]).unwrap();
    let names: Vec<&str> = sites.iter().map(|s| s.name.as_str()).collect();

    assert_eq!(names, vec!["laravel-app", "plain", "vue-app"], "sorted, ignored dirs excluded");

    let laravel_site = sites.iter().find(|s| s.name == "laravel-app").unwrap();
    assert_eq!(laravel_site.php_version, "8.2");
    assert_eq!(laravel_site.domain, "laravel-app.test");
    assert!(laravel_site.ssl);
    assert!(laravel_site.stack.iter().any(|s| s.name == "laravel"));

    let vue_site = sites.iter().find(|s| s.name == "vue-app").unwrap();
    assert_eq!(vue_site.php_version, "8.4", "default when no PHP constraint");
    let stack: Vec<&str> = vue_site.stack.iter().map(|s| s.name.as_str()).collect();
    assert!(stack.contains(&"vue"));
    assert!(stack.contains(&"vite"));
    assert!(stack.contains(&"bun"));
}

#[test]
fn invalid_directories_yield_no_sites() {
    let sites = scan_sites(vec!["Z:/definitely/not/here/at/all".into()]).unwrap();
    assert!(sites.is_empty());
}
