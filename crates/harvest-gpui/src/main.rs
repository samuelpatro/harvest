mod app;
mod assets;
mod bridge;
mod dashboard;
mod extensions;
mod services;
mod settings;
mod sites;
mod store;
mod theme;
mod tray;
mod ui;
mod versions;
mod wizard;

use gpui_kit::component::TitleBar;
use gpui_kit::*;

use crate::app::HarvestApp;
use crate::store::Store;

fn main() {
    gpui_kit::application().with_assets(assets::AppAssets).run(|cx| {
        gpui_kit::init(cx);
        theme::apply(cx);

        let store = cx.new(Store::new);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1200.), px(800.)), cx)),
            window_min_size: Some(size(px(900.), px(600.))),
            window_background: theme::window_background(),
            // Traffic lights centered in the 52pt unified toolbar.
            titlebar: Some(TitlebarOptions {
                traffic_light_position: Some(point(px(20.), px(19.))),
                ..TitleBar::title_bar_options()
            }),
            ..TitleBar::window_options()
        };
        let (main, app) = gpui_kit::open_window(options, cx, |window, cx| {
            let store = store.clone();
            cx.new(|cx| HarvestApp::new(store, window, cx))
        })
        .expect("failed to open main window");
        ui::clear_root_background(main, cx);

        // Closing the main window quits, as the Slint app did.
        cx.on_window_closed(move |cx, closed| {
            if closed == main.window_id() {
                cx.quit();
            }
        })
        .detach();

        tray::setup(main, app, store, cx);
        cx.activate(true);
    });
}
