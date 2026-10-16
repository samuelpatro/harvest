# Migration: Tauri + Vue → native Slint (Rust)

Harvest was originally a Tauri 2.0 app: a Rust backend plus a Vue 3 + TypeScript frontend rendered
in a webview. It is now a fully native Rust application rendered by Slint — no webview, no DOM, no
CDN fonts, one language end to end.

## Why

- One language across the whole app (Rust).
- Native rendering (winit + femtovg) instead of an embedded webview.
- Smaller, self-contained binary; no bundled web runtime.

## Architecture after migration

```
harvest/  (cargo workspace)
├── crates/harvest-core   # backend logic, zero UI dependencies
└── crates/harvest-app    # native Slint binary (UI + controllers)
```

`harvest-core` is plain Rust (`reqwest`, `std::fs`, `std::process`, `dirs`, `winreg`). The Slint app
calls it directly from `controllers.rs`; there is no IPC boundary.

## How it was done (phased)

- **Phase 0 — decouple the backend.** Moved all logic out of the Tauri command layer into
  `harvest-core`. The key coupling was install progress, which used Tauri's `AppHandle`/`emit`. That
  became a transport-agnostic `ProgressSink` callback (`progress.rs`). During the migration the Tauri
  commands were kept as thin wrappers that re-emitted the exact same `install-progress` event, so the
  old Vue UI kept working while the native UI was built in parallel.
- **Phase 1 — native shell.** Frameless window via the winit attributes hook
  (`decorations(false)`), a custom titlebar with drag + minimize/maximize/close through
  `slint::winit_030::WinitWindowAccessor`, a themed app shell, and the system tray
  (`tray-icon`) with a popup window that toggles and hides on blur.
- **Phases 2–3 — design system + screens.** Reusable Slint components and the full screens
  (Dashboard, Sites, PHP, Node, Services, Settings) wired to `harvest-core` via the `Backend`
  global, plus the PHP extensions modal, Node package managers panel, and first-run wizard.
- **Phase 4 — realtime + polish.** 5s service-status poll (declarative `Timer`), install progress
  bars driven by the `ProgressSink`, bottom-right auto-dismiss toasts, real app/tray icon.
- **Phase 5 — cutover.** Removed the Tauri layer (`src-tauri`) and the Vue frontend (`src`,
  `index.html`, `vite.config.ts`, `package.json`, etc.) and promoted `harvest-app` as the app entry.

## Async bridging

Slint runs a single-threaded event loop. Blocking and network work (the `reqwest` calls, process
spawning, registry edits) runs on a tokio runtime via `bridge::run(fut, on_done)`; the result is
delivered back on the UI thread with `slint::invoke_from_event_loop`, where the controllers update
the Slint `VecModel`s. This pattern is used by every controller.

## Notes

- Slint is licensed under GPLv3 OR a royalty-free desktop license OR a paid license. Confirm the
  chosen option before distributing builds.
- `harvest-core` builds and tests without any system libraries; the Slint UI builds natively on
  Windows (winit + femtovg).
