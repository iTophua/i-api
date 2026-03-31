<script setup lang="ts">
import { NButton, NSelect, NIcon, NDropdown, NTooltip } from 'naive-ui'
import {
  SettingsSharp,
  MoonOutline,
  SunnyOutline,
  DownloadOutline,
  CloudUploadOutline,
  CodeSlashOutline,
} from '@vicons/ionicons5'
import { computed } from 'vue'
import { useI18n } from '@/composables/useI18n'
import { useSettingsStore, useEnvironmentStore } from '@/stores'
import Logo from './Logo.vue'

const settingsStore = useSettingsStore()
const environmentStore = useEnvironmentStore()
const { t } = useI18n()

const emit = defineEmits<{
  import: [key: string]
  export: []
  code: [key: string]
  'toggle-theme': []
  'select-environment': [id: string]
  'open-settings': []
}>()

const importOptions = [
  { label: () => t('import.curl'), key: 'curl' },
  { label: () => t('import.postman'), key: 'postman' },
  { label: () => t('import.openapi'), key: 'openapi' },
  { label: () => t('import.har'), key: 'har' },
]

const codeOptions = [
  { label: () => t('request.copyAsCurl'), key: 'copy-curl' },
  { label: () => t('codeGenerator.generateCode'), key: 'generate' },
]

function handleImportSelect(key: string) {
  emit('import', key)
}

function handleCodeSelect(key: string) {
  emit('code', key)
}

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

    <div class="toolbar-actions">
      <NDropdown :options="importOptions" @select="handleImportSelect">
        <NButton quaternary size="small">
          <template #icon>
            <NIcon :component="CloudUploadOutline" />
          </template>
          {{ t('import.import') }}
        </NButton>
      </NDropdown>
      <NButton quaternary size="small" @click="$emit('export')">
        <template #icon>
          <NIcon :component="DownloadOutline" />
        </template>
        {{ t('export.export') }}
      </NButton>
      <NDropdown :options="codeOptions" @select="handleCodeSelect">
        <NButton quaternary size="small">
          <template #icon>
            <NIcon :component="CodeSlashOutline" />
          </template>
          {{ t('codeGenerator.codeGenerator') }}
        </NButton>
      </NDropdown>
      <div class="divider"></div>
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
          <NButton quaternary circle size="small" @click="$emit('toggle-theme')">
            <template #icon>
              <NIcon :component="isDark ? SunnyOutline : MoonOutline" />
            </template>
          </NButton>
        </template>
        {{ isDark ? t('settings.light') : t('settings.dark') }}
      </NTooltip>
      <NButton quaternary circle size="small" @click="$emit('open-settings')">
        <template #icon>
          <NIcon :component="SettingsSharp" />
        </template>
      </NButton>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  height: 38px;
  display: flex;
  align-items: center;
  background: var(--n-color);
  user-select: none;
  -webkit-app-region: drag;
}

.titlebar-traffic-light-spacer {
  width: 80px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.titlebar-logo {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  -webkit-app-region: no-drag;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 1;
  justify-content: flex-end;
  padding-right: 12px;
}

.divider {
  width: 1px;
  height: 20px;
  background: var(--n-border-color);
  margin: 0 8px;
}

.toolbar-actions :deep(.n-button),
.toolbar-actions :deep(.n-select) {
  -webkit-app-region: no-drag;
}
</style>
