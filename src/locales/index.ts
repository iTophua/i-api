import { createI18n } from 'vue-i18n'
import enUS from './en-US'
import zhCN from './zh-CN'
import type { Locale } from '@/types'

export type { Locale }

export const availableLocales: { label: string; value: Locale }[] = [
  { label: '简体中文', value: 'zh-CN' },
  { label: 'English', value: 'en-US' },
]

const messages = {
  'zh-CN': zhCN,
  'en-US': enUS,
}

export function createI18nInstance(locale: Locale = 'zh-CN') {
  return createI18n({
    legacy: false,
    locale,
    fallbackLocale: 'en-US',
    messages,
    missingWarn: false,
    fallbackWarn: false,
  })
}

export default createI18nInstance
