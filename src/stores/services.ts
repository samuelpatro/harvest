import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Service {
  id: string
  name: string
  port: number
  status: 'running' | 'stopped' | 'not_installed'
  pid: number | null
  path: string | null
  description: string
  version: string | null
}

const SERVICE_ICONS: Record<string, string> = {
  nginx: '🌐',
  apache: '🪶',
  mysql: '🗄️',
  redis: '⚡',
  postgresql: '🐘',
}

export const useServicesStore = defineStore('services', () => {
  const services = ref<Service[]>([])
  const loading = ref<Record<string, boolean>>({})
  const initialLoading = ref(true)
  const error = ref<string | null>(null)

  async function fetchServices() {
    try {
      error.value = null
      const result = await invoke<Service[]>('get_services')
      services.value = result
    } catch (e) {
      error.value = e as string
      console.error('Failed to fetch services:', e)
    } finally {
      initialLoading.value = false
    }
  }

  async function startService(id: string) {
    loading.value[id] = true
    error.value = null
    try {
      await invoke('start_service', { id })
      await fetchServices()
    } catch (e) {
      error.value = e as string
      console.error('Failed to start service:', e)
    } finally {
      loading.value[id] = false
    }
  }

  async function stopService(id: string) {
    loading.value[id] = true
    error.value = null
    try {
      await invoke('stop_service', { id })
      await fetchServices()
    } catch (e) {
      error.value = e as string
      console.error('Failed to stop service:', e)
    } finally {
      loading.value[id] = false
    }
  }

  async function restartService(id: string) {
    loading.value[id] = true
    error.value = null
    try {
      await invoke('restart_service', { id })
      await fetchServices()
    } catch (e) {
      error.value = e as string
      console.error('Failed to restart service:', e)
    } finally {
      loading.value[id] = false
    }
  }

  async function toggleService(id: string) {
    const service = services.value.find(s => s.id === id)
    if (!service || service.status === 'not_installed') return

    if (service.status === 'running') {
      await stopService(id)
    } else {
      await startService(id)
    }
  }

  function getIcon(id: string): string {
    return SERVICE_ICONS[id] || '⚙️'
  }

  function isLoading(id: string): boolean {
    return loading.value[id] || false
  }

  return {
    services,
    loading,
    initialLoading,
    error,
    fetchServices,
    startService,
    stopService,
    restartService,
    toggleService,
    getIcon,
    isLoading,
  }
})
