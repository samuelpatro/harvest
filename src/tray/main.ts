import { createApp } from 'vue'
import { createPinia } from 'pinia'
import TrayPopup from './TrayPopup.vue'
import '../styles/tray.css'

const app = createApp(TrayPopup)
app.use(createPinia())
app.mount('#app')
