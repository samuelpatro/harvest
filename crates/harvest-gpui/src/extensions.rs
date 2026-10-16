//! Sheet listing and toggling the extensions of one PHP version.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{h_flex, v_flex, Disableable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::store::Store;
use crate::theme::{self, Type as _};
use crate::ui;

pub struct ExtensionsDialog {
    store: Entity<Store>,
    filter: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl ExtensionsDialog {
    pub fn new(store: Entity<Store>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let filter = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));
        let subscriptions = vec![
            cx.observe(&store, |_, _, cx| cx.notify()),
            cx.subscribe(&filter, |_, _, ev: &InputEvent, cx| {
                if matches!(ev, InputEvent::Change) {
                    cx.notify();
                }
            }),
        ];
        Self { store, filter, _subscriptions: subscriptions }
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.filter.update(cx, |f, cx| f.set_value("", window, cx));
        self.store.update(cx, |s, cx| s.close_extensions(cx));
    }
}

impl Render for ExtensionsDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(panel) = self.store.read(cx).extension_panel.as_ref() else {
            return div().into_any_element();
        };
        let q = self.filter.read(cx).value().to_lowercase();
        let exts: Vec<_> = panel
            .extensions
            .iter()
            .filter(|e| q.is_empty() || e.name.to_lowercase().contains(&q))
            .cloned()
            .collect();
        let title = format!("PHP {} Extensions", panel.label);
        let enabled = panel.extensions.iter().filter(|e| e.enabled).count();
        let summary = format!("{enabled} of {} enabled", panel.extensions.len());
        let height = (window.viewport_size().height * 0.75).min(px(620.));

        div()
            .id("ext-backdrop")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0., 0., 0., 0.4))
            // Backdrop click dismisses; clicks inside the sheet stop here.
            .on_click(cx.listener(|this, _, window, cx| this.close(window, cx)))
            .child(
                v_flex()
                    .id("ext-sheet")
                    .w(px(500.))
                    .h(height)
                    .p(px(20.))
                    .gap(px(14.))
                    .bg(theme::sheet())
                    .border_1()
                    .border_color(theme::cream(0.1))
                    .rounded(px(12.))
                    .shadow_2xl()
                    .on_click(|_, _, cx| cx.stop_propagation())
                    .child(
                        h_flex()
                            .gap(px(12.))
                            .child(ui::symbol_tile(IconName::Puzzle, theme::violet(), 32.))
                            .child(
                                v_flex()
                                    .flex_1()
                                    .child(div().headline().child(title))
                                    .child(div().caption().text_color(theme::secondary()).child(summary)),
                            ),
                    )
                    .child(
                        Input::new(&self.filter)
                            .small()
                            .prefix(ui::icon(IconName::Search, 12., theme::secondary()))
                            .cleanable(true),
                    )
                    .child(
                        ui::group()
                            .id("ext-list")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .when(exts.is_empty(), |el| {
                                el.child(ui::unavailable(IconName::Puzzle, "No Extensions", "Nothing matches your search."))
                            })
                            .children(exts.into_iter().enumerate().map(|(ix, e)| {
                                let store = self.store.clone();
                                let name = e.name.clone();
                                ui::row(("ext", ix), ix == 0, 12.)
                                    .min_h(px(36.))
                                    .py(px(6.))
                                    .child(div().flex_1().body().child(e.name.clone()))
                                    .when(e.builtin, |el| el.child(ui::capsule("Built In", theme::stone())))
                                    .child(
                                        Switch::new(("ext-switch", ix))
                                            .small()
                                            .checked(e.enabled)
                                            .disabled(e.builtin)
                                            .on_change(move |on, _, cx| {
                                                store.update(cx, |s, cx| s.toggle_extension(&name, *on, cx))
                                            }),
                                    )
                            })),
                    )
                    .child(
                        h_flex().justify_end().child(
                            Button::new("done")
                                .primary()
                                .label("Done")
                                .on_click(cx.listener(|this, _, window, cx| this.close(window, cx))),
                        ),
                    ),
            )
            .into_any_element()
    }
}
