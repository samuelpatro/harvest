use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::commands::config::get_harvest_home;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VhostConfig {
    pub site_name: String,
    pub domain: String,
    pub root_path: String,
    pub php_version: Option<String>,
    pub ssl: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VhostStatus {
    pub site_name: String,
    pub domain: String,
    pub nginx_config_exists: bool,
    pub hosts_entry_exists: bool,
    pub is_linked: bool,
}

/// Dynamic path resolver based on harvest_home configuration
struct HarvestPaths {
    nginx_sites_path: String,
    ssl_cert_path: String,
    ssl_key_path: String,
    nginx_exe: String,
    nginx_conf: String,
}

impl HarvestPaths {
    fn new(harvest_home: &str) -> Self {
        #[cfg(target_os = "windows")]
        let nginx_exe_name = "nginx.exe";
        #[cfg(not(target_os = "windows"))]
        let nginx_exe_name = "nginx";

        Self {
            nginx_sites_path: format!("{}/etc/nginx/sites-enabled", harvest_home),
            ssl_cert_path: format!("{}/etc/ssl/harvest.crt", harvest_home),
            ssl_key_path: format!("{}/etc/ssl/harvest.key", harvest_home),
            nginx_exe: format!("{}/bin/nginx/{}", harvest_home, nginx_exe_name),
            nginx_conf: format!("{}/etc/nginx/nginx.conf", harvest_home),
        }
    }
}

fn get_hosts_file_path() -> &'static str {
    #[cfg(target_os = "windows")]
    { "C:/Windows/System32/drivers/etc/hosts" }
    #[cfg(not(target_os = "windows"))]
    { "/etc/hosts" }
}

fn generate_nginx_config(config: &VhostConfig, paths: &HarvestPaths) -> String {
    let root = config.root_path.replace('\\', "/");

    // Determine document root - check for public folder
    let public_path = PathBuf::from(&config.root_path).join("public");
    let document_root = if public_path.exists() {
        format!("{}/public", root)
    } else {
        root.clone()
    };

    format!(r#"server {{
    listen 80;
    listen 8443 ssl;
    server_name {domain} *.{domain};
    root "{document_root}";

    index index.html index.htm index.php;

    location / {{
        try_files $uri $uri/ /index.php$is_args$args;
        autoindex on;
    }}

    location ~ \.php$ {{
        include snippets/fastcgi-php.conf;
        fastcgi_pass php_upstream;
    }}

    # Enable SSL
    ssl_certificate "{ssl_cert}";
    ssl_certificate_key "{ssl_key}";
    ssl_session_timeout 5m;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    charset utf-8;

    location = /favicon.ico {{ access_log off; log_not_found off; }}
    location = /robots.txt  {{ access_log off; log_not_found off; }}
    location ~ /\.ht {{
        deny all;
    }}
}}
"#,
        domain = config.domain,
        document_root = document_root,
        ssl_cert = paths.ssl_cert_path,
        ssl_key = paths.ssl_key_path,
    )
}

fn get_nginx_config_path(site_name: &str, paths: &HarvestPaths) -> PathBuf {
    PathBuf::from(&paths.nginx_sites_path).join(format!("auto.{}.test.conf", site_name))
}

fn nginx_config_exists(site_name: &str, paths: &HarvestPaths) -> bool {
    get_nginx_config_path(site_name, paths).exists()
}

fn hosts_entry_exists(domain: &str) -> bool {
    let hosts_path = get_hosts_file_path();
    if let Ok(content) = fs::read_to_string(hosts_path) {
        content.lines().any(|line| {
            let line = line.trim();
            !line.starts_with('#') && line.contains(domain)
        })
    } else {
        false
    }
}

#[tauri::command]
pub fn get_vhost_status(site_name: String) -> VhostStatus {
    let harvest_home = get_harvest_home();
    let paths = HarvestPaths::new(&harvest_home);
    let domain = format!("{}.test", site_name);
    let nginx_exists = nginx_config_exists(&site_name, &paths);
    let hosts_exists = hosts_entry_exists(&domain);

    VhostStatus {
        site_name: site_name.clone(),
        domain: domain.clone(),
        nginx_config_exists: nginx_exists,
        hosts_entry_exists: hosts_exists,
        is_linked: nginx_exists && hosts_exists,
    }
}

#[tauri::command]
pub fn get_all_vhost_statuses(site_names: Vec<String>) -> Vec<VhostStatus> {
    site_names.into_iter().map(|name| get_vhost_status(name)).collect()
}

#[tauri::command]
pub async fn link_site(site_name: String, root_path: String) -> Result<String, String> {
    let harvest_home = get_harvest_home();
    let paths = HarvestPaths::new(&harvest_home);
    let domain = format!("{}.test", site_name);

    let config = VhostConfig {
        site_name: site_name.clone(),
        domain: domain.clone(),
        root_path,
        php_version: None,
        ssl: true,
    };

    // Step 1: Create nginx config
    let nginx_config = generate_nginx_config(&config, &paths);
    let nginx_path = get_nginx_config_path(&site_name, &paths);

    fs::write(&nginx_path, &nginx_config)
        .map_err(|e| format!("Failed to write nginx config: {}", e))?;

    // Step 2: Add hosts entry (requires elevation)
    if !hosts_entry_exists(&domain) {
        add_hosts_entry(&domain)?;
    }

    // Step 3: Reload nginx
    reload_nginx(&paths)?;

    Ok(format!("Site {} linked successfully at https://{}", site_name, domain))
}

#[tauri::command]
pub async fn unlink_site(site_name: String) -> Result<String, String> {
    let harvest_home = get_harvest_home();
    let paths = HarvestPaths::new(&harvest_home);
    let domain = format!("{}.test", site_name);

    // Step 1: Remove nginx config
    let nginx_path = get_nginx_config_path(&site_name, &paths);
    if nginx_path.exists() {
        fs::remove_file(&nginx_path)
            .map_err(|e| format!("Failed to remove nginx config: {}", e))?;
    }

    // Step 2: Remove hosts entry
    if hosts_entry_exists(&domain) {
        remove_hosts_entry(&domain)?;
    }

    // Step 3: Reload nginx
    reload_nginx(&paths)?;

    Ok(format!("Site {} unlinked successfully", site_name))
}

fn add_hosts_entry(domain: &str) -> Result<(), String> {
    let hosts_path = get_hosts_file_path();
    let entry = format!("127.0.0.1 {}", domain);

    #[cfg(target_os = "windows")]
    {
        // Use PowerShell with elevation to add entry (hidden window)
        let script = format!(
            r#"Add-Content -Path '{}' -Value "`n{}" -Force"#,
            hosts_path, entry
        );

        let output = Command::new("powershell")
            .args([
                "-WindowStyle", "Hidden",
                "-Command",
                &format!("Start-Process powershell -ArgumentList '-WindowStyle Hidden -Command {}' -Verb RunAs -Wait -WindowStyle Hidden",
                    script.replace("'", "''").replace("\"", "\\\""))
            ])
            .output()
            .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

        if !output.status.success() {
            // Try without elevation as fallback
            let direct_result = Command::new("powershell")
                .args(["-WindowStyle", "Hidden", "-Command", &script])
                .output();

            if direct_result.is_err() || !direct_result.unwrap().status.success() {
                return Err("Failed to add hosts entry. Please run Harvest as Administrator.".to_string());
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Use sudo on macOS/Linux
        let output = Command::new("sudo")
            .args(["sh", "-c", &format!("echo '{}' >> {}", entry, hosts_path)])
            .output()
            .map_err(|e| format!("Failed to run sudo: {}", e))?;

        if !output.status.success() {
            return Err("Failed to add hosts entry. Please ensure you have sudo permissions.".to_string());
        }
    }

    Ok(())
}

fn remove_hosts_entry(domain: &str) -> Result<(), String> {
    let hosts_path = get_hosts_file_path();

    // Read current hosts file
    let content = fs::read_to_string(hosts_path)
        .map_err(|e| format!("Failed to read hosts file: {}", e))?;

    // Filter out the domain line
    let new_content: Vec<&str> = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.is_empty() || trimmed.starts_with('#') || !trimmed.contains(domain)
        })
        .collect();

    let new_content = new_content.join("\n");

    #[cfg(target_os = "windows")]
    {
        // Write back using PowerShell with elevation (hidden window)
        // Use a temp file approach to avoid escaping issues
        let temp_path = std::env::temp_dir().join("harvest_hosts_temp.txt");
        std::fs::write(&temp_path, &new_content)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        let script = format!(
            r#"Copy-Item -Path '{}' -Destination '{}' -Force"#,
            temp_path.to_string_lossy().replace('\\', "/"),
            hosts_path
        );

        let output = Command::new("powershell")
            .args([
                "-WindowStyle", "Hidden",
                "-Command",
                &format!("Start-Process powershell -ArgumentList '-WindowStyle Hidden -Command {}' -Verb RunAs -Wait -WindowStyle Hidden",
                    script.replace("'", "''"))
            ])
            .output()
            .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        if !output.status.success() {
            return Err("Failed to remove hosts entry. Please run Harvest as Administrator.".to_string());
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Use sudo on macOS/Linux
        let escaped_content = new_content.replace("'", "'\\''");
        let output = Command::new("sudo")
            .args(["sh", "-c", &format!("echo '{}' > {}", escaped_content, hosts_path)])
            .output()
            .map_err(|e| format!("Failed to run sudo: {}", e))?;

        if !output.status.success() {
            return Err("Failed to remove hosts entry. Please ensure you have sudo permissions.".to_string());
        }
    }

    Ok(())
}

fn reload_nginx(paths: &HarvestPaths) -> Result<(), String> {
    let nginx_exe = PathBuf::from(&paths.nginx_exe);
    if !nginx_exe.exists() {
        return Err("Nginx executable not found".to_string());
    }

    let output = Command::new(&nginx_exe)
        .args(["-c", &paths.nginx_conf, "-s", "reload"])
        .output()
        .map_err(|e| format!("Failed to reload nginx: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Ignore "no process" error - nginx might not be running
        if !stderr.contains("No mapping for the Unicode character") && !stderr.contains("error") {
            return Err(format!("Nginx reload failed: {}", stderr));
        }
    }

    Ok(())
}

#[tauri::command]
pub fn check_nginx_config() -> Result<String, String> {
    let harvest_home = get_harvest_home();
    let paths = HarvestPaths::new(&harvest_home);

    let nginx_exe = PathBuf::from(&paths.nginx_exe);
    if !nginx_exe.exists() {
        return Err("Nginx executable not found".to_string());
    }

    let output = Command::new(&nginx_exe)
        .args(["-c", &paths.nginx_conf, "-t"])
        .output()
        .map_err(|e| format!("Failed to test nginx config: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() || stderr.contains("syntax is ok") {
        Ok(format!("{}\n{}", stdout, stderr))
    } else {
        Err(format!("Nginx config error:\n{}\n{}", stdout, stderr))
    }
}
