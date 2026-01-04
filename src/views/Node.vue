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

interface NodeVersion {
  version: string
  installed: boolean
  installedVersion?: string
  latestVersion?: string
  hasUpdate?: boolean
  installing?: boolean
  progress?: number
  status?: string
  path?: string
  lts?: boolean
  downloadUrl?: string
}

interface NodeVersionGroup {
  majorVersion: string
  lts: boolean
  versions: NodeVersion[]
  latestVersion?: string
  hasUpdate: boolean
  isExpanded: boolean
}

interface PackageManager {
  name: string
  command: string
  installed: boolean
  version: string | null
  description: string
}

const versions = ref<NodeVersion[]>([])
const versionGroups = ref<NodeVersionGroup[]>([])
const packageManagers = ref<PackageManager[]>([])
const loading = ref(true)
const loadingPM = ref(true)
const error = ref<string | null>(null)
const defaultPath = ref<string | null>(null)
let unlisten: UnlistenFn | null = null

const installedCount = computed(() => versions.value.filter(v => v.installed).length)
const updatesAvailable = computed(() => versions.value.filter(v => v.hasUpdate).length)

async function loadDefaultPath() {
  try {
    defaultPath.value = await invoke<string | null>('get_default_node_path')
  } catch (e) {
    console.error('Failed to get default Node path:', e)
    defaultPath.value = null
  }
}

async function setAsDefault(version: NodeVersion) {
  if (!version.path) return
  try {
    // Remove old path if exists
    if (defaultPath.value) {
      await invoke('remove_from_path', { path: defaultPath.value })
    }
    // Add new path
    await invoke('add_to_path', { path: version.path })
    await loadDefaultPath()
  } catch (e) {
    console.error('Failed to set as default:', e)
    error.value = `Failed to set Node ${version.installedVersion || version.version} as default: ${e}`
  }
}

async function removeFromPath(version: NodeVersion) {
  if (!version.path) return
  try {
    await invoke('remove_from_path', { path: version.path })
    await loadDefaultPath()
  } catch (e) {
    console.error('Failed to remove from PATH:', e)
    error.value = `Failed to remove Node ${version.installedVersion || version.version} from PATH: ${e}`
  }
}

function isDefaultVersion(version: NodeVersion): boolean {
  if (!defaultPath.value || !version.path) return false
  return defaultPath.value.toLowerCase().replace(/\\/g, '/') === version.path.toLowerCase().replace(/\\/g, '/')
}

async function loadVersions() {
  loading.value = true
  error.value = null
  try {
    // Fetch both installed and available versions in parallel
    const [installed, available] = await Promise.all([
      invoke<{ version: string; path: string }[]>('get_installed_node_versions'),
      invoke<AvailableVersion[]>('get_available_node_versions')
    ])

    // Build versions list from available versions
    versions.value = available.map(av => {
      const found = installed.find(i => i.version.startsWith(av.version))
      const hasUpdate = found && compareVersions(av.full_version, found.version) > 0
      return {
        version: av.version,
        installed: !!found,
        installedVersion: found?.version,
        latestVersion: av.full_version,
        hasUpdate,
        path: found?.path,
        installing: false,
        progress: 0,
        status: '',
        lts: av.lts,
        downloadUrl: av.download_url
      }
    })

    // Build grouped versions (group by major version)
    const groups = new Map<string, NodeVersionGroup>()

    for (const v of versions.value) {
      const majorVersion = v.version.split('.')[0]

      if (!groups.has(majorVersion)) {
        groups.set(majorVersion, {
          majorVersion,
          lts: v.lts || false,
          versions: [],
          latestVersion: v.latestVersion,
          hasUpdate: false,
          isExpanded: false
        })
      }

      const group = groups.get(majorVersion)!
      group.versions.push(v)
      if (v.hasUpdate) group.hasUpdate = true
      if (v.lts) group.lts = true
    }

    // Sort groups by major version descending
    versionGroups.value = Array.from(groups.values()).sort(
      (a, b) => parseInt(b.majorVersion) - parseInt(a.majorVersion)
    )

    // Auto-expand groups with installed versions
    for (const group of versionGroups.value) {
      if (group.versions.some(v => v.installed)) {
        group.isExpanded = true
      }
    }
  } catch (e) {
    console.error('Failed to load Node versions:', e)
    error.value = String(e)
    // Fallback to hardcoded versions
    const fallbackVersions = [
      { version: '23', lts: false },
      { version: '22', lts: true },
      { version: '21', lts: false },
      { version: '20', lts: true },
      { version: '18', lts: true },
      { version: '16', lts: true },
    ]
    versions.value = fallbackVersions.map(v => ({
      version: v.version,
      installed: false,
      installing: false,
      progress: 0,
      status: '',
      lts: v.lts
    }))

    // Build fallback groups
    versionGroups.value = fallbackVersions.map(v => ({
      majorVersion: v.version,
      lts: v.lts,
      versions: [{
        version: v.version,
        installed: false,
        installing: false,
        progress: 0,
        status: '',
        lts: v.lts
      }],
      hasUpdate: false,
      isExpanded: false
    }))
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

function toggleGroup(group: NodeVersionGroup) {
  group.isExpanded = !group.isExpanded
}

function getInstalledVersionsInGroup(group: NodeVersionGroup): NodeVersion[] {
  return group.versions.filter(v => v.installed)
}

async function installVersion(version: NodeVersion) {
  version.installing = true
  version.progress = 0
  version.status = 'starting'
  error.value = null
  try {
    await invoke('install_node_version', { version: version.version })
    await loadVersions()
  } catch (e) {
    console.error('Failed to install Node:', e)
    error.value = `Failed to install Node ${version.version}: ${e}`
    version.installing = false
    version.progress = 0
    version.status = ''
  }
}

async function uninstallVersion(version: NodeVersion) {
  if (!confirm(`Are you sure you want to uninstall Node ${version.installedVersion || version.version}?`)) {
    return
  }
  try {
    // If this is the default version, remove from PATH first
    if (isDefaultVersion(version)) {
      await removeFromPath(version)
    }
    await invoke('uninstall_node_version', { version: version.version })
    await loadVersions()
  } catch (e) {
    console.error('Failed to uninstall Node:', e)
    error.value = `Failed to uninstall Node ${version.version}: ${e}`
  }
}

// Package Manager functions
async function loadPackageManagers() {
  loadingPM.value = true
  try {
    const managers = await invoke<PackageManager[]>('get_package_managers')
    packageManagers.value = managers.map(m => ({ ...m, loading: false }))
  } catch (e) {
    console.error('Failed to load package managers:', e)
  } finally {
    loadingPM.value = false
  }
}


onMounted(async () => {
  await Promise.all([loadVersions(), loadPackageManagers(), loadDefaultPath()])

  // Listen for progress events
  unlisten = await listen<{ type: string; version: string; status: string; progress: number }>('install-progress', (event) => {
    if (event.payload.type !== 'node') return

    const version = versions.value.find(v => v.version === event.payload.version)
    if (version) {
      version.progress = event.payload.progress
      version.status = event.payload.status

      if (event.payload.status === 'complete') {
        version.installing = false
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
    <PageHeader title="Node" :subtitle="`${installedCount} installed${updatesAvailable > 0 ? ` · ${updatesAvailable} update${updatesAvailable > 1 ? 's' : ''} available` : ''}`" />

    <div class="page-content">
      <!-- Error Banner -->
      <div v-if="error" class="error-banner">
        <span>{{ error }}</span>
        <button class="error-dismiss" @click="error = null">&times;</button>
      </div>

      <!-- Versions Section -->
      <section class="content-section">
        <h2 class="section-title">Versions</h2>
        <p class="section-desc">Manage Node.js versions for your projects. Each project can use a different version.</p>

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
            <template v-for="group in versionGroups" :key="group.majorVersion">
              <!-- Group Header -->
              <div
                class="table-row group-header"
                :class="{
                  'has-installed': group.versions.some(v => v.installed),
                  'has-update': group.hasUpdate
                }"
                @click="toggleGroup(group)"
              >
                <span class="col-version">
                  <span class="expand-icon" :class="{ expanded: group.isExpanded }">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M9 18l6-6-6-6" />
                    </svg>
                  </span>
                  Node {{ group.majorVersion }}
                  <span v-if="group.lts" class="lts-badge">LTS</span>
                  <span v-if="getInstalledVersionsInGroup(group).length > 0" class="installed-count">
                    {{ getInstalledVersionsInGroup(group).length }} installed
                  </span>
                  <span v-if="group.hasUpdate" class="update-badge">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M12 19V5M5 12l7-7 7 7" />
                    </svg>
                    Update
                  </span>
                </span>
                <span class="col-status">
                  <span v-if="group.versions.some(v => v.installed)" class="status-installed">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <polyline points="20 6 9 17 4 12" />
                    </svg>
                  </span>
                </span>
              </div>

              <!-- Expanded Version List -->
              <template v-if="group.isExpanded">
                <div
                  v-for="version in group.versions"
                  :key="version.version"
                  class="table-row version-row"
                  :class="{
                    installed: version.installed,
                    'has-update': version.hasUpdate,
                    'is-default': isDefaultVersion(version)
                  }"
                >
                  <span class="col-version">
                    <span class="version-indent"></span>
                    {{ version.latestVersion || version.version }}
                    <span v-if="version.lts" class="lts-badge">LTS</span>
                    <span v-if="isDefaultVersion(version)" class="default-badge">PATH</span>
                    <span v-if="version.installedVersion && version.installedVersion !== version.latestVersion" class="installed-version">
                      (installed: {{ version.installedVersion }})
                    </span>
                    <span v-if="version.hasUpdate" class="update-badge" :title="`Update to ${version.latestVersion}`">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M12 19V5M5 12l7-7 7 7" />
                      </svg>
                      {{ version.latestVersion }}
                    </span>
                  </span>
                  <span class="col-status">
                    <div v-if="version.installing" class="install-progress">
                      <div class="progress-bar">
                        <div class="progress-fill" :style="{ width: `${version.progress}%` }" />
                      </div>
                      <span class="progress-text">{{ version.status === 'extracting' ? 'Extracting...' : `${version.progress}%` }}</span>
                    </div>
                    <template v-else-if="version.installed">
                      <div class="installed-row-actions">
                        <button
                          v-if="version.hasUpdate"
                          class="btn-update"
                          @click.stop="installVersion(version)"
                        >
                          Update
                        </button>
                        <template v-else>
                          <button
                            v-if="!isDefaultVersion(version)"
                            class="btn-default"
                            @click.stop="setAsDefault(version)"
                            title="Add to PATH"
                          >
                            Set as Default
                          </button>
                          <button
                            v-else
                            class="btn-remove-path"
                            @click.stop="removeFromPath(version)"
                            title="Remove from PATH"
                          >
                            Remove
                          </button>
                        </template>
                        <button
                          class="btn-uninstall"
                          @click.stop="uninstallVersion(version)"
                          title="Uninstall"
                        >
                          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                            <path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                          </svg>
                        </button>
                      </div>
                    </template>
                    <button
                      v-else
                      class="btn-install"
                      @click.stop="installVersion(version)"
                    >
                      Install
                    </button>
                  </span>
                </div>
              </template>
            </template>
          </div>
        </div>
      </section>

      <!-- Package Managers Section -->
      <section class="content-section">
        <h2 class="section-title">Package Managers</h2>
        <p class="section-desc">Manage npm, yarn, pnpm, and bun for your Node.js environment.</p>

        <div v-if="loadingPM" class="loading-state">
          <div class="loading-spinner" />
          <span>Detecting package managers...</span>
        </div>

        <div v-else class="pm-grid">
          <div
            v-for="pm in packageManagers"
            :key="pm.name"
            class="pm-card"
            :class="{ installed: pm.installed }"
          >
            <div class="pm-header">
              <div class="pm-icon">
                <svg v-if="pm.name === 'npm'" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M0 7.334v8h6.666v1.332H12v-1.332h12v-8H0zm6.666 6.664H5.334v-4H3.999v4H1.335V8.667h5.331v5.331zm4 0v1.336H8.001V8.667h5.334v5.332h-2.669v-.001zm12.001 0h-1.33v-4h-1.336v4h-1.335v-4h-1.33v4h-2.671V8.667h8.002v5.331zM10.665 10H12v2.667h-1.335V10z"/>
                </svg>
                <svg v-else-if="pm.name === 'yarn'" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 0C5.375 0 0 5.375 0 12s5.375 12 12 12 12-5.375 12-12S18.625 0 12 0zm.768 4.105c.183 0 .363.053.525.157.125.083.287.185.755 1.154.31-.088.468-.042.551-.019.204.056.366.19.463.375.477.917.542 2.553.334 3.605-.241 1.232-.755 2.029-1.131 2.576.324.329.778.899 1.117 1.825.278.774.31 1.478.273 2.015a5.51 5.51 0 0 0 .602-.329c.593-.366 1.487-.917 2.553-.931.714-.009 1.269.445 1.353 1.103a1.23 1.23 0 0 1-.945 1.362c-.649.158-.95.278-1.821.843-1.232.799-2.539 1.242-3.012 1.39a1.686 1.686 0 0 1-.704.343c-.737.181-3.266.315-3.466.315h-.046c-.783 0-1.214-.241-1.45-.491-.658.329-1.51.19-2.122-.134a1.078 1.078 0 0 1-.58-1.153 1.243 1.243 0 0 1-.153-.195c-.162-.25-.528-.936-.454-1.946.056-.723.556-1.367.88-1.71a5.522 5.522 0 0 1 .408-2.256c.306-.727.885-1.348 1.32-1.737-.32-.537-.644-1.367-.329-2.21.227-.602.412-.936.82-1.08h-.005c.199-.074.389-.153.486-.259a3.418 3.418 0 0 1 2.298-1.103c.037-.093.079-.185.125-.283.31-.658.639-1.029 1.024-1.168a.94.94 0 0 1 .328-.06z"/>
                </svg>
                <svg v-else-if="pm.name === 'pnpm'" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M0 0v7.5h7.5V0zm8.25 0v7.5h7.498V0zm8.25 0v7.5H24V0zM8.25 8.25v7.5h7.498v-7.5zm8.25 0v7.5H24v-7.5zm-8.25 8.25v7.5h7.498V16.5zm8.25 0v7.5H24v-7.5z"/>
                </svg>
                <svg v-else-if="pm.name === 'bun'" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 22.596c6.628 0 12-4.338 12-9.688 0-3.318-2.057-6.248-5.219-7.986-.424 2.5-2.673 4.406-5.373 4.406-1.586 0-3.021-.651-4.051-1.7a5.455 5.455 0 0 1-4.051 1.7C2.616 9.328.373 7.442.03 4.98.01 5.1 0 5.22 0 5.34c0 5.35 5.372 9.688 12 9.688z"/>
                </svg>
              </div>
              <div class="pm-info">
                <h3 class="pm-name">{{ pm.name }}</h3>
                <span v-if="pm.installed" class="pm-version">v{{ pm.version }}</span>
                <span v-else class="pm-not-installed">Not installed</span>
              </div>
              <span v-if="pm.installed" class="pm-status-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
              </span>
            </div>
            <p class="pm-desc">{{ pm.description }}</p>
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

.table-row.is-default {
  background: rgba(245, 158, 11, 0.08);
  border-left: 3px solid var(--color-accent);
}

/* Group Header Styles */
.group-header {
  cursor: pointer;
  background: var(--color-bg-elevated);
  font-weight: 500;
}

.group-header:hover {
  background: var(--color-bg-card);
}

.group-header.has-installed {
  background: rgba(245, 158, 11, 0.05);
}

.expand-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  margin-right: 4px;
  transition: transform 0.2s ease;
}

.expand-icon svg {
  width: 14px;
  height: 14px;
}

.expand-icon.expanded {
  transform: rotate(90deg);
}

.version-row {
  padding-left: 24px;
}

.version-indent {
  display: inline-block;
  width: 16px;
}

.installed-count {
  font-size: 11px;
  color: var(--color-accent);
  margin-left: 8px;
}

.col-version {
  flex: 1;
  font-size: 14px;
  color: var(--color-text);
  display: flex;
  align-items: center;
  gap: 8px;
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

.default-badge {
  padding: 2px 6px;
  background: rgba(245, 158, 11, 0.2);
  color: var(--color-accent);
  font-size: 10px;
  font-weight: 600;
  border-radius: 4px;
  text-transform: uppercase;
}

.installed-version {
  font-size: 12px;
  color: var(--color-text-muted);
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
  width: 200px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
}

.installed-row-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-default {
  padding: 6px 12px;
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  border-radius: 6px;
  color: var(--color-accent);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.btn-default:hover {
  background: rgba(245, 158, 11, 0.2);
  border-color: var(--color-accent);
}

.btn-remove-path {
  padding: 6px 12px;
  background: transparent;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.btn-remove-path:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
  color: #ef4444;
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

/* Package Managers Grid */
.pm-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.pm-card {
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 16px;
  transition: all 0.15s ease;
}

.pm-card:hover {
  border-color: var(--color-border-light, #3a3a44);
}

.pm-card.installed {
  border-color: rgba(245, 158, 11, 0.3);
  background: rgba(245, 158, 11, 0.03);
}

.pm-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.pm-status-icon {
  margin-left: auto;
  color: var(--color-accent);
}

.pm-status-icon svg {
  width: 18px;
  height: 18px;
}

.pm-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-elevated);
  border-radius: 8px;
  color: var(--color-text-muted);
}

.pm-card.installed .pm-icon {
  color: var(--color-accent);
  background: rgba(245, 158, 11, 0.1);
}

.pm-icon svg {
  width: 20px;
  height: 20px;
}

.pm-info {
  flex: 1;
}

.pm-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
  text-transform: capitalize;
}

.pm-version {
  font-size: 12px;
  color: var(--color-accent);
  font-family: var(--font-mono);
}

.pm-not-installed {
  font-size: 12px;
  color: var(--color-text-muted);
}

.pm-desc {
  font-size: 12px;
  color: var(--color-text-muted);
  margin: 0;
  line-height: 1.4;
}

.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 8px;
  color: #ef4444;
  font-size: 13px;
  margin-bottom: 16px;
}

.error-dismiss {
  background: none;
  border: none;
  color: #ef4444;
  font-size: 18px;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
}

.error-dismiss:hover {
  opacity: 0.7;
}
</style>
