<script setup lang="ts">
import type { Service } from '../stores/services'

defineProps<{
  service: Service
  loading?: boolean
  icon: string
}>()

const emit = defineEmits<{
  toggle: [id: string]
}>()

function getStatusLabel(status: string): string {
  switch (status) {
    case 'running': return 'Running'
    case 'stopped': return 'Stopped'
    case 'not_installed': return 'Not Installed'
    default: return status
  }
}

function getStatusClass(status: string): string {
  switch (status) {
    case 'running': return 'status-running'
    case 'stopped': return 'status-stopped'
    case 'not_installed': return 'status-not-installed'
    default: return ''
  }
}

function formatPath(path: string | null): string {
  if (!path) return ''
  // Shorten the path for display
  return path.replace(/\\/g, '/').replace(/^C:\/harvest\/bin\//, '')
}
</script>

<template>
  <div
    class="service-row"
    :class="{ 'not-installed': service.status === 'not_installed' }"
  >
    <div class="service-main">
      <span class="service-icon">{{ icon }}</span>

      <div class="service-info">
        <div class="service-header">
          <h3 class="service-name">{{ service.name }}</h3>
          <span v-if="service.version" class="service-version">v{{ service.version }}</span>
          <span class="service-port">:{{ service.port }}</span>
        </div>
        <p class="service-desc">{{ service.description }}</p>
        <div v-if="service.path && service.status !== 'not_installed'" class="service-meta">
          <span class="service-path">{{ formatPath(service.path) }}</span>
          <span v-if="service.status === 'running' && service.pid" class="service-pid">
            PID {{ service.pid }}
          </span>
        </div>
      </div>
    </div>

    <div class="service-actions">
      <span class="status-badge" :class="getStatusClass(service.status)">
        {{ getStatusLabel(service.status) }}
      </span>

      <button
        v-if="service.status !== 'not_installed'"
        @click="emit('toggle', service.id)"
        :disabled="loading"
        class="toggle-btn"
        :class="{
          'btn-stop': service.status === 'running',
          'btn-start': service.status === 'stopped',
          'btn-loading': loading
        }"
      >
        <span v-if="loading" class="spinner"></span>
        <span v-else>{{ service.status === 'running' ? 'Stop' : 'Start' }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.service-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 16px 20px;
  transition: border-color 0.2s;
  gap: 16px;
}

.service-row:hover {
  border-color: var(--color-border-light, #3a3a44);
}

.service-row.not-installed {
  opacity: 0.5;
}

.service-main {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  flex: 1;
  min-width: 0;
}

.service-icon {
  font-size: 28px;
  flex-shrink: 0;
  margin-top: 2px;
}

.service-info {
  flex: 1;
  min-width: 0;
}

.service-header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex-wrap: wrap;
}

.service-name {
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
  font-size: 15px;
}

.service-version {
  font-size: 12px;
  font-family: 'JetBrains Mono', monospace;
  color: var(--color-accent);
  background: rgba(245, 158, 11, 0.1);
  padding: 1px 6px;
  border-radius: 4px;
}

.service-port {
  font-size: 12px;
  font-family: 'JetBrains Mono', monospace;
  color: var(--color-text-muted);
}

.service-desc {
  font-size: 13px;
  color: var(--color-text-muted);
  margin: 4px 0 0 0;
  line-height: 1.4;
}

.service-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 6px;
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  color: var(--color-text-muted);
  opacity: 0.7;
}

.service-path {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.service-pid {
  flex-shrink: 0;
}

.service-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.status-badge {
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.status-running {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.status-stopped {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.status-not-installed {
  background: rgba(113, 113, 122, 0.1);
  color: #71717a;
}

.toggle-btn {
  padding: 6px 16px;
  border-radius: 6px;
  font-weight: 500;
  font-size: 13px;
  border: none;
  cursor: pointer;
  transition: background 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-width: 70px;
}

.toggle-btn:disabled {
  cursor: not-allowed;
  opacity: 0.7;
}

.btn-stop {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.btn-stop:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.2);
}

.btn-start {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.btn-start:hover:not(:disabled) {
  background: rgba(34, 197, 94, 0.2);
}

.btn-loading {
  background: rgba(113, 113, 122, 0.1);
  color: var(--color-text-muted);
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
