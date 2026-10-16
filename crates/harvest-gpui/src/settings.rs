//! Settings: sites folder and external tool commands (System Settings form).

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::{h_flex, v_flex, Sizable as _};
use gpui_kit::*;
use harvest_core as core;

use crate::store::Store;
use crate::theme::{self, Type as _};
use crate::ui::{self, Toolbar};

pub struct SettingsView {
    store: Entity<Store>,
    sites_folder: Entity<InputState>,
    editor: Entity<InputState>,
    terminal: Entity<InputState>,
    harvest_home: String,
}

impl SettingsView {
    pub fn new(store: Entity<Store>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let cfg = core::config::get_config();
        let field = |value: String, placeholder: &'static str, window: &mut Window, cx: &mut Context<Self>| {
            cx.new(|cx| {
                let mut state = InputState::new(window, cx).placeholder(placeholder);
                state.set_value(value, window, cx);
                state
            })
        };
        Self {
            store,
            sites_folder: field(cfg.harvest_paths.first().cloned().unwrap_or_default(), "", window, cx),
            editor: field(cfg.editor_command, "code", window, cx),
            terminal: field(cfg.terminal_command, "wt -d", window, cx),
            harvest_home: cfg.harvest_home,
        }
    }

    pub fn toolbar(this: &Entity<Self>, _: &App) -> Toolbar {
        let this = this.clone();
        Toolbar::new("Settings").actions(
            Button::new("save")
                .primary()
                .label("Save")
                .on_click(move |_, _, cx| this.update(cx, |view, cx| view.save(cx))),
        )
    }

    fn browse(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.sites_folder.clone();
        cx.spawn_in(window, async move |_, cx| {
            if let Some(folder) = ui::pick_folder(cx).await {
                let _ = input.update_in(cx, |state, window, cx| state.set_value(folder, window, cx));
            }
        })
        .detach();
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        let value = |input: &Entity<InputState>, cx: &App| input.read(cx).value().to_string();
        let (folder, editor, terminal) = (value(&self.sites_folder, cx), value(&self.editor, cx), value(&self.terminal, cx));
        self.store.update(cx, |s, cx| s.save_settings(folder, editor, terminal, cx));
    }
}

/// Label on the left, control on the right.
fn setting(ix: usize, label: &'static str, control: impl IntoElement) -> impl IntoElement {
    ui::row(("setting", ix), ix == 0, 12.)
        .child(div().flex_1().body().child(label))
        .child(h_flex().w(px(360.)).flex_shrink_0().justify_end().child(control))
}

impl Render for SettingsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        ui::page_body(
            "settings",
            v_flex()
                .gap(px(24.))
                .child(
                    ui::section(
                        "Sites",
                        ui::group().child(setting(
                            0,
                            "Sites folder",
                            h_flex()
                                .w_full()
                                .gap(px(6.))
                                .child(div().flex_1().child(Input::new(&self.sites_folder).small()))
                                .child(
                                    Button::new("browse")
                                        .small()
                                        .label("Choose…")
                                        .on_click(cx.listener(|this, _, window, cx| this.browse(window, cx))),
                                ),
                        )),
                    )
                    .child(ui::footnote("Every project in this folder is served at its-name.test.")),
                )
                .child(
                    ui::section(
                        "Tools",
                        ui::group()
                            .child(setting(0, "Editor", div().w_full().child(Input::new(&self.editor).small())))
                            .child(setting(1, "Terminal", div().w_full().child(Input::new(&self.terminal).small()))),
                    )
                    .child(ui::footnote("Commands used by Open in Editor and Open in Terminal.")),
                )
                .child(
                    ui::section(
                        "Harvest Home",
                        ui::group().child(setting(
                            0,
                            "Location",
                            div().callout().text_color(theme::secondary()).truncate().child(ui::tilde(&self.harvest_home)),
                        )),
                    )
                    .child(ui::footnote("PHP, Node.js, services and site configs are stored here.")),
                ),
        )
    }
}
