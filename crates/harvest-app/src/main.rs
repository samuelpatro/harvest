mod window;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Frameless: drop native window decorations — we render our own titlebar.
    slint::BackendSelector::new()
        .with_winit_window_attributes_hook(|attrs| attrs.with_decorations(false))
        .select()?;

    let app = AppWindow::new()?;
    window::wire_window_controls(&app);
    app.run()?;
    Ok(())
}
