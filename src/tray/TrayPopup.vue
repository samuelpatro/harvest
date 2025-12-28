<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useServicesStore } from '../stores/services'

const store = useServicesStore()
const selectedPhpVersion = ref('8.4')

const phpVersions = [
  { version: '8.4', label: 'Use PHP 8.4' },
  { version: '8.3', label: 'Use PHP 8.3' }
]

function selectPhpVersion(version: string) {
  selectedPhpVersion.value = version
}

function stopAllServices() {
  store.services.forEach(service => {
    if (service.status === 'running') {
      store.toggleService(service.id)
    }
  })
}

function hidePopup() {
  invoke('hide_tray_popup')
}

function handleBlur() {
  hidePopup()
}

onMounted(async () => {
  window.addEventListener('blur', handleBlur)
  await store.fetchServices()
})

onUnmounted(() => {
  window.removeEventListener('blur', handleBlur)
})
</script>

<template>
  <div class="tray-popup">
    <div class="popup-container">
      <!-- Scrollable Content -->
      <div class="popup-content">
        <!-- Quick Access Section -->
        <div class="section">
        <div class="section-header">Quick Access</div>
        <button class="menu-item">
          <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="7" height="7" rx="1" />
            <rect x="14" y="3" width="7" height="7" rx="1" />
            <rect x="3" y="14" width="7" height="7" rx="1" />
            <rect x="14" y="14" width="7" height="7" rx="1" />
          </svg>
          <span>Services</span>
        </button>
        <button class="menu-item">
          <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="4" width="18" height="16" rx="2" />
            <path d="M3 10h18" />
          </svg>
          <span>Sites</span>
        </button>
      </div>

      <!-- Services Section -->
      <div class="section">
        <div class="section-header">Services</div>
        <button class="menu-item stop-all" @click="stopAllServices">
          <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="6" y="4" width="4" height="16" rx="1" />
            <rect x="14" y="4" width="4" height="16" rx="1" />
          </svg>
          <span>Stop all services</span>
        </button>

        <div class="services-list">
          <div
            v-for="service in store.services"
            :key="service.id"
            class="service-item"
          >
            <div class="service-left">
              <span
                class="status-dot"
                :class="service.status === 'running' ? 'running' : 'stopped'"
              />
              <span class="service-name">{{ service.name }}</span>
            </div>
            <button class="info-btn">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="12" cy="12" r="10" />
                <path d="M12 16v-4M12 8h.01" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- PHP Version Section -->
      <div class="section php-section">
        <button
          v-for="php in phpVersions"
          :key="php.version"
          class="php-item"
          :class="{ active: selectedPhpVersion === php.version }"
          @click="selectPhpVersion(php.version)"
        >
          <svg
            v-if="selectedPhpVersion === php.version"
            class="check-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <polyline points="20 6 9 17 4 12" />
          </svg>
          <span :class="{ 'ml-6': selectedPhpVersion !== php.version }">{{ php.label }}</span>
        </button>
        <button class="menu-item config-link">
          <span>Open configuration files</span>
        </button>
      </div>
      </div>

      <!-- Footer (sticky) -->
      <div class="footer">
        <button class="footer-btn">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
        <button class="footer-btn">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
          </svg>
        </button>
        <button class="footer-btn" @click="hidePopup">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M18.36 6.64a9 9 0 1 1-12.73 0M12 2v10" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tray-popup {
  width: 280px;
  height: 100vh;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  background: #18181b;
  user-select: none;
  overflow: hidden;
}

.popup-container {
  background: #18181b;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.popup-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.popup-content::-webkit-scrollbar {
  width: 6px;
}

.popup-content::-webkit-scrollbar-track {
  background: transparent;
}

.popup-content::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 3px;
}

.popup-content::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.25);
}

.section {
  padding: 8px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.section-header {
  padding: 8px 16px 4px;
  font-size: 11px;
  font-weight: 500;
  color: #71717a;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 16px;
  font-size: 13px;
  color: #fafafa;
  background: none;
  border: none;
  cursor: pointer;
  transition: background-color 0.15s ease;
  text-align: left;
}

.menu-item:hover {
  background-color: rgba(255, 255, 255, 0.06);
}

.menu-icon {
  width: 18px;
  height: 18px;
  color: #a1a1aa;
  flex-shrink: 0;
}

.stop-all {
  color: #fafafa;
}

.services-list {
  padding: 4px 0;
}

.service-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  transition: background-color 0.15s ease;
}

.service-item:hover {
  background-color: rgba(255, 255, 255, 0.06);
}

.service-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.running {
  background-color: #22c55e;
  box-shadow: 0 0 8px rgba(34, 197, 94, 0.5);
}

.status-dot.stopped {
  background-color: #ef4444;
  box-shadow: 0 0 8px rgba(239, 68, 68, 0.4);
}

.service-name {
  font-size: 13px;
  color: #fafafa;
}

.info-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: none;
  border: none;
  cursor: pointer;
  color: #71717a;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.info-btn:hover {
  background-color: rgba(255, 255, 255, 0.1);
  color: #a1a1aa;
}

.info-btn svg {
  width: 16px;
  height: 16px;
}

.php-section {
  border-bottom: none;
}

.php-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 16px;
  font-size: 13px;
  color: #fafafa;
  background: none;
  border: none;
  cursor: pointer;
  transition: background-color 0.15s ease;
  text-align: left;
}

.php-item:hover {
  background-color: rgba(255, 255, 255, 0.06);
}

.php-item.active {
  color: #f59e0b;
}

.check-icon {
  width: 16px;
  height: 16px;
  color: #f59e0b;
  flex-shrink: 0;
}

.ml-6 {
  margin-left: 24px;
}

.config-link {
  color: #a1a1aa;
  padding-left: 40px;
}

.footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
  padding: 12px 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.footer-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: none;
  border: none;
  cursor: pointer;
  color: #71717a;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.footer-btn:hover {
  background-color: rgba(255, 255, 255, 0.08);
  color: #a1a1aa;
}

.footer-btn svg {
  width: 18px;
  height: 18px;
}
</style>
