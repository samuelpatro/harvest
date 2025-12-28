import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Site } from '../types'

interface AppConfig {
  harvest_paths: string[]
  editor_command: string
  terminal_command: string
  default_php: string
}

export const useSitesStore = defineStore('sites', {
  state: () => ({
    sites: [] as Site[],
    loading: false,
    searchQuery: '',
    config: null as AppConfig | null,
    error: null as string | null,
  }),

  getters: {
    filteredSites: (state) => {
      if (!state.searchQuery) return state.sites
      const query = state.searchQuery.toLowerCase()
      return state.sites.filter(site =>
        site.name.toLowerCase().includes(query) ||
        site.domain.toLowerCase().includes(query)
      )
    },
    siteCount: (state) => state.sites.length,
  },

  actions: {
    async fetchConfig() {
      try {
        this.config = await invoke('get_config')
      } catch (error) {
        console.error('Failed to fetch config:', error)
      }
    },

    async fetchSites() {
      this.loading = true
      this.error = null
      try {
        // Get config first
        if (!this.config) {
          await this.fetchConfig()
        }

        if (!this.config?.harvest_paths?.length) {
          this.error = 'No harvest paths configured'
          return
        }

        const sites = await invoke<Site[]>('scan_sites', {
          directories: this.config.harvest_paths
        })

        // Map snake_case from Rust to camelCase in TypeScript
        this.sites = sites.map(site => ({
          ...site,
          phpVersion: (site as any).php_version || site.phpVersion,
        }))
      } catch (error) {
        console.error('Failed to scan sites:', error)
        this.error = String(error)
        this.sites = []
      } finally {
        this.loading = false
      }
    },

    async openInBrowser(site: Site) {
      const url = site.ssl ? `https://${site.domain}` : `http://${site.domain}`
      try {
        await invoke('open_in_browser', { url })
      } catch (error) {
        // Fallback to window.open
        window.open(url, '_blank')
      }
    },

    async openInEditor(site: Site) {
      if (!this.config) await this.fetchConfig()
      try {
        await invoke('open_in_editor', {
          path: site.path,
          editor: this.config?.editor_command || 'code'
        })
      } catch (error) {
        console.error('Failed to open in editor:', error)
      }
    },

    async openInTerminal(site: Site) {
      if (!this.config) await this.fetchConfig()
      try {
        await invoke('open_in_terminal', {
          path: site.path,
          terminal: this.config?.terminal_command || 'wt -d'
        })
      } catch (error) {
        console.error('Failed to open terminal:', error)
      }
    },

    setSearchQuery(query: string) {
      this.searchQuery = query
    }
  },
})
