<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from 'vue'
import { useServicesStore, type InstallableService } from '../stores/services'
import PageHeader from '../components/PageHeader.vue'
import ServiceCard from '../components/ServiceCard.vue'

const store = useServicesStore()
const refreshInterval = ref<number | null>(null)
const activeTab = ref<'status' | 'install'>('status')
const selectedVersions = ref<Record<string, string>>({})

onMounted(async () => {
  await Promise.all([
    store.fetchServices(),
    store.fetchInstallableServices()
  ])
  // Poll for status updates every 5 seconds
  refreshInterval.value = window.setInterval(() => {
    store.fetchServices()
  }, 5000)

  // Set default selected versions
  store.installableServices.forEach(s => {
    if (s.available_versions.length > 0) {
      selectedVersions.value[s.id] = s.available_versions[0].version
    }
  })
})

onUnmounted(() => {
  if (refreshInterval.value) {
    clearInterval(refreshInterval.value)
  }
})

const installedServices = computed(() => store.services.filter(s => s.status !== 'not_installed'))
const notInstalledServices = computed(() => store.services.filter(s => s.status === 'not_installed'))

async function handleInstall(serviceId: string) {
  const version = selectedVersions.value[serviceId]
  if (version) {
    await store.installService(serviceId, version)
  }
}

async function handleUninstall(serviceId: string) {
  if (confirm(`Are you sure you want to uninstall ${serviceId}? This will remove all service files.`)) {
    await store.uninstallService(serviceId)
  }
}
</script>

<template>
  <div class="page-layout">
    <PageHeader title="Services" subtitle="Manage local development services" />

    <!-- Tabs -->
    <div class="tabs-container">
      <div class="tabs">
        <button
          class="tab"
          :class="{ active: activeTab === 'status' }"
          @click="activeTab = 'status'"
        >
          Status
        </button>
        <button
          class="tab"
          :class="{ active: activeTab === 'install' }"
          @click="activeTab = 'install'"
        >
          Install Services
        </button>
      </div>
    </div>

    <div class="page-content">
      <!-- Error Banner -->
      <div v-if="store.error" class="error-banner">
        {{ store.error }}
      </div>

      <!-- Status Tab -->
      <div v-if="activeTab === 'status'">
        <!-- Installed Services -->
        <div v-if="!store.initialLoading && installedServices.length > 0" class="section">
          <div class="services-grid">
            <ServiceCard
              v-for="service in installedServices"
              :key="service.id"
              :service="service"
              :loading="store.isLoading(service.id)"
              :icon="store.getIcon(service.id)"
              @toggle="store.toggleService"
            />
          </div>
        </div>

        <!-- Not Installed Services -->
        <div v-if="!store.initialLoading && notInstalledServices.length > 0" class="section">
          <h3 class="section-title">Not Installed</h3>
          <p class="section-hint">Go to "Install Services" tab to download and install services.</p>
          <div class="services-grid">
            <ServiceCard
              v-for="service in notInstalledServices"
              :key="service.id"
              :service="service"
              :loading="false"
              :icon="store.getIcon(service.id)"
              @toggle="store.toggleService"
            />
          </div>
        </div>

        <!-- Loading State -->
        <div v-if="store.initialLoading" class="loading-state">
          <div class="loading-spinner"></div>
          <p>Loading services...</p>
        </div>

        <!-- Empty State (after loading) -->
        <div v-else-if="store.services.length === 0" class="empty-state">
          <span class="empty-icon">⚙️</span>
          <p>No services found</p>
          <button class="btn-primary" @click="activeTab = 'install'">Install Services</button>
        </div>
      </div>

      <!-- Install Tab -->
      <div v-if="activeTab === 'install'" class="install-section">
        <div class="install-list">
          <div
            v-for="service in store.installableServices"
            :key="service.id"
            class="install-card"
          >
            <div class="install-card-header">
              <span class="service-icon">{{ store.getIcon(service.id) }}</span>
              <div class="service-info">
                <h3>{{ service.name }}</h3>
                <p>{{ service.description }}</p>
              </div>
              <div class="status-badge" :class="{ installed: service.installed }">
                {{ service.installed ? `v${service.version}` : 'Not Installed' }}
              </div>
            </div>

            <!-- Installing Progress -->
            <div v-if="store.isInstalling(service.id)" class="install-progress">
              <div class="progress-bar">
                <div
                  class="progress-fill"
                  :style="{ width: `${store.getInstallProgress(service.id)?.progress || 0}%` }"
                ></div>
              </div>
              <span class="progress-text">
                {{ store.getInstallProgress(service.id)?.status === 'downloading' ? 'Downloading...' : 'Extracting...' }}
                {{ store.getInstallProgress(service.id)?.progress || 0 }}%
              </span>
            </div>

            <!-- Actions -->
            <div v-else class="install-card-actions">
              <template v-if="!service.installed">
                <select
                  v-model="selectedVersions[service.id]"
                  class="version-select"
                >
                  <option
                    v-for="ver in service.available_versions"
                    :key="ver.version"
                    :value="ver.version"
                  >
                    v{{ ver.version }}
                  </option>
                </select>
                <button
                  class="btn-install"
                  @click="handleInstall(service.id)"
                  :disabled="store.isInstalling(service.id)"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                    <polyline points="7 10 12 15 17 10" />
                    <line x1="12" y1="15" x2="12" y2="3" />
                  </svg>
                  Install
                </button>
              </template>
              <template v-else>
                <button
                  class="btn-uninstall"
                  @click="handleUninstall(service.id)"
                  :disabled="store.isInstalling(service.id)"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="3 6 5 6 21 6" />
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                  </svg>
                  Uninstall
                </button>
              </template>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.tabs-container {
  padding: 0 32px;
  border-bottom: 1px solid var(--color-border);
}

.tabs {
  display: flex;
  gap: 4px;
}

.tab {
  padding: 12px 20px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--color-text-muted);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab:hover {
  color: var(--color-text);
}

.tab.active {
  color: var(--color-accent);
  border-bottom-color: var(--color-accent);
}

.page-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px 32px 32px;
}

.error-banner {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #ef4444;
  padding: 12px 16px;
  border-radius: 8px;
  margin-bottom: 20px;
  font-size: 14px;
}

.section {
  margin-bottom: 32px;
}

.section-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-muted);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.section-hint {
  font-size: 13px;
  color: var(--color-text-muted);
  margin-bottom: 16px;
}

.services-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 600px;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--color-text-muted);
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--color-text-muted);
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.btn-primary {
  margin-top: 16px;
  padding: 10px 20px;
  background: var(--color-accent);
  border: none;
  border-radius: 8px;
  color: #000;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s ease;
}

.btn-primary:hover {
  background: #d97706;
}

/* Install Section */
.install-section {
  max-width: 700px;
}

.install-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.install-card {
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 20px;
}

.install-card-header {
  display: flex;
  align-items: center;
  gap: 14px;
}

.service-icon {
  font-size: 28px;
}

.service-info {
  flex: 1;
}

.service-info h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 4px;
}

.service-info p {
  font-size: 13px;
  color: var(--color-text-muted);
  margin: 0;
}

.status-badge {
  padding: 6px 12px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-muted);
}

.status-badge.installed {
  background: rgba(34, 197, 94, 0.1);
  border-color: rgba(34, 197, 94, 0.3);
  color: #22c55e;
}

.install-progress {
  margin-top: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
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
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 12px;
  color: var(--color-text-muted);
  min-width: 120px;
}

.install-card-actions {
  margin-top: 16px;
  display: flex;
  gap: 10px;
}

.version-select {
  padding: 8px 12px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  color: var(--color-text);
  font-size: 13px;
  cursor: pointer;
}

.version-select:focus {
  outline: none;
  border-color: var(--color-accent);
}

.btn-install {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--color-accent);
  border: none;
  border-radius: 6px;
  color: #000;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s ease;
}

.btn-install:hover:not(:disabled) {
  background: #d97706;
}

.btn-install:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-install svg {
  width: 16px;
  height: 16px;
}

.btn-uninstall {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
  color: #ef4444;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-uninstall:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.2);
}

.btn-uninstall:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-uninstall svg {
  width: 16px;
  height: 16px;
}
</style>
