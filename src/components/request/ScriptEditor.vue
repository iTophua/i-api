<script setup lang="ts">
import { NCollapse, NCollapseItem, NButton } from 'naive-ui'
import { computed, shallowRef, onMounted } from 'vue'

const MonacoEditor = shallowRef<unknown>(null)

onMounted(async () => {
  const mod = await import('@/components/common/MonacoEditor.vue')
  MonacoEditor.value = mod.default
})

const props = withDefaults(
  defineProps<{
    script?: string
    type?: 'preRequest' | 'test'
  }>(),
  {
    type: 'test',
    script: '',
  }
)

const emit = defineEmits<{
  'update:script': [script: string]
}>()

const scriptContent = computed({
  get: () => props.script || '',
  set: (val: string) => emit('update:script', val),
})

const preRequestSnippets = [
  {
    name: '设置环境变量',
    code: 'pm.environment.set("variableName", "variableValue");',
  },
  {
    name: '修改请求 URL',
    code: 'pm.request.url = "https://api.example.com/new-endpoint";',
  },
  {
    name: '修改请求方法',
    code: 'pm.request.method = "POST";',
  },
  {
    name: '使用动态变量',
    code: '// 通过 {{动态变量}} 语法在值中嵌入动态值，引擎会自动替换\npm.environment.set("requestId", "{{$randomUUID}}");\npm.environment.set("timestamp", "{{$timestamp}}");',
  },
  {
    name: '设置多个变量',
    code: 'pm.environment.set("baseUrl", "https://api.example.com");\npm.environment.set("apiKey", "your-api-key");',
  },
]

const testSnippets = [
  {
    name: '断言状态码',
    code: 'pm.test("状态码为 200", () => {\n  pm.expect(pm.response.status).to.equal(200);\n});',
  },
  {
    name: '断言响应时间',
    code: 'pm.test("响应时间小于 500ms", () => {\n  pm.expect(pm.response.time).to.be.below(500);\n});',
  },
  {
    name: '断言响应头存在',
    code: 'pm.test("Content-Type 存在", () => {\n  pm.response.to.have.header("Content-Type");\n});',
  },
  {
    name: '提取响应 JSON',
    code: '// 将整个响应体解析为 JSON 存入变量，后续请求可通过 {{responseData}} 使用\nconst responseData = pm.response.json();',
  },
  {
    name: '提取响应文本',
    code: '// 后续请求可通过 {{responseText}} 使用\nconst responseText = pm.response.text();',
  },
  {
    name: '提取响应头',
    code: '// 后续请求可通过 {{contentType}} 使用\nconst contentType = pm.response.headers.get("Content-Type");',
  },
  {
    name: '提取状态码',
    code: '// 后续请求可通过 {{responseStatus}} 使用\nconst responseStatus = pm.response.status;',
  },
  {
    name: '提取 Token（链式传递）',
    code: '// 从响应中提取 token，变量名即为 token，后续请求可通过 {{token}} 使用\nconst token = pm.response.json();',
  },
]

const snippets = computed(() => {
  return props.type === 'preRequest' ? preRequestSnippets : testSnippets
})

function insertSnippet(code: string) {
  scriptContent.value = scriptContent.value ? `${scriptContent.value}\n${code}` : code
}
</script>

<template>
  <div class="script-editor">
    <div class="editor-toolbar">
      <NCollapse>
        <NCollapseItem title="代码片段" name="snippets">
          <div class="snippet-list">
            <NButton
              v-for="snippet in snippets"
              :key="snippet.name"
              size="small"
              quaternary
              @click="insertSnippet(snippet.code)"
            >
              {{ snippet.name }}
            </NButton>
          </div>
        </NCollapseItem>
      </NCollapse>
    </div>
    <div class="editor-wrapper">
      <MonacoEditor
        v-if="MonacoEditor"
        v-model="scriptContent"
        language="javascript"
        height="100%"
      />
    </div>
  </div>
</template>

<style scoped>
.script-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.editor-toolbar {
  margin-bottom: 4px;
  flex-shrink: 0;
}

.editor-toolbar :deep(.n-collapse) {
  --n-title-font-size: 12px;
  --n-content-font-size: 12px;
  --n-title-padding: 4px 0 0 0;
  --n-content-padding: 2px 0 0 0;
}

.editor-toolbar :deep(.n-collapse-item) {
  margin: 0 !important;
}

.editor-toolbar :deep(.n-collapse-item__header) {
  padding: 2px 0 !important;
  font-size: 12px !important;
  height: 22px !important;
  min-height: 22px !important;
  line-height: 22px !important;
}

.editor-toolbar :deep(.n-collapse-item__content-wrapper) {
  padding: 0 !important;
}

.editor-toolbar :deep(.n-collapse-item__content-inner) {
  padding: 2px 0 !important;
}

.snippet-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.snippet-list :deep(.n-button) {
  --n-font-size: var(--font-size-compact-sm);
  --n-height: 20px;
  padding: 0 6px;
}

.snippet-list :deep(.n-button__content) {
  font-size: var(--font-size-compact-sm) !important;
}

.editor-wrapper {
  flex: 1;
  min-height: 0;
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>

<style>
.script-editor .n-collapse-item__content-wrapper {
  padding: 0 !important;
}

.script-editor .n-collapse-item__content-inner {
  padding: 4px 0 !important;
}

.script-editor .n-collapse-item__header {
  padding: 4px 0 !important;
  height: 24px !important;
  min-height: 24px !important;
  font-size: 11px !important;
}

.script-editor .n-collapse-item__header .n-collapse-item-arrow {
  font-size: 12px !important;
}

.script-editor .n-collapse-item {
  margin: 0 !important;
}
</style>
