//! Installed/available version manager, shared by the PHP and Node.js pages.
//! PHP rows open the extensions sheet; the Node.js page adds package managers.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::Button;
use gpui_kit::component::progress::Progress;
use gpui_kit::component::{h_flex, Icon, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::store::{Runtime, Store, VersionRow};
use crate::theme::{self, Type as _};
use crate::ui::{self, Toolbar};

pub struct VersionsView {
    store: Entity<Store>,
    runtime: Runtime,
    _observe: Subscription,
}

fn symbol(rt: Runtime) -> (IconName, Hsla) {
    match rt {
        Runtime::Php => (IconName::CodeXml, theme::violet()),
        Runtime::Node => (IconName::Hexagon, theme::leaf()),
    }
}

impl VersionsView {
    pub fn new(store: Entity<Store>, runtime: Runtime, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&store, |_, _, cx| cx.notify());
        Self { store, runtime, _observe: observe }
    }

    pub fn toolbar(this: &Entity<Self>, cx: &App) -> Toolbar {
        let view = this.read(cx);
        let (rt, store) = (view.runtime, view.store.clone());
        let s = store.read(cx);
        let default = s
            .versions(rt)
            .iter()
            .find(|v| v.group.installed && v.group.is_default)
            .map(|v| format!("Default {}", v.group.label))
            .unwrap_or_else(|| "No default version".into());
        let loading = s.versions_loading(rt);
        Toolbar::new(match rt {
            Runtime::Php => "PHP",
            Runtime::Node => "Node.js",
        })
        .subtitle(default)
        .actions(
            ui::toolbar_button("refresh", IconName::RefreshCw, "Check for Updates")
                .loading(loading)
                .on_click(move |_, _, cx| store.update(cx, |s, cx| s.load_versions(rt, true, cx))),
        )
    }

    fn installed_row(&self, ix: usize, v: &VersionRow) -> impl IntoElement {
        let (rt, g) = (self.runtime, &v.group);
        let store = self.store.clone();
        let (symbol, color) = symbol(rt);
        let (label, path) = (g.label.clone(), g.path.clone());

        ui::row(("installed", ix), ix == 0, 50.)
            .child(ui::symbol_tile(symbol, color, 28.))
            .child(ui::row_text(
                h_flex()
                    .gap(px(6.))
                    .child(div().headline().child(g.label.clone()))
                    .when(g.is_default, |el| el.child(ui::capsule("Default", theme::gold())))
                    .when(g.lts, |el| el.child(ui::capsule("LTS", theme::leaf())))
                    .when(g.update_available, |el| el.child(ui::capsule("Update", theme::sky()))),
                ui::tilde(&g.path),
            ))
            .child(
                h_flex()
                    .flex_shrink_0()
                    .gap(px(2.))
                    .when(!g.is_default, |el| {
                        let (store, path) = (store.clone(), path.clone());
                        el.child(
                            Button::new(("default", ix))
                                .small()
                                .label("Make Default")
                                .on_click(move |_, _, cx| store.update(cx, |s, cx| s.set_default_version(rt, &path, cx))),
                        )
                        .child(div().w(px(4.)))
                    })
                    .when(rt == Runtime::Php, |el| {
                        let (store, label, path) = (store.clone(), label.clone(), path.clone());
                        el.child(
                            ui::icon_button(("ext", ix), IconName::Puzzle, "Extensions")
                                .on_click(move |_, _, cx| store.update(cx, |s, cx| s.open_extensions(&label, &path, cx))),
                        )
                    })
                    .child(
                        ui::icon_button(("uninstall", ix), IconName::Trash, "Uninstall")
                            .on_click(move |_, _, cx| store.update(cx, |s, cx| s.uninstall_version(rt, &label, cx))),
                    ),
            )
    }

    fn available_row(&self, ix: usize, v: &VersionRow) -> impl IntoElement {
        let (rt, g) = (self.runtime, &v.group);
        let store = self.store.clone();
        let id = g.id.clone();

        ui::row(("available", ix), ix == 0, 12.)
            .min_h(px(40.))
            .child(
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(6.))
                    .child(div().body().child(g.label.clone()))
                    .when(g.lts, |el| el.child(ui::capsule("LTS", theme::leaf()))),
            )
            .when(v.installing, |el| {
                el.child(
                    h_flex()
                        .w(px(240.))
                        .gap(px(10.))
                        .child(div().flex_1().child(Progress::new(("progress", ix)).value(v.progress as f32)))
                        .child(
                            div()
                                .w(px(80.))
                                .caption()
                                .text_color(theme::secondary())
                                .truncate()
                                .child(if v.status.is_empty() { format!("{}%", v.progress) } else { v.status.clone() }),
                        ),
                )
            })
            .when(!v.installing, |el| {
                el.child(
                    Button::new(("install", ix))
                        .small()
                        .icon(Icon::new(IconName::Download))
                        .label("Install")
                        .on_click(move |_, _, cx| store.update(cx, |s, cx| s.install_version(rt, &id, cx))),
                )
            })
    }

    fn package_managers(&self, cx: &App) -> impl IntoElement {
        let pms = self.store.read(cx).package_managers.clone();
        ui::section(
            "Package Managers",
            ui::group().children(pms.iter().enumerate().map(|(ix, pm)| {
                ui::row(("pm", ix), ix == 0, 46.)
                    .child(ui::symbol_tile(IconName::Package, if pm.installed { theme::gold() } else { theme::stone() }, 24.))
                    .child(ui::row_text(div().headline().child(pm.name.clone()), pm.description.clone()))
                    .child(match (&pm.version, pm.installed) {
                        (Some(v), true) => ui::capsule(v.clone(), theme::leaf()).into_any_element(),
                        _ => div().callout().text_color(theme::tertiary()).child("Not Installed").into_any_element(),
                    })
            })),
        )
    }
}

impl Render for VersionsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rt = self.runtime;
        let store = self.store.read(cx);
        let (installed, available): (Vec<VersionRow>, Vec<VersionRow>) =
            store.versions(rt).iter().cloned().partition(|v| v.group.installed);
        let loading = store.versions_loading(rt);
        let (symbol, _) = symbol(rt);

        let installed_group = if installed.is_empty() {
            ui::group().child(ui::unavailable(symbol, "No Versions Installed", "Install one from the list below."))
        } else {
            ui::group().children(installed.iter().enumerate().map(|(ix, v)| self.installed_row(ix, v)))
        };

        ui::page_body(
            match rt {
                Runtime::Php => "php",
                Runtime::Node => "node",
            },
            gpui_kit::component::v_flex()
                .gap(px(24.))
                .child(
                    ui::section("Installed", installed_group)
                        .child(ui::footnote("The default version is the one on your PATH.")),
                )
                .when(!available.is_empty(), |el| {
                    el.child(ui::section(
                        "Available",
                        ui::group().children(available.iter().enumerate().map(|(ix, v)| self.available_row(ix, v))),
                    ))
                })
                .when(available.is_empty() && loading, |el| {
                    el.child(div().px(px(10.)).callout().text_color(theme::secondary()).child("Checking for versions…"))
                })
                .when(rt == Runtime::Node, |el| el.child(self.package_managers(cx))),
        )
    }
}
