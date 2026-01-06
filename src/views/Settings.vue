<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import PageHeader from '../components/PageHeader.vue'
import { useSitesStore } from '../stores/sites'

const sitesStore = useSitesStore()

interface AppConfig {
  harvest_home: string
  harvest_paths: string[]
  editor_command: string
  terminal_command: string
  first_run_complete: boolean
}

const config = ref<AppConfig>({
  harvest_home: 'C:/harvest',
  harvest_paths: [],
  editor_command: 'code',
  terminal_command: 'wt -d',
  first_run_complete: true
})

const saving = ref(false)
const saved = ref(false)

async function loadConfig() {
  try {
    config.value = await invoke('get_config')
  } catch (error) {
    console.error('Failed to load config:', error)
  }
}

async function saveConfig() {
  saving.value = true
  try {
    await invoke('save_config', { config: config.value })
    saved.value = true
    setTimeout(() => { saved.value = false }, 2000)
  } catch (error) {
    console.error('Failed to save config:', error)
  } finally {
    saving.value = false
  }
}

async function addHarvestPath() {
  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath: config.value.harvest_paths[0] || 'C:/'
  })
  if (selected && !config.value.harvest_paths.includes(selected as string)) {
    config.value.harvest_paths.push(selected as string)
    await saveConfig()
    // Refresh sites to pick up the new path
    await sitesStore.fetchSites()
  }
}

async function removeHarvestPath(index: number) {
  if (config.value.harvest_paths.length > 1) {
    config.value.harvest_paths.splice(index, 1)
    await saveConfig()
    // Refresh sites to reflect removed path
    await sitesStore.fetchSites()
  }
}

onMounted(() => {
  loadConfig()
})
</script>

<template>
  <div class="page-layout">
    <PageHeader title="Settings" subtitle="Configuration" />

    <div class="page-content">
      <div class="settings-sections">
        <!-- Harvest Home -->
        <section class="settings-section">
          <div class="section-header">
            <svg class="section-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
              <polyline points="9 22 9 12 15 12 15 22" />
            </svg>
            <div>
              <h2 class="section-title">Harvest Home</h2>
              <p class="section-desc">Where Harvest stores services and configuration</p>
            </div>
          </div>
          <div class="path-item readonly">
            <span class="path-text">{{ config.harvest_home }}</span>
            <span class="path-badge">Read-only</span>
          </div>
          <div class="input-hint">
            Change this by running setup wizard again or editing the config file directly.
          </div>
        </section>

        <!-- Sites Paths -->
        <section class="settings-section">
          <div class="section-header">
            <svg class="section-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            <div>
              <h2 class="section-title">Sites Directories</h2>
              <p class="section-desc">Directories where your local sites are stored</p>
            </div>
          </div>
          <div class="paths-list">
            <div
              v-for="(path, index) in config.harvest_paths"
              :key="index"
              class="path-item"
            >
              <span class="path-text">{{ path }}</span>
              <button
                v-if="config.harvest_paths.length > 1"
                class="btn-remove"
                @click="removeHarvestPath(index)"
                title="Remove path"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>
          </div>
          <button class="btn-add" @click="addHarvestPath">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            Add Path
          </button>
        </section>

        <!-- Editor -->
        <section class="settings-section">
          <div class="section-header">
            <svg class="section-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
            </svg>
            <div>
              <h2 class="section-title">Code Editor</h2>
              <p class="section-desc">Command to open sites in your preferred editor</p>
            </div>
          </div>
          <div class="input-group">
            <input
              type="text"
              class="text-input"
              v-model="config.editor_command"
              placeholder="code"
              @change="saveConfig"
            />
          </div>
          <div class="input-hint">
            Examples: <code>code</code> (VS Code), <code>cursor</code>, <code>phpstorm</code>, <code>subl</code>
          </div>
        </section>

        <!-- Terminal -->
        <section class="settings-section">
          <div class="section-header">
            <svg class="section-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <polyline points="4 17 10 11 4 5" />
              <line x1="12" y1="19" x2="20" y2="19" />
            </svg>
            <div>
              <h2 class="section-title">Terminal</h2>
              <p class="section-desc">Command to open terminal in site directory</p>
            </div>
          </div>
          <div class="input-group">
            <input
              type="text"
              class="text-input"
              v-model="config.terminal_command"
              placeholder="wt -d"
              @change="saveConfig"
            />
          </div>
          <div class="input-hint">
            Examples: <code>wt -d</code> (Windows Terminal), <code>cmd /k cd</code>
          </div>
        </section>

      </div>
    </div>

    <!-- Save indicator -->
    <Transition name="fade">
      <div v-if="saved" class="save-indicator">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="20 6 9 17 4 12" />
        </svg>
        Settings saved
      </div>
    </Transition>
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

.settings-sections {
  display: flex;
  flex-direction: column;
  gap: 24px;
  max-width: 600px;
}

.settings-section {
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 20px;
}

.section-header {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  margin-bottom: 16px;
}

.section-icon {
  width: 24px;
  height: 24px;
  color: var(--color-accent);
  flex-shrink: 0;
  margin-top: 2px;
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 4px 0;
}

.section-desc {
  font-size: 13px;
  color: var(--color-text-muted);
  margin: 0;
}

.paths-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.path-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
}

.path-text {
  flex: 1;
  font-size: 13px;
  font-family: var(--font-mono);
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.path-item.readonly {
  background: var(--color-bg-elevated);
  border-style: dashed;
}

.path-badge {
  font-size: 11px;
  padding: 4px 8px;
  background: var(--color-bg);
  border-radius: 4px;
  color: var(--color-text-muted);
}

.btn-remove {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: none;
  border-radius: 6px;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-remove:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.btn-remove svg {
  width: 16px;
  height: 16px;
}

.btn-add {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 10px;
  background: var(--color-bg);
  border: 1px dashed var(--color-border);
  border-radius: 8px;
  color: var(--color-text-muted);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-add:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.btn-add svg {
  width: 16px;
  height: 16px;
}

.input-group {
  display: flex;
  gap: 10px;
}

.text-input {
  flex: 1;
  padding: 10px 14px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  color: var(--color-text);
  font-size: 13px;
  font-family: var(--font-mono);
  transition: all 0.2s ease;
}

.text-input:focus {
  outline: none;
  border-color: var(--color-accent);
  box-shadow: 0 0 0 3px rgba(245, 158, 11, 0.1);
}

.select-input {
  flex: 1;
  padding: 10px 14px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  color: var(--color-text);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.select-input:focus {
  outline: none;
  border-color: var(--color-accent);
}

.input-hint {
  margin-top: 10px;
  font-size: 12px;
  color: var(--color-text-muted);
}

.input-hint code {
  background: var(--color-bg-elevated);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 11px;
}

.save-indicator {
  position: fixed;
  bottom: 24px;
  right: 24px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  background: var(--color-accent);
  border-radius: 8px;
  color: #000;
  font-size: 13px;
  font-weight: 500;
  box-shadow: 0 4px 12px rgba(245, 158, 11, 0.3);
}

.save-indicator svg {
  width: 18px;
  height: 18px;
}

.fade-enter-active,
.fade-leave-active {
  transition: all 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(10px);
}
</style>
