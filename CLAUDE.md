# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Harvest is a native desktop application for managing local PHP/OctoberCMS development environments, similar to Laravel Herd. Built with Tauri 2.0 (Rust backend) and Vue 3 + TypeScript frontend.

## Development Commands

```bash
# Install dependencies
bun install

# Run development server (starts both Vite and Tauri)
bun run tauri dev

# Build for production
bun run tauri build

# Type check
bunx vue-tsc --noEmit

# Frontend only dev server
bun run dev
```

## Architecture

### Frontend (Vue 3 + TypeScript)
- **Entry**: `src/main.ts` → `src/App.vue`
- **Routing**: `src/router.ts` - Vue Router with views (Dashboard, Sites, Php, Node, Services, Settings)
- **State**: Pinia stores in `src/stores/` - `sites.ts`, `services.ts`, `config.ts`
- **Types**: `src/types/index.ts` - TypeScript interfaces (Site, StackItem, ServiceInfo, etc.)
- **Styling**: Tailwind CSS 4 with CSS variables in `src/styles/main.css`

### Backend (Rust/Tauri)
```
src-tauri/
├── src/
│   ├── main.rs              # Tauri entry point
│   ├── lib.rs               # App initialization, command registration
│   ├── types/
│   │   └── mod.rs           # Shared data structures
│   └── commands/
│       ├── mod.rs           # Module exports
│       ├── config.rs        # App configuration (get/save config)
│       ├── sites.rs         # Site scanning, detection, open actions
│       ├── php.rs           # PHP version management, extensions
│       ├── node.rs          # Node.js versions, package managers
│       └── services.rs      # Service management (Phase 2)
├── Cargo.toml
└── tauri.conf.json
```

### Key Tauri Plugins
- `tauri-plugin-dialog` - Native folder picker
- `tauri-plugin-shell` - Open external applications
- `tauri-plugin-opener` - Default browser opening

## Design System

- **Background**: `#0a0a0b` (primary), `#111113` (elevated), `#161618` (card)
- **Accent**: `#f59e0b` (amber)
- **Fonts**: System UI, JetBrains Mono (code)
- **Frameless window** with custom titlebar

## Tauri Commands

### Config Commands (`commands/config.rs`)
- `get_config()` → `AppConfig`
- `save_config(config)` → `Result<(), String>`

### Site Commands (`commands/sites.rs`)
- `scan_sites(directories)` → `Result<Vec<Site>, String>`
- `open_in_editor(path, editor)` → `Result<(), String>`
- `open_in_terminal(path, terminal)` → `Result<(), String>`
- `open_in_browser(url)` → `Result<(), String>`

### PHP Commands (`commands/php.rs`)
- `get_available_php_versions()` → `Result<Vec<AvailableVersion>, String>`
- `get_installed_php_versions()` → `Vec<InstalledVersion>`
- `install_php_version(app, version)` → `Result<(), String>`
- `uninstall_php_version(version)` → `Result<(), String>`
- `get_php_extensions(php_path)` → `Result<Vec<PhpExtension>, String>`
- `toggle_php_extension(php_path, extension, enable)` → `Result<(), String>`

### Node Commands (`commands/node.rs`)
- `get_available_node_versions()` → `Result<Vec<AvailableVersion>, String>`
- `get_installed_node_versions()` → `Vec<InstalledVersion>`
- `install_node_version(app, version)` → `Result<(), String>`
- `uninstall_node_version(version)` → `Result<(), String>`
- `get_package_managers()` → `Vec<PackageManager>`

## Site Detection

The Rust backend (`commands/sites.rs`) scans project directories and detects:
- **PHP frameworks**: OctoberCMS, Laravel, plain PHP (via `composer.json`, `artisan`, module directories)
- **Frontend stack**: Vue, React, Vite, Tailwind, TypeScript (via `package.json`)
- **Package managers**: Bun, pnpm, Yarn (via lockfiles)
- **PHP version**: From `.php-version` file or `composer.json` require

## Frontend-Backend Communication

Vue components call Rust via `invoke` from `@tauri-apps/api/core`:
```typescript
import { invoke } from '@tauri-apps/api/core'
const sites = await invoke<Site[]>('scan_sites', { directories: ['/path'] })
```

Rust returns snake_case fields; the frontend handles mapping as needed.

## Data Types (`types/mod.rs`)

```rust
pub struct AppConfig { harvest_paths, editor_command, terminal_command }
pub struct Site { id, name, path, domain, php_version, stack, ssl }
pub struct StackItem { name, version }
pub struct InstalledVersion { version, path }
pub struct AvailableVersion { version, full_version, lts, download_url }
pub struct PhpExtension { name, enabled, builtin }
pub struct PackageManager { name, command, installed, version, description }
```

See `README.md` for full roadmap and feature checklist.
