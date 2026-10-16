//! First-run onboarding: choose and initialize the Harvest home directory.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::{h_flex, v_flex, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use harvest_core as core;

use crate::store::Store;
use crate::theme::{self, Type as _};
use crate::ui;

pub struct SetupWizard {
    store: Entity<Store>,
    home: Entity<InputState>,
    _observe: Subscription,
}

impl SetupWizard {
    pub fn new(store: Entity<Store>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let cfg = core::config::get_config();
        let default_home = if cfg.harvest_home.is_empty() {
            core::types::get_default_harvest_home()
        } else {
            cfg.harvest_home
        };
        let home = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_value(default_home, window, cx);
            state
        });
        let observe = cx.observe(&store, |_, _, cx| cx.notify());
        Self { store, home, _observe: observe }
    }

    fn browse(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.home.clone();
        cx.spawn_in(window, async move |_, cx| {
            if let Some(folder) = ui::pick_folder(cx).await {
                let _ = input.update_in(cx, |state, window, cx| state.set_value(folder, window, cx));
            }
        })
        .detach();
    }

    fn finish(&mut self, cx: &mut Context<Self>) {
        let home = self.home.read(cx).value().to_string();
        self.store.update(cx, |s, cx| s.finish_wizard(home, cx));
    }
}

impl Render for SetupWizard {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let error = self.store.read(cx).wizard_error.clone();

        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme::content())
            .child(
                v_flex()
                    .w(px(440.))
                    .items_center()
                    .child(ui::brand_mark(84.))
                    .child(div().pt(px(24.)).large_title().child("Welcome to Harvest"))
                    .child(
                        div()
                            .pt(px(6.))
                            .body()
                            .text_center()
                            .text_color(theme::secondary())
                            .child("Choose a home folder. Harvest keeps PHP, Node.js, services and site configs there."),
                    )
                    .child(
                        v_flex().w_full().pt(px(28.)).gap(px(6.)).child(
                            ui::group().child(
                                ui::row("home", true, 12.)
                                    .child(div().body().flex_shrink_0().child("Harvest home"))
                                    .child(div().flex_1().child(Input::new(&self.home).small()))
                                    .child(
                                        Button::new("browse")
                                            .small()
                                            .label("Choose…")
                                            .on_click(cx.listener(|this, _, window, cx| this.browse(window, cx))),
                                    ),
                            ),
                        )
                        .when_some(error, |el, e| {
                            el.child(div().px(px(10.)).caption().text_color(theme::rose()).child(e))
                        }),
                    )
                    .child(
                        h_flex().pt(px(28.)).child(
                            Button::new("finish")
                                .primary()
                                .large()
                                .w(px(200.))
                                .label("Continue")
                                .on_click(cx.listener(|this, _, _, cx| this.finish(cx))),
                        ),
                    ),
            )
    }
}
