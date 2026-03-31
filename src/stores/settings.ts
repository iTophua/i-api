import { ref, watch } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Settings, AppState, Locale } from '@/types'

const defaultSettings: Settings = {
  theme: 'system',
  language: 'zh-CN',
  historyLimit: 100,
  timeout: 30000,
  downloadPath: '',
  downloadAsk: true,
}

const defaultAppState: AppState = {
  sidebarCollapsed: false,
}

let i18nInstance: any = null

export function setI18nInstance(instance: any) {
  i18nInstance = instance
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Settings>({ ...defaultSettings })
  const appState = ref<AppState>({ ...defaultAppState })
  let mediaQuery: MediaQueryList | null = null

  function updateSettings(updates: Partial<Settings>) {
    Object.assign(settings.value, updates)
  }

  function setTheme(theme: Settings['theme']) {
    settings.value.theme = theme
    applyTheme(theme)
  }

  function setLanguage(language: Locale) {
    settings.value.language = language
    localStorage.setItem('iapi-locale', language)
    if (i18nInstance) {
      i18nInstance.global.locale.value = language
      document.documentElement.setAttribute('lang', language)
    }
  }

  function applyTheme(theme: Settings['theme']) {
    let actualTheme = theme
    if (theme === 'system') {
      actualTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    }
    document.documentElement.setAttribute('data-theme', actualTheme)
  }

  function handleSystemThemeChange(_e: MediaQueryListEvent) {
    if (settings.value.theme === 'system') {
      applyTheme('system')
    }
  }

  function setupMediaQueryListener() {
    if (typeof window !== 'undefined') {
      mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      mediaQuery.addEventListener('change', handleSystemThemeChange)
    }
  }

  function cleanupMediaQueryListener() {
    if (mediaQuery) {
      mediaQuery.removeEventListener('change', handleSystemThemeChange)
    }
  }

  async function loadSettings() {
    try {
      const savedSettings = localStorage.getItem('iapi-settings')
      if (savedSettings) {
        settings.value = { ...defaultSettings, ...JSON.parse(savedSettings) }
      }

      const savedLocale = localStorage.getItem('iapi-locale') as Locale | null
      if (savedLocale && (savedLocale === 'zh-CN' || savedLocale === 'en-US')) {
        settings.value.language = savedLocale
      }
    } catch (e) {
      console.error('加载设置失败:', e)
    }
  }

  async function saveSettings() {
    try {
      localStorage.setItem('iapi-settings', JSON.stringify(settings.value))
    } catch (e) {
      console.error('保存设置失败:', e)
    }
  }

  async function loadAppState() {
    try {
      const state = await invoke<AppState>('get_app_state')
      appState.value = { ...defaultAppState, ...state }
    } catch (e) {
      console.error('加载应用状态失败:', e)
    }
  }

  async function saveAppState() {
    try {
      await invoke('save_app_state', { state: appState.value })
    } catch (e) {
      console.error('保存应用状态失败:', e)
    }
  }

  function updateAppState(updates: Partial<AppState>) {
    Object.assign(appState.value, updates)
  }

  watch(
    () => settings.value.theme,
    (theme) => applyTheme(theme),
    { immediate: true }
  )

  watch(
    () => settings.value,
    () => saveSettings(),
    { deep: true }
  )

  setupMediaQueryListener()

  return {
    settings,
    appState,
    updateSettings,
    setTheme,
    setLanguage,
    applyTheme,
    loadSettings,
    saveSettings,
    loadAppState,
    saveAppState,
    updateAppState,
    cleanupMediaQueryListener,
  }
})
