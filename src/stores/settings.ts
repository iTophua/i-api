import { ref, watch, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { createI18n } from 'vue-i18n'
import type { Settings, AppState, Locale } from '@/types'

const defaultSettings: Settings = {
  theme: 'system',
  language: 'zh-CN',
  historyLimit: 100,
  timeout: 30000,
  downloadPath: '',
  downloadAsk: true,
  followRedirects: true,
  verifySsl: true,
}

const defaultAppState: AppState = {
  sidebarCollapsed: false,
}

let i18nInstance: ReturnType<typeof createI18n> | null = null

export function setI18nInstance(instance: ReturnType<typeof createI18n>) {
  i18nInstance = instance
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Settings>({ ...defaultSettings })
  const appState = ref<AppState>({ ...defaultAppState })
  let mediaQuery: MediaQueryList | null = null
  let saveTimeout: ReturnType<typeof setTimeout> | null = null

  // 计算属性优化：避免重复计算
  const isDarkTheme = computed(() => {
    if (settings.value.theme === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return settings.value.theme === 'dark'
  })

  const currentLocale = computed(() => settings.value.language)

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

  // 防抖保存设置
  function debouncedSaveSettings() {
    if (saveTimeout) {
      clearTimeout(saveTimeout)
    }
    saveTimeout = setTimeout(() => {
      saveSettings()
    }, 500)
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

  // 优化 watch：只监听必要字段
  watch(
    () => settings.value.theme,
    (theme) => applyTheme(theme),
    { immediate: true }
  )

  // 使用防抖保存
  watch(
    () => settings.value,
    () => debouncedSaveSettings(),
    { deep: true }
  )

  setupMediaQueryListener()

  return {
    settings,
    appState,
    isDarkTheme,
    currentLocale,
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
