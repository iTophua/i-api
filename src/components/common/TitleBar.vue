<script setup lang="ts">
import { NButton, NSelect, NIcon, NTooltip } from 'naive-ui'
import {
  SettingsSharp,
  MoonOutline,
  SunnyOutline,
  GlobeOutline,
} from '@vicons/ionicons5'
import { computed } from 'vue'
import { useI18n } from '@/composables/useI18n'
import { useSettingsStore, useEnvironmentStore } from '@/stores'
import Logo from './Logo.vue'

const settingsStore = useSettingsStore()
const environmentStore = useEnvironmentStore()
const { t } = useI18n()

defineEmits<{
  'toggle-theme': []
  'select-environment': [id: string]
  'open-settings': []
  'open-environment-manager': []
}>()

const isDark = computed(() => {
  const theme = settingsStore.settings.theme
  if (theme === 'dark') return true
  if (theme === 'light') return false
  return window.matchMedia('(prefers-color-scheme: dark)').matches
})

const environmentOptions = computed(() =>
  environmentStore.environments.map((e) => ({
    label: e.name,
    value: e.id,
  }))
)
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-traffic-light-spacer"></div>

    <div class="titlebar-logo">
      <Logo :size="24" :show-text="false" />
    </div>

    <div class="titlebar-drag-area"></div>

    <div class="toolbar-actions">
      <NSelect
        :value="environmentStore.currentEnvironmentId"
        :options="environmentOptions"
        style="width: 140px"
        size="small"
        :placeholder="t('environment.selectEnvironment')"
        @update:value="(id: string) => $emit('select-environment', id)"
      />
      <NTooltip :show-arrow="false">
        <template #trigger>
          <NButton quaternary circle size="small" @click="$emit('open-environment-manager')">
            <template #icon>
              <NIcon :component="GlobeOutline" />
            </template>
          </NButton>
        </template>
        {{ t('environment.manageEnvironments') }}
      </NTooltip>
      <NTooltip :show-arrow="false">
        <template #trigger>
          <NButton quaternary circle size="small" @click="$emit('toggle-theme')">
            <template #icon>
              <NIcon :component="isDark ? SunnyOutline : MoonOutline" />
            </template>
          </NButton>
        </template>
        {{ isDark ? t('settings.light') : t('settings.dark') }}
      </NTooltip>
      <NTooltip :show-arrow="false">
        <template #trigger>
          <NButton quaternary circle size="small" @click="$emit('open-settings')">
            <template #icon>
              <NIcon :component="SettingsSharp" />
            </template>
          </NButton>
        </template>
        {{ t('settings.settings') }}
      </NTooltip>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  height: 38px;
  display: flex;
  align-items: center;
  background: var(--color-bg-base);
  user-select: none;
  -webkit-app-region: drag;
}

.titlebar-traffic-light-spacer {
  width: 80px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.titlebar-logo {
  display: flex;
  align-items: center;
  margin-left: 8px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.titlebar-drag-area {
  flex: 1;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  justify-content: flex-end;
  padding-right: 12px;
  -webkit-app-region: no-drag;
}

.toolbar-actions :deep(.n-button),
.toolbar-actions :deep(.n-select) {
  -webkit-app-region: no-drag;
}
</style>
