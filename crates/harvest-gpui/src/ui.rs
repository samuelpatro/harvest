//! Shared building blocks in the style of macOS System Settings: page bodies,
//! inset grouped lists, capsules, icon tiles and empty states.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{h_flex, v_flex, Icon, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::{self, Type as _};

/// Scrolling page body below the toolbar.
pub fn page_body(id: &'static str, content: impl IntoElement) -> impl IntoElement {
    v_flex()
        .id(id)
        .size_full()
        .overflow_y_scroll()
        .px(px(24.))
        .pt(px(8.))
        .pb(px(28.))
        .gap(px(24.))
        .child(content)
}

/// Group with a title above it and an optional footnote below (Apple style).
pub fn section(title: impl Into<SharedString>, content: impl IntoElement) -> Div {
    v_flex()
        .gap(px(6.))
        .child(div().px(px(10.)).headline().text_color(theme::label()).child(title.into()))
        .child(content)
}

/// Explanatory text under a group.
pub fn footnote(text: impl Into<SharedString>) -> impl IntoElement {
    div().px(px(10.)).pt(px(2.)).caption().text_color(theme::secondary()).child(text.into())
}

/// Inset grouped list container.
pub fn group() -> Div {
    v_flex()
        .bg(theme::card())
        .border_1()
        .border_color(theme::card_border())
        .rounded(px(12.))
        .overflow_hidden()
}

/// A row of a [`group`]. Rows after the first get a separator inset to the text
/// column (`inset`), like AppKit's inset grouped lists.
pub fn row(id: impl Into<ElementId>, first: bool, inset: f32) -> Stateful<Div> {
    h_flex()
        .id(id)
        .relative()
        .min_h(px(44.))
        .px(px(12.))
        .py(px(8.))
        .gap(px(10.))
        .hover(|s| s.bg(theme::cream(0.025)))
        .when(!first, |el| {
            el.child(
                div()
                    .absolute()
                    .top_0()
                    .left(px(inset))
                    .right_0()
                    .h(px(1.))
                    .bg(theme::separator()),
            )
        })
}

pub fn icon(name: IconName, size: f32, color: Hsla) -> Icon {
    Icon::new(name).size(px(size)).text_color(color)
}

/// Rounded chip with a colored glyph on a soft gradient of the same color.
pub fn symbol_tile(name: IconName, color: Hsla, size: f32) -> impl IntoElement {
    div()
        .size(px(size))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(size * 0.28))
        .bg(linear_gradient(
            135.,
            linear_color_stop(theme::tint(color, 0.26), 0.),
            linear_color_stop(theme::tint(color, 0.1), 1.),
        ))
        .border_1()
        .border_color(theme::tint(color, 0.22))
        .child(icon(name, size * 0.55, color))
}

/// The Harvest mark: a wheat glyph on the brand gradient.
pub fn brand_mark(size: f32) -> impl IntoElement {
    div()
        .size(px(size))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(size * 0.28))
        .bg(theme::brand_gradient(135.))
        .shadow(vec![BoxShadow {
            color: theme::tint(theme::pumpkin(), 0.35),
            offset: point(px(0.), px(size * 0.12)),
            blur_radius: px(size * 0.45),
            spread_radius: px(0.),
            inset: false,
        }])
        .child(icon(IconName::Wheat, size * 0.58, theme::on_gold()))
}

/// Rounded square with the first letter of `name` on a gradient picked from it.
pub fn monogram(name: &str, size: f32) -> impl IntoElement {
    let colors = [theme::gold(), theme::pumpkin(), theme::violet(), theme::leaf(), theme::teal(), theme::sky(), theme::rose()];
    let hash = name.bytes().fold(0usize, |h, b| h.wrapping_mul(31).wrapping_add(b as usize));
    let color = colors[hash % colors.len()];
    let letter = name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
    div()
        .size(px(size))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(size * 0.28))
        .bg(linear_gradient(
            135.,
            linear_color_stop(color, 0.),
            linear_color_stop(theme::shade(color, -0.14), 1.),
        ))
        .text_color(theme::on_gold())
        .text_size(px(size * 0.46))
        .font_weight(FontWeight::BOLD)
        .child(letter)
}

/// Small tinted capsule ("Default", "LTS", "Linked").
pub fn capsule(text: impl Into<SharedString>, color: Hsla) -> impl IntoElement {
    div()
        .flex_shrink_0()
        .h(px(17.))
        .px(px(7.))
        .flex()
        .items_center()
        .rounded_full()
        .bg(theme::tint(color, 0.18))
        .text_color(color)
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(text.into())
}

pub fn dot(color: Hsla) -> impl IntoElement {
    div().size(px(8.)).flex_shrink_0().rounded_full().bg(color)
}

/// "Running" / "Stopped" / "Not Installed" with a status dot.
pub fn service_status(status: &str) -> impl IntoElement {
    let (label, color) = match status {
        "running" => ("Running", theme::leaf()),
        "stopped" => ("Stopped", theme::stone()),
        _ => ("Not Installed", theme::tertiary()),
    };
    h_flex()
        .flex_shrink_0()
        .gap(px(6.))
        .callout()
        .text_color(theme::secondary())
        .child(dot(color))
        .child(label)
}

/// Borderless icon button with a tooltip, for row and toolbar actions.
pub fn icon_button(id: impl Into<ElementId>, name: IconName, tooltip: &'static str) -> Button {
    Button::new(id).ghost().small().icon(Icon::new(name)).tooltip(tooltip)
}

/// Title and subtitle stacked, the standard two-line row text.
pub fn row_text(title: impl IntoElement, subtitle: impl Into<SharedString>) -> impl IntoElement {
    v_flex()
        .flex_1()
        .min_w_0()
        .gap(px(2.))
        .child(title)
        .child(div().caption().text_color(theme::secondary()).truncate().child(subtitle.into()))
}

/// `path` with the user's home folder shown as `~`.
pub fn tilde(path: &str) -> String {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_default();
    match path.strip_prefix(home.as_str()) {
        Some(rest) if !home.is_empty() => format!("~{rest}"),
        _ => path.to_string(),
    }
}

/// " · "-joined, skipping empty parts.
pub fn joined(parts: &[&str]) -> String {
    parts.iter().filter(|p| !p.is_empty()).copied().collect::<Vec<_>>().join("  ·  ")
}

/// Centered placeholder for an empty list (ContentUnavailableView).
pub fn unavailable(name: IconName, title: &'static str, description: &'static str) -> impl IntoElement {
    v_flex()
        .items_center()
        .gap(px(6.))
        .py(px(36.))
        .child(icon(name, 34., theme::tertiary()))
        .child(div().pt(px(6.)).title3().child(title))
        .child(div().body().text_color(theme::secondary()).child(description))
}

/// Ask the OS for a folder; resolves to `None` when cancelled.
pub async fn pick_folder(cx: &mut AsyncApp) -> Option<String> {
    let rx = cx.update(|cx| {
        cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: None,
        })
    });
    let paths = rx.await.ok()?.ok()??;
    paths.into_iter().next().map(|p| p.to_string_lossy().into_owned())
}

/// What a page puts in the window toolbar.
pub struct Toolbar {
    pub title: SharedString,
    pub subtitle: Option<SharedString>,
    pub actions: Option<AnyElement>,
}

impl Toolbar {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self { title: title.into(), subtitle: None, actions: None }
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(actions.into_any_element());
        self
    }
}

/// Borderless glyph button sized for the toolbar.
pub fn toolbar_button(id: impl Into<ElementId>, name: IconName, tooltip: &'static str) -> Button {
    Button::new(id).ghost().icon(Icon::new(name)).tooltip(tooltip)
}

/// gpui-component's root paints the theme background over the whole window;
/// clear it so the window's blur (or transparency) shows where a view leaves
/// its own background unpainted.
pub fn clear_root_background(window: AnyWindowHandle, cx: &mut App) {
    let _ = window.update(cx, |_, window, cx| {
        gpui_kit::component::Root::update(window, cx, |root, _, cx| {
            root.style().background = Some(gpui_kit::transparent_black().into());
            cx.notify();
        });
    });
}
