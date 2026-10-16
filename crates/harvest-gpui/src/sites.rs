//! Sites view: scanned projects with open/link actions and a search filter.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::Button;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{h_flex, Disableable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::store::{SiteRow, Store};
use crate::theme::{self, Type as _};
use crate::ui::{self, Toolbar};

pub struct SitesView {
    store: Entity<Store>,
    search: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl SitesView {
    pub fn new(store: Entity<Store>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));
        let subscriptions = vec![
            cx.observe(&store, |_, _, cx| cx.notify()),
            cx.subscribe(&search, |_, _, ev: &InputEvent, cx| {
                if matches!(ev, InputEvent::Change) {
                    cx.notify();
                }
            }),
        ];
        Self { store, search, _subscriptions: subscriptions }
    }

    pub fn toolbar(this: &Entity<Self>, cx: &App) -> Toolbar {
        let view = this.read(cx);
        let store = view.store.clone();
        let count = store.read(cx).sites.len();
        Toolbar::new("Sites")
            .subtitle(match count {
                1 => "1 site".to_string(),
                n => format!("{n} sites"),
            })
            .actions(
                h_flex()
                    .gap(px(6.))
                    .child(
                        div().w(px(200.)).child(
                            Input::new(&view.search)
                                .prefix(ui::icon(IconName::Search, 13., theme::secondary()))
                                .cleanable(true),
                        ),
                    )
                    .child(
                        ui::toolbar_button("rescan", IconName::RefreshCw, "Rescan sites folder")
                            .on_click(move |_, _, cx| store.update(cx, |s, cx| s.load_sites(cx))),
                    ),
            )
    }
}

/// One site row; shared with the dashboard.
pub fn site_row(store: &Entity<Store>, ix: usize, site: &SiteRow) -> impl IntoElement {
    let (url, path, name) = (site.url(), site.path.clone(), site.name.clone());
    let php = format!("PHP {}", site.php_version);
    let (link_icon, link_tip) = if site.linked {
        (IconName::Unlink, "Unlink from nginx")
    } else {
        (IconName::Link, "Link to nginx")
    };

    ui::row(("site", ix), ix == 0, 54.)
        .child(ui::monogram(&site.name, 32.))
        .child(ui::row_text(
            h_flex()
                .gap(px(6.))
                .child(div().headline().truncate().child(site.name.clone()))
                .when(site.linked, |el| el.child(ui::capsule("Linked", theme::leaf()))),
            ui::joined(&[&site.domain, &php, &site.stack]),
        ))
        .child(
            h_flex()
                .flex_shrink_0()
                .gap(px(2.))
                .child(ui::icon_button(("code", ix), IconName::CodeXml, "Open in Editor").on_click({
                    let (store, path) = (store.clone(), path.clone());
                    move |_, _, cx| store.read(cx).open_in_editor(path.clone())
                }))
                .child(ui::icon_button(("term", ix), IconName::Terminal, "Open in Terminal").on_click({
                    let store = store.clone();
                    move |_, _, cx| store.read(cx).open_in_terminal(path.clone())
                }))
                .child(
                    ui::icon_button(("link", ix), link_icon, link_tip)
                        .loading(site.linking)
                        .disabled(site.linking)
                        .on_click({
                            let store = store.clone();
                            move |_, _, cx| store.update(cx, |s, cx| s.toggle_link(&name, cx))
                        }),
                )
                .child(div().w(px(6.)))
                .child(Button::new(("open", ix)).small().label("Open").on_click({
                    let store = store.clone();
                    move |_, _, cx| store.read(cx).open_in_browser(url.clone())
                })),
        )
}

impl Render for SitesView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let q = self.search.read(cx).value().to_lowercase();
        let all = &self.store.read(cx).sites;
        let total = all.len();
        let sites: Vec<SiteRow> = all
            .iter()
            .filter(|s| q.is_empty() || s.name.to_lowercase().contains(&q) || s.domain.to_lowercase().contains(&q))
            .cloned()
            .collect();

        let content = if sites.is_empty() {
            if total == 0 {
                ui::unavailable(IconName::Globe, "No Sites", "Add projects to your sites folder, then rescan.").into_any_element()
            } else {
                ui::unavailable(IconName::Search, "No Results", "Try a different name or domain.").into_any_element()
            }
        } else {
            ui::group()
                .children(sites.iter().enumerate().map(|(ix, s)| site_row(&self.store, ix, s)))
                .into_any_element()
        };

        ui::page_body("sites", content)
    }
}
