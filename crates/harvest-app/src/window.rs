//! Frameless-window controls (Spike A). The window has no native decorations
//! (set via the winit attributes hook in `main`), so drag/minimize/maximize/
//! close are driven through the underlying winit window.

use slint::winit_030::{winit, WinitWindowAccessor};
use slint::ComponentHandle;

use crate::AppWindow;

pub fn wire_window_controls(app: &AppWindow) {
    let weak = app.as_weak();
    app.on_start_drag(move || {
        if let Some(app) = weak.upgrade() {
            app.window().with_winit_window(|w: &winit::window::Window| {
                let _ = w.drag_window();
            });
        }
    });

    let weak = app.as_weak();
    app.on_minimize(move || {
        if let Some(app) = weak.upgrade() {
            app.window().with_winit_window(|w| w.set_minimized(true));
        }
    });

    let weak = app.as_weak();
    app.on_toggle_maximize(move || {
        if let Some(app) = weak.upgrade() {
            app.window()
                .with_winit_window(|w| w.set_maximized(!w.is_maximized()));
        }
    });

    app.on_close(|| {
        let _ = slint::quit_event_loop();
    });
}
