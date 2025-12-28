<script setup lang="ts">
import type { Service } from '../stores/services'

defineProps<{
  service: Service
}>()

const emit = defineEmits<{
  toggle: [id: string]
}>()
</script>

<template>
  <div class="bg-bg-card border border-border rounded-xl p-5 hover:border-border/80 transition-colors">
    <div class="flex items-start justify-between mb-4">
      <div class="flex items-center gap-3">
        <span class="text-3xl">{{ service.icon }}</span>
        <div>
          <h3 class="font-semibold text-text">{{ service.name }}</h3>
          <p class="text-sm font-mono text-text-muted">:{{ service.port }}</p>
        </div>
      </div>
      <span
        class="px-2.5 py-1 rounded-full text-xs font-medium"
        :class="service.status === 'running'
          ? 'bg-success/10 text-success'
          : 'bg-danger/10 text-danger'"
      >
        {{ service.status === 'running' ? 'Running' : 'Stopped' }}
      </span>
    </div>

    <button
      @click="emit('toggle', service.id)"
      class="w-full py-2.5 rounded-lg font-medium transition-colors cursor-pointer"
      :class="service.status === 'running'
        ? 'bg-danger/10 text-danger hover:bg-danger/20'
        : 'bg-success/10 text-success hover:bg-success/20'"
    >
      {{ service.status === 'running' ? 'Stop' : 'Start' }}
    </button>
  </div>
</template>
