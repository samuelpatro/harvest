import { defineStore } from 'pinia'
import type { AppConfig, PhpVersion } from '../types'

const mockPhpVersions: PhpVersion[] = [
  { version: '8.4', path: 'C:/laragon/bin/php/php-8.4', isDefault: true, isInstalled: true },
  { version: '8.3', path: 'C:/laragon/bin/php/php-8.3', isDefault: false, isInstalled: true },
  { version: '8.2', path: 'C:/laragon/bin/php/php-8.2', isDefault: false, isInstalled: true },
  { version: '8.1', path: 'C:/laragon/bin/php/php-8.1', isDefault: false, isInstalled: true },
]

export const useConfigStore = defineStore('config', {
  state: () => ({
    config: {
      sitesDirectory: 'C:/laragon/www',
      defaultPhp: '8.4',
      editorCommand: 'code',
      autoStart: true,
    } as AppConfig,
    phpVersions: [] as PhpVersion[],
    loading: false,
  }),

  getters: {
    currentPhp: (state) => state.phpVersions.find(v => v.isDefault),
    installedPhpVersions: (state) => state.phpVersions.filter(v => v.isInstalled),
  },

  actions: {
    async fetchConfig() {
      this.loading = true
      // TODO: Replace with actual Tauri invoke
      // this.config = await invoke('get_config')
      await new Promise(r => setTimeout(r, 200))
      this.loading = false
    },

    async fetchPhpVersions() {
      // TODO: Replace with actual Tauri invoke
      // this.phpVersions = await invoke('get_php_versions')
      await new Promise(r => setTimeout(r, 200))
      this.phpVersions = mockPhpVersions
    },

    async setGlobalPhp(version: string) {
      // TODO: Replace with actual Tauri invoke
      // await invoke('set_global_php', { version })
      this.phpVersions.forEach(v => {
        v.isDefault = v.version === version
      })
      this.config.defaultPhp = version
    },

    async updateConfig(config: Partial<AppConfig>) {
      // TODO: Replace with actual Tauri invoke
      // await invoke('update_config', { config: { ...this.config, ...config } })
      this.config = { ...this.config, ...config }
    },
  },
})
