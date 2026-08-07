import { createPinia } from 'pinia'
import { createApp } from 'vue'

import App from './App.vue'
import router from './router'
// Bundled, not referenced by name: JetBrains Mono is not a system font, so a
// bare font-family would silently fall back to the old stack on any machine
// that lacks it. The variable cut is deliberate — the workbench uses weights
// 550 and 650, which static cuts would snap to the nearest available.
import '@fontsource-variable/jetbrains-mono'
import './assets/app.scss'

createApp(App).use(createPinia()).use(router).mount('#app')
