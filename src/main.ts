import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import { createI18nInstance } from './locales'
import { setI18nInstance } from './stores/settings'
import App from './App.vue'
import type { Locale } from './locales'
import './styles/theme.css'
import './monaco-setup'

const savedLocale = (localStorage.getItem('iapi-locale') as Locale) || 'zh-CN'

const i18n = createI18nInstance(savedLocale)

setI18nInstance(i18n)

const app = createApp(App)

const pinia = createPinia()
app.use(pinia)
app.use(router)
app.use(i18n)

router.replace('/splash')

app.mount('#app')

if (import.meta.hot) {
  import.meta.hot.accept(() => {
    console.log('Hot module replacement')
  })
}

if (import.meta.env.DEV) {
  console.log('Development mode')
}

window.addEventListener('error', (event) => {
  console.error('Global error:', event.error)
})

window.addEventListener('unhandledrejection', (event) => {
  console.error('Unhandled rejection:', event.reason)
})

document.addEventListener('dragover', (event) => {
  event.preventDefault()
})

document.addEventListener('drop', (event) => {
  event.preventDefault()
})

if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (event) => {
    event.preventDefault()
  })
}

if (import.meta.env.PROD) {
  document.addEventListener('keydown', (event) => {
    if (event.ctrlKey || event.metaKey) {
      if (['c', 'v', 'a', 'x'].includes(event.key.toLowerCase())) {
        return
      }
    }

    if (event.key === 'F12' || (event.ctrlKey && event.shiftKey && event.key === 'I')) {
      event.preventDefault()
    }
  })
}

if ('serviceWorker' in navigator && import.meta.env.PROD) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').catch(() => {})
  })
}

console.log('iApi started')
