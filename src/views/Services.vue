<script setup lang="ts">
import { useServicesStore } from '../stores/services'
import PageHeader from '../components/PageHeader.vue'
import ServiceCard from '../components/ServiceCard.vue'

const store = useServicesStore()
</script>

<template>
  <div class="page-layout">
    <PageHeader title="Services" subtitle="All services" />

    <div class="page-content">
      <div class="services-grid">
        <ServiceCard
          v-for="service in store.services"
          :key="service.id"
          :service="service"
          @toggle="store.toggleService"
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
}

.services-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
  max-width: 1200px;
}
</style>
