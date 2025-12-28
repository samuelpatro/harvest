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
- **Routing**: `src/router.ts` - Vue Router with 4 views (Dashboard, Sites, Services, Settings)
- **State**: Pinia stores in `src/stores/` - `sites.ts`, `services.ts`, `config.ts`
- **Types**: `src/types/index.ts` - TypeScript interfaces (Site, StackItem, ServiceInfo, etc.)
- **Styling**: Tailwind CSS 4 with CSS variables in `src/styles/main.css`

### Backend (Rust/Tauri)
- **Entry**: `src-tauri/src/main.rs` → `src-tauri/src/lib.rs`
- **Config**: Settings persisted to `%APPDATA%/harvest/config.json` (Windows)
- **Tauri Commands**:
  - `get_config` / `save_config` - App configuration
  - `scan_sites` - Scans directory for PHP projects, detects framework & stack
  - `open_in_browser` / `open_in_editor` / `open_in_terminal` - Site actions

### Key Tauri Plugins
- `tauri-plugin-dialog` - Native folder picker
- `tauri-plugin-shell` - Open external applications
- `tauri-plugin-opener` - Default browser opening

## Design System

- **Background**: `#0d0d0f` (primary), `#18181b` (elevated), `#1c1c1f` (card)
- **Accent**: `#f59e0b` (amber)
- **Fonts**: Outfit (UI), JetBrains Mono (code)
- **Frameless window** with custom titlebar in `src/components/Titlebar.vue`

## Site Detection

The Rust backend (`detect_stack` function) scans project directories and detects:
- **PHP frameworks**: OctoberCMS, Laravel, plain PHP (via `composer.json`, `artisan`, module directories)
- **Frontend stack**: Vue, React, Vite, Tailwind, TypeScript (via `package.json`)
- **Package managers**: Bun, pnpm, Yarn (via lockfiles)
- **PHP version**: From `.php-version` file or `composer.json` require

## Frontend-Backend Communication

Vue components call Rust via `invoke` from `@tauri-apps/api/core`:
```typescript
import { invoke } from '@tauri-apps/api/core'
const sites = await invoke<Site[]>('scan_sites', { directory: '/path' })
```

Rust returns snake_case fields; map to camelCase in TypeScript stores.
