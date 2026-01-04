<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const emit = defineEmits<{
  (e: 'complete'): void
}>()

const step = ref(1)
const harvestHome = ref('')
const isValidating = ref(false)
const isInitializing = ref(false)
const error = ref('')

// Get default path on mount
invoke<string>('get_harvest_home').then(path => {
  harvestHome.value = path
})

const canProceed = computed(() => {
  return harvestHome.value.trim().length > 0
})

async function selectFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath: harvestHome.value || 'C:/'
  })
  if (selected) {
    harvestHome.value = selected as string
    error.value = ''
  }
}

async function validateAndNext() {
  isValidating.value = true
  error.value = ''

  try {
    await invoke('validate_harvest_path', { path: harvestHome.value })
    step.value = 2
  } catch (e) {
    error.value = String(e)
  } finally {
    isValidating.value = false
  }
}

async function initializeHarvest() {
  isInitializing.value = true
  error.value = ''

  try {
    await invoke('initialize_harvest_home', { path: harvestHome.value })
    step.value = 3
  } catch (e) {
    error.value = String(e)
  } finally {
    isInitializing.value = false
  }
}

function finish() {
  emit('complete')
}
</script>

<template>
  <div class="wizard-overlay">
    <div class="wizard-container">
      <!-- Header -->
      <div class="wizard-header">
        <div class="logo">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M12 2L2 7l10 5 10-5-10-5z" />
            <path d="M2 17l10 5 10-5" />
            <path d="M2 12l10 5 10-5" />
          </svg>
        </div>
        <h1>Welcome to Harvest</h1>
        <p>Let's set up your local development environment</p>
      </div>

      <!-- Progress -->
      <div class="progress-bar">
        <div class="progress-step" :class="{ active: step >= 1, completed: step > 1 }">
          <div class="step-circle">1</div>
          <span>Location</span>
        </div>
        <div class="progress-line" :class="{ active: step > 1 }"></div>
        <div class="progress-step" :class="{ active: step >= 2, completed: step > 2 }">
          <div class="step-circle">2</div>
          <span>Initialize</span>
        </div>
        <div class="progress-line" :class="{ active: step > 2 }"></div>
        <div class="progress-step" :class="{ active: step >= 3 }">
          <div class="step-circle">3</div>
          <span>Complete</span>
        </div>
      </div>

      <!-- Step 1: Choose Location -->
      <div v-if="step === 1" class="wizard-content">
        <h2>Choose Harvest Location</h2>
        <p class="step-description">
          Select where Harvest will store its services (nginx, MySQL, Redis, PHP) and configuration files.
          This folder will be portable - you can move it anywhere.
        </p>

        <div class="path-selector">
          <input
            type="text"
            class="path-input"
            v-model="harvestHome"
            placeholder="C:\harvest"
          />
          <button class="btn-browse" @click="selectFolder">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            Browse
          </button>
        </div>

        <div v-if="error" class="error-message">
          {{ error }}
        </div>

        <div class="folder-structure">
          <p class="structure-title">This will create:</p>
          <div class="structure-tree">
            <div class="tree-item"><span class="folder">{{ harvestHome || 'harvest' }}/</span></div>
            <div class="tree-item indent"><span class="folder">bin/</span> <span class="desc">- nginx, mysql, redis, php</span></div>
            <div class="tree-item indent"><span class="folder">etc/</span> <span class="desc">- configuration files</span></div>
            <div class="tree-item indent"><span class="folder">logs/</span> <span class="desc">- service logs</span></div>
            <div class="tree-item indent"><span class="folder">www/</span> <span class="desc">- your sites</span></div>
          </div>
        </div>

        <div class="wizard-actions">
          <button
            class="btn-primary"
            :disabled="!canProceed || isValidating"
            @click="validateAndNext"
          >
            {{ isValidating ? 'Validating...' : 'Continue' }}
          </button>
        </div>
      </div>

      <!-- Step 2: Initialize -->
      <div v-if="step === 2" class="wizard-content">
        <h2>Initialize Harvest</h2>
        <p class="step-description">
          We'll create the folder structure and prepare Harvest for use.
          You can install services like nginx, MySQL, and Redis after setup.
        </p>

        <div class="init-summary">
          <div class="summary-item">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            <div>
              <strong>Harvest Home</strong>
              <span>{{ harvestHome }}</span>
            </div>
          </div>
          <div class="summary-item">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="9 11 12 14 22 4" />
              <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
            </svg>
            <div>
              <strong>Ready to Initialize</strong>
              <span>Click below to create the folder structure</span>
            </div>
          </div>
        </div>

        <div v-if="error" class="error-message">
          {{ error }}
        </div>

        <div class="wizard-actions">
          <button class="btn-secondary" @click="step = 1">
            Back
          </button>
          <button
            class="btn-primary"
            :disabled="isInitializing"
            @click="initializeHarvest"
          >
            {{ isInitializing ? 'Initializing...' : 'Initialize Harvest' }}
          </button>
        </div>
      </div>

      <!-- Step 3: Complete -->
      <div v-if="step === 3" class="wizard-content">
        <div class="success-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        </div>
        <h2>Setup Complete!</h2>
        <p class="step-description">
          Harvest is ready to use. You can now install services from the Services page,
          add PHP/Node.js versions, and start managing your local development sites.
        </p>

        <div class="next-steps">
          <h3>Next Steps:</h3>
          <ul>
            <li>Install nginx from the Services page</li>
            <li>Install PHP from the PHP page</li>
            <li>Add your sites directory in Settings</li>
          </ul>
        </div>

        <div class="wizard-actions">
          <button class="btn-primary" @click="finish">
            Get Started
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.wizard-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.wizard-container {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: 16px;
  width: 100%;
  max-width: 560px;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.wizard-header {
  text-align: center;
  padding: 32px 32px 24px;
  border-bottom: 1px solid var(--color-border);
}

.logo {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  background: linear-gradient(135deg, var(--color-accent), #d97706);
  border-radius: 14px;
  margin-bottom: 16px;
}

.logo svg {
  width: 32px;
  height: 32px;
  color: #000;
}

.wizard-header h1 {
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 8px;
}

.wizard-header p {
  font-size: 14px;
  color: var(--color-text-muted);
  margin: 0;
}

.progress-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px 32px;
  gap: 12px;
  border-bottom: 1px solid var(--color-border);
}

.progress-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.step-circle {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--color-bg);
  border: 2px solid var(--color-border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  transition: all 0.3s ease;
}

.progress-step.active .step-circle {
  background: var(--color-accent);
  border-color: var(--color-accent);
  color: #000;
}

.progress-step.completed .step-circle {
  background: var(--color-accent);
  border-color: var(--color-accent);
  color: #000;
}

.progress-step span {
  font-size: 12px;
  color: var(--color-text-muted);
}

.progress-step.active span {
  color: var(--color-text);
}

.progress-line {
  width: 40px;
  height: 2px;
  background: var(--color-border);
  margin-bottom: 20px;
  transition: background 0.3s ease;
}

.progress-line.active {
  background: var(--color-accent);
}

.wizard-content {
  padding: 32px;
}

.wizard-content h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 12px;
  text-align: center;
}

.step-description {
  font-size: 14px;
  color: var(--color-text-muted);
  text-align: center;
  margin: 0 0 24px;
  line-height: 1.6;
}

.path-selector {
  display: flex;
  gap: 10px;
  margin-bottom: 16px;
}

.path-input {
  flex: 1;
  padding: 12px 16px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  color: var(--color-text);
  font-size: 14px;
  font-family: var(--font-mono);
}

.path-input:focus {
  outline: none;
  border-color: var(--color-accent);
}

.btn-browse {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  color: var(--color-text);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-browse:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.btn-browse svg {
  width: 18px;
  height: 18px;
}

.error-message {
  padding: 12px 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 8px;
  color: #ef4444;
  font-size: 13px;
  margin-bottom: 16px;
}

.folder-structure {
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 24px;
}

.structure-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  margin: 0 0 12px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.structure-tree {
  font-family: var(--font-mono);
  font-size: 13px;
}

.tree-item {
  padding: 4px 0;
  color: var(--color-text);
}

.tree-item.indent {
  padding-left: 20px;
}

.tree-item .folder {
  color: var(--color-accent);
}

.tree-item .desc {
  color: var(--color-text-muted);
  font-size: 12px;
}

.init-summary {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 24px;
}

.summary-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
}

.summary-item svg {
  width: 24px;
  height: 24px;
  color: var(--color-accent);
  flex-shrink: 0;
}

.summary-item div {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.summary-item strong {
  font-size: 14px;
  color: var(--color-text);
}

.summary-item span {
  font-size: 12px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}

.success-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto 20px;
  background: linear-gradient(135deg, #22c55e, #16a34a);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.success-icon svg {
  width: 32px;
  height: 32px;
  color: white;
}

.next-steps {
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px 20px;
  margin-bottom: 24px;
}

.next-steps h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 12px;
}

.next-steps ul {
  margin: 0;
  padding-left: 20px;
}

.next-steps li {
  font-size: 14px;
  color: var(--color-text-muted);
  padding: 4px 0;
}

.wizard-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn-primary {
  padding: 12px 24px;
  background: var(--color-accent);
  border: none;
  border-radius: 8px;
  color: #000;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary:hover:not(:disabled) {
  background: #d97706;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  padding: 12px 24px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  color: var(--color-text);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-secondary:hover {
  border-color: var(--color-text-muted);
}
</style>
