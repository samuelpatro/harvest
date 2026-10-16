# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Harvest is a fully native desktop application for managing local PHP/OctoberCMS development
environments on Windows, similar to Laravel Herd. It is written in Rust with a GPUI UI (via
gpui-kit; native GPU rendering, no webview). It was migrated from Tauri 2.0 + Vue 3, then from
Slint (see `docs/gpui-migration.md`).

## Development Commands

```bash
# Run the app (workspace default member is harvest-gpui, binary `harvest`)
cargo run

# Build the whole workspace
cargo build

# Release build (single native binary)
cargo build --release

# Backend only (fast; no system libraries needed)
cargo build -p harvest-core
cargo test  -p harvest-core

# UI crate tests (icon rendering)
cargo test -p harvest-gpui
```

GPUI needs the latest stable Rust. On macOS it renders with Metal (needs Xcode); on Windows it
needs the MSVC toolchain (Visual Studio Build Tools, "Desktop development with C++").

## Architecture

Cargo workspace:

### `crates/harvest-core` (backend, no UI dependencies)
- `config.rs`: app config (JSON in the OS config dir) + Windows PATH management (winreg).
- `sites.rs`: scan project directories, detect framework/stack/PHP version, open actions.
- `php.rs` / `node.rs`: installed + available versions (cached 24h), install/uninstall, extensions
  (PHP) / package managers (Node).
- `services.rs`: nginx/MySQL/Redis detection, start/stop/restart, install/uninstall.
- `vhosts.rs`: nginx vhost + hosts file management for `.test` domains.
- `versions.rs`: grouping installed/available versions into rows.
- `types.rs`: shared serde structs.
- `progress.rs`: `Progress` / `ProgressSink` (a transport-agnostic callback) for install progress.
- Pure helpers are unit-tested in `tests/`.

### `crates/harvest-gpui` (native GPUI binary `harvest`)
- `main.rs`: opens the main window (blurred on macOS, Mica on Windows; traffic lights centered in
  the 52pt toolbar), the shared `Store`, and the tray.
- `store.rs`: the `Store` entity. Owns all data loaded from harvest-core and implements every
  action (sites, services, PHP/Node versions, extensions, settings, first run). Emits
  `StoreEvent::{Info, Error}`; polls service status every 5s.
- `bridge.rs`: shared tokio runtime. `bridge::spawn(fut)` starts core work off the UI thread and
  returns a future that GPUI tasks await.
- `app.rs`: main window shell. Unified toolbar (drawn into gpui-component's `TitleBar`), sidebar,
  page switch, overlays (extensions sheet, first-run wizard), notifications.
- Pages: `dashboard.rs`, `sites.rs`, `versions.rs` (PHP and Node.js), `services.rs`,
  `settings.rs`, plus `extensions.rs` (sheet) and `wizard.rs` (first run).
- `tray.rs`: `tray-icon` icon + popup window. Icon and tooltip follow service status.
- `theme.rs`: brand tokens, type scale (`Type` trait), gpui-component theme overrides.
- `ui.rs`: shared building blocks (page body, sections, inset grouped lists, chips, capsules,
  brand mark, toolbar type, folder picker).
- `assets.rs`: icon asset source (default gpui-kit icons + the extra Lucide icons we use) and
  rasterized brand/tray icons (resvg).

### `crates/harvest-app` (legacy)
The previous Slint UI, kept until the GPUI app is verified on Windows. Not a default member, so
`cargo run` / `cargo build` skip it; build it with `-p harvest-app`.

## UI <-> backend pattern

Every view holds an `Entity<Store>` and `cx.observe`s it, so a store `cx.notify()` re-renders the
views. Actions are `Store` methods called through `store.update(cx, ...)`. Blocking or async core
calls go through `bridge::spawn(...)`; the store awaits them in `cx.spawn` and writes results back
with `this.update(cx, ...)`. Install progress arrives through a `ProgressSink` that forwards to a
futures channel drained on the UI thread. User-facing results are `StoreEvent`s, which `HarvestApp`
turns into notifications.

Each page also has `fn toolbar(this: &Entity<Self>, cx: &App) -> ui::Toolbar` (title, subtitle,
actions) that `app.rs` renders into the window toolbar.

## GPUI gotchas

- gpui-kit pins its GPUI snapshot (`gpui-pre`); depend on `gpui-kit` only, never on `gpui`.
- The theme's `font_size` is the window rem size: 13px here, so rem-based spacing and component
  heights are macOS sized. Use the `Type` trait (`.body()`, `.caption()`, ...) for text sizes.
- gpui-component's root paints the theme background; `ui::clear_root_background` removes it so
  the window blur shows through the sidebar and the tray popup's rounded corners.
- Toolbar controls sit inside an `.occlude()` element. Without it, clicks start a window drag on
  macOS and are reported to Windows as caption area.
- Gradients between a translucent and an opaque stop render with a bright band; use
  `theme::mix` for solid stops.
- Icons: add new Lucide names to `ExtraIcons` in `assets.rs`, or they render empty.

## Design System

- **Brand gradient**: wheat gold `#f5b83d` to pumpkin `#ef6f2e` (logo, selection, primary buttons).
- **Surfaces**: warm darks, content `#15120f`, cards `#1e1a16`, sidebar `#0f0d0b` over the window
  blur, sheets `#231e19`. Text is cream `#f6efe6` (secondary 58%, tertiary 34%).
- **Category colors**: leaf `#7bcf6b` (running/success), violet `#9d8cff` (PHP), teal `#3cc9b8`
  (services), sky `#5b9dff`, rose `#f2667a` (errors), stone `#a39a90`.
- **Type**: system font (SF Pro / Segoe UI), SF sizes (13 body, 11 caption, 15 title3, 20 title2).
- **Layout**: macOS style. Unified 52pt toolbar, sidebar, inset grouped lists with inset
  separators, section titles above groups and footnotes below.

## Site Detection (`harvest-core/src/sites.rs`)

Scans project directories and detects:
- **PHP frameworks**: OctoberCMS, Laravel, plain PHP (via `composer.json`, `artisan`, module dirs)
- **Frontend stack**: Vue, React, Vite, Tailwind, TypeScript (via `package.json`)
- **Package managers**: Bun, pnpm, Yarn (via lockfiles)
- **PHP version**: from `.php-version` or `composer.json` require

## Backend API

See `README.md` for the full function list per `harvest-core` module, plus the feature checklist
and roadmap.
