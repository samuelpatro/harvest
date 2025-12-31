<script setup lang="ts">
import { onMounted } from 'vue'
import { useSitesStore } from '../stores/sites'
import PageHeader from '../components/PageHeader.vue'
import SiteCard from '../components/SiteCard.vue'

const store = useSitesStore()

onMounted(async () => {
  await store.fetchSites()
  await store.fetchVhostStatuses()
})
</script>

<template>
  <div class="page-layout">
    <PageHeader title="Sites" :subtitle="`${store.siteCount} sites`">
      <div class="search-wrapper">
        <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8" />
          <path d="M21 21l-4.35-4.35" />
        </svg>
        <input
          type="text"
          class="search-input"
          placeholder="Search sites..."
          :value="store.searchQuery"
          @input="store.setSearchQuery(($event.target as HTMLInputElement).value)"
        />
      </div>
      <button class="btn-icon" title="Refresh sites" @click="store.fetchSites()">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="23 4 23 10 17 10" />
          <polyline points="1 20 1 14 7 14" />
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
        </svg>
      </button>
    </PageHeader>

    <div class="page-content">
      <div v-if="store.loading" class="loading-state">
        <div class="loading-spinner" />
        <p>Scanning sites...</p>
      </div>

      <div v-else-if="store.filteredSites.length === 0" class="empty-state">
        <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10" />
          <line x1="2" y1="12" x2="22" y2="12" />
          <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
        </svg>
        <p v-if="store.searchQuery">No sites matching "{{ store.searchQuery }}"</p>
        <p v-else>No sites found in your sites directory</p>
      </div>

      <div v-else class="sites-grid">
        <SiteCard
          v-for="site in store.filteredSites"
          :key="site.id"
          :site="site"
          :is-linked="store.isLinked(site.name)"
          :is-linking="store.isLinking(site.name)"
          @open="store.openInBrowser"
          @editor="store.openInEditor"
          @terminal="store.openInTerminal"
          @link="store.linkSite"
          @unlink="store.unlinkSite"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.page-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.page-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px 32px 32px;
  max-width: 1200px;
}

.search-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  width: 16px;
  height: 16px;
  color: var(--color-text-muted);
  pointer-events: none;
}

.search-input {
  width: 240px;
  padding: 8px 12px 8px 36px;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  color: var(--color-text);
  font-size: 13px;
  transition: all 0.2s ease;
}

.search-input::placeholder {
  color: var(--color-text-muted);
}

.search-input:focus {
  outline: none;
  border-color: var(--color-accent);
  box-shadow: 0 0 0 3px rgba(245, 158, 11, 0.1);
}

.btn-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-icon:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.btn-icon svg {
  width: 18px;
  height: 18px;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 0;
  color: var(--color-text-muted);
  gap: 16px;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 0;
  color: var(--color-text-muted);
  gap: 16px;
}

.empty-icon {
  width: 48px;
  height: 48px;
  opacity: 0.5;
}

.sites-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
</style>
