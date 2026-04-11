<script setup lang="ts">
import { NModal, NCard, NSelect, NButton, NSpace, useMessage } from 'naive-ui'
import { ref, computed, defineAsyncComponent } from 'vue'
import type { Request } from '@/types'
import type { Language } from '@/composables/useMonacoEditor'
import { generateCode, type CodeLanguage } from '@/utils/codeGenerator'

const MonacoEditor = defineAsyncComponent(() =>
  import('@/components/common/MonacoEditor.vue')
)
const message = useMessage()

const props = defineProps<{
  show: boolean
  request: Request | null
}>()

const emit = defineEmits<{
  'update:show': [show: boolean]
}>()

const languageOptions = [
  { label: 'cURL', value: 'curl' },
  { label: 'JavaScript (Axios)', value: 'javascript-axios' },
  { label: 'JavaScript (Fetch)', value: 'javascript-fetch' },
  { label: 'Python (requests)', value: 'python' },
  { label: 'Java (OkHttp)', value: 'java' },
  { label: 'Go', value: 'go' },
]

const selectedLanguage = ref<CodeLanguage>('curl')
const isCopying = ref(false)

const code = computed(() => {
  if (!props.request) return ''
  return generateCode(props.request, selectedLanguage.value)
})

const editorLanguage = computed<Language>(() => {
  const langMap: Record<CodeLanguage, Language> = {
    curl: 'plaintext',
    'javascript-axios': 'javascript',
    'javascript-fetch': 'javascript',
    python: 'plaintext',
    java: 'plaintext',
    go: 'plaintext',
  }
  return langMap[selectedLanguage.value] || 'plaintext'
})

async function copyCode() {
  if (!code.value) {
    message.warning('没有可复制的代码')
    return
  }
  
  try {
    isCopying.value = true
    await navigator.clipboard.writeText(code.value)
    message.success('代码已复制到剪贴板')
  } catch (err) {
    console.error('复制失败:', err)
    message.error('复制失败，请手动复制')
  } finally {
    isCopying.value = false
  }
}

function handleClose() {
  emit('update:show', false)
}
</script>

<template>
  <NModal :show="props.show" @update:show="emit('update:show', $event)">
    <NCard style="width: 700px" title="生成代码" :bordered="false" size="medium">
      <div class="code-generator">
        <div class="language-selector">
          <NSelect
            v-model:value="selectedLanguage"
            :options="languageOptions"
            style="width: 200px"
          />
          <NButton :loading="isCopying" @click="copyCode">复制代码</NButton>
        </div>

        <div class="code-preview">
          <MonacoEditor
            v-if="MonacoEditor"
            :model-value="code"
            :language="editorLanguage"
            read-only
            height="400px"
          />
        </div>
      </div>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="handleClose">关闭</NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>

<style scoped>
.code-generator {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.language-selector {
  display: flex;
  gap: 12px;
  align-items: center;
}

.code-preview {
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  overflow: hidden;
}
</style>
