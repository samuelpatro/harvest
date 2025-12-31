<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { useServicesStore } from '../stores/services'
import PageHeader from '../components/PageHeader.vue'
import ServiceCard from '../components/ServiceCard.vue'

const store = useServicesStore()
const refreshInterval = ref<number | null>(null)

onMounted(async () => {
  await store.fetchServices()
  // Poll for status updates every 5 seconds
  refreshInterval.value = window.setInterval(() => {
    store.fetchServices()
  }, 5000)
})

onUnmounted(() => {
  if (refreshInterval.value) {
    clearInterval(refreshInterval.value)
  }
})

const installedServices = () => store.services.filter(s => s.status !== 'not_installed')
const notInstalledServices = () => store.services.filter(s => s.status === 'not_installed')
</script>

<template>
  <div class="page-layout">
    <PageHeader title="Services" subtitle="Manage local development services" />

    <div class="page-content">
      <!-- Error Banner -->
      <div v-if="store.error" class="error-banner">
        {{ store.error }}
      </div>

      <!-- Installed Services -->
      <div v-if="!store.initialLoading && installedServices().length > 0" class="section">
        <div class="services-grid">
          <ServiceCard
            v-for="service in installedServices()"
            :key="service.id"
            :service="service"
            :loading="store.isLoading(service.id)"
            :icon="store.getIcon(service.id)"
            @toggle="store.toggleService"
          />
        </div>
      </div>

      <!-- Not Installed Services -->
      <div v-if="!store.initialLoading && notInstalledServices().length > 0" class="section">
        <h3 class="section-title">Not Installed</h3>
        <div class="services-grid">
          <ServiceCard
            v-for="service in notInstalledServices()"
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
  margin-bottom: 16px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
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
</style>
