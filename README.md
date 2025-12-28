# Harvest - Local Development Environment Manager

A native desktop application for managing local PHP/Node.js development environments on Windows. Built with Tauri 2.0, Vue 3, and TypeScript.

## Tech Stack

- **Backend:** Tauri 2.0 (Rust)
- **Frontend:** Vue 3 + TypeScript
- **Styling:** Custom CSS (dark theme, amber accent)
- **State:** Pinia
- **Routing:** Vue Router

---

## Feature Checklist

### Core Infrastructure
- [x] Tauri 2.0 + Vue 3 + TypeScript setup
- [x] Dark theme with amber accent (#f59e0b)
- [x] Custom titlebar with window controls
- [x] Sidebar navigation
- [x] Page header component
- [x] Responsive layout

### PHP Management
- [x] Fetch available PHP versions from windows.php.net
- [x] Show supported/LTS versions from php.net/releases/active
- [x] Download and install PHP versions
- [x] Uninstall PHP versions
- [x] Grouped version display (multiple versions per major.minor)
- [x] Update detection (show when newer version available)
- [x] PHP extensions management (enable/disable via php.ini)
- [ ] Per-site PHP version switching
- [ ] Add PHP to Windows PATH

### Node.js Management
- [x] Fetch available Node.js versions from nodejs.org
- [x] Show LTS versions
- [x] Download and install Node.js versions
- [x] Uninstall Node.js versions
- [x] Update detection
- [x] Package managers (npm, yarn, pnpm, bun) - detect, install, update, uninstall
- [ ] Grouped version display (like PHP)
- [ ] Add Node to Windows PATH

### Site Management
- [x] Scan directories for projects
- [x] Display site list with framework detection
- [x] Open site in browser
- [x] Open site in editor (VS Code, Cursor, PhpStorm)
- [x] Open site in terminal
- [ ] Auto-detect OctoberCMS, Laravel, vanilla PHP
- [ ] Create `.test` domain routing (hosts file)
- [ ] Per-site PHP version pinning
- [ ] SSL certificate generation

### Service Management
- [x] Services view UI
- [ ] Start/stop nginx
- [ ] Start/stop PHP-FPM
- [ ] Start/stop MySQL
- [ ] Start/stop Redis
- [ ] Status monitoring (running/stopped)
- [ ] Auto-start on app launch
- [ ] View service logs

### Settings
- [x] Sites directory configuration
- [x] Editor selection (code, cursor, phpstorm)
- [x] Terminal selection
- [ ] Auto-start services toggle
- [ ] Theme selection (dark/light)

### System Integration
- [x] Configuration persistence (JSON file)
- [ ] System tray icon
- [ ] Minimize to tray
- [ ] SQLite database for config
- [ ] Windows startup option

---

## Project Structure

```
harvest/
├── src/                          # Vue frontend
│   ├── components/
│   │   ├── PageHeader.vue        # Page title + subtitle
│   │   ├── Sidebar.vue           # Navigation sidebar
│   │   ├── ServiceCard.vue       # Service status card
│   │   ├── SiteCard.vue          # Site display card
│   │   └── PhpSelector.vue       # (deprecated)
│   ├── views/
│   │   ├── Dashboard.vue         # Overview page
│   │   ├── Sites.vue             # Site management
│   │   ├── Php.vue               # PHP version management
│   │   ├── Node.vue              # Node.js version management
│   │   ├── Services.vue          # Service management
│   │   └── Settings.vue          # App settings
│   ├── stores/
│   │   ├── services.ts           # Services state
│   │   ├── sites.ts              # Sites state
│   │   └── config.ts             # App configuration
│   ├── styles/
│   │   └── main.css              # Global styles + CSS variables
│   ├── types/
│   │   └── index.ts              # TypeScript interfaces
│   ├── App.vue
│   ├── main.ts
│   └── router.ts
├── src-tauri/
│   ├── src/
│   │   ├── main.rs               # Tauri entry point
│   │   └── lib.rs                # All Tauri commands
│   ├── Cargo.toml
│   └── tauri.conf.json
├── index.html
├── package.json
├── vite.config.ts
└── README.md
```

---

## Implemented Tauri Commands

```rust
// Configuration
get_config() -> AppConfig
save_config(config: AppConfig) -> Result<(), String>

// Sites
scan_sites(directories: Vec<String>) -> Result<Vec<Site>, String>
open_in_editor(path: String, editor: String) -> Result<(), String>
open_in_terminal(path: String, terminal: String) -> Result<(), String>
open_in_browser(url: String) -> Result<(), String>

// PHP
get_available_php_versions() -> Result<Vec<AvailableVersion>, String>
get_installed_php_versions() -> Vec<InstalledVersion>
install_php_version(version: String) -> Result<(), String>
uninstall_php_version(version: String) -> Result<(), String>
get_php_extensions(php_path: String) -> Result<Vec<PhpExtension>, String>
toggle_php_extension(php_path: String, extension: String, enable: bool) -> Result<(), String>

// Node.js
get_available_node_versions() -> Result<Vec<AvailableVersion>, String>
get_installed_node_versions() -> Vec<InstalledVersion>
install_node_version(version: String) -> Result<(), String>
uninstall_node_version(version: String) -> Result<(), String>

// Package Managers
get_package_managers() -> Vec<PackageManager>
install_package_manager(name: String) -> Result<(), String>
uninstall_package_manager(name: String) -> Result<(), String>
update_package_manager(name: String) -> Result<(), String>
```

---

## Development

```bash
# Install dependencies
bun install

# Run development
bun run tauri dev

# Build for production
bun run tauri build
```

---

## Design Theme

```css
:root {
  --color-bg: #0a0a0b;
  --color-bg-elevated: #111113;
  --color-bg-card: #161618;
  --color-border: #232328;
  --color-text: #fafafa;
  --color-text-muted: #71717a;
  --color-accent: #f59e0b;        /* Amber/Harvest */
  --color-success: #22c55e;
  --color-danger: #ef4444;
}
```

**Fonts:** System UI (Segoe UI on Windows), JetBrains Mono (code)

---

## Roadmap

### Phase 1 (Current) - Version Management
- [x] PHP version management
- [x] Node.js version management
- [x] Basic site scanning

### Phase 2 - Service Management
- [ ] nginx bundling and management
- [ ] PHP-FPM configuration
- [ ] MySQL/MariaDB integration
- [ ] Redis integration

### Phase 3 - Domain Routing
- [ ] Hosts file management
- [ ] .test domain routing
- [ ] SSL certificate generation
- [ ] Per-site configuration

### Phase 4 - Polish
- [ ] System tray integration
- [ ] Auto-updates
- [ ] Backup/restore settings
- [ ] Plugin system for additional services
