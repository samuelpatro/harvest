//! Services view: start, stop and restart nginx/MySQL/Redis.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::Button;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{h_flex, Disableable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::store::{ServiceAction, ServiceRow, Store};
use crate::theme::{self, Type as _};
use crate::ui::{self, Toolbar};

pub struct ServicesView {
    store: Entity<Store>,
    _observe: Subscription,
}

impl ServicesView {
    pub fn new(store: Entity<Store>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&store, |_, _, cx| cx.notify());
        Self { store, _observe: observe }
    }

    pub fn toolbar(this: &Entity<Self>, cx: &App) -> Toolbar {
        let store = this.read(cx).store.clone();
        let s = store.read(cx);
        let running = s.services.iter().filter(|s| s.running()).count();
        let loading = s.services_loading;
        let stop_all = store.clone();
        Toolbar::new("Services")
            .subtitle(format!("{running} of {} running", s.services.len()))
            .actions(
                h_flex()
                    .gap(px(6.))
                    .child(
                        Button::new("stop-all")
                            .label("Stop All")
                            .disabled(running == 0)
                            .on_click(move |_, _, cx| stop_all.update(cx, |s, cx| s.stop_all_services(cx))),
                    )
                    .child(
                        ui::toolbar_button("refresh", IconName::RefreshCw, "Refresh")
                            .loading(loading)
                            .on_click(move |_, _, cx| store.update(cx, |s, cx| s.load_services(false, cx))),
                    ),
            )
    }
}

pub fn service_symbol(id: &str) -> (IconName, Hsla) {
    match id {
        "nginx" => (IconName::Server, theme::leaf()),
        "mysql" | "mariadb" => (IconName::Database, theme::sky()),
        "redis" => (IconName::Database, theme::rose()),
        _ => (IconName::Server, theme::stone()),
    }
}

/// On/off switch that starts or stops a service.
pub fn service_switch(store: &Entity<Store>, ix: usize, s: &ServiceRow) -> impl IntoElement {
    let (store, id) = (store.clone(), s.info.id.clone());
    Switch::new(("svc-switch", ix))
        .checked(s.running())
        .disabled(s.busy || !s.installed())
        .on_change(move |on, _, cx| {
            let action = if *on { ServiceAction::Start } else { ServiceAction::Stop };
            store.update(cx, |st, cx| st.service_action(&id, action, cx))
        })
}

/// One service row; shared with the dashboard.
pub fn service_row(store: &Entity<Store>, ix: usize, s: &ServiceRow) -> impl IntoElement {
    let (symbol, color) = service_symbol(&s.info.id);
    let version = s.info.version.clone().unwrap_or_default();
    let port = format!("Port {}", s.info.port);
    let detail = ui::joined(&[&version, &port]);
    let restart = {
        let (store, id) = (store.clone(), s.info.id.clone());
        move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
            store.update(cx, |st, cx| st.service_action(&id, ServiceAction::Restart, cx))
        }
    };

    ui::row(("svc", ix), ix == 0, 50.)
        .child(ui::symbol_tile(symbol, if s.installed() { color } else { theme::stone() }, 28.))
        .child(ui::row_text(div().headline().child(s.info.name.clone()), detail))
        .child(
            h_flex()
                .flex_shrink_0()
                .gap(px(10.))
                .child(div().w(px(100.)).child(ui::service_status(&s.info.status)))
                .when(s.running(), |el| {
                    el.child(
                        ui::icon_button(("restart", ix), IconName::RotateCw, "Restart")
                            .disabled(s.busy)
                            .on_click(restart),
                    )
                })
                .when(s.installed(), |el| el.child(service_switch(store, ix, s))),
        )
}

impl Render for ServicesView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let services = self.store.read(cx).services.clone();
        let content = if services.is_empty() {
            ui::unavailable(IconName::Server, "No Services", "Install nginx, MySQL or Redis into your Harvest home.")
                .into_any_element()
        } else {
            ui::section(
                "Local Services",
                ui::group().children(services.iter().enumerate().map(|(ix, s)| service_row(&self.store, ix, s))),
            )
            .child(ui::footnote("Switch a service on to start it. Harvest checks their status every few seconds."))
            .into_any_element()
        };
        ui::page_body("services", content)
    }
}
