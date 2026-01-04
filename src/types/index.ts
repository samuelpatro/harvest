export interface ServiceInfo {
  id: string
  name: string
  status: 'running' | 'stopped' | 'not_installed'
  port: number
  pid?: number
  path?: string
  description: string
  version?: string
}

export interface StackItem {
  name: string
  version: string | null
}

export interface Site {
  id: string
  name: string
  path: string
  domain: string
  phpVersion: string
  stack: StackItem[]
  ssl: boolean
}

export interface PhpVersion {
  version: string
  path: string
  isDefault: boolean
  isInstalled: boolean
}

export interface AppConfig {
  harvest_paths: string[]
  editor_command: string
  terminal_command: string
}
