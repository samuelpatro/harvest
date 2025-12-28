<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'

defineProps<{
  title: string
  subtitle?: string
}>()

const appWindow = getCurrentWindow()

async function minimize() {
  await appWindow.minimize()
}

async function toggleMaximize() {
  await appWindow.toggleMaximize()
}

async function close() {
  await appWindow.close()
}
</script>

<template>
  <header class="page-header" data-tauri-drag-region>
    <div class="header-left">
      <h1 class="page-title">{{ title }}</h1>
      <span v-if="subtitle" class="page-subtitle">{{ subtitle }}</span>
    </div>
    <div class="header-center">
      <slot />
    </div>
    <div class="window-controls">
      <button class="control-btn" @click="minimize" title="Minimize">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect fill="currentColor" width="10" height="1" x="1" y="6" />
        </svg>
      </button>
      <button class="control-btn" @click="toggleMaximize" title="Maximize">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect stroke="currentColor" fill="none" width="9" height="9" x="1.5" y="1.5" stroke-width="1" />
        </svg>
      </button>
      <button class="control-btn close" @click="close" title="Close">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <path fill="currentColor" d="M6.707 6l3.147-3.146a.5.5 0 00-.708-.708L6 5.293 2.854 2.146a.5.5 0 10-.708.708L5.293 6l-3.147 3.146a.5.5 0 00.708.708L6 6.707l3.146 3.147a.5.5 0 00.708-.708L6.707 6z" />
        </svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.page-header {
  display: flex;
  align-items: center;
  height: 68px;
  padding: 0 0 0 32px;
  background: var(--color-bg);
  border-bottom: 1px solid var(--color-border);
  -webkit-app-region: drag;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: baseline;
  gap: 12px;
  -webkit-app-region: no-drag;
}

.page-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.page-subtitle {
  font-size: 13px;
  color: var(--color-text-muted);
  background: var(--color-bg-elevated);
  padding: 4px 10px;
  border-radius: 10px;
}

.header-center {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  padding: 0 16px;
  -webkit-app-region: no-drag;
}

.window-controls {
  display: flex;
  height: 100%;
  -webkit-app-region: no-drag;
}

.control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 100%;
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.control-btn:hover {
  background-color: var(--color-bg-elevated);
  color: var(--color-text);
}

.control-btn.close:hover {
  background-color: #e81123;
  color: white;
}
</style>
