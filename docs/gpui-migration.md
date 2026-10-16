# Migration: Slint to GPUI

Harvest's UI moved from Slint (`crates/harvest-app`) to GPUI (`crates/harvest-gpui`, binary
`harvest`). `harvest-core` did not change. The earlier move from Tauri + Vue is described in
`slint-migration.md`.

## Why

- UI written in plain Rust, with the same tooling as the rest of the app.
- GPU-accelerated rendering from the framework behind Zed, with Win32/DirectWrite on Windows.
- gpui-kit provides a maintained component library (inputs, switches, notifications, title bar).

## Dependency choice

GPUI's crates.io release (`gpui` 0.2.2) is old and `gpui_platform` is not published, so the README
snippet on gpui.rs only works against Zed's git repository. gpui-kit (Longbridge, Apache-2.0)
publishes a matching snapshot as `gpui-pre-*` crates and re-exports it, so the app depends on
`gpui-kit` alone, from crates.io, with no git pin. Upgrades follow gpui-kit releases and are pre-1.0
(expect breaking changes).

## Architecture changes

| Slint | GPUI |
| --- | --- |
| `Backend` global with `VecModel`s in thread-locals | `Store` entity owning all state and actions |
| Callbacks wired in `controllers.rs` | `Store` methods, called via `store.update(cx, ...)` |
| `bridge::run(fut, on_done)` + `invoke_from_event_loop` | `bridge::spawn(fut)` awaited in `cx.spawn` |
| Status toast in the window | gpui-component notifications from `StoreEvent`s |
| winit frameless window + manual drag | gpui-component `TitleBar` as a unified toolbar |
| Colored PNG tray icon | Rendered icon that follows service status |

## Design

The UI follows macOS conventions (unified toolbar, sidebar, inset grouped lists, SF sizes) with
Harvest's own brand: warm darks, cream text and a wheat gold to pumpkin gradient. Tokens are in
`theme.rs`, shared pieces in `ui.rs`.

## Things that were not obvious

- The theme's `font_size` becomes the window rem size. It is 13px, which makes rem-based spacing
  and component heights macOS sized.
- gpui-component's root paints the theme background over the whole window. It is cleared so the
  window blur (macOS) or Mica (Windows 11) shows through the sidebar and around the tray popup.
- Interactive controls inside the title bar must be wrapped in an `.occlude()` element. On macOS
  the title bar otherwise starts a window drag on mouse down, and on Windows the click is reported
  to the OS as caption area, so the control never receives it.
- `tray-icon`'s `set_icon_with_as_template` does nothing outside macOS; use `set_icon` there.
- GPUI's runtime window icon is X11 only. A Windows executable icon needs a resource embedded at
  build time (not done yet).
- Only the icons listed in `assets.rs` are embedded. An icon missing there renders empty.

## Status

- All screens are ported: dashboard, sites, PHP, Node.js (with package managers), services,
  settings, PHP extensions sheet, first-run wizard, tray popup.
- Verified on macOS (rendering of every page and the tray popup).
- Not yet verified on Windows: Mica backdrop, title bar controls, tray icon position and popup.
  `harvest-app` (Slint) stays in the workspace until that is done.
- Building for Windows from macOS fails in `harvest-core`'s zip dependency (bzip2, zstd and lzma
  need the MSVC C toolchain); build on Windows.
