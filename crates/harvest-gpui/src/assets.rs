//! Icon assets: gpui-kit's default component icons plus the extra Lucide
//! icons Harvest uses (only those are embedded, not the whole catalog).

use std::borrow::Cow;

use gpui_kit::assets::{icon_assets, Assets};
use gpui_kit::{AssetSource, Result, SharedString};

icon_assets!(ExtraIcons, [
    Activity, CircleStop, CodeXml, Database, Download, Hexagon, Link, Package, Power, Puzzle,
    Server, Terminal, Trash, Unlink, Wheat,
]);

pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = ExtraIcons.load(path)? {
            return Ok(Some(bytes));
        }
        Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = Assets.list(path)?;
        paths.extend(ExtraIcons.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

/// Straight (not premultiplied) RGBA pixels.
pub struct Rgba {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// The Lucide wheat glyph's path elements, from the bundled icon.
fn wheat_glyph() -> String {
    let svg = AppAssets
        .load("icons/wheat.svg")
        .ok()
        .flatten()
        .expect("wheat icon is bundled");
    let svg = String::from_utf8_lossy(&svg);
    let start = svg.find('>').map_or(0, |i| i + 1);
    let end = svg.rfind("</svg>").unwrap_or(svg.len());
    svg[start..end].to_string()
}

fn rasterize(svg: &str) -> Rgba {
    use resvg::{tiny_skia, usvg};
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).expect("valid icon svg");
    let size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height()).expect("non-empty icon");
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    let data = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    Rgba { data, width: size.width(), height: size.height() }
}

/// The Harvest mark (wheat on the gold to pumpkin gradient) at `size` px, with a
/// green status dot in the corner when `active`.
pub fn brand_mark(size: u32, active: bool) -> Rgba {
    let glyph = wheat_glyph();
    let dot = if active {
        r##"<circle cx="25.5" cy="25.5" r="5.5" fill="#7BCF6B" stroke="#15120F" stroke-width="2"/>"##
    } else {
        ""
    };
    rasterize(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 32 32">
  <defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0" stop-color="#F5B83D"/><stop offset="1" stop-color="#EF6F2E"/>
  </linearGradient></defs>
  <rect width="32" height="32" rx="8" fill="url(#g)"/>
  <g transform="translate(5 5) scale(0.9167)" fill="none" stroke="#1F1406" stroke-width="2.3" stroke-linecap="round" stroke-linejoin="round">{glyph}</g>
  {dot}
</svg>"##
    ))
}

/// Monochrome wheat for the macOS menu bar. Used as a template image, so only
/// its alpha matters and macOS tints it to match the menu bar. When `active`, a
/// dot is cut into the corner.
pub fn menu_bar_glyph(active: bool) -> Rgba {
    let glyph = wheat_glyph();
    let (cut, dot) = if active {
        (
            r##"<circle cx="19.5" cy="19.5" r="5.5" fill="#000"/>"##,
            r##"<circle cx="19.5" cy="19.5" r="3.5" fill="#000"/>"##,
        )
    } else {
        ("", "")
    };
    rasterize(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 24 24">
  <defs><mask id="m"><rect width="24" height="24" fill="#fff"/>{cut}</mask></defs>
  <g mask="url(#m)" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{glyph}</g>
  {dot}
</svg>"##
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn visible_pixels(img: &Rgba) -> usize {
        img.data.chunks(4).filter(|px| px[3] > 0).count()
    }

    #[test]
    fn status_icons_render_and_show_activity() {
        for (idle, active, size) in [
            (brand_mark(32, false), brand_mark(32, true), 32),
            (menu_bar_glyph(false), menu_bar_glyph(true), 36),
        ] {
            assert_eq!((idle.width, idle.height), (size, size));
            assert_eq!(idle.data.len(), (size * size * 4) as usize);
            assert!(visible_pixels(&idle) > 0);
            assert_ne!(idle.data, active.data, "the running state must change the icon");
        }
    }
}
