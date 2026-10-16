//! Main window: unified toolbar, translucent sidebar, the active page, and the
//! overlays (extensions sheet, first-run wizard, notifications).

use gpui_kit::assets::IconName;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{h_flex, v_flex, TitleBar, WindowExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::dashboard::DashboardView;
use crate::extensions::ExtensionsDialog;
use crate::services::ServicesView;
use crate::settings::SettingsView;
use crate::sites::SitesView;
use crate::store::{Runtime, Store, StoreEvent};
use crate::theme::{self, Type as _, SIDEBAR_WIDTH, TOOLBAR_HEIGHT};
use crate::ui::{self, Toolbar};
use crate::versions::VersionsView;
use crate::wizard::SetupWizard;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Sites,
    Php,
    Node,
    Services,
    Settings,
}

/// Sidebar sections: an optional heading and its pages.
const SECTIONS: [(Option<&str>, &[Page]); 3] = [
    (None, &[Page::Dashboard]),
    (Some("Develop"), &[Page::Sites, Page::Php, Page::Node]),
    (Some("Environment"), &[Page::Services, Page::Settings]),
];

impl Page {
    fn symbol(self) -> (IconName, Hsla) {
        match self {
            Page::Dashboard => (IconName::LayoutDashboard, theme::sky()),
            Page::Sites => (IconName::Globe, theme::gold()),
            Page::Php => (IconName::CodeXml, theme::violet()),
            Page::Node => (IconName::Hexagon, theme::leaf()),
            Page::Services => (IconName::Server, theme::teal()),
            Page::Settings => (IconName::Settings, theme::stone()),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Page::Dashboard => "Dashboard",
            Page::Sites => "Sites",
            Page::Php => "PHP",
            Page::Node => "Node.js",
            Page::Services => "Services",
            Page::Settings => "Settings",
        }
    }
}

pub struct HarvestApp {
    page: Page,
    store: Entity<Store>,
    dashboard: Entity<DashboardView>,
    sites: Entity<SitesView>,
    php: Entity<VersionsView>,
    node: Entity<VersionsView>,
    services: Entity<ServicesView>,
    settings: Entity<SettingsView>,
    extensions: Entity<ExtensionsDialog>,
    wizard: Entity<SetupWizard>,
    _subscriptions: Vec<Subscription>,
}

impl HarvestApp {
    pub fn new(store: Entity<Store>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let subscriptions = vec![
            cx.observe(&store, |_, _, cx| cx.notify()),
            cx.subscribe_in(&store, window, |_, _, event: &StoreEvent, window, cx| {
                let note = match event {
                    StoreEvent::Info(msg) => Notification::success(msg.clone()),
                    StoreEvent::Error(msg) => Notification::error(msg.clone()),
                };
                window.push_notification(note, cx);
            }),
        ];
        Self {
            page: Page::Dashboard,
            dashboard: cx.new(|cx| DashboardView::new(store.clone(), cx)),
            sites: cx.new(|cx| SitesView::new(store.clone(), window, cx)),
            php: cx.new(|cx| VersionsView::new(store.clone(), Runtime::Php, cx)),
            node: cx.new(|cx| VersionsView::new(store.clone(), Runtime::Node, cx)),
            services: cx.new(|cx| ServicesView::new(store.clone(), cx)),
            settings: cx.new(|cx| SettingsView::new(store.clone(), window, cx)),
            extensions: cx.new(|cx| ExtensionsDialog::new(store.clone(), window, cx)),
            wizard: cx.new(|cx| SetupWizard::new(store.clone(), window, cx)),
            store,
            _subscriptions: subscriptions,
        }
    }

    pub fn show_page(&mut self, page: Page, cx: &mut Context<Self>) {
        self.page = page;
        cx.notify();
    }

    fn badge(&self, page: Page, cx: &App) -> Option<String> {
        let store = self.store.read(cx);
        match page {
            Page::Sites if !store.sites.is_empty() => Some(store.sites.len().to_string()),
            Page::Services => {
                let running = store.services.iter().filter(|s| s.running()).count();
                (running > 0).then(|| running.to_string())
            }
            _ => None,
        }
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let services = &self.store.read(cx).services;
        let running: Vec<&str> = services.iter().filter(|s| s.running()).map(|s| s.info.name.as_str()).collect();
        let installed = services.iter().filter(|s| s.installed()).count();
        let (status, detail, color) = match (installed, running.len()) {
            (0, _) => ("No services yet".to_string(), "Install nginx, MySQL or Redis".to_string(), theme::stone()),
            (_, 0) => ("Services stopped".to_string(), "Open Services to start them".to_string(), theme::stone()),
            (_, n) => (
                if n == 1 { "1 service running".to_string() } else { format!("{n} services running") },
                running.join(", "),
                theme::leaf(),
            ),
        };

        v_flex()
            .w(SIDEBAR_WIDTH)
            .h_full()
            .flex_shrink_0()
            .pt(TOOLBAR_HEIGHT)
            .bg(theme::sidebar())
            .border_r_1()
            .border_color(theme::separator())
            .child(
                h_flex()
                    .gap(px(10.))
                    .px(px(18.))
                    .pt(px(2.))
                    .pb(px(20.))
                    .child(ui::brand_mark(32.))
                    .child(
                        v_flex()
                            .child(div().title3().child("Harvest"))
                            .child(div().caption().text_color(theme::tertiary()).child("Local dev environment")),
                    ),
            )
            .child(v_flex().px(px(10.)).children(SECTIONS.iter().map(|(heading, pages)| {
                v_flex()
                    .gap(px(2.))
                    .pb(px(14.))
                    .when_some(*heading, |el, heading| {
                        el.child(
                            div()
                                .px(px(10.))
                                .pb(px(5.))
                                .caption()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::tertiary())
                                .child(heading),
                        )
                    })
                    .children(pages.iter().map(|&page| self.render_nav_item(page, cx)))
            })))
            .child(div().flex_1())
            .child(
                v_flex()
                    .id("service-status")
                    .m(px(10.))
                    .p(px(12.))
                    .gap(px(3.))
                    .rounded(px(10.))
                    .bg(theme::cream(0.035))
                    .border_1()
                    .border_color(theme::cream(0.06))
                    .hover(|s| s.bg(theme::cream(0.06)))
                    .on_click(cx.listener(|this, _, _, cx| this.show_page(Page::Services, cx)))
                    .child(
                        h_flex()
                            .gap(px(7.))
                            .child(ui::dot(color))
                            .child(div().callout().font_weight(FontWeight::MEDIUM).child(status)),
                    )
                    .child(div().pl(px(15.)).caption().text_color(theme::tertiary()).truncate().child(detail)),
            )
    }

    fn render_nav_item(&self, page: Page, cx: &mut Context<Self>) -> impl IntoElement {
        let active = page == self.page;
        let (symbol, _) = page.symbol();
        h_flex()
            .id(page.label())
            .relative()
            .h(px(32.))
            .px(px(10.))
            .gap(px(10.))
            .rounded(px(8.))
            .body()
            .text_color(if active { theme::label() } else { theme::cream(0.74) })
            .when(active, |el| {
                el.font_weight(FontWeight::MEDIUM)
                    .bg(linear_gradient(
                        90.,
                        linear_color_stop(theme::tint(theme::gold(), 0.17), 0.),
                        linear_color_stop(theme::tint(theme::gold(), 0.04), 1.),
                    ))
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .top(px(8.))
                            .w(px(3.))
                            .h(px(16.))
                            .rounded_full()
                            .bg(theme::brand_gradient(180.)),
                    )
            })
            .when(!active, |el| el.hover(|s| s.bg(theme::cream(0.04))))
            .on_click(cx.listener(move |this, _, _, cx| this.show_page(page, cx)))
            .child(ui::icon(symbol, 16., if active { theme::gold() } else { theme::secondary() }))
            .child(div().flex_1().child(page.label()))
            .when_some(self.badge(page, cx), |el, badge| {
                el.child(
                    div()
                        .px(px(7.))
                        .h(px(18.))
                        .flex()
                        .items_center()
                        .rounded_full()
                        .caption()
                        .font_weight(FontWeight::MEDIUM)
                        .bg(if active { theme::tint(theme::gold(), 0.2) } else { theme::cream(0.07) })
                        .text_color(if active { theme::gold() } else { theme::secondary() })
                        .child(badge),
                )
            })
    }

    fn toolbar(&self, cx: &App) -> Option<Toolbar> {
        if self.store.read(cx).first_run {
            return None;
        }
        Some(match self.page {
            Page::Dashboard => DashboardView::toolbar(&self.dashboard, cx),
            Page::Sites => SitesView::toolbar(&self.sites, cx),
            Page::Php => VersionsView::toolbar(&self.php, cx),
            Page::Node => VersionsView::toolbar(&self.node, cx),
            Page::Services => ServicesView::toolbar(&self.services, cx),
            Page::Settings => SettingsView::toolbar(&self.settings, cx),
        })
    }
}

/// The window toolbar, drawn into the (transparent) title bar so it drags the
/// window. Actions occlude it, so clicks on them never start a drag, and on
/// Windows they are not reported to the OS as caption area.
fn render_toolbar(toolbar: Option<Toolbar>) -> impl IntoElement {
    TitleBar::new()
        .h(TOOLBAR_HEIGHT)
        .pl_0()
        .bg(gpui_kit::transparent_black())
        .border_b_0()
        .child(
            h_flex()
                .size_full()
                .child(div().w(SIDEBAR_WIDTH + px(24.)).flex_shrink_0())
                .when_some(toolbar, |el, toolbar| {
                    el.child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(div().headline().truncate().child(toolbar.title))
                            .when_some(toolbar.subtitle, |el, subtitle| {
                                el.child(div().caption().text_color(theme::secondary()).truncate().child(subtitle))
                            }),
                    )
                    .when_some(toolbar.actions, |el, actions| {
                        el.child(h_flex().occlude().gap(px(6.)).pr(px(14.)).child(actions))
                    })
                }),
        )
}

impl Render for HarvestApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.store.read(cx);
        let (show_extensions, show_wizard) = (store.extension_panel.is_some(), store.first_run);
        let toolbar = self.toolbar(cx);

        let content = match self.page {
            Page::Dashboard => self.dashboard.clone().into_any_element(),
            Page::Sites => self.sites.clone().into_any_element(),
            Page::Php => self.php.clone().into_any_element(),
            Page::Node => self.node.clone().into_any_element(),
            Page::Services => self.services.clone().into_any_element(),
            Page::Settings => self.settings.clone().into_any_element(),
        };

        // The window background is the platform blur; the sidebar stays
        // translucent while the content column is opaque. The toolbar is
        // overlaid last so the window can always be dragged and closed.
        div()
            .relative()
            .size_full()
            .text_color(theme::label())
            .child(
                h_flex()
                    .size_full()
                    .items_stretch()
                    .child(self.render_sidebar(cx))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .pt(TOOLBAR_HEIGHT)
                            .bg(theme::content_background())
                            .child(content),
                    ),
            )
            .when(show_extensions, |el| el.child(self.extensions.clone()))
            .when(show_wizard, |el| el.child(self.wizard.clone()))
            .child(div().absolute().top_0().left_0().right_0().child(render_toolbar(toolbar)))
    }
}
