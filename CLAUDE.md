# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Harvest is a fully native desktop application for managing local PHP/OctoberCMS development
environments on Windows, similar to Laravel Herd. It is written in Rust with a Slint UI (native
rendering, no webview). It was migrated from an earlier Tauri 2.0 + Vue 3 stack.

## Development Commands

```bash
# Run the app (workspace default member is harvest-app)
cargo run

# Build the whole workspace
cargo build

# Release build (single native binary)
cargo build --release

# Backend only (fast; no system libraries needed)
cargo build -p harvest-core
cargo test  -p harvest-core
```

Use the Slint VS Code extension for a live preview of `crates/harvest-app/ui/*.slint`.

## Architecture

Cargo workspace with two crates:

### `crates/harvest-core` — backend (no UI dependencies)
- `config.rs` — app config (JSON in the OS config dir) + Windows PATH management (winreg).
- `sites.rs` — scan project directories, detect framework/stack/PHP version, open actions.
- `php.rs` / `node.rs` — installed + available versions (cached 24h), install/uninstall, extensions
  (PHP) / package managers (Node).
- `services.rs` — nginx/MySQL/Redis detection, start/stop/restart, install/uninstall.
- `vhosts.rs` — nginx vhost + hosts file management for `.test` domains.
- `types.rs` — shared serde structs.
- `progress.rs` — `Progress` / `ProgressSink` (a transport-agnostic callback) used to report
  install progress. The UI decides how to surface it.
- Pure helpers are unit-tested in `tests/logic.rs`.

### `crates/harvest-app` — native Slint binary
- `ui/` — Slint markup: `theme.slint` (design tokens), `components.slint`, `app.slint` (frameless
  window + sidebar + view switch), per-screen views (`dashboard`, `sites`, `services`, `versions`,
  `settings`), `ext.slint` (PHP extensions modal), `wizard.slint` (first-run), `tray.slint`,
  `globals.slint` (the `Backend` global: data models + action callbacks).
- `src/main.rs` — selects the winit backend with `decorations(false)` for a frameless window,
  creates the main window + tray popup, wires everything.
- `src/window.rs` — titlebar drag + min/max/close via `slint::winit_030::WinitWindowAccessor`.
- `src/tray.rs` — `tray-icon` creation + popup show/position/hide-on-blur.
- `src/bridge.rs` — runs blocking/async work on a tokio runtime, marshals results back to the Slint
  event loop with `slint::invoke_from_event_loop`.
- `src/controllers.rs` — fills the Slint `Backend` models from `harvest-core` and implements every
  callback. Owns the `VecModel`s (thread-local, UI thread).

## UI <-> backend pattern

The Slint `Backend` global (in `ui/globals.slint`) declares data properties (models of `SiteItem`,
`ServiceItem`, `VersionItem`, `PmItem`, `ExtItem`) and action callbacks. `controllers::setup`
attaches `Rc<VecModel<…>>` to those properties and wires each callback to a `harvest-core` function.
Long/blocking calls go through `bridge::run(fut, on_done)`; `on_done` runs on the UI thread and
updates the models. Installs pass a `ProgressSink` that updates the relevant row's progress.

## Design System

- **Background**: `#0d0d0f` (primary), `#18181b` (elevated), `#1c1c1f` (card)
- **Accent**: `#f59e0b` (amber), hover `#d97706`; success `#22c55e`, danger `#ef4444`
- **Fonts**: Outfit (UI), JetBrains Mono (code)
- **Frameless window** with a custom titlebar; tokens live in `ui/theme.slint`.

## Site Detection (`harvest-core/src/sites.rs`)

Scans project directories and detects:
- **PHP frameworks**: OctoberCMS, Laravel, plain PHP (via `composer.json`, `artisan`, module dirs)
- **Frontend stack**: Vue, React, Vite, Tailwind, TypeScript (via `package.json`)
- **Package managers**: Bun, pnpm, Yarn (via lockfiles)
- **PHP version**: from `.php-version` or `composer.json` require

## Backend API

See `README.md` for the full function list per `harvest-core` module, plus the feature checklist
and roadmap. `docs/slint-migration.md` describes the migration from Tauri + Vue.
