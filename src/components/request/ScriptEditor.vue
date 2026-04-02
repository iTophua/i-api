<script setup lang="ts">
import { NCollapse, NCollapseItem, NButton } from 'naive-ui'
import { computed, shallowRef, onMounted } from 'vue'

const MonacoEditor = shallowRef<any>(null)

onMounted(async () => {
  const mod = await import('@/components/common/MonacoEditor.vue')
  MonacoEditor.value = mod.default
})

const props = defineProps<{
  script?: string
}>()

const emit = defineEmits<{
  'update:script': [script: string]
}>()

const scriptContent = computed({
  get: () => props.script || '',
  set: (val: string) => emit('update:script', val),
})

const snippets = [
  {
    name: '获取响应状态码',
    code: 'const status = pm.response.status;\nconsole.log("状态码:", status);',
  },
  {
    name: '提取 JSON 字段',
    code: 'const json = JSON.parse(pm.response.body);\nconst value = json.data.key;\npm.environment.set("extractedValue", value);',
  },
  {
    name: '断言状态码',
    code: 'pm.test("状态码为 200", () => {\n  pm.expect(pm.response.status).to.equal(200);\n});',
  },
  {
    name: '断言响应时间',
    code: 'pm.test("响应时间小于 500ms", () => {\n  pm.expect(pm.response.responseTime).to.be.below(500);\n});',
  },
  {
    name: '设置环境变量',
    code: 'pm.environment.set("variableName", "variableValue");',
  },
  {
    name: '获取环境变量',
    code: 'const value = pm.environment.get("variableName");\nconsole.log("变量值:", value);',
  },
]

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
        v-if="MonacoEditor.value"
        v-model="scriptContent"
        language="javascript"
        height="300px"
      />
    </div>
  </div>
</template>

<style scoped>
.script-editor {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.editor-toolbar {
  margin-bottom: 8px;
  flex-shrink: 0;
}

.snippet-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.editor-wrapper {
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  overflow: hidden;
}
</style>
