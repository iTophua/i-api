import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import { createI18nInstance } from './locales'
import { setI18nInstance, useSettingsStore } from './stores/settings'
import { useHistoryStore } from './stores/history'
import App from './App.vue'
import type { Locale } from './locales'
import './styles/theme.css'

const savedLocale = (localStorage.getItem('iapi-locale') as Locale) || 'zh-CN'

const i18n = createI18nInstance(savedLocale)

setI18nInstance(i18n)

const app = createApp(App)

const pinia = createPinia()
app.use(pinia)
app.use(router)
app.use(i18n)

// 应用启动时加载历史记录
router.isReady().then(async () => {
  const settingsStore = useSettingsStore()
  await settingsStore.loadSettings()

  const historyStore = useHistoryStore()
  await historyStore.loadHistory()

  router.replace('/splash')
})

app.mount('#app')
