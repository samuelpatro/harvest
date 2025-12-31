pub mod commands;
pub mod types;

use commands::{
    config::{get_config, save_config},
    sites::{scan_sites, open_in_editor, open_in_terminal, open_in_browser},
    php::{
        get_available_php_versions, get_installed_php_versions,
        install_php_version, uninstall_php_version,
        get_php_extensions, toggle_php_extension,
    },
    node::{
        get_available_node_versions, get_installed_node_versions,
        install_node_version, uninstall_node_version,
        get_package_managers,
    },
    services::{get_services, start_service, stop_service, restart_service},
    vhosts::{get_vhost_status, get_all_vhost_statuses, link_site, unlink_site, check_nginx_config},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Config
            get_config,
            save_config,
            // Sites
            scan_sites,
            open_in_editor,
            open_in_terminal,
            open_in_browser,
            // PHP
            get_installed_php_versions,
            get_available_php_versions,
            install_php_version,
            uninstall_php_version,
            get_php_extensions,
            toggle_php_extension,
            // Node
            get_installed_node_versions,
            get_available_node_versions,
            install_node_version,
            uninstall_node_version,
            get_package_managers,
            // Services
            get_services,
            start_service,
            stop_service,
            restart_service,
            // Vhosts
            get_vhost_status,
            get_all_vhost_statuses,
            link_site,
            unlink_site,
            check_nginx_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
