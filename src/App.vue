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
:root {
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 12px;
  --spacing-lg: 16px;
  --spacing-xl: 24px;

  --z-dropdown: 1000;
  --z-modal: 2000;
  --z-toast: 3000;
  --z-tooltip: 4000;
}

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
  background-color: #ffffff;
  color: #333333;
  transition:
    background-color 0.3s ease,
    color 0.3s ease;
}

[data-theme='dark'] html,
[data-theme='dark'] body,
[data-theme='dark'] #app {
  background-color: #1a1a1a;
  color: #e0e0e0;
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
  border-radius: 4px;
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
</style>
