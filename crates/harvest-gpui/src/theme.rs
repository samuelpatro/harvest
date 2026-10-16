//! Harvest's brand on a macOS-style layout: warm "soil" darks, cream text and a
//! wheat-gold to pumpkin gradient, with SF type sizes. Mapped onto
//! gpui-component's theme.

use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{
    linear_color_stop, linear_gradient, px, rgb, App, Background, Hsla, Pixels, Styled,
    WindowBackgroundAppearance,
};

pub const SIDEBAR_WIDTH: Pixels = px(232.);
pub const TOOLBAR_HEIGHT: Pixels = px(52.);

fn c(hex: u32) -> Hsla {
    rgb(hex).into()
}

/// `color` at `alpha`, for tinted capsules, icon chips and glows.
pub fn tint(color: Hsla, alpha: f32) -> Hsla {
    Hsla { a: alpha, ..color }
}

/// `color` with its lightness shifted by `delta` (negative darkens).
pub fn shade(color: Hsla, delta: f32) -> Hsla {
    Hsla { l: (color.l + delta).clamp(0., 1.), ..color }
}

/// `over` blended into `base` by `t`, as a solid color. Gradients between a
/// translucent and an opaque stop render with a bright band; mix instead.
pub fn mix(base: Hsla, over: Hsla, t: f32) -> Hsla {
    let (a, b) = (base.to_rgb(), over.to_rgb());
    gpui_kit::Rgba {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: 1.,
    }
    .into()
}

/// Cream at `alpha`: hairlines and hover washes on the warm darks.
pub fn cream(alpha: f32) -> Hsla {
    tint(c(0xf6efe6), alpha)
}

// Brand.
pub fn gold() -> Hsla {
    c(0xf5b83d)
}
pub fn pumpkin() -> Hsla {
    c(0xef6f2e)
}
/// The signature wheat-gold to pumpkin gradient.
pub fn brand_gradient(angle: f32) -> Background {
    linear_gradient(angle, linear_color_stop(gold(), 0.), linear_color_stop(pumpkin(), 1.))
}
/// Dark text for gold fills (white fails contrast on gold).
pub fn on_gold() -> Hsla {
    c(0x1f1406)
}

// Category colors, tuned to sit next to the gold.
pub fn leaf() -> Hsla {
    c(0x7bcf6b)
}
pub fn violet() -> Hsla {
    c(0x9d8cff)
}
pub fn teal() -> Hsla {
    c(0x3cc9b8)
}
pub fn sky() -> Hsla {
    c(0x5b9dff)
}
pub fn rose() -> Hsla {
    c(0xf2667a)
}
pub fn stone() -> Hsla {
    c(0xa39a90)
}

// Text.
pub fn label() -> Hsla {
    c(0xf6efe6)
}
pub fn secondary() -> Hsla {
    cream(0.58)
}
pub fn tertiary() -> Hsla {
    cream(0.34)
}
pub fn separator() -> Hsla {
    cream(0.07)
}

// Surfaces.
pub fn content() -> Hsla {
    c(0x15120f)
}
/// Content background: a faint warm glow fading down from the toolbar.
pub fn content_background() -> Background {
    linear_gradient(180., linear_color_stop(c(0x1e1813), 0.), linear_color_stop(content(), 0.4))
}
pub fn card() -> Hsla {
    c(0x1e1a16)
}
pub fn card_border() -> Hsla {
    cream(0.065)
}
pub fn sheet() -> Hsla {
    c(0x231e19)
}

/// Near-opaque so the sidebar stays dark on any desktop, with a hint of the
/// window's blur (macOS) or Mica (Windows 11) coming through.
pub fn sidebar() -> Hsla {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        tint(c(0x0f0d0b), 0.9)
    } else {
        c(0x0f0d0b)
    }
}

pub fn window_background() -> WindowBackgroundAppearance {
    if cfg!(target_os = "macos") {
        WindowBackgroundAppearance::Blurred
    } else if cfg!(target_os = "windows") {
        WindowBackgroundAppearance::MicaBackdrop
    } else {
        WindowBackgroundAppearance::Opaque
    }
}

/// SF type scale (size / line height in points).
pub trait Type: Styled + Sized {
    fn large_title(self) -> Self {
        self.text_size(px(26.)).line_height(px(32.)).font_weight(gpui_kit::FontWeight::BOLD)
    }
    fn title2(self) -> Self {
        self.text_size(px(20.)).line_height(px(25.)).font_weight(gpui_kit::FontWeight::BOLD)
    }
    fn title3(self) -> Self {
        self.text_size(px(15.)).line_height(px(20.)).font_weight(gpui_kit::FontWeight::SEMIBOLD)
    }
    fn headline(self) -> Self {
        self.text_size(px(13.)).line_height(px(16.)).font_weight(gpui_kit::FontWeight::SEMIBOLD)
    }
    fn body(self) -> Self {
        self.text_size(px(13.)).line_height(px(16.))
    }
    fn callout(self) -> Self {
        self.text_size(px(12.)).line_height(px(15.))
    }
    fn caption(self) -> Self {
        self.text_size(px(11.)).line_height(px(13.))
    }
}

impl<T: Styled> Type for T {}

pub fn apply(cx: &mut App) {
    // Switching mode reloads the registered dark theme, so override it in a
    // separate update afterwards.
    Theme::change(ThemeMode::Dark, None, cx);
    Theme::update(cx, |t| {
        // macOS body text is 13pt; the theme's font size is the window rem size,
        // so this also sets component proportions (controls 20/26pt tall).
        t.font_size = px(13.);
        t.radius = px(7.);
        t.radius_lg = px(12.);
        t.shadow = true;

        t.background = content();
        t.foreground = label();
        t.muted = cream(0.08);
        t.muted_foreground = secondary();
        t.border = separator();
        t.input = cream(0.13);
        t.ring = tint(gold(), 0.55);
        t.caret = gold();
        t.selection = tint(gold(), 0.3);

        t.title_bar = content();
        t.title_bar_border = separator();
        t.sidebar = sidebar();
        t.sidebar_foreground = label();
        t.sidebar_border = separator();
        t.popover = sheet();
        t.popover_foreground = label();
        t.list_hover = cream(0.05);

        t.primary = gold();
        t.primary_hover = c(0xf8c862);
        t.primary_active = c(0xe9a623);
        t.primary_foreground = on_gold();
        t.button_primary = gold();
        t.button_primary_hover = c(0xf8c862);
        t.button_primary_active = c(0xe9a623);
        t.button_primary_foreground = on_gold();

        t.secondary = cream(0.1);
        t.secondary_hover = cream(0.15);
        t.secondary_active = cream(0.19);
        t.secondary_foreground = label();
        t.accent = cream(0.08);
        t.accent_foreground = label();

        t.success = leaf();
        t.danger = rose();
        t.warning = gold();
        t.info = sky();
        t.progress_bar = gold();
        t.switch = cream(0.16);
        t.scrollbar = gpui_kit::transparent_black();
        t.scrollbar_thumb = cream(0.22);
    });
}
