import { createRouter, createWebHistory } from 'vue-router'
import Dashboard from './views/Dashboard.vue'
import Sites from './views/Sites.vue'
import Php from './views/Php.vue'
import Node from './views/Node.vue'
import Services from './views/Services.vue'
import Settings from './views/Settings.vue'

const routes = [
  { path: '/', name: 'Dashboard', component: Dashboard },
  { path: '/sites', name: 'Sites', component: Sites },
  { path: '/php', name: 'PHP', component: Php },
  { path: '/node', name: 'Node', component: Node },
  { path: '/services', name: 'Services', component: Services },
  { path: '/settings', name: 'Settings', component: Settings }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
