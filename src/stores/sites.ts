import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Site } from '../types'

interface AppConfig {
  harvest_paths: string[]
  editor_command: string
  terminal_command: string
  default_php: string
}

interface VhostStatus {
  site_name: string
  domain: string
  nginx_config_exists: boolean
  hosts_entry_exists: boolean
  is_linked: boolean
}

export const useSitesStore = defineStore('sites', {
  state: () => ({
    sites: [] as Site[],
    vhostStatuses: {} as Record<string, VhostStatus>,
    loading: false,
    searchQuery: '',
    config: null as AppConfig | null,
    error: null as string | null,
    linkingInProgress: {} as Record<string, boolean>,
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
        // Always fetch fresh config to pick up any changes
        await this.fetchConfig()

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
    },

    async fetchVhostStatuses() {
      if (this.sites.length === 0) return

      try {
        const siteNames = this.sites.map(s => s.name)
        const statuses = await invoke<VhostStatus[]>('get_all_vhost_statuses', { siteNames })

        this.vhostStatuses = {}
        for (const status of statuses) {
          this.vhostStatuses[status.site_name] = status
        }
      } catch (error) {
        console.error('Failed to fetch vhost statuses:', error)
      }
    },

    async linkSite(site: Site) {
      this.linkingInProgress[site.name] = true
      try {
        await invoke<string>('link_site', { siteName: site.name, rootPath: site.path })
        await this.fetchVhostStatuses()
      } catch (error) {
        console.error('Failed to link site:', error)
        throw error
      } finally {
        this.linkingInProgress[site.name] = false
      }
    },

    async unlinkSite(site: Site) {
      this.linkingInProgress[site.name] = true
      try {
        await invoke<string>('unlink_site', { siteName: site.name })
        await this.fetchVhostStatuses()
      } catch (error) {
        console.error('Failed to unlink site:', error)
        throw error
      } finally {
        this.linkingInProgress[site.name] = false
      }
    },

    isLinked(siteName: string): boolean {
      return this.vhostStatuses[siteName]?.is_linked ?? false
    },

    isLinking(siteName: string): boolean {
      return this.linkingInProgress[siteName] ?? false
    },

    async refreshAfterConfigChange() {
      // Clear cached config so fetchSites will reload it
      this.config = null
      await this.fetchSites()
      await this.fetchVhostStatuses()
    }
  },
})
