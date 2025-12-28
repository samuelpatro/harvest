<script setup lang="ts">
import { useRoute } from 'vue-router'

const route = useRoute()

const navItems = [
  { path: '/', label: 'Dashboard', icon: 'dashboard' },
  { path: '/sites', label: 'Sites', icon: 'sites' },
  { path: '/php', label: 'PHP', icon: 'php' },
  { path: '/node', label: 'Node', icon: 'node' },
  { path: '/services', label: 'Services', icon: 'services' },
  { path: '/settings', label: 'Settings', icon: 'settings' }
]
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-header" data-tauri-drag-region>
      <div class="logo">
        <svg class="logo-icon" viewBox="0 0 24 24" fill="none">
          <!-- Wheat stalk -->
          <path d="M12 22V8" stroke="#f59e0b" stroke-width="2" stroke-linecap="round"/>
          <!-- Left leaves -->
          <path d="M12 8C12 8 8 6 6 3" stroke="#f59e0b" stroke-width="2" stroke-linecap="round"/>
          <path d="M12 11C12 11 8.5 9.5 7 7" stroke="#f59e0b" stroke-width="2" stroke-linecap="round"/>
          <path d="M12 14C12 14 9 13 8 11" stroke="#f59e0b" stroke-width="2" stroke-linecap="round"/>
          <!-- Right leaves -->
          <path d="M12 8C12 8 16 6 18 3" stroke="#fbbf24" stroke-width="2" stroke-linecap="round"/>
          <path d="M12 11C12 11 15.5 9.5 17 7" stroke="#fbbf24" stroke-width="2" stroke-linecap="round"/>
          <path d="M12 14C12 14 15 13 16 11" stroke="#fbbf24" stroke-width="2" stroke-linecap="round"/>
        </svg>
        <span class="logo-text">Harvest</span>
      </div>
    </div>

    <nav class="sidebar-nav">
      <ul class="nav-list">
        <li v-for="item in navItems" :key="item.path">
          <router-link
            :to="item.path"
            class="nav-link"
            :class="{ active: route.path === item.path }"
          >
            <!-- Dashboard Icon -->
            <svg v-if="item.icon === 'dashboard'" class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="3" y="3" width="7" height="7" rx="1" />
              <rect x="14" y="3" width="7" height="7" rx="1" />
              <rect x="3" y="14" width="7" height="7" rx="1" />
              <rect x="14" y="14" width="7" height="7" rx="1" />
            </svg>
            <!-- Sites Icon -->
            <svg v-else-if="item.icon === 'sites'" class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="12" cy="12" r="10" />
              <line x1="2" y1="12" x2="22" y2="12" />
              <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            </svg>
            <!-- PHP Icon -->
            <svg v-else-if="item.icon === 'php'" class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <ellipse cx="12" cy="12" rx="10" ry="6" />
              <path d="M6 12v-1.5c0-1.1.9-2 2-2h1c.6 0 1 .4 1 1v1" />
              <path d="M14 12v-3h2c.6 0 1 .4 1 1v1c0 .6-.4 1-1 1h-2" />
            </svg>
            <!-- Node Icon -->
            <svg v-else-if="item.icon === 'node'" class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M12 2l8 4.5v9L12 20l-8-4.5v-9L12 2z" />
              <path d="M12 7v6" />
              <path d="M12 13l5-3" />
              <path d="M12 13l-5-3" />
            </svg>
            <!-- Services Icon -->
            <svg v-else-if="item.icon === 'services'" class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
            </svg>
            <!-- Settings Icon -->
            <svg v-else-if="item.icon === 'settings'" class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
            <span class="nav-label">{{ item.label }}</span>
          </router-link>
        </li>
      </ul>
    </nav>

    <div class="sidebar-bottom">
      <div class="version-info">v0.1.0</div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 224px;
  height: 100vh;
  background: var(--color-bg-elevated);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  height: 68px;
  display: flex;
  align-items: center;
  padding: 0 20px;
  border-bottom: 1px solid var(--color-border);
  -webkit-app-region: drag;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
}

.logo-icon {
  width: 28px;
  height: 28px;
}

.logo-text {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text);
}

.sidebar-nav {
  flex: 1;
  padding: 12px;
  overflow-y: auto;
}

.nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-link {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 8px;
  color: var(--color-text-muted);
  text-decoration: none;
  transition: all 0.15s ease;
}

.nav-link:hover {
  background: var(--color-bg-card);
  color: var(--color-text);
}

.nav-link.active {
  background: rgba(245, 158, 11, 0.1);
  color: var(--color-accent);
}

.nav-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.nav-label {
  font-size: 14px;
  font-weight: 500;
}

.sidebar-bottom {
  padding: 16px;
  border-top: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.version-info {
  font-size: 11px;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  text-align: center;
}
</style>
