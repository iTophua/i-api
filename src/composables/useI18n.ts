import { useI18n as useVueI18n } from 'vue-i18n'
import { computed } from 'vue'
import type { Locale } from '@/types'
import { availableLocales } from '@/locales'

export function useI18n() {
  const { t, locale } = useVueI18n()

  const currentLocale = computed({
    get: () => locale.value as Locale,
    set: (val: Locale) => {
      locale.value = val
    },
  })

  const locales = availableLocales

  function setLocale(newLocale: Locale) {
    locale.value = newLocale
    document.documentElement.setAttribute('lang', newLocale)
  }

  return {
    t,
    locale: currentLocale,
    locales,
    setLocale,
  }
}

export function useLocale() {
  const { locale } = useVueI18n()

  const isZhCN = computed(() => locale.value === 'zh-CN')
  const isEnUS = computed(() => locale.value === 'en-US')

  function toggleLocale() {
    locale.value = locale.value === 'zh-CN' ? 'en-US' : 'zh-CN'
    document.documentElement.setAttribute('lang', locale.value)
  }

  return {
    locale,
    isZhCN,
    isEnUS,
    toggleLocale,
  }
}
