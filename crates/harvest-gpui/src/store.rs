//! App state and actions, shared by every view and the tray popup.
//!
//! The store owns all data loaded from harvest-core and runs every action
//! against it. Views observe it and re-render; user-facing results are emitted
//! as [`StoreEvent`]s, which the main window turns into notifications.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::channel::mpsc;
use futures::StreamExt as _;
use gpui_kit::*;
use harvest_core as core;
use harvest_core::progress::{Progress, ProgressSink};
use harvest_core::services::ServiceInfo;
use harvest_core::types::{PackageManager, PhpExtension};
use harvest_core::versions::VersionGroup;

use crate::bridge;

const SERVICE_POLL: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    Php,
    Node,
}

#[derive(Clone)]
pub struct SiteRow {
    pub name: String,
    pub domain: String,
    pub path: String,
    pub php_version: String,
    pub stack: String,
    pub ssl: bool,
    pub linked: bool,
    pub linking: bool,
}

impl SiteRow {
    pub fn url(&self) -> String {
        format!("{}://{}", if self.ssl { "https" } else { "http" }, self.domain)
    }
}

#[derive(Clone)]
pub struct ServiceRow {
    pub info: ServiceInfo,
    pub busy: bool,
}

impl ServiceRow {
    pub fn running(&self) -> bool {
        self.info.status == "running"
    }

    pub fn installed(&self) -> bool {
        self.info.status != "not_installed"
    }
}

#[derive(Clone)]
pub struct VersionRow {
    pub group: VersionGroup,
    pub installing: bool,
    pub progress: u32,
    pub status: String,
}

/// The PHP version whose extensions are open in the extensions dialog.
pub struct ExtensionPanel {
    pub label: String,
    pub path: String,
    pub extensions: Vec<PhpExtension>,
}

#[derive(Clone, Copy)]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
}

pub enum StoreEvent {
    Info(String),
    Error(String),
}

pub struct Store {
    pub sites: Vec<SiteRow>,
    pub services: Vec<ServiceRow>,
    pub services_loading: bool,
    pub php: Vec<VersionRow>,
    pub php_loading: bool,
    pub node: Vec<VersionRow>,
    pub node_loading: bool,
    pub package_managers: Vec<PackageManager>,
    pub extension_panel: Option<ExtensionPanel>,
    pub first_run: bool,
    pub wizard_error: Option<String>,
    _poll: Task<()>,
}

impl EventEmitter<StoreEvent> for Store {}

impl Store {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Keep service status live (main window and tray popup).
        let poll = cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(SERVICE_POLL).await;
            if this.update(cx, |this, cx| this.load_services(true, cx)).is_err() {
                break;
            }
        });

        let mut this = Self {
            sites: Vec::new(),
            services: Vec::new(),
            services_loading: false,
            php: Vec::new(),
            php_loading: false,
            node: Vec::new(),
            node_loading: false,
            package_managers: Vec::new(),
            extension_panel: None,
            first_run: core::config::is_first_run(),
            wizard_error: None,
            _poll: poll,
        };
        this.load_all(cx);
        this
    }

    fn load_all(&mut self, cx: &mut Context<Self>) {
        self.load_sites(cx);
        self.load_services(false, cx);
        self.load_versions(Runtime::Php, false, cx);
        self.load_versions(Runtime::Node, false, cx);
    }

    fn report(&mut self, res: Result<(), String>, cx: &mut Context<Self>) {
        if let Err(e) = res {
            cx.emit(StoreEvent::Error(e));
        }
    }

    // ---------------- sites ----------------

    pub fn load_sites(&mut self, cx: &mut Context<Self>) {
        let job = bridge::spawn(async {
            let cfg = core::config::get_config();
            let sites = core::sites::scan_sites(cfg.harvest_paths).unwrap_or_default();
            let names = sites.iter().map(|s| s.name.clone()).collect();
            let statuses = core::vhosts::get_all_vhost_statuses(names);
            (sites, statuses)
        });
        cx.spawn(async move |this, cx| {
            let (sites, statuses) = job.await;
            let linked: HashMap<String, bool> =
                statuses.into_iter().map(|s| (s.site_name, s.is_linked)).collect();
            let rows = sites
                .into_iter()
                .map(|s| SiteRow {
                    linked: linked.get(&s.name).copied().unwrap_or(false),
                    linking: false,
                    stack: s.stack.iter().map(|x| x.name.clone()).collect::<Vec<_>>().join(", "),
                    name: s.name,
                    domain: s.domain,
                    path: s.path,
                    php_version: s.php_version,
                    ssl: s.ssl,
                })
                .collect();
            let _ = this.update(cx, |this, cx| {
                this.sites = rows;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn toggle_link(&mut self, name: &str, cx: &mut Context<Self>) {
        let Some(site) = self.sites.iter_mut().find(|s| s.name == name) else {
            return;
        };
        site.linking = true;
        cx.notify();

        let (name, path, linked) = (site.name.clone(), site.path.clone(), site.linked);
        let job = bridge::spawn(async move {
            if linked {
                core::vhosts::unlink_site(name).await
            } else {
                core::vhosts::link_site(name, path).await
            }
        });
        cx.spawn(async move |this, cx| {
            let res = job.await.map(|_| ());
            let _ = this.update(cx, |this, cx| {
                this.report(res, cx);
                this.load_sites(cx);
            });
        })
        .detach();
    }

    pub fn open_in_browser(&self, url: String) {
        let _ = bridge::spawn(core::sites::open_in_browser(url));
    }

    pub fn open_in_editor(&self, path: String) {
        let _ = bridge::spawn(async move {
            let editor = core::config::get_config().editor_command;
            core::sites::open_in_editor(path, editor).await
        });
    }

    pub fn open_in_terminal(&self, path: String) {
        let _ = bridge::spawn(async move {
            let terminal = core::config::get_config().terminal_command;
            core::sites::open_in_terminal(path, terminal).await
        });
    }

    // ---------------- services ----------------

    /// `quiet` skips the loading indicator (used by the background poll).
    pub fn load_services(&mut self, quiet: bool, cx: &mut Context<Self>) {
        if !quiet {
            self.services_loading = true;
            cx.notify();
        }
        let job = bridge::spawn(async { core::services::get_services() });
        cx.spawn(async move |this, cx| {
            let list = job.await;
            let _ = this.update(cx, |this, cx| {
                // Keep the busy flag of rows with an action still in flight.
                let busy: Vec<String> =
                    this.services.iter().filter(|s| s.busy).map(|s| s.info.id.clone()).collect();
                this.services = list
                    .into_iter()
                    .map(|info| ServiceRow { busy: busy.contains(&info.id), info })
                    .collect();
                this.services_loading = false;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn service_action(&mut self, id: &str, action: ServiceAction, cx: &mut Context<Self>) {
        if let Some(row) = self.services.iter_mut().find(|s| s.info.id == id) {
            row.busy = true;
            cx.notify();
        }
        let id = id.to_string();
        let job = bridge::spawn({
            let id = id.clone();
            async move {
                match action {
                    ServiceAction::Start => core::services::start_service(id).await,
                    ServiceAction::Stop => core::services::stop_service(id).await,
                    ServiceAction::Restart => core::services::restart_service(id).await,
                }
            }
        });
        cx.spawn(async move |this, cx| {
            let res = job.await;
            let _ = this.update(cx, |this, cx| {
                if let Some(row) = this.services.iter_mut().find(|s| s.info.id == id) {
                    row.busy = false;
                }
                this.report(res, cx);
                this.load_services(false, cx);
            });
        })
        .detach();
    }

    pub fn start_all_services(&mut self, cx: &mut Context<Self>) {
        let stopped: Vec<String> = self
            .services
            .iter()
            .filter(|s| s.info.status == "stopped")
            .map(|s| s.info.id.clone())
            .collect();
        for id in stopped {
            self.service_action(&id, ServiceAction::Start, cx);
        }
    }

    pub fn stop_all_services(&mut self, cx: &mut Context<Self>) {
        let running: Vec<String> =
            self.services.iter().filter(|s| s.running()).map(|s| s.info.id.clone()).collect();
        for id in running {
            self.service_action(&id, ServiceAction::Stop, cx);
        }
    }

    // ---------------- versions (php / node) ----------------

    pub fn versions(&self, rt: Runtime) -> &[VersionRow] {
        match rt {
            Runtime::Php => &self.php,
            Runtime::Node => &self.node,
        }
    }

    pub fn versions_loading(&self, rt: Runtime) -> bool {
        match rt {
            Runtime::Php => self.php_loading,
            Runtime::Node => self.node_loading,
        }
    }

    fn versions_mut(&mut self, rt: Runtime) -> &mut Vec<VersionRow> {
        match rt {
            Runtime::Php => &mut self.php,
            Runtime::Node => &mut self.node,
        }
    }

    fn set_versions_loading(&mut self, rt: Runtime, loading: bool) {
        match rt {
            Runtime::Php => self.php_loading = loading,
            Runtime::Node => self.node_loading = loading,
        }
    }

    /// `force` bypasses harvest-core's 24h cache of available versions.
    pub fn load_versions(&mut self, rt: Runtime, force: bool, cx: &mut Context<Self>) {
        self.set_versions_loading(rt, true);
        cx.notify();
        let job = bridge::spawn(async move {
            let node = rt == Runtime::Node;
            let (installed, available, default, pms) = if node {
                (
                    core::node::get_installed_node_versions(),
                    core::node::get_available_node_versions(Some(force)).await.unwrap_or_default(),
                    core::config::get_default_node_path(),
                    Some(core::node::get_package_managers()),
                )
            } else {
                (
                    core::php::get_installed_php_versions(),
                    core::php::get_available_php_versions(Some(force)).await.unwrap_or_default(),
                    core::config::get_default_php_path(),
                    None,
                )
            };
            let groups = core::versions::group_versions(&installed, &available, default.as_deref(), node);
            (groups, pms)
        });
        cx.spawn(async move |this, cx| {
            let (groups, pms) = job.await;
            let _ = this.update(cx, |this, cx| {
                *this.versions_mut(rt) = groups
                    .into_iter()
                    .map(|group| VersionRow { group, installing: false, progress: 0, status: String::new() })
                    .collect();
                if let Some(pms) = pms {
                    this.package_managers = pms;
                }
                this.set_versions_loading(rt, false);
                cx.notify();
            });
        })
        .detach();
    }

    fn set_version_progress(&mut self, rt: Runtime, id: &str, installing: bool, progress: u32, status: &str) {
        if let Some(row) = self.versions_mut(rt).iter_mut().find(|r| r.group.id == id) {
            row.installing = installing;
            row.progress = progress;
            row.status = status.to_string();
        }
    }

    pub fn install_version(&mut self, rt: Runtime, id: &str, cx: &mut Context<Self>) {
        self.set_version_progress(rt, id, true, 0, "starting");
        cx.notify();

        // The sink runs on harvest-core's worker threads; forward to the UI thread.
        let (tx, mut rx) = mpsc::unbounded::<Progress>();
        let sink: ProgressSink = Arc::new(move |p| {
            let _ = tx.unbounded_send(p);
        });
        let id = id.to_string();
        let job = bridge::spawn({
            let id = id.clone();
            async move {
                match rt {
                    Runtime::Php => core::php::install_php_version(id, sink).await,
                    Runtime::Node => core::node::install_node_version(id, sink).await,
                }
            }
        });
        cx.spawn(async move |this, cx| {
            // Ends once the install drops its sink.
            while let Some(p) = rx.next().await {
                let _ = this.update(cx, |this, cx| {
                    this.set_version_progress(rt, &p.id, p.status != "complete", p.progress, &p.status);
                    cx.notify();
                });
            }
            let res = job.await;
            let _ = this.update(cx, |this, cx| {
                if let Err(e) = res {
                    this.set_version_progress(rt, &id, false, 0, "");
                    cx.emit(StoreEvent::Error(format!("Install failed: {e}")));
                }
                this.load_versions(rt, false, cx);
            });
        })
        .detach();
    }

    pub fn uninstall_version(&mut self, rt: Runtime, label: &str, cx: &mut Context<Self>) {
        let label = label.to_string();
        let job = bridge::spawn(async move {
            match rt {
                Runtime::Php => core::php::uninstall_php_version(label).await,
                Runtime::Node => core::node::uninstall_node_version(label).await,
            }
        });
        cx.spawn(async move |this, cx| {
            let res = job.await;
            let _ = this.update(cx, |this, cx| {
                this.report(res, cx);
                this.load_versions(rt, false, cx);
            });
        })
        .detach();
    }

    /// Put `path` on PATH as the default version, replacing the previous default
    /// of the same runtime.
    pub fn set_default_version(&mut self, rt: Runtime, path: &str, cx: &mut Context<Self>) {
        let path = path.to_string();
        let job = bridge::spawn(async move {
            let current = match rt {
                Runtime::Php => core::config::get_default_php_path(),
                Runtime::Node => core::config::get_default_node_path(),
            };
            if let Some(c) = current {
                if core::config::normalize_path_for_comparison(&c)
                    != core::config::normalize_path_for_comparison(&path)
                {
                    core::config::remove_from_path(c)?;
                }
            }
            core::config::add_to_path(path)
        });
        cx.spawn(async move |this, cx| {
            let res = job.await;
            let _ = this.update(cx, |this, cx| {
                this.report(res, cx);
                this.load_versions(rt, false, cx);
            });
        })
        .detach();
    }

    // ---------------- php extensions ----------------

    pub fn open_extensions(&mut self, label: &str, path: &str, cx: &mut Context<Self>) {
        self.extension_panel = Some(ExtensionPanel {
            label: label.to_string(),
            path: path.to_string(),
            extensions: Vec::new(),
        });
        cx.notify();
        self.load_extensions(cx);
    }

    pub fn close_extensions(&mut self, cx: &mut Context<Self>) {
        self.extension_panel = None;
        cx.notify();
    }

    fn load_extensions(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.extension_panel.as_ref().map(|p| p.path.clone()) else {
            return;
        };
        let job = bridge::spawn({
            let path = path.clone();
            async move { core::php::get_php_extensions(path) }
        });
        cx.spawn(async move |this, cx| {
            let res = job.await;
            let _ = this.update(cx, |this, cx| match res {
                Ok(exts) => {
                    // Ignore a result for a panel that was closed or switched meanwhile.
                    if let Some(panel) = this.extension_panel.as_mut().filter(|p| p.path == path) {
                        panel.extensions = exts;
                        cx.notify();
                    }
                }
                Err(e) => cx.emit(StoreEvent::Error(e)),
            });
        })
        .detach();
    }

    pub fn toggle_extension(&mut self, name: &str, enable: bool, cx: &mut Context<Self>) {
        let Some(panel) = self.extension_panel.as_mut() else {
            return;
        };
        // Optimistic: the switch is controlled, so flip it now.
        if let Some(ext) = panel.extensions.iter_mut().find(|e| e.name == name) {
            ext.enabled = enable;
        }
        cx.notify();

        let (path, name) = (panel.path.clone(), name.to_string());
        let job = bridge::spawn(async move { core::php::toggle_php_extension(path, name, enable) });
        cx.spawn(async move |this, cx| {
            let res = job.await;
            let _ = this.update(cx, |this, cx| {
                this.report(res, cx);
                this.load_extensions(cx);
            });
        })
        .detach();
    }

    // ---------------- settings / first run ----------------

    pub fn save_settings(&mut self, sites_folder: String, editor: String, terminal: String, cx: &mut Context<Self>) {
        let job = bridge::spawn(async move {
            let mut cfg = core::config::get_config();
            cfg.harvest_paths = vec![sites_folder];
            cfg.editor_command = editor;
            cfg.terminal_command = terminal;
            core::config::save_config(cfg)
        });
        cx.spawn(async move |this, cx| {
            let res = job.await;
            let _ = this.update(cx, |this, cx| {
                match res {
                    Ok(()) => cx.emit(StoreEvent::Info("Settings saved".into())),
                    Err(e) => cx.emit(StoreEvent::Error(format!("Save failed: {e}"))),
                }
                this.load_sites(cx);
            });
        })
        .detach();
    }

    pub fn finish_wizard(&mut self, home: String, cx: &mut Context<Self>) {
        if home.trim().is_empty() {
            self.wizard_error = Some("Please choose a folder.".into());
            cx.notify();
            return;
        }
        self.wizard_error = None;
        let job = bridge::spawn(async move {
            core::config::validate_harvest_path(home.clone())
                .and_then(|_| core::config::initialize_harvest_home(home))
        });
        cx.spawn(async move |this, cx| {
            let res = job.await;
            let _ = this.update(cx, |this, cx| {
                match res {
                    Ok(()) => {
                        this.first_run = false;
                        this.load_all(cx);
                    }
                    Err(e) => this.wizard_error = Some(e),
                }
                cx.notify();
            });
        })
        .detach();
    }
}
