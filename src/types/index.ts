export interface ServiceInfo {
  id: string
  name: string
  status: 'running' | 'stopped' | 'error'
  port: number
  pid?: number
  icon: string
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
  sitesDirectory: string
  defaultPhp: string
  editorCommand: string
  autoStart: boolean
}
