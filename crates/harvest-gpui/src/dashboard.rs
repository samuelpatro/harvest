//! Dashboard: environment overview, services and recent sites.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{h_flex, v_flex, Icon, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::services::service_row;
use crate::sites::site_row;
use crate::store::{Store, VersionRow};
use crate::theme::{self, Type as _};
use crate::ui::{self, Toolbar};

const RECENT_SITES: usize = 5;

pub struct DashboardView {
    store: Entity<Store>,
    _observe: Subscription,
}

impl DashboardView {
    pub fn new(store: Entity<Store>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&store, |_, _, cx| cx.notify());
        Self { store, _observe: observe }
    }

    pub fn toolbar(this: &Entity<Self>, cx: &App) -> Toolbar {
        let store = this.read(cx).store.read(cx);
        let running = store.services.iter().filter(|s| s.running()).count();
        let sites = match store.sites.len() {
            1 => "1 site".to_string(),
            n => format!("{n} sites"),
        };
        Toolbar::new("Dashboard").subtitle(ui::joined(&[&sites, &format!("{running} services running")]))
    }
}

/// Summary tile with a soft glow of its color.
fn widget(symbol: IconName, color: Hsla, label: &'static str, value: String, detail: String) -> impl IntoElement {
    v_flex()
        .flex_1()
        .min_w_0()
        .gap(px(12.))
        .p(px(16.))
        .bg(linear_gradient(
            160.,
            linear_color_stop(theme::mix(theme::card(), color, 0.11), 0.),
            linear_color_stop(theme::card(), 0.65),
        ))
        .border_1()
        .border_color(theme::card_border())
        .rounded(px(14.))
        .child(
            h_flex()
                .justify_between()
                .child(div().callout().font_weight(FontWeight::MEDIUM).text_color(theme::secondary()).child(label))
                .child(ui::symbol_tile(symbol, color, 26.)),
        )
        .child(
            v_flex()
                .gap(px(2.))
                .child(div().text_size(px(30.)).line_height(px(34.)).font_weight(FontWeight::BOLD).child(value))
                .child(div().caption().text_color(theme::secondary()).truncate().child(detail)),
        )
}

/// Headline banner: overall status and the one action that matters most.
fn hero(store: &Entity<Store>, cx: &App) -> impl IntoElement {
    let s = store.read(cx);
    let installed = s.services.iter().filter(|s| s.installed()).count();
    let running = s.services.iter().filter(|s| s.running()).count();
    let (headline, action) = match (installed, running) {
        (0, _) => ("Let's get you set up", None),
        (i, 0) => ("Ready when you are", Some((true, i))),
        (i, r) if r < i => ("Partly running", Some((true, i - r))),
        _ => ("All systems go", Some((false, running))),
    };
    let php = default_version(&s.php);
    let node = default_version(&s.node);
    let sites = match s.sites.len() {
        1 => "1 site on .test".to_string(),
        n => format!("{n} sites on .test"),
    };
    let summary = ui::joined(&[&sites, &format!("PHP {php}"), &format!("Node.js {node}")]);
    let store = store.clone();

    div()
        .relative()
        .overflow_hidden()
        .rounded(px(16.))
        .border_1()
        .border_color(theme::tint(theme::gold(), 0.16))
        .bg(linear_gradient(
            115.,
            linear_color_stop(theme::tint(theme::gold(), 0.17), 0.),
            linear_color_stop(theme::tint(theme::pumpkin(), 0.03), 1.),
        ))
        .child(
            div()
                .absolute()
                .right(px(-18.))
                .top(px(-26.))
                .child(ui::icon(IconName::Wheat, 170., theme::tint(theme::gold(), 0.08))),
        )
        .child(
            h_flex()
                .p(px(22.))
                .gap(px(16.))
                .child(
                    v_flex()
                        .flex_1()
                        .gap(px(4.))
                        .child(
                            div()
                                .caption()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::gold())
                                .child("HARVEST"),
                        )
                        .child(div().title2().child(headline))
                        .child(div().body().text_color(theme::secondary()).child(summary)),
                )
                .when_some(action, |el, (start, n)| {
                    let label = match (start, n) {
                        (true, 1) => "Start Service".to_string(),
                        (true, _) => format!("Start {n} Services"),
                        (false, _) => "Stop All".to_string(),
                    };
                    el.child(
                        Button::new("hero-action")
                            .when(start, |b| b.primary())
                            .large()
                            .icon(Icon::new(if start { IconName::Play } else { IconName::CircleStop }))
                            .label(label)
                            .on_click(move |_, _, cx| {
                                store.update(cx, |s, cx| {
                                    if start {
                                        s.start_all_services(cx)
                                    } else {
                                        s.stop_all_services(cx)
                                    }
                                })
                            }),
                    )
                }),
        )
}

fn default_version(rows: &[VersionRow]) -> String {
    rows.iter()
        .find(|v| v.group.installed && v.group.is_default)
        .map(|v| v.group.label.clone())
        .unwrap_or_else(|| "not set".into())
}

impl Render for DashboardView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.store.read(cx);
        let running = store.services.iter().filter(|s| s.running()).count();
        let installed_services = store.services.iter().filter(|s| s.installed()).count();
        let php_installed = store.php.iter().filter(|v| v.group.installed).count();
        let node_installed = store.node.iter().filter(|v| v.group.installed).count();
        let linked = store.sites.iter().filter(|s| s.linked).count();

        let widgets = h_flex()
            .gap(px(12.))
            .child(widget(
                IconName::Globe,
                theme::gold(),
                "Sites",
                store.sites.len().to_string(),
                format!("{linked} linked"),
            ))
            .child(widget(
                IconName::Server,
                theme::teal(),
                "Services",
                format!("{running}/{installed_services}"),
                "Running now".into(),
            ))
            .child(widget(
                IconName::CodeXml,
                theme::violet(),
                "PHP",
                php_installed.to_string(),
                format!("Default {}", default_version(&store.php)),
            ))
            .child(widget(
                IconName::Hexagon,
                theme::leaf(),
                "Node.js",
                node_installed.to_string(),
                format!("Default {}", default_version(&store.node)),
            ));

        let services = store.services.clone();
        let sites: Vec<_> = store.sites.iter().take(RECENT_SITES).cloned().collect();

        ui::page_body(
            "dashboard",
            v_flex()
                .gap(px(24.))
                .child(hero(&self.store, cx))
                .child(widgets)
                .when(!services.is_empty(), |el| {
                    el.child(ui::section(
                        "Services",
                        ui::group().children(services.iter().enumerate().map(|(ix, s)| service_row(&self.store, ix, s))),
                    ))
                })
                .when(!sites.is_empty(), |el| {
                    el.child(ui::section(
                        "Sites",
                        ui::group().children(sites.iter().enumerate().map(|(ix, s)| site_row(&self.store, ix, s))),
                    ))
                }),
        )
    }
}
