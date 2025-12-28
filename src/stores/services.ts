import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface Service {
  id: string
  name: string
  port: number
  status: 'running' | 'stopped'
  icon: string
}

export const useServicesStore = defineStore('services', () => {
  const services = ref<Service[]>([
    {
      id: 'nginx',
      name: 'Nginx',
      port: 80,
      status: 'running',
      icon: '🌐'
    },
    {
      id: 'php-fpm',
      name: 'PHP-FPM',
      port: 9000,
      status: 'running',
      icon: '🐘'
    },
    {
      id: 'mysql',
      name: 'MySQL',
      port: 3306,
      status: 'stopped',
      icon: '🗄️'
    },
    {
      id: 'redis',
      name: 'Redis',
      port: 6379,
      status: 'stopped',
      icon: '⚡'
    }
  ])

  function toggleService(id: string) {
    const service = services.value.find(s => s.id === id)
    if (service) {
      service.status = service.status === 'running' ? 'stopped' : 'running'
    }
  }

  async function fetchServices() {
    // TODO: Replace with actual Tauri invoke
    // const result = await invoke('get_services')
    // services.value = result
  }

  return {
    services,
    toggleService,
    fetchServices
  }
})
