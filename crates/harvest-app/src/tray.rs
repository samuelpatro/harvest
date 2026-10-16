//! System-tray integration (Spike B).
//!
//! Creates the tray icon once the winit event loop is running, then a repeating
//! timer drains tray-icon's event channel. A left click toggles the frameless
//! `TrayPopup` window, positioned just above the tray icon. The popup hides on
//! focus loss; a short guard after a blur-hide stops the very next tray click
//! (which caused that blur) from immediately reopening it.

use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

use slint::winit_030::{winit, EventResult, WinitWindowAccessor};
use slint::ComponentHandle;
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::TrayPopup;

thread_local! {
    static TRAY: RefCell<Option<TrayIcon>> = const { RefCell::new(None) };
    static POLL_TIMER: RefCell<Option<slint::Timer>> = const { RefCell::new(None) };
    static BLUR_HOOKED: Cell<bool> = const { Cell::new(false) };
    static LAST_HIDE: Cell<Option<Instant>> = const { Cell::new(None) };
}

const POPUP_W: f32 = 280.0;
const POPUP_H: f32 = 440.0;

pub fn setup_tray(popup: &TrayPopup) {
    // Footer power button closes the popup.
    let weak = popup.as_weak();
    popup.on_hide_requested(move || {
        if let Some(p) = weak.upgrade() {
            hide_popup(&p);
        }
    });

    // Create the tray icon after the event loop is live, then poll its events.
    let weak = popup.as_weak();
    slint::Timer::single_shot(Duration::from_millis(100), move || {
        let tray = TrayIconBuilder::new()
            .with_tooltip("Harvest")
            .with_icon(build_icon())
            .with_menu_on_left_click(false)
            .build();
        match tray {
            Ok(t) => TRAY.with(|c| *c.borrow_mut() = Some(t)),
            Err(e) => {
                eprintln!("Failed to create tray icon: {e}");
                return;
            }
        }

        let timer = slint::Timer::default();
        let weak = weak.clone();
        timer.start(slint::TimerMode::Repeated, Duration::from_millis(100), move || {
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    rect,
                    ..
                } = event
                {
                    if let Some(popup) = weak.upgrade() {
                        toggle_popup(&popup, rect);
                    }
                }
            }
        });
        POLL_TIMER.with(|c| *c.borrow_mut() = Some(timer));
    });
}

fn toggle_popup(popup: &TrayPopup, rect: tray_icon::Rect) {
    if popup.window().is_visible() {
        hide_popup(popup);
        return;
    }

    // The tray click that reaches us also blurred (and thus hid) an open popup a
    // moment ago. Within the guard window, treat this click as "close", not reopen.
    if let Some(t) = LAST_HIDE.with(|c| c.get()) {
        if t.elapsed() < Duration::from_millis(250) {
            return;
        }
    }

    show_popup(popup, rect);
}

fn show_popup(popup: &TrayPopup, rect: tray_icon::Rect) {
    let win = popup.window();
    let scale = win.scale_factor() as f64;
    let pw = POPUP_W as f64 * scale;
    let ph = POPUP_H as f64 * scale;

    // Bottom-right of the popup aligned to the tray icon, opening upward.
    let x = (rect.position.x + rect.size.width as f64 - pw).max(0.0);
    let y = (rect.position.y - ph).max(0.0);
    win.set_position(slint::PhysicalPosition::new(x as i32, y as i32));

    let _ = popup.show();

    // The winit window only exists after the first show, so hook blur here once.
    BLUR_HOOKED.with(|hooked| {
        if !hooked.get() {
            let weak = popup.as_weak();
            win.on_winit_window_event(move |_w, event| {
                if matches!(event, winit::event::WindowEvent::Focused(false)) {
                    if let Some(p) = weak.upgrade() {
                        hide_popup(&p);
                    }
                }
                EventResult::Propagate
            });
            hooked.set(true);
        }
    });

    win.with_winit_window(|w| {
        w.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowExtWindows;
            w.set_skip_taskbar(true);
        }
        w.focus_window();
    });
}

fn hide_popup(popup: &TrayPopup) {
    if popup.window().is_visible() {
        LAST_HIDE.with(|c| c.set(Some(Instant::now())));
        let _ = popup.hide();
    }
}

/// The app icon, decoded from the bundled PNG.
fn build_icon() -> Icon {
    let png = include_bytes!("../assets/icons/32x32.png");
    let img = image::load_from_memory(png)
        .expect("decode tray icon")
        .to_rgba8();
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.into_raw(), w, h).expect("valid icon")
}
