import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

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

export interface ServiceVersion {
  version: string
  download_url: string
  filename: string
}

export interface InstallableService {
  id: string
  name: string
  description: string
  installed: boolean
  version: string | null
  available_versions: ServiceVersion[]
}

export interface InstallProgress {
  type: string
  service: string
  version: string
  status: 'downloading' | 'extracting' | 'complete'
  progress: number
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
  const installableServices = ref<InstallableService[]>([])
  const loading = ref<Record<string, boolean>>({})
  const installing = ref<Record<string, boolean>>({})
  const installProgress = ref<Record<string, InstallProgress>>({})
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

  async function fetchInstallableServices() {
    try {
      error.value = null
      const result = await invoke<InstallableService[]>('get_installable_services')
      installableServices.value = result
    } catch (e) {
      error.value = e as string
      console.error('Failed to fetch installable services:', e)
    }
  }

  async function installService(serviceId: string, version: string) {
    installing.value[serviceId] = true
    installProgress.value[serviceId] = {
      type: 'service',
      service: serviceId,
      version,
      status: 'downloading',
      progress: 0
    }
    error.value = null

    // Listen for progress events
    const unlisten = await listen<InstallProgress>('install-progress', (event) => {
      if (event.payload.service === serviceId) {
        installProgress.value[serviceId] = event.payload
        if (event.payload.status === 'complete') {
          unlisten()
        }
      }
    })

    try {
      await invoke('install_service', { serviceId, version })
      await fetchServices()
      await fetchInstallableServices()
    } catch (e) {
      error.value = e as string
      console.error('Failed to install service:', e)
    } finally {
      installing.value[serviceId] = false
      delete installProgress.value[serviceId]
    }
  }

  async function uninstallService(serviceId: string) {
    installing.value[serviceId] = true
    error.value = null
    try {
      await invoke('uninstall_service', { serviceId })
      await fetchServices()
      await fetchInstallableServices()
    } catch (e) {
      error.value = e as string
      console.error('Failed to uninstall service:', e)
    } finally {
      installing.value[serviceId] = false
    }
  }

  function getIcon(id: string): string {
    return SERVICE_ICONS[id] || '⚙️'
  }

  function isLoading(id: string): boolean {
    return loading.value[id] || false
  }

  function isInstalling(id: string): boolean {
    return installing.value[id] || false
  }

  function getInstallProgress(id: string): InstallProgress | null {
    return installProgress.value[id] || null
  }

  return {
    services,
    installableServices,
    loading,
    installing,
    installProgress,
    initialLoading,
    error,
    fetchServices,
    fetchInstallableServices,
    startService,
    stopService,
    restartService,
    toggleService,
    installService,
    uninstallService,
    getIcon,
    isLoading,
    isInstalling,
    getInstallProgress,
  }
})
