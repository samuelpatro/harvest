mod bridge;
mod controllers;
mod tray;
mod window;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Frameless: drop native window decorations — we render our own titlebar.
    // Applies to both the main window and the tray popup.
    slint::BackendSelector::new()
        .with_winit_window_attributes_hook(|attrs| attrs.with_decorations(false))
        .select()?;

    let app = AppWindow::new()?;
    window::wire_window_controls(&app);
    controllers::setup(&app);

    // Tray popup window: created hidden, shown on tray click. Kept alive for the
    // whole session so the handle isn't dropped (which would destroy it).
    let popup = TrayPopup::new()?;
    tray::setup_tray(&popup);

    app.run()?;
    Ok(())
}
