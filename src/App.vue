<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Sidebar from './components/Sidebar.vue'
import SetupWizard from './components/SetupWizard.vue'

const showWizard = ref(false)
const loading = ref(true)

async function checkFirstRun() {
  try {
    const isFirst = await invoke<boolean>('is_first_run')
    showWizard.value = isFirst
  } catch (error) {
    console.error('Failed to check first run:', error)
    showWizard.value = false
  } finally {
    loading.value = false
  }
}

function onSetupComplete() {
  showWizard.value = false
}

onMounted(() => {
  checkFirstRun()
})
</script>

<template>
  <div v-if="loading" class="loading-screen">
    <div class="loading-spinner"></div>
  </div>
  <SetupWizard v-else-if="showWizard" @complete="onSetupComplete" />
  <div v-else class="app-layout">
    <Sidebar />
    <main class="main-content">
      <router-view />
    </main>
  </div>
</template>

<style scoped>
.loading-screen {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  background: var(--color-bg);
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.app-layout {
  display: flex;
  height: 100vh;
  background: var(--color-bg);
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>
