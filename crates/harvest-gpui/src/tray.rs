//! System tray: a `tray-icon` icon whose left click toggles a small popup
//! window next to it. The popup closes when it loses focus; a short guard after
//! that stops the same tray click (which caused the blur) from reopening it.
//!
//! The icon and tooltip follow service status: a monochrome template glyph in
//! the macOS menu bar, the colored brand mark in the Windows tray, each with a
//! status dot while services run.

use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use tray_icon::{Icon, MouseButton as TrayButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::app::{HarvestApp, Page};
use crate::assets;
use crate::services::{service_switch, service_symbol};
use crate::store::Store;
use crate::theme::{self, Type as _};
use crate::ui;

const POPUP_W: f32 = 300.0;
/// Popup height without service rows, and per service row.
const POPUP_BASE_H: f32 = 224.0;
const SERVICE_ROW_H: f32 = 34.0;
const POPUP_MAX_H: f32 = 520.0;

thread_local! {
    static TRAY: RefCell<Option<TrayIcon>> = const { RefCell::new(None) };
    /// (running, installed) services last shown by the tray.
    static SHOWN_STATUS: Cell<Option<(usize, usize)>> = const { Cell::new(None) };
    static POPUP: RefCell<Option<AnyWindowHandle>> = const { RefCell::new(None) };
    static LAST_HIDE: Cell<Option<Instant>> = const { Cell::new(None) };
}

pub fn setup(main: AnyWindowHandle, app: Entity<HarvestApp>, store: Entity<Store>, cx: &mut App) {
    let tray = TrayIconBuilder::new()
        .with_tooltip("Harvest")
        .with_icon(tray_image(false))
        .with_icon_as_template(cfg!(target_os = "macos"))
        .with_menu_on_left_click(false)
        .build();
    let tray = match tray {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to create tray icon: {e}");
            return;
        }
    };

    TRAY.with(|t| *t.borrow_mut() = Some(tray));
    cx.observe(&store, |store, cx| show_status(&store, cx)).detach();

    cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(Duration::from_millis(100)).await;
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                if let TrayIconEvent::Click {
                    button: TrayButton::Left,
                    button_state: MouseButtonState::Up,
                    rect,
                    ..
                } = event
                {
                    let (app, store) = (app.clone(), store.clone());
                    let _ = cx.update(|cx| toggle_popup(main, app, store, rect, cx));
                }
            }
        }
    })
    .detach();
}

fn toggle_popup(
    main: AnyWindowHandle,
    app: Entity<HarvestApp>,
    store: Entity<Store>,
    rect: tray_icon::Rect,
    cx: &mut App,
) {
    if let Some(popup) = POPUP.with(|p| p.borrow_mut().take()) {
        let _ = popup.update(cx, |_, window, _| window.remove_window());
        return;
    }
    if LAST_HIDE.with(|c| c.get()).is_some_and(|t| t.elapsed() < Duration::from_millis(250)) {
        return;
    }

    let popup_h = popup_height(&store, cx);

    // tray-icon reports physical pixels; GPUI windows are placed in logical ones.
    let scale = main
        .update(cx, |_, window, _| window.scale_factor())
        .unwrap_or(1.0) as f64;
    let (ix, iy) = (rect.position.x / scale, rect.position.y / scale);
    let (iw, ih) = (rect.size.width as f64 / scale, rect.size.height as f64 / scale);
    let x = (ix + iw - POPUP_W as f64).max(0.0);
    // Taskbar at the bottom (Windows): open upward. Menu bar at the top (macOS): open downward.
    let y = if iy > popup_h as f64 { iy - popup_h as f64 } else { iy + ih };

    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            point(px(x as f32), px(y as f32)),
            size(px(POPUP_W), px(popup_h)),
        ))),
        titlebar: None,
        kind: WindowKind::PopUp,
        is_movable: false,
        is_resizable: false,
        is_minimizable: false,
        window_background: WindowBackgroundAppearance::Transparent,
        focus: true,
        show: true,
        ..Default::default()
    };
    match gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| TrayPopup::new(main, app, store, window, cx))) {
        Ok((handle, _)) => {
            ui::clear_root_background(handle, cx);
            POPUP.with(|p| *p.borrow_mut() = Some(handle));
        }
        Err(e) => eprintln!("Failed to open tray popup: {e}"),
    }
}

/// Height that fits the popup's content for the current services.
fn popup_height(store: &Entity<Store>, cx: &App) -> f32 {
    let rows = store.read(cx).services.len() as f32;
    (POPUP_BASE_H + rows * SERVICE_ROW_H).min(POPUP_MAX_H)
}

fn close_popup(window: &mut Window) {
    POPUP.with(|p| p.borrow_mut().take());
    LAST_HIDE.with(|c| c.set(Some(Instant::now())));
    window.remove_window();
}

struct TrayPopup {
    main: AnyWindowHandle,
    app: Entity<HarvestApp>,
    store: Entity<Store>,
    _subscriptions: Vec<Subscription>,
}

impl TrayPopup {
    fn new(
        main: AnyWindowHandle,
        app: Entity<HarvestApp>,
        store: Entity<Store>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscriptions = vec![
            // Re-fit when the services list changes (e.g. opened before it loaded).
            cx.observe_in(&store, window, |_, store, window, cx| {
                let height = px(popup_height(&store, cx));
                if window.viewport_size().height != height {
                    window.resize(size(px(POPUP_W), height));
                }
                cx.notify();
            }),
            cx.observe_window_activation(window, |_, window, _| {
                if !window.is_window_active() {
                    close_popup(window);
                }
            }),
        ];
        Self { main, app, store, _subscriptions: subscriptions }
    }

    fn open_main(&self, page: Page, window: &mut Window, cx: &mut App) {
        self.app.update(cx, |app, cx| app.show_page(page, cx));
        let _ = self.main.update(cx, |_, window, _| window.activate_window());
        close_popup(window);
    }
}

fn menu_row(id: &'static str, label: &'static str) -> Stateful<Div> {
    h_flex()
        .id(id)
        .mx(px(6.))
        .px(px(8.))
        .h(px(26.))
        .rounded(px(6.))
        .body()
        .hover(|s| s.bg(theme::cream(0.1)))
        .child(label)
}

fn separator() -> impl IntoElement {
    div().mx(px(14.)).my(px(5.)).h(px(1.)).bg(theme::separator())
}

impl Render for TrayPopup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let services = self.store.read(cx).services.clone();
        let running = services.iter().filter(|s| s.running()).count();
        let status = match running {
            0 => "All services stopped".to_string(),
            1 => "1 service running".to_string(),
            n => format!("{n} services running"),
        };

        // Control Center style: a rounded translucent-looking panel with a
        // switch per service, then menu items.
        v_flex()
            .size_full()
            .py(px(6.))
            .bg(theme::sheet())
            .text_color(theme::label())
            .border_1()
            .border_color(theme::cream(0.12))
            .rounded(px(12.))
            .overflow_hidden()
            .child(
                h_flex()
                    .gap(px(10.))
                    .px(px(14.))
                    .py(px(8.))
                    .child(ui::brand_mark(28.))
                    .child(
                        v_flex()
                            .child(div().headline().child("Harvest"))
                            .child(div().caption().text_color(theme::secondary()).child(status)),
                    ),
            )
            .child(separator())
            .child(
                div()
                    .px(px(14.))
                    .pt(px(2.))
                    .pb(px(4.))
                    .caption()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme::tertiary())
                    .child("Services"),
            )
            .child(
                v_flex()
                    .id("tray-services")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children(services.iter().enumerate().map(|(ix, s)| {
                        let (symbol, color) = service_symbol(&s.info.id);
                        h_flex()
                            .px(px(14.))
                            .h(px(SERVICE_ROW_H))
                            .gap(px(10.))
                            .child(ui::symbol_tile(symbol, if s.installed() { color } else { theme::stone() }, 22.))
                            .child(
                                v_flex()
                                    .flex_1()
                                    .child(div().body().child(s.info.name.clone()))
                                    .child(
                                        div()
                                            .caption()
                                            .text_color(theme::secondary())
                                            .child(if s.installed() { format!("Port {}", s.info.port) } else { "Not Installed".into() }),
                                    ),
                            )
                            .when(s.installed(), |el| el.child(service_switch(&self.store, ix, s)))
                    })),
            )
            .child(separator())
            .child(
                menu_row("open", "Open Harvest")
                    .on_click(cx.listener(|this, _, window, cx| this.open_main(Page::Dashboard, window, cx))),
            )
            .child(
                menu_row("sites", "Sites…")
                    .on_click(cx.listener(|this, _, window, cx| this.open_main(Page::Sites, window, cx))),
            )
            .child(
                menu_row("services", "Services…")
                    .on_click(cx.listener(|this, _, window, cx| this.open_main(Page::Services, window, cx))),
            )
            .child(separator())
            .child(menu_row("quit", "Quit Harvest").on_click(|_, _, cx| cx.quit()))
    }
}

fn tray_image(active: bool) -> Icon {
    let img = if cfg!(target_os = "macos") {
        assets::menu_bar_glyph(active)
    } else {
        assets::brand_mark(32, active)
    };
    Icon::from_rgba(img.data, img.width, img.height).expect("valid tray icon")
}

/// Refresh the tray icon and tooltip when the number of running or installed
/// services changes.
fn show_status(store: &Entity<Store>, cx: &App) {
    let services = &store.read(cx).services;
    let running = services.iter().filter(|s| s.running()).count();
    let installed = services.iter().filter(|s| s.installed()).count();
    if SHOWN_STATUS.with(|c| c.get()) == Some((running, installed)) {
        return;
    }
    SHOWN_STATUS.with(|c| c.set(Some((running, installed))));

    let tooltip = match (installed, running) {
        (0, _) => "Harvest · No services installed".to_string(),
        (_, 0) => "Harvest · All services stopped".to_string(),
        (_, 1) => "Harvest · 1 service running".to_string(),
        (_, n) => format!("Harvest · {n} services running"),
    };
    TRAY.with(|t| {
        let Some(tray) = t.borrow().as_ref().cloned() else { return };
        let icon = tray_image(running > 0);
        let _ = if cfg!(target_os = "macos") {
            tray.set_icon_with_as_template(Some(icon), true)
        } else {
            tray.set_icon(Some(icon))
        };
        let _ = tray.set_tooltip(Some(tooltip));
    });
}
