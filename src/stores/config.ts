import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, PhpVersion } from '../types'

interface InstalledVersion {
  version: string
  path: string
}

export const useConfigStore = defineStore('config', {
  state: () => ({
    config: {
      harvest_paths: ['C:/laragon/www'],
      editor_command: 'code',
      terminal_command: 'wt -d',
    } as AppConfig,
    phpVersions: [] as PhpVersion[],
    loading: false,
    defaultPhpVersion: null as string | null,
  }),

  getters: {
    currentPhp: (state) => state.phpVersions.find(v => v.isDefault),
    installedPhpVersions: (state) => state.phpVersions.filter(v => v.isInstalled),
  },

  actions: {
    async fetchConfig() {
      this.loading = true
      try {
        this.config = await invoke('get_config')
      } catch (error) {
        console.error('Failed to fetch config:', error)
      } finally {
        this.loading = false
      }
    },

    async fetchPhpVersions() {
      try {
        const installed = await invoke<InstalledVersion[]>('get_installed_php_versions')
        this.phpVersions = installed.map((v, index) => ({
          version: v.version,
          path: v.path,
          isDefault: index === 0,
          isInstalled: true,
        }))
      } catch (error) {
        console.error('Failed to fetch PHP versions:', error)
      }
    },

    async setGlobalPhp(version: string) {
      this.phpVersions.forEach(v => {
        v.isDefault = v.version === version
      })
      this.defaultPhpVersion = version
    },

    async updateConfig(config: Partial<AppConfig>) {
      try {
        const newConfig = { ...this.config, ...config }
        await invoke('save_config', { config: newConfig })
        this.config = newConfig
      } catch (error) {
        console.error('Failed to save config:', error)
      }
    },
  },
})
