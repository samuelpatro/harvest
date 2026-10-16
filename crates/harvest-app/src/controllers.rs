//! Wires the Slint `Backend` global to `harvest-core`: fills the data models and
//! implements every action callback. Long/blocking work goes through `bridge`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use slint::{ComponentHandle, Model, SharedString, VecModel, Weak};

use harvest_core as core;
use harvest_core::progress::{Progress, ProgressSink};
use harvest_core::types::{AvailableVersion, InstalledVersion};

use crate::bridge;
use crate::{AppWindow, Backend, ExtItem, PmItem, ServiceItem, SiteItem, TrayPopup, VersionItem};

thread_local! {
    static SITES_MODEL: Rc<VecModel<SiteItem>> = Rc::new(VecModel::default());
    static SITES_ALL: RefCell<Vec<SiteItem>> = const { RefCell::new(Vec::new()) };
    static SERVICES_MODEL: Rc<VecModel<ServiceItem>> = Rc::new(VecModel::default());
    static PHP_MODEL: Rc<VecModel<VersionItem>> = Rc::new(VecModel::default());
    static NODE_MODEL: Rc<VecModel<VersionItem>> = Rc::new(VecModel::default());
    static EXT_MODEL: Rc<VecModel<ExtItem>> = Rc::new(VecModel::default());
    static EXT_ALL: RefCell<Vec<ExtItem>> = const { RefCell::new(Vec::new()) };
    static EXT_FILTER: RefCell<String> = const { RefCell::new(String::new()) };
    static PM_MODEL: Rc<VecModel<PmItem>> = Rc::new(VecModel::default());
}

#[derive(Clone, Copy)]
enum ServiceAction {
    Start,
    Stop,
    Restart,
}

pub fn setup(app: &AppWindow) {
    let b = app.global::<Backend>();

    // Attach the shared models.
    SITES_MODEL.with(|m| b.set_sites(m.clone().into()));
    SERVICES_MODEL.with(|m| b.set_services(m.clone().into()));
    PHP_MODEL.with(|m| b.set_php_versions(m.clone().into()));
    NODE_MODEL.with(|m| b.set_node_versions(m.clone().into()));
    EXT_MODEL.with(|m| b.set_extensions(m.clone().into()));
    PM_MODEL.with(|m| b.set_package_managers(m.clone().into()));

    // Load current config into the editable fields.
    let cfg = core::config::get_config();
    b.set_harvest_home(cfg.harvest_home.clone().into());
    b.set_harvest_path(cfg.harvest_paths.first().cloned().unwrap_or_default().into());
    b.set_editor_command(cfg.editor_command.clone().into());
    b.set_terminal_command(cfg.terminal_command.clone().into());

    // First-run wizard state.
    b.set_show_wizard(core::config::is_first_run());
    let default_home = if cfg.harvest_home.is_empty() {
        core::types::get_default_harvest_home()
    } else {
        cfg.harvest_home.clone()
    };
    b.set_wizard_path(default_home.into());

    // ---- sites ----
    let weak = app.as_weak();
    b.on_refresh_sites(move || load_sites(weak.clone()));
    b.on_search_changed(|q| apply_site_filter(q.as_str()));
    b.on_open_browser(|domain, ssl| {
        let url = format!("{}://{}", if ssl { "https" } else { "http" }, domain);
        bridge::run(async move { core::sites::open_in_browser(url).await }, |_| {});
    });
    b.on_open_editor(|path| {
        let editor = core::config::get_config().editor_command;
        let p = path.to_string();
        bridge::run(async move { core::sites::open_in_editor(p, editor).await }, |_| {});
    });
    b.on_open_terminal(|path| {
        let terminal = core::config::get_config().terminal_command;
        let p = path.to_string();
        bridge::run(async move { core::sites::open_in_terminal(p, terminal).await }, |_| {});
    });
    let weak = app.as_weak();
    b.on_toggle_link(move |name, path, linked| {
        let weak = weak.clone();
        let (n, p) = (name.to_string(), path.to_string());
        bridge::run(
            async move {
                if linked {
                    core::vhosts::unlink_site(n).await
                } else {
                    core::vhosts::link_site(n, p).await
                }
            },
            move |res| {
                if let (Err(e), Some(app)) = (&res, weak.upgrade()) {
                    app.global::<Backend>().set_status_message(e.as_str().into());
                }
                load_sites(weak.clone());
            },
        );
    });

    // ---- services ----
    let weak = app.as_weak();
    b.on_refresh_services(move || load_services(weak.clone(), false));
    let weak = app.as_weak();
    b.on_poll_services(move || load_services(weak.clone(), true));
    let weak = app.as_weak();
    b.on_start_service(move |id| service_action(weak.clone(), id.to_string(), ServiceAction::Start));
    let weak = app.as_weak();
    b.on_stop_service(move |id| service_action(weak.clone(), id.to_string(), ServiceAction::Stop));
    let weak = app.as_weak();
    b.on_restart_service(move |id| service_action(weak.clone(), id.to_string(), ServiceAction::Restart));

    // ---- php ----
    let weak = app.as_weak();
    b.on_refresh_php(move |force| load_php(weak.clone(), force));
    let weak = app.as_weak();
    b.on_install_php(move |id| install_version(weak.clone(), false, id.to_string()));
    let weak = app.as_weak();
    b.on_uninstall_php(move |label| {
        let weak = weak.clone();
        let v = label.to_string();
        bridge::run(async move { core::php::uninstall_php_version(v).await }, move |res| {
            report_err(&weak, res);
            load_php(weak.clone(), false);
        });
    });
    let weak = app.as_weak();
    b.on_set_default_php(move |path| {
        let weak = weak.clone();
        let p = path.to_string();
        bridge::run(async move { swap_default(p) }, move |_| load_php(weak.clone(), false));
    });

    // ---- php extensions ----
    let weak = app.as_weak();
    b.on_load_extensions(move |path, label| {
        if let Some(app) = weak.upgrade() {
            let g = app.global::<Backend>();
            g.set_ext_version_path(path.clone());
            g.set_ext_version_label(label);
            g.set_ext_filter(SharedString::default());
        }
        EXT_FILTER.with(|c| c.borrow_mut().clear());
        load_extensions(path.to_string());
    });
    b.on_ext_filter_changed(|q| apply_ext_filter(q.as_str()));
    b.on_toggle_extension(|path, name, enable| {
        let (p, n) = (path.to_string(), name.to_string());
        bridge::run(
            async move { core::php::toggle_php_extension(p.clone(), n, enable).map(|_| p) },
            move |res| {
                if let Ok(p) = res {
                    load_extensions(p);
                }
            },
        );
    });
    let weak = app.as_weak();
    b.on_close_extensions(move || {
        if let Some(app) = weak.upgrade() {
            let g = app.global::<Backend>();
            g.set_ext_version_path(SharedString::default());
            g.set_ext_version_label(SharedString::default());
        }
        EXT_ALL.with(|c| c.borrow_mut().clear());
        EXT_MODEL.with(|m| m.set_vec(Vec::new()));
    });

    // ---- node ----
    let weak = app.as_weak();
    b.on_refresh_node(move |force| load_node(weak.clone(), force));
    let weak = app.as_weak();
    b.on_install_node(move |id| install_version(weak.clone(), true, id.to_string()));
    let weak = app.as_weak();
    b.on_uninstall_node(move |label| {
        let weak = weak.clone();
        let v = label.to_string();
        bridge::run(async move { core::node::uninstall_node_version(v).await }, move |res| {
            report_err(&weak, res);
            load_node(weak.clone(), false);
        });
    });
    let weak = app.as_weak();
    b.on_set_default_node(move |path| {
        let weak = weak.clone();
        let p = path.to_string();
        bridge::run(async move { swap_default(p) }, move |_| load_node(weak.clone(), false));
    });

    // ---- settings ----
    let weak = app.as_weak();
    b.on_browse_harvest_path(move || {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            if let Some(app) = weak.upgrade() {
                app.global::<Backend>()
                    .set_harvest_path(folder.to_string_lossy().to_string().into());
            }
        }
    });
    let weak = app.as_weak();
    b.on_save_settings(move || {
        let Some(app) = weak.upgrade() else { return };
        let g = app.global::<Backend>();
        let mut cfg = core::config::get_config();
        cfg.harvest_paths = vec![g.get_harvest_path().to_string()];
        cfg.editor_command = g.get_editor_command().to_string();
        cfg.terminal_command = g.get_terminal_command().to_string();
        let weak = weak.clone();
        bridge::run(async move { core::config::save_config(cfg) }, move |res| {
            if let Some(app) = weak.upgrade() {
                let g = app.global::<Backend>();
                match res {
                    Ok(()) => g.set_status_message("Settings saved".into()),
                    Err(e) => g.set_status_message(format!("Save failed: {e}").into()),
                }
            }
            load_sites(weak.clone());
        });
    });

    // ---- first-run wizard ----
    let weak = app.as_weak();
    b.on_wizard_browse(move || {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            if let Some(app) = weak.upgrade() {
                app.global::<Backend>()
                    .set_wizard_path(folder.to_string_lossy().to_string().into());
            }
        }
    });
    let weak = app.as_weak();
    b.on_wizard_finish(move || {
        let Some(app) = weak.upgrade() else { return };
        let g = app.global::<Backend>();
        let path = g.get_wizard_path().to_string();
        if path.trim().is_empty() {
            g.set_wizard_error("Please choose a folder.".into());
            return;
        }
        g.set_wizard_error(SharedString::default());
        let weak = weak.clone();
        bridge::run(
            async move {
                core::config::validate_harvest_path(path.clone())
                    .and_then(|_| core::config::initialize_harvest_home(path))
            },
            move |res| {
                let Some(app) = weak.upgrade() else { return };
                let g = app.global::<Backend>();
                match res {
                    Ok(()) => {
                        g.set_show_wizard(false);
                        let cfg = core::config::get_config();
                        g.set_harvest_home(cfg.harvest_home.clone().into());
                        g.set_harvest_path(cfg.harvest_paths.first().cloned().unwrap_or_default().into());
                        load_sites(weak.clone());
                        load_services(weak.clone(), false);
                        load_php(weak.clone(), false);
                        load_node(weak.clone(), false);
                    }
                    Err(e) => g.set_wizard_error(e.into()),
                }
            },
        );
    });

    // Initial data load.
    let weak = app.as_weak();
    load_sites(weak.clone());
    load_services(weak.clone(), false);
    load_php(weak.clone(), false);
    load_node(weak, false);
}

/// Share the live services model with the tray popup (a separate window with its
/// own Backend global) and wire the popup's quick actions.
pub fn wire_tray(app: &AppWindow, popup: &TrayPopup) {
    SERVICES_MODEL.with(|m| popup.global::<Backend>().set_services(m.clone().into()));

    let appw = app.as_weak();
    let popupw = popup.as_weak();
    popup.on_open_services(move || {
        if let Some(a) = appw.upgrade() {
            a.set_current_view(4);
            let _ = a.show();
        }
        if let Some(p) = popupw.upgrade() {
            let _ = p.hide();
        }
    });

    let appw = app.as_weak();
    let popupw = popup.as_weak();
    popup.on_open_sites(move || {
        if let Some(a) = appw.upgrade() {
            a.set_current_view(1);
            let _ = a.show();
        }
        if let Some(p) = popupw.upgrade() {
            let _ = p.hide();
        }
    });

    let appw = app.as_weak();
    popup.on_stop_all(move || {
        let ids: Vec<String> = SERVICES_MODEL.with(|m| {
            (0..m.row_count())
                .filter_map(|i| m.row_data(i))
                .filter(|r| r.status.as_str() == "running")
                .map(|r| r.id.to_string())
                .collect()
        });
        for id in ids {
            service_action(appw.clone(), id, ServiceAction::Stop);
        }
    });
}

// ---------------- sites ----------------

fn load_sites(weak: Weak<AppWindow>) {
    bridge::run(
        async move {
            let cfg = core::config::get_config();
            let sites = core::sites::scan_sites(cfg.harvest_paths.clone()).unwrap_or_default();
            let names: Vec<String> = sites.iter().map(|s| s.name.clone()).collect();
            let statuses = core::vhosts::get_all_vhost_statuses(names);
            (sites, statuses)
        },
        move |(sites, statuses)| {
            let items = build_site_items(sites, statuses);
            SITES_ALL.with(|c| *c.borrow_mut() = items);
            let q = weak
                .upgrade()
                .map(|a| a.global::<Backend>().get_search_query().to_string())
                .unwrap_or_default();
            apply_site_filter(&q);
        },
    );
}

fn apply_site_filter(query: &str) {
    let q = query.to_lowercase();
    SITES_ALL.with(|c| {
        let filtered: Vec<SiteItem> = c
            .borrow()
            .iter()
            .filter(|s| {
                q.is_empty()
                    || s.name.as_str().to_lowercase().contains(&q)
                    || s.domain.as_str().to_lowercase().contains(&q)
            })
            .cloned()
            .collect();
        SITES_MODEL.with(|m| m.set_vec(filtered));
    });
}

fn build_site_items(sites: Vec<core::types::Site>, statuses: Vec<core::vhosts::VhostStatus>) -> Vec<SiteItem> {
    let linked: HashMap<String, bool> = statuses.into_iter().map(|s| (s.site_name, s.is_linked)).collect();
    sites
        .into_iter()
        .map(|s| {
            let stack = s.stack.iter().map(|x| x.name.clone()).collect::<Vec<_>>().join(", ");
            SiteItem {
                linked: *linked.get(&s.name).unwrap_or(&false),
                linking: false,
                name: s.name.into(),
                domain: s.domain.into(),
                path: s.path.into(),
                php_version: s.php_version.into(),
                stack: stack.into(),
                ssl: s.ssl,
            }
        })
        .collect()
}

// ---------------- services ----------------

fn load_services(weak: Weak<AppWindow>, quiet: bool) {
    if !quiet {
        if let Some(app) = weak.upgrade() {
            app.global::<Backend>().set_services_loading(true);
        }
    }
    bridge::run(
        async move { core::services::get_services() },
        move |list| {
            let items: Vec<ServiceItem> = list
                .into_iter()
                .map(|s| ServiceItem {
                    busy: false,
                    id: s.id.into(),
                    name: s.name.into(),
                    description: s.description.into(),
                    status: s.status.into(),
                    port: s.port as i32,
                    version: s.version.unwrap_or_default().into(),
                })
                .collect();
            SERVICES_MODEL.with(|m| m.set_vec(items));
            if let Some(app) = weak.upgrade() {
                app.global::<Backend>().set_services_loading(false);
            }
        },
    );
}

fn service_action(weak: Weak<AppWindow>, id: String, action: ServiceAction) {
    set_service_busy(&id, true);
    bridge::run(
        async move {
            match action {
                ServiceAction::Start => core::services::start_service(id).await,
                ServiceAction::Stop => core::services::stop_service(id).await,
                ServiceAction::Restart => core::services::restart_service(id).await,
            }
        },
        move |res| {
            report_err(&weak, res);
            load_services(weak.clone(), false);
        },
    );
}

fn set_service_busy(id: &str, busy: bool) {
    SERVICES_MODEL.with(|m| {
        for i in 0..m.row_count() {
            if let Some(mut row) = m.row_data(i) {
                if row.id.as_str() == id {
                    row.busy = busy;
                    m.set_row_data(i, row);
                    break;
                }
            }
        }
    });
}

// ---------------- versions (php / node) ----------------

fn load_php(weak: Weak<AppWindow>, force: bool) {
    if let Some(app) = weak.upgrade() {
        app.global::<Backend>().set_php_loading(true);
    }
    bridge::run(
        async move {
            let installed = core::php::get_installed_php_versions();
            let available = core::php::get_available_php_versions(Some(force)).await.unwrap_or_default();
            let default = core::config::get_default_php_path();
            (installed, available, default)
        },
        move |(installed, available, default)| {
            let items = build_version_items(installed, available, default, false);
            PHP_MODEL.with(|m| m.set_vec(items));
            if let Some(app) = weak.upgrade() {
                app.global::<Backend>().set_php_loading(false);
            }
        },
    );
}

fn load_node(weak: Weak<AppWindow>, force: bool) {
    if let Some(app) = weak.upgrade() {
        app.global::<Backend>().set_node_loading(true);
    }
    bridge::run(
        async move {
            let installed = core::node::get_installed_node_versions();
            let available = core::node::get_available_node_versions(Some(force)).await.unwrap_or_default();
            let default = core::config::get_default_node_path();
            let pms = core::node::get_package_managers();
            (installed, available, default, pms)
        },
        move |(installed, available, default, pms)| {
            let items = build_version_items(installed, available, default, true);
            NODE_MODEL.with(|m| m.set_vec(items));
            let pm_items: Vec<PmItem> = pms
                .into_iter()
                .map(|p| PmItem {
                    name: p.name.into(),
                    version: p.version.unwrap_or_default().into(),
                    installed: p.installed,
                    description: p.description.into(),
                })
                .collect();
            PM_MODEL.with(|m| m.set_vec(pm_items));
            if let Some(app) = weak.upgrade() {
                app.global::<Backend>().set_node_loading(false);
            }
        },
    );
}

/// Map core version groups to Slint rows (grouping logic lives in harvest-core).
fn build_version_items(
    installed: Vec<InstalledVersion>,
    available: Vec<AvailableVersion>,
    default: Option<String>,
    node: bool,
) -> Vec<VersionItem> {
    core::versions::group_versions(&installed, &available, default.as_deref(), node)
        .into_iter()
        .map(|g| VersionItem {
            id: g.id.into(),
            label: g.label.into(),
            path: g.path.into(),
            installed: g.installed,
            is_default: g.is_default,
            lts: g.lts,
            update_available: g.update_available,
            installing: false,
            progress: 0,
            status: SharedString::default(),
        })
        .collect()
}

fn install_version(weak: Weak<AppWindow>, node: bool, id: String) {
    set_version_row(node, &id, true, 0, "starting");
    let sink = make_version_sink(weak.clone(), node);
    let id_done = id.clone();
    bridge::run(
        async move {
            if node {
                core::node::install_node_version(id, sink).await
            } else {
                core::php::install_php_version(id, sink).await
            }
        },
        move |res| {
            if let Err(e) = res {
                set_version_row(node, &id_done, false, 0, "");
                if let Some(app) = weak.upgrade() {
                    app.global::<Backend>().set_status_message(format!("Install failed: {e}").into());
                }
            }
        },
    );
}

fn make_version_sink(weak: Weak<AppWindow>, node: bool) -> ProgressSink {
    Arc::new(move |p: Progress| {
        let weak = weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let installing = p.status != "complete";
            set_version_row(node, &p.id, installing, p.progress, &p.status);
            if !installing {
                if node {
                    load_node(weak, false);
                } else {
                    load_php(weak, false);
                }
            }
        });
    })
}

fn set_version_row(node: bool, id: &str, installing: bool, progress: u32, status: &str) {
    let update = |m: &VecModel<VersionItem>| {
        for i in 0..m.row_count() {
            if let Some(mut row) = m.row_data(i) {
                if row.id.as_str() == id {
                    row.installing = installing;
                    row.progress = progress as i32;
                    row.status = status.into();
                    m.set_row_data(i, row);
                    break;
                }
            }
        }
    };
    if node {
        NODE_MODEL.with(|m| update(m));
    } else {
        PHP_MODEL.with(|m| update(m));
    }
}

/// Move the default (PATH) entry to `new_path`: drop the previous default of the
/// same family (PHP vs Node), then add the new one.
fn swap_default(new_path: String) {
    let current = if is_node_path(&new_path) {
        core::config::get_default_node_path()
    } else {
        core::config::get_default_php_path()
    };
    if let Some(c) = current {
        if core::config::normalize_path_for_comparison(&c)
            != core::config::normalize_path_for_comparison(&new_path)
        {
            let _ = core::config::remove_from_path(c);
        }
    }
    let _ = core::config::add_to_path(new_path);
}

fn is_node_path(p: &str) -> bool {
    core::config::normalize_path_for_comparison(p).contains("\\bin\\nodejs\\")
}

// ---------------- php extensions ----------------

fn load_extensions(path: String) {
    bridge::run(
        async move { core::php::get_php_extensions(path).unwrap_or_default() },
        |exts| {
            let items: Vec<ExtItem> = exts
                .into_iter()
                .map(|e| ExtItem {
                    name: e.name.into(),
                    enabled: e.enabled,
                    builtin: e.builtin,
                })
                .collect();
            EXT_ALL.with(|c| *c.borrow_mut() = items);
            let q = EXT_FILTER.with(|c| c.borrow().clone());
            apply_ext_filter(&q);
        },
    );
}

fn apply_ext_filter(query: &str) {
    EXT_FILTER.with(|c| *c.borrow_mut() = query.to_string());
    let q = query.to_lowercase();
    EXT_ALL.with(|c| {
        let filtered: Vec<ExtItem> = c
            .borrow()
            .iter()
            .filter(|e| q.is_empty() || e.name.as_str().to_lowercase().contains(&q))
            .cloned()
            .collect();
        EXT_MODEL.with(|m| m.set_vec(filtered));
    });
}

// ---------------- helpers ----------------

fn report_err(weak: &Weak<AppWindow>, res: Result<(), String>) {
    if let (Err(e), Some(app)) = (res, weak.upgrade()) {
        app.global::<Backend>().set_status_message(e.into());
    }
}
