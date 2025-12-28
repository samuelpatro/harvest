<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import PageHeader from '../components/PageHeader.vue'

interface AvailableVersion {
  version: string
  full_version: string
  lts: boolean
  download_url: string
}

interface InstalledPhp {
  version: string
  path: string
  hasUpdate?: boolean
}

interface PhpVersionGroup {
  majorMinor: string
  latestVersion: string
  lts: boolean
  downloadUrl: string
  installed: InstalledPhp[]
  installing?: boolean
  progress?: number
  status?: string
}

interface PhpExtension {
  name: string
  enabled: boolean
  builtin: boolean
}

const versionGroups = ref<PhpVersionGroup[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
let unlisten: UnlistenFn | null = null

// Extensions state
const selectedVersionForExt = ref<string | null>(null)
const selectedVersionPath = ref<string | null>(null)
const extensions = ref<PhpExtension[]>([])
const loadingExtensions = ref(false)
const extensionFilter = ref('')

const installedCount = computed(() => versionGroups.value.reduce((acc, g) => acc + g.installed.length, 0))
const updatesAvailable = computed(() => versionGroups.value.filter(g =>
  g.installed.length > 0 && !g.installed.some(i => i.version === g.latestVersion)
).length)
const allInstalledVersions = computed(() => {
  const result: { version: string; path: string }[] = []
  for (const g of versionGroups.value) {
    for (const inst of g.installed) {
      result.push({ version: inst.version, path: inst.path })
    }
  }
  return result
})
const filteredExtensions = computed(() => {
  if (!extensionFilter.value) return extensions.value
  return extensions.value.filter(e => e.name.toLowerCase().includes(extensionFilter.value.toLowerCase()))
})

async function loadVersions() {
  loading.value = true
  error.value = null
  try {
    // Fetch both installed and available versions in parallel
    const [installed, available] = await Promise.all([
      invoke<{ version: string; path: string }[]>('get_installed_php_versions'),
      invoke<AvailableVersion[]>('get_available_php_versions')
    ])

    // Build version groups
    const groupMap = new Map<string, PhpVersionGroup>()

    // First, add all available versions as groups
    for (const av of available) {
      const installedInGroup = installed
        .filter(i => i.version.startsWith(av.version))
        .map(i => ({
          version: i.version,
          path: i.path,
          hasUpdate: compareVersions(av.full_version, i.version) > 0
        }))
        .sort((a, b) => compareVersions(b.version, a.version))

      groupMap.set(av.version, {
        majorMinor: av.version,
        latestVersion: av.full_version,
        lts: av.lts,
        downloadUrl: av.download_url,
        installed: installedInGroup,
        installing: false,
        progress: 0,
        status: ''
      })
    }

    // Add any installed versions not in available list (e.g., EOL versions)
    for (const inst of installed) {
      const majorMinor = inst.version.split('.').slice(0, 2).join('.')
      if (!groupMap.has(majorMinor)) {
        groupMap.set(majorMinor, {
          majorMinor,
          latestVersion: inst.version,
          lts: false,
          downloadUrl: '',
          installed: [{ version: inst.version, path: inst.path }],
          installing: false,
          progress: 0,
          status: ''
        })
      }
    }

    // Sort groups by version descending
    versionGroups.value = Array.from(groupMap.values()).sort((a, b) => {
      return compareVersions(b.majorMinor, a.majorMinor)
    })
  } catch (e) {
    console.error('Failed to load PHP versions:', e)
    error.value = String(e)
    versionGroups.value = []
  } finally {
    loading.value = false
  }
}

function compareVersions(a: string, b: string): number {
  const aParts = a.split('.').map(Number)
  const bParts = b.split('.').map(Number)
  for (let i = 0; i < Math.max(aParts.length, bParts.length); i++) {
    const aVal = aParts[i] || 0
    const bVal = bParts[i] || 0
    if (aVal > bVal) return 1
    if (aVal < bVal) return -1
  }
  return 0
}

async function installVersion(group: PhpVersionGroup) {
  group.installing = true
  group.progress = 0
  group.status = 'starting'
  error.value = null
  try {
    await invoke('install_php_version', { version: group.majorMinor })
    await loadVersions()
  } catch (e) {
    console.error('Failed to install PHP:', e)
    error.value = `Failed to install PHP ${group.majorMinor}: ${e}`
    group.installing = false
    group.progress = 0
    group.status = ''
  }
}

async function uninstallVersion(installed: InstalledPhp) {
  if (!confirm(`Are you sure you want to uninstall PHP ${installed.version}?`)) {
    return
  }
  try {
    await invoke('uninstall_php_version', { version: installed.version })
    await loadVersions()
  } catch (e) {
    console.error('Failed to uninstall PHP:', e)
    error.value = `Failed to uninstall PHP ${installed.version}: ${e}`
  }
}

async function selectVersionForExtensions(version: string, path: string) {
  selectedVersionForExt.value = version
  selectedVersionPath.value = path
  loadingExtensions.value = true
  extensions.value = []
  try {
    extensions.value = await invoke<PhpExtension[]>('get_php_extensions', { phpPath: path })
  } catch (e) {
    console.error('Failed to load extensions:', e)
    error.value = `Failed to load extensions: ${e}`
  } finally {
    loadingExtensions.value = false
  }
}

async function toggleExtension(ext: PhpExtension) {
  if (!selectedVersionPath.value) return

  try {
    await invoke('toggle_php_extension', {
      phpPath: selectedVersionPath.value,
      extension: ext.name,
      enable: !ext.enabled
    })
    ext.enabled = !ext.enabled
  } catch (e) {
    console.error('Failed to toggle extension:', e)
    error.value = `Failed to toggle extension: ${e}`
  }
}

onMounted(async () => {
  await loadVersions()

  // Listen for progress events
  unlisten = await listen<{ type: string; version: string; status: string; progress: number }>('install-progress', (event) => {
    if (event.payload.type !== 'php') return

    const group = versionGroups.value.find(g => g.majorMinor === event.payload.version)
    if (group) {
      group.progress = event.payload.progress
      group.status = event.payload.status

      if (event.payload.status === 'complete') {
        group.installing = false
        loadVersions()
      }
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>

<template>
  <div class="page-layout">
    <PageHeader title="PHP" :subtitle="`${installedCount} installed${updatesAvailable > 0 ? ` · ${updatesAvailable} update${updatesAvailable > 1 ? 's' : ''} available` : ''}`" />

    <div class="page-content">
      <!-- Versions Section -->
      <section class="content-section">
        <h2 class="section-title">Versions</h2>
        <p class="section-desc">Manage PHP versions for your projects. Each project can use a different version.</p>

        <div v-if="loading" class="loading-state">
          <div class="loading-spinner" />
          <span>Fetching available versions...</span>
        </div>

        <div v-else class="versions-table">
          <div class="table-header">
            <span class="col-version">Version</span>
            <span class="col-status">Status</span>
          </div>
          <div class="table-body">
            <div
              v-for="group in versionGroups"
              :key="group.majorMinor"
              class="version-group"
            >
              <!-- Main version row -->
              <div class="table-row" :class="{ 'has-installed': group.installed.length > 0 }">
                <span class="col-version">
                  {{ group.majorMinor }}
                  <span v-if="group.lts" class="lts-badge">Supported</span>
                  <span class="latest-version">({{ group.latestVersion }})</span>
                </span>
                <span class="col-status">
                  <div v-if="group.installing" class="install-progress">
                    <div class="progress-bar">
                      <div class="progress-fill" :style="{ width: `${group.progress}%` }" />
                    </div>
                    <span class="progress-text">{{ group.status === 'extracting' ? 'Extracting...' : `${group.progress}%` }}</span>
                  </div>
                  <button
                    v-else-if="group.installed.length === 0"
                    class="btn-install"
                    @click="installVersion(group)"
                  >
                    Install
                  </button>
                  <button
                    v-else-if="!group.installed.some(i => i.version === group.latestVersion)"
                    class="btn-update"
                    @click="installVersion(group)"
                  >
                    Update
                  </button>
                  <span v-else class="installed-status">
                    <svg class="check-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <polyline points="20 6 9 17 4 12" />
                    </svg>
                    {{ group.installed.length > 1 ? `${group.installed.length} installed` : 'Installed' }}
                  </span>
                </span>
              </div>
              <!-- Installed versions sub-rows -->
              <div v-if="group.installed.length > 0" class="installed-list">
                <div
                  v-for="inst in group.installed"
                  :key="inst.version"
                  class="installed-row"
                  :class="{ 'has-update': inst.hasUpdate }"
                >
                  <span class="installed-version-name">
                    {{ inst.version }}
                    <span v-if="inst.hasUpdate" class="update-badge" :title="`Update available: ${group.latestVersion}`">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M12 19V5M5 12l7-7 7 7" />
                      </svg>
                    </span>
                  </span>
                  <button class="btn-uninstall" @click="uninstallVersion(inst)" title="Uninstall">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                      <path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Extensions Section -->
      <section class="content-section">
        <h2 class="section-title">Extensions</h2>
        <p class="section-desc">PHP extensions are managed per-version. Select a version to manage its extensions.</p>

        <div v-if="allInstalledVersions.length === 0" class="empty-state">
          No PHP versions installed. Install a version above to manage extensions.
        </div>

        <div v-else class="extensions-panel">
          <div class="version-selector">
            <label>Select PHP Version:</label>
            <div class="version-buttons">
              <button
                v-for="v in allInstalledVersions"
                :key="v.version"
                class="version-btn"
                :class="{ active: selectedVersionForExt === v.version }"
                @click="selectVersionForExtensions(v.version, v.path)"
              >
                {{ v.version }}
              </button>
            </div>
          </div>

          <div v-if="selectedVersionForExt" class="extensions-content">
            <div v-if="loadingExtensions" class="loading-state">
              <div class="loading-spinner" />
              <span>Loading extensions...</span>
            </div>

            <template v-else>
              <div class="extensions-header">
                <input
                  v-model="extensionFilter"
                  type="text"
                  class="search-input"
                  placeholder="Search extensions..."
                />
                <span class="extensions-count">{{ extensions.length }} extensions</span>
              </div>

              <div class="extensions-grid">
                <div
                  v-for="ext in filteredExtensions"
                  :key="ext.name"
                  class="extension-item"
                  :class="{ enabled: ext.enabled }"
                >
                  <div class="extension-info">
                    <span class="extension-name">{{ ext.name }}</span>
                  </div>
                  <button
                    class="toggle-btn"
                    :class="{ enabled: ext.enabled }"
                    @click="toggleExtension(ext)"
                    :title="ext.enabled ? 'Disable' : 'Enable'"
                  >
                    <span class="toggle-track">
                      <span class="toggle-thumb" />
                    </span>
                  </button>
                </div>
              </div>
            </template>
          </div>

          <div v-else class="select-hint">
            Select a PHP version above to view and manage its extensions
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.page-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.page-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px 32px 32px;
}

.page-content > .content-section {
  max-width: 800px;
}

.content-section {
  margin-bottom: 32px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 8px 0;
}

.section-desc {
  font-size: 13px;
  color: var(--color-text-muted);
  margin: 0 0 20px 0;
}

.loading-state {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 24px;
  color: var(--color-text-muted);
  font-size: 13px;
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--color-border);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.versions-table {
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  overflow: hidden;
}

.table-header {
  display: flex;
  padding: 12px 16px;
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border);
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.table-body {
  display: flex;
  flex-direction: column;
}

.table-row {
  display: flex;
  padding: 14px 16px;
  border-bottom: 1px solid var(--color-border);
  transition: background 0.15s ease;
}

.table-row:last-child {
  border-bottom: none;
}

.table-row:hover {
  background: var(--color-bg-elevated);
}

.table-row.installed {
  background: rgba(245, 158, 11, 0.03);
}

.col-version {
  flex: 1;
  font-size: 14px;
  color: var(--color-text);
  display: flex;
  align-items: center;
  gap: 8px;
}

.installed-version {
  font-size: 12px;
  color: var(--color-text-muted);
}

.lts-badge {
  padding: 2px 6px;
  background: rgba(34, 197, 94, 0.15);
  color: #22c55e;
  font-size: 10px;
  font-weight: 600;
  border-radius: 4px;
  text-transform: uppercase;
}

.update-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  background: rgba(59, 130, 246, 0.15);
  color: #3b82f6;
  font-size: 11px;
  font-weight: 500;
  border-radius: 4px;
  margin-left: 4px;
}

.update-badge svg {
  width: 12px;
  height: 12px;
}

.table-row.has-update {
  background: rgba(59, 130, 246, 0.03);
}

.btn-update {
  padding: 6px 14px;
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 6px;
  color: #3b82f6;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-update:hover {
  background: rgba(59, 130, 246, 0.2);
  border-color: #3b82f6;
}

.col-status {
  width: 100px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

.status-installed {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  color: var(--color-accent);
}

.status-installed svg {
  width: 18px;
  height: 18px;
}

.btn-install {
  padding: 6px 14px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  color: var(--color-text);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  gap: 6px;
}

.btn-install:hover:not(:disabled) {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.btn-install:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.install-progress {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 120px;
}

.progress-bar {
  flex: 1;
  height: 6px;
  background: var(--color-bg);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--color-accent);
  border-radius: 3px;
  transition: width 0.2s ease;
}

.progress-text {
  font-size: 11px;
  color: var(--color-text-muted);
  width: 32px;
  text-align: right;
  flex-shrink: 0;
}

.empty-state {
  padding: 24px;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  color: var(--color-text-muted);
  font-size: 13px;
  text-align: center;
}

.extensions-panel {
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  overflow: hidden;
}

.version-selector {
  padding: 16px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  gap: 12px;
}

.version-selector label {
  font-size: 13px;
  color: var(--color-text-muted);
  white-space: nowrap;
}

.version-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.version-btn {
  padding: 6px 12px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  color: var(--color-text);
  font-size: 12px;
  font-family: var(--font-mono);
  cursor: pointer;
  transition: all 0.15s ease;
}

.version-btn:hover {
  border-color: var(--color-accent);
}

.version-btn.active {
  background: rgba(245, 158, 11, 0.15);
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.extensions-content {
  padding: 16px;
}

.extensions-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.search-input {
  padding: 8px 12px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  color: var(--color-text);
  font-size: 13px;
  width: 200px;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-accent);
}

.extensions-count {
  font-size: 12px;
  color: var(--color-text-muted);
}

.extensions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 8px;
}

.extension-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  transition: all 0.15s ease;
}

.extension-item.enabled {
  border-color: rgba(34, 197, 94, 0.3);
  background: rgba(34, 197, 94, 0.05);
}

.extension-name {
  font-size: 12px;
  font-family: var(--font-mono);
  color: var(--color-text);
}

.toggle-btn {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
}

.toggle-track {
  display: block;
  width: 36px;
  height: 20px;
  background: var(--color-border);
  border-radius: 10px;
  position: relative;
  transition: background 0.2s ease;
}

.toggle-btn.enabled .toggle-track {
  background: #22c55e;
}

.toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  background: white;
  border-radius: 50%;
  transition: transform 0.2s ease;
}

.toggle-btn.enabled .toggle-thumb {
  transform: translateX(16px);
}

.select-hint {
  padding: 32px;
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
}

/* Grouped version styles */
.version-group {
  border-bottom: 1px solid var(--color-border);
}

.version-group:last-child {
  border-bottom: none;
}

.version-group .table-row {
  border-bottom: none;
}

.version-group .table-row.has-installed {
  background: rgba(245, 158, 11, 0.03);
}

.latest-version {
  font-size: 12px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}

.installed-count {
  font-size: 12px;
  color: var(--color-accent);
  font-weight: 500;
}

.installed-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--color-accent);
  font-weight: 500;
}

.check-icon {
  width: 16px;
  height: 16px;
}

.installed-list {
  background: var(--color-bg-elevated);
  border-top: 1px solid var(--color-border);
}

.installed-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px 10px 32px;
  border-bottom: 1px solid var(--color-border);
  transition: background 0.15s ease;
}

.installed-row:last-child {
  border-bottom: none;
}

.installed-row:hover {
  background: rgba(255, 255, 255, 0.02);
}

.installed-row.has-update {
  background: rgba(59, 130, 246, 0.03);
}

.installed-version-name {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-family: var(--font-mono);
  color: var(--color-text);
}

.btn-uninstall {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-uninstall:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
  color: #ef4444;
}

.btn-uninstall svg {
  width: 16px;
  height: 16px;
}
</style>
