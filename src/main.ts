import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import { createI18nInstance } from './locales'
import { setI18nInstance } from './stores/settings'
import App from './App.vue'
import type { Locale } from './locales'

const savedLocale = (localStorage.getItem('iapi-locale') as Locale) || 'zh-CN'

const i18n = createI18nInstance(savedLocale)

setI18nInstance(i18n)

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(i18n)

router.isReady().then(() => {
  router.replace('/splash')
})

app.mount('#app')
