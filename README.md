# Harvest - Local Development Environment Manager

A native desktop application for managing local PHP/Node.js development environments on Windows. Built with Tauri 2.0, Vue 3, and TypeScript.

## Tech Stack

- **Backend:** Tauri 2.0 (Rust)
- **Frontend:** Vue 3 + TypeScript
- **Styling:** Tailwind CSS 4 (dark theme, amber accent)
- **State:** Pinia
- **Routing:** Vue Router

---

## Feature Checklist

### Core Infrastructure
- [x] Tauri 2.0 + Vue 3 + TypeScript setup
- [x] Dark theme with amber accent (#f59e0b)
- [x] Custom frameless window
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
- [x] Add PHP to Windows PATH

### Node.js Management
- [x] Fetch available Node.js versions from nodejs.org
- [x] Show LTS versions
- [x] Download and install Node.js versions
- [x] Uninstall Node.js versions
- [x] Update detection
- [x] Package managers detection (npm, yarn, pnpm, bun)
- [x] Grouped version display (like PHP)
- [x] Add Node to Windows PATH

### Site Management
- [x] Scan directories for projects
- [x] Display site list with framework detection
- [x] Open site in browser
- [x] Open site in editor (VS Code, Cursor, PhpStorm)
- [x] Open site in terminal
- [x] Auto-detect OctoberCMS, Laravel, vanilla PHP
- [x] Create `.test` domain routing (nginx vhost + hosts file)
- [x] Link/unlink sites UI
- [ ] Per-site PHP version pinning
- [ ] SSL certificate generation

### Service Management
- [x] Services view UI
- [x] Start/stop nginx
- [x] Start/stop MySQL
- [x] Start/stop Redis
- [x] Status monitoring (running/stopped)
- [x] Version detection
- [ ] Start/stop PHP-FPM
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
- [x] Cross-platform path handling (Windows/macOS/Linux)
- [ ] System tray icon
- [ ] Minimize to tray
- [ ] SQLite database for config
- [ ] Windows startup option
- [ ] Auto-updater (Tauri updater plugin)

### Harvest Standalone Structure
- [x] `C:\harvest\bin\` - Service binaries (nginx, mysql, redis, php)
- [x] `C:\harvest\etc\` - Config files (nginx.conf, vhosts, ssl)
- [x] `C:\harvest\logs\` - Service logs
- [ ] Configurable harvest home path
- [ ] First-run setup wizard

---

## Project Structure

```
harvest/
├── src/                          # Vue frontend
│   ├── components/
│   │   ├── PageHeader.vue        # Page title + subtitle
│   │   ├── PhpSelector.vue       # Global PHP version selector
│   │   ├── Sidebar.vue           # Navigation sidebar
│   │   ├── ServiceCard.vue       # Service status card
│   │   └── SiteCard.vue          # Site display card
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
│   ├── tray/
│   │   ├── TrayPopup.vue         # System tray popup UI
│   │   └── main.ts               # Tray window entry point
│   ├── types/
│   │   └── index.ts              # TypeScript interfaces
│   ├── App.vue
│   ├── main.ts
│   └── router.ts
├── src-tauri/
│   ├── src/
│   │   ├── main.rs               # Tauri entry point
│   │   ├── lib.rs                # App init, command registration
│   │   ├── types/
│   │   │   └── mod.rs            # Shared data structures
│   │   └── commands/
│   │       ├── mod.rs            # Module exports
│   │       ├── config.rs         # App configuration
│   │       ├── sites.rs          # Site scanning & actions
│   │       ├── php.rs            # PHP version management
│   │       ├── node.rs           # Node.js & package managers
│   │       ├── services.rs       # Service management
│   │       └── vhosts.rs         # Vhost & domain management
│   ├── Cargo.toml
│   └── tauri.conf.json
├── index.html
├── package.json
├── vite.config.ts
└── README.md
```

---

## Implemented Tauri Commands

### Config (`commands/config.rs`)
```rust
get_config() -> AppConfig
save_config(config: AppConfig) -> Result<(), String>
is_first_run() -> bool
get_harvest_home() -> String
initialize_harvest_home(path: String) -> Result<(), String>
validate_harvest_path(path: String) -> Result<bool, String>
// PATH Management
is_in_path(path: String) -> bool
add_to_path(path: String) -> Result<(), String>
remove_from_path(path: String) -> Result<(), String>
get_default_php_path() -> Option<String>
get_default_node_path() -> Option<String>
```

### Sites (`commands/sites.rs`)
```rust
scan_sites(directories: Vec<String>) -> Result<Vec<Site>, String>
open_in_editor(path: String, editor: String) -> Result<(), String>
open_in_terminal(path: String, terminal: String) -> Result<(), String>
open_in_browser(url: String) -> Result<(), String>
```

### PHP (`commands/php.rs`)
```rust
get_available_php_versions() -> Result<Vec<AvailableVersion>, String>
get_installed_php_versions() -> Vec<InstalledVersion>
install_php_version(app: AppHandle, version: String) -> Result<(), String>
uninstall_php_version(version: String) -> Result<(), String>
get_php_extensions(php_path: String) -> Result<Vec<PhpExtension>, String>
toggle_php_extension(php_path: String, extension: String, enable: bool) -> Result<(), String>
```

### Node.js (`commands/node.rs`)
```rust
get_available_node_versions() -> Result<Vec<AvailableVersion>, String>
get_installed_node_versions() -> Vec<InstalledVersion>
install_node_version(app: AppHandle, version: String) -> Result<(), String>
uninstall_node_version(version: String) -> Result<(), String>
get_package_managers() -> Vec<PackageManager>
```

### Services (`commands/services.rs`)
```rust
get_services() -> Vec<ServiceInfo>
start_service(id: String) -> Result<(), String>
stop_service(id: String) -> Result<(), String>
restart_service(id: String) -> Result<(), String>
```

### Vhosts (`commands/vhosts.rs`)
```rust
get_vhost_status(site_name: String) -> VhostStatus
get_all_vhost_statuses(site_names: Vec<String>) -> Vec<VhostStatus>
link_site(site_name: String, root_path: String) -> Result<String, String>
unlink_site(site_name: String) -> Result<String, String>
check_nginx_config() -> Result<String, String>
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

## Testing

### Rust Backend Tests

The Rust backend includes comprehensive integration tests in the `tests/` directory:

```bash
# Run all Rust tests
cd src-tauri && cargo test

# Run tests with output
cd src-tauri && cargo test -- --nocapture

# Run specific test file
cd src-tauri && cargo test --test sites_tests
cd src-tauri && cargo test --test vhosts_tests
cd src-tauri && cargo test --test services_tests
cd src-tauri && cargo test --test node_tests
cd src-tauri && cargo test --test php_tests
cd src-tauri && cargo test --test types_tests
cd src-tauri && cargo test --test config_tests
```

### Test Structure

```
src-tauri/
├── src/           # Source code (no inline tests)
└── tests/         # Integration tests
    ├── types_tests.rs      # 9 tests - AppConfig, Site, StackItem, etc.
    ├── sites_tests.rs      # 16 tests - Version parsing, site ID generation
    ├── vhosts_tests.rs     # 16 tests - Nginx config, hosts file parsing
    ├── services_tests.rs   # 9 tests - Version extraction from CLI output
    ├── node_tests.rs       # 13 tests - Download URLs, folder parsing
    ├── php_tests.rs        # 12 tests - PHP version parsing
    └── config_tests.rs     # 27 tests - PATH normalization and manipulation
```

### Prerequisites for Running Tests

On Linux, you'll need Tauri development dependencies:
```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
```

On Windows, no additional dependencies are required.

---

## Design Theme

```css
:root {
  --color-bg: #0d0d0f;
  --color-bg-elevated: #18181b;
  --color-bg-card: #1c1c1f;
  --color-border: #27272a;
  --color-text: #fafafa;
  --color-text-muted: #a1a1aa;
  --color-accent: #f59e0b;        /* Amber/Harvest */
  --color-success: #22c55e;
  --color-danger: #ef4444;
}
```

**Fonts:** Outfit (headings/UI), JetBrains Mono (code)

---

## Roadmap

### Phase 1 (Complete) - Version Management
- [x] PHP version management
- [x] Node.js version management
- [x] Package managers detection
- [x] Basic site scanning

### Phase 2 (Complete) - Service Management
- [x] Standalone `C:\harvest` folder structure
- [x] nginx, MySQL, Redis service detection
- [x] Start/stop/restart services
- [x] Service status monitoring
- [x] Version detection for services

### Phase 3 (In Progress) - Domain Routing
- [x] Nginx vhost config generation
- [x] Hosts file management (with UAC elevation)
- [x] Site link/unlink UI
- [ ] SSL certificate generation (self-signed with local CA)
- [ ] Per-site PHP version switching

### Phase 4 (Complete) - Standalone Mode
- [x] Configurable Harvest home path (e.g., `D:\Harvest`, `C:\Dev\Harvest`)
- [x] First-run setup wizard to pick install location
- [x] Small config in AppData pointing to real Harvest folder
- [x] Fully portable - move the whole folder anywhere
- [x] **Download services from official sources:**
  - [x] nginx (from nginx.org)
  - [x] MySQL (from dev.mysql.com)
  - [x] Redis (from GitHub releases)
  - [x] PHP versions (already implemented)
- [x] Service install/uninstall UI (like PHP versions page)
- [x] No Laragon dependency - completely standalone

### Phase 5 - Dashboard & Polish
- [ ] Quick stats overview (sites count, services status)
- [ ] Services compact status cards with toggle
- [ ] Recent sites quick access
- [ ] System tray integration
- [ ] Auto-updates
- [ ] Backup/restore settings

### Future Ideas

**Database Management**
- [ ] Create/drop MySQL databases
- [ ] Database GUI (like TablePlus/HeidiSQL embedded)
- [ ] Import/export SQL dumps
- [ ] Database backups scheduling

**Developer Tools**
- [ ] Log viewer for nginx/PHP/MySQL (tail -f style)
- [ ] PHP error log viewer with syntax highlighting
- [ ] Composer integration (install, update, require)
- [ ] Artisan command runner for Laravel
- [ ] October CMS artisan commands

**Site Features**
- [ ] Site templates (create new Laravel, October, vanilla PHP)
- [ ] Clone site from Git repository
- [ ] Site backups (files + database)
- [ ] Favorite/pin sites to top
- [ ] Site groups/folders
- [ ] Site notes/descriptions

**Sharing & Collaboration**
- [ ] Share sites via tunneling (like Expose/ngrok)
- [ ] QR code for mobile testing
- [ ] Local network sharing (access from phone)

**Quality of Life**
- [ ] Global search (sites, services, settings)
- [ ] Keyboard shortcuts
- [ ] Command palette (Ctrl+K)
- [ ] Notifications for service crashes
- [ ] Health checks (disk space, memory)
- [ ] Dark/light theme toggle

**Advanced**
- [ ] Multiple PHP-FPM pools
- [ ] Xdebug configuration per site
- [ ] Mail catcher (like Mailhog/Mailpit)
- [ ] Queue worker management (Laravel Horizon)
- [ ] Cron job scheduler
- [ ] Docker integration (optional)
