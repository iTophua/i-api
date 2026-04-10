import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import { createI18nInstance } from './locales'
import { setI18nInstance } from './stores/settings'
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

// 直接设置初始路由为 splash
router.replace('/splash')

app.mount('#app')
