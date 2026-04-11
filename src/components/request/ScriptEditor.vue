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
    name: '获取环境变量',
    code: 'const value = pm.environment.get("variableName");\nconsole.log("变量值:", value);',
  },
  {
    name: '设置请求头',
    code: 'pm.request.headers.add({\n  key: "Header-Name",\n  value: "Header-Value"\n});',
  },
  {
    name: '设置 URL 参数',
    code: 'pm.request.url.query.add({\n  key: "paramName",\n  value: "paramValue"\n});',
  },
  {
    name: '生成时间戳',
    code: 'const timestamp = Date.now();\npm.environment.set("timestamp", timestamp);',
  },
  {
    name: '生成随机字符串',
    code: 'const randomStr = Math.random().toString(36).substring(7);\npm.environment.set("randomStr", randomStr);',
  },
  {
    name: '生成 UUID',
    code: 'const uuid = crypto.randomUUID();\npm.environment.set("uuid", uuid);',
  },
  {
    name: 'Base64 编码',
    code: 'const encoded = btoa("Hello World");\npm.environment.set("encoded", encoded);',
  },
]

const testSnippets = [
  {
    name: '断言状态码',
    code: 'pm.test("状态码为 200", () => {\n  pm.expect(pm.response.status).to.equal(200);\n});',
  },
  {
    name: '断言响应时间',
    code: 'pm.test("响应时间小于 500ms", () => {\n  pm.expect(pm.response.responseTime).to.be.below(500);\n});',
  },
  {
    name: '获取响应状态码',
    code: 'const status = pm.response.status;\nconsole.log("状态码:", status);',
  },
  {
    name: '提取 JSON 字段',
    code: 'const json = JSON.parse(pm.response.body);\nconst value = json.data.key;\npm.environment.set("extractedValue", value);',
  },
  {
    name: '断言 JSON 字段',
    code: 'const json = JSON.parse(pm.response.body);\npm.test("字段存在", () => {\n  pm.expect(json.data).to.have.property("key");\n});',
  },
  {
    name: '断言响应包含文本',
    code: 'pm.test("响应包含预期文本", () => {\n  pm.expect(pm.response.body).to.include("expectedText");\n});',
  },
  {
    name: '断言数组长度',
    code: 'const json = JSON.parse(pm.response.body);\npm.test("数组长度正确", () => {\n  pm.expect(json.data).to.be.an("array");\n  pm.expect(json.data).to.have.lengthOf(10);\n});',
  },
  {
    name: '设置下次请求变量',
    code: 'const json = JSON.parse(pm.response.body);\npm.environment.set("nextRequestId", json.data.id);',
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
