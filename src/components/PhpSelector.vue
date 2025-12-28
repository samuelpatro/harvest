<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useConfigStore } from '../stores/config'

const store = useConfigStore()
const isOpen = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)

const currentVersion = computed(() => store.currentPhp)

function toggleDropdown() {
  isOpen.value = !isOpen.value
}

function selectVersion(version: string) {
  store.setGlobalPhp(version)
  isOpen.value = false
}

function handleClickOutside(event: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    isOpen.value = false
  }
}

onMounted(() => {
  store.fetchPhpVersions()
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div class="php-selector" ref="dropdownRef">
    <button class="selector-trigger" @click="toggleDropdown">
      <div class="selector-left">
        <svg class="php-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <ellipse cx="12" cy="12" rx="10" ry="7" />
          <text x="12" y="15" font-size="8" font-weight="bold" fill="currentColor" stroke="none" text-anchor="middle">PHP</text>
        </svg>
        <div class="selector-info">
          <span class="selector-label">PHP Version</span>
          <span class="selector-value">{{ currentVersion?.version || '...' }}</span>
        </div>
      </div>
      <svg
        class="chevron"
        :class="{ rotated: isOpen }"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <polyline points="6 9 12 15 18 9" />
      </svg>
    </button>

    <Transition name="dropdown">
      <div v-if="isOpen" class="dropdown-menu">
        <button
          v-for="php in store.installedPhpVersions"
          :key="php.version"
          class="dropdown-item"
          :class="{ active: php.isDefault }"
          @click="selectVersion(php.version)"
        >
          <svg
            v-if="php.isDefault"
            class="check-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <polyline points="20 6 9 17 4 12" />
          </svg>
          <span :class="{ 'ml-offset': !php.isDefault }">PHP {{ php.version }}</span>
        </button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.php-selector {
  position: relative;
}

.selector-trigger {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.selector-trigger:hover {
  border-color: rgba(245, 158, 11, 0.4);
}

.selector-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.php-icon {
  width: 28px;
  height: 28px;
  color: #777bb4;
}

.selector-info {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
}

.selector-label {
  font-size: 10px;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.selector-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  font-family: var(--font-mono);
}

.chevron {
  width: 16px;
  height: 16px;
  color: var(--color-text-muted);
  transition: transform 0.2s ease;
}

.chevron.rotated {
  transform: rotate(180deg);
}

.dropdown-menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 4px;
  z-index: 50;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.dropdown-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: none;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  color: var(--color-text);
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: left;
}

.dropdown-item:hover {
  background: var(--color-bg-elevated);
}

.dropdown-item.active {
  color: var(--color-accent);
}

.check-icon {
  width: 16px;
  height: 16px;
  color: var(--color-accent);
  flex-shrink: 0;
}

.ml-offset {
  margin-left: 24px;
}

.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
