<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider, darkTheme, lightTheme } from 'naive-ui'
import { computed, onMounted, onUnmounted } from 'vue'
import { useSettingsStore, useRequestStore } from '@/stores'

const settingsStore = useSettingsStore()
const requestStore = useRequestStore()

const theme = computed(() => {
  const currentTheme = settingsStore.settings.theme
  if (currentTheme === 'dark') return darkTheme
  if (currentTheme === 'light') return lightTheme
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? darkTheme : lightTheme
})

async function handleBeforeUnload() {
  await requestStore.saveTabs()
}

onMounted(() => {
  window.addEventListener('beforeunload', handleBeforeUnload)
})

onUnmounted(() => {
  window.removeEventListener('beforeunload', handleBeforeUnload)
  settingsStore.cleanupMediaQueryListener()
})
</script>

<template>
  <NConfigProvider :theme="theme">
    <NMessageProvider>
      <NDialogProvider>
        <router-view />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>

<style>
@import './styles/theme.css';

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html,
body,
#app {
  height: 100%;
  overflow: hidden;
  background-color: var(--color-bg-base);
  color: var(--color-text-primary);
  font-family: var(--font-family-base);
  font-size: var(--font-size-sm);
  line-height: var(--line-height-base);
  transition:
    background-color var(--transition-base),
    color var(--transition-base);
}

/* 禁用浏览器自动填充和拼写检查 */
input,
textarea,
[contenteditable],
[contenteditable="true"],
[contenteditable="false"] {
  autocomplete: off !important;
  -webkit-autocomplete: off !important;
  spellcheck: false !important;
}

input:-webkit-autofill,
input:-webkit-autofill:hover,
input:-webkit-autofill:focus,
input-autofill,
textarea:-webkit-autofill,
textarea:-webkit-autofill:hover,
textarea:-webkit-autofill:focus,
textarea-autofill {
  autocomplete: off !important;
  -webkit-autocomplete: off !important;
  spellcheck: false !important;
}

input:not([autocomplete]),
textarea:not([autocomplete]),
input[autocomplete="off"],
textarea[autocomplete="off"],
input[spellcheck="false"],
textarea[spellcheck="false"] {
  autocomplete: off !important;
  spellcheck: false !important;
}

.n-input input[type="text"],
.n-input input[type="url"],
.n-input input[type="search"],
.n-input input:not([type]),
.n-input textarea,
.n-base-selection-input,
.n-input-wrapper input,
.n-input-wrapper textarea,
.n-input .n-input__input,
.n-input__input-el,
.url-input input,
.request-panel input,
input.url-input,
.n-input *,
.n-input-wrapper * {
  autocomplete: off !important;
  -webkit-autocomplete: off !important;
  spellcheck: false !important;
}

/* Webkit 滚动条 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(128, 128, 128, 0.4);
  border-radius: var(--radius-sm);
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(128, 128, 128, 0.6);
}

[data-theme='dark'] ::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
}

[data-theme='dark'] ::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}

/* Firefox 滚动条 */
* {
  scrollbar-width: thin;
  scrollbar-color: rgba(128, 128, 128, 0.4) transparent;
}

[data-theme='dark'] * {
  scrollbar-color: rgba(255, 255, 255, 0.2) transparent;
}

/* ===== Naive UI Tooltip 全局紧凑样式 ===== */
.n-tooltip {
  font-size: var(--font-size-compact-sm) !important;
  --n-padding: 2px 6px !important;
  --n-height: 22px !important;
  --n-font-size: var(--font-size-compact-sm) !important;
}

.n-tooltip .n-tooltip-trigger {
  font-size: var(--font-size-compact-sm) !important;
}

.n-tooltip .n-tooltip-content {
  font-size: var(--font-size-compact-sm) !important;
  padding: 0 !important;
  margin: 0 !important;
}

.n-tooltip .n-tooltip-content .n-tooltip-arrow {
  display: none !important;
}

.n-tooltip .n-tooltip-content .n-tooltip-ref {
  font-size: var(--font-size-compact-sm) !important;
  padding: 1px 4px !important;
  line-height: 1.2 !important;
}

.n-popover {
  font-size: var(--font-size-compact-sm) !important;
  --n-padding: 2px 6px !important;
}

.n-popover .n-popover-content {
  font-size: var(--font-size-compact-sm) !important;
  padding: 0 !important;
  margin: 0 !important;
}

/* ===== 下拉菜单紧凑样式 ===== */
.n-dropdown {
  font-size: var(--font-size-compact-sm) !important;
}

.n-dropdown .n-dropdown-menu .n-dropdown-item {
  font-size: var(--font-size-compact-sm) !important;
  padding: 4px 12px !important;
  min-height: 28px !important;
}
</style>
