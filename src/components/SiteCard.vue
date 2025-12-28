<script setup lang="ts">
import type { Site } from '../types'

defineProps<{
  site: Site
}>()

const emit = defineEmits<{
  open: [site: Site]
  editor: [site: Site]
  terminal: [site: Site]
}>()

const stackColors: Record<string, string> = {
  octobercms: '#f59e0b',
  laravel: '#ef4444',
  php: '#777bb4',
  vue: '#42b883',
  react: '#61dafb',
  vite: '#bd34fe',
  tailwind: '#38bdf8',
  typescript: '#3178c6',
  bun: '#fbf0df',
  pnpm: '#f9ad00',
  yarn: '#2c8ebb',
}

const stackLabels: Record<string, string> = {
  octobercms: 'October',
  laravel: 'Laravel',
  php: 'PHP',
  vue: 'Vue',
  react: 'React',
  vite: 'Vite',
  tailwind: 'Tailwind',
  typescript: 'TS',
  bun: 'Bun',
  pnpm: 'pnpm',
  yarn: 'Yarn',
}

function getStackLabel(name: string, version: string | null): string {
  const label = stackLabels[name] || name
  if (version) {
    // Shorten version (e.g., "3.5.13" -> "3.5")
    const shortVersion = version.split('.').slice(0, 2).join('.')
    return `${label} ${shortVersion}`
  }
  return label
}

// Filter out plain "php" from stack since we show PHP version separately
function getFilteredStack(stack: { name: string; version: string | null }[]) {
  return stack.filter(item => item.name !== 'php')
}

// Check if this is a PHP project
function isPhpProject(stack: { name: string; version: string | null }[]) {
  return stack.some(item => ['php', 'laravel', 'octobercms'].includes(item.name))
}
</script>

<template>
  <div class="site-card">
    <div class="site-main">
      <div class="site-icon">
        <svg v-if="site.ssl" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
          <path d="M7 11V7a5 5 0 0 1 10 0v4" />
        </svg>
        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10" />
          <line x1="2" y1="12" x2="22" y2="12" />
          <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
        </svg>
      </div>
      <div class="site-info">
        <h3 class="site-name">
          <button class="site-link" @click="emit('open', site)">
            {{ site.name }}<span class="tld">.test</span>
          </button>
        </h3>
        <span class="site-path">{{ site.path }}</span>
      </div>
    </div>
    <div class="site-meta">
      <div class="stack-badges">
        <span
          v-for="item in getFilteredStack(site.stack)"
          :key="item.name"
          class="stack-badge"
          :style="{ '--stack-color': stackColors[item.name] || '#71717a' }"
        >
          {{ getStackLabel(item.name, item.version) }}
        </span>
        <span v-if="isPhpProject(site.stack)" class="php-badge">PHP {{ site.phpVersion }}</span>
      </div>
      <div class="site-actions">
        <button class="action-btn" title="Open in Browser" @click="emit('open', site)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
            <polyline points="15 3 21 3 21 9" />
            <line x1="10" y1="14" x2="21" y2="3" />
          </svg>
        </button>
        <button class="action-btn" title="Open in Editor" @click="emit('editor', site)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
          </svg>
        </button>
        <button class="action-btn" title="Open Terminal" @click="emit('terminal', site)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <polyline points="4 17 10 11 4 5" />
            <line x1="12" y1="19" x2="20" y2="19" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.site-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  transition: all 0.2s ease;
}

.site-card:hover {
  border-color: rgba(245, 158, 11, 0.3);
  background: var(--color-bg-elevated);
}

.site-main {
  display: flex;
  align-items: center;
  gap: 14px;
}

.site-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-elevated);
  border-radius: 8px;
  color: var(--color-text-muted);
}

.site-icon svg {
  width: 20px;
  height: 20px;
}

.site-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.site-name {
  font-size: 15px;
  font-weight: 500;
  margin: 0;
}

.site-link {
  background: none;
  border: none;
  color: var(--color-text);
  cursor: pointer;
  padding: 0;
  font: inherit;
  text-align: left;
  transition: color 0.15s ease;
}

.site-link:hover {
  color: var(--color-accent);
}

.tld {
  color: var(--color-text-muted);
  font-weight: 400;
}

.site-path {
  font-size: 12px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}

.site-meta {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stack-badges {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.stack-badge {
  padding: 4px 8px;
  border-radius: 5px;
  font-size: 10px;
  font-weight: 600;
  background: color-mix(in srgb, var(--stack-color) 15%, transparent);
  color: var(--stack-color);
  text-transform: uppercase;
  letter-spacing: 0.3px;
  white-space: nowrap;
}

.php-badge {
  padding: 4px 8px;
  border-radius: 5px;
  font-size: 10px;
  font-weight: 600;
  background: rgba(119, 123, 180, 0.15);
  color: #a5a9d6;
  font-family: var(--font-mono);
  white-space: nowrap;
}

.site-actions {
  display: flex;
  gap: 4px;
}

.action-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: var(--color-bg-card);
  border-color: var(--color-border);
  color: var(--color-text);
}

.action-btn svg {
  width: 16px;
  height: 16px;
}
</style>
