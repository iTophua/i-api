import * as monaco from 'monaco-editor'
import { ref, onMounted, onUnmounted, type Ref } from 'vue'

export type Language = 'json' | 'javascript' | 'typescript' | 'html' | 'xml' | 'plaintext'

export interface EditorOptions {
  language: Language
  readOnly?: boolean
  minimap?: boolean
  lineNumbers?: boolean
  wordWrap?: 'on' | 'off'
  fontSize?: number
  tabSize?: number
  automaticLayout?: boolean
}

export function useMonacoEditor(
  containerRef: Ref<HTMLElement | null>,
  initialValue: string,
  options: EditorOptions
) {
  const editor = ref<monaco.editor.IStandaloneCodeEditor | null>(null)
  const content = ref(initialValue)

  const defaultOptions: monaco.editor.IStandaloneEditorConstructionOptions = {
    language: options.language,
    readOnly: options.readOnly ?? false,
    minimap: { enabled: options.minimap ?? false },
    lineNumbers: options.lineNumbers ?? 'on' ? 'on' : 'off',
    wordWrap: options.wordWrap ?? 'on',
    fontSize: options.fontSize ?? 13,
    tabSize: options.tabSize ?? 2,
    automaticLayout: options.automaticLayout ?? true,
    scrollBeyondLastLine: false,
    folding: true,
    foldingStrategy: 'indentation',
    renderWhitespace: 'selection',
    formatOnPaste: true,
    formatOnType: true,
    theme: 'vs',
    padding: { top: 8, bottom: 8 },
    scrollbar: {
      verticalScrollbarSize: 8,
      horizontalScrollbarSize: 8,
    },
  }

  function initEditor() {
    if (!containerRef.value) return

    editor.value = monaco.editor.create(containerRef.value, {
      ...defaultOptions,
      value: content.value,
    })

    editor.value.onDidChangeModelContent(() => {
      content.value = editor.value?.getValue() || ''
    })
  }

  function setValue(value: string) {
    if (editor.value) {
      editor.value.setValue(value)
      content.value = value
    }
  }

  function getValue(): string {
    return editor.value?.getValue() || ''
  }

  function formatJson() {
    if (!editor.value || options.language !== 'json') return

    const value = editor.value.getValue()
    const formatted = formatJsonString(value, 2)
    if (formatted !== value) {
      editor.value.setValue(formatted)
    }
  }

  function setLanguage(lang: Language) {
    if (!editor.value) return

    const model = editor.value.getModel()
    if (model) {
      monaco.editor.setModelLanguage(model, lang)
    }
  }

  function setTheme(isDark: boolean) {
    monaco.editor.setTheme(isDark ? 'vs-dark' : 'vs')
  }

  function focus() {
    editor.value?.focus()
  }

  function dispose() {
    editor.value?.dispose()
    editor.value = null
  }

  onMounted(() => {
    initEditor()
  })

  onUnmounted(() => {
    dispose()
  })

  return {
    editor,
    content,
    setValue,
    getValue,
    formatJson,
    setLanguage,
    setTheme,
    focus,
    dispose,
  }
}

export function formatJsonString(json: string, indent = 2): string {
  try {
    const parsed = JSON.parse(json)
    return JSON.stringify(parsed, null, indent)
  } catch {
    return json
  }
}

export function minifyJsonString(json: string): string {
  try {
    const parsed = JSON.parse(json)
    return JSON.stringify(parsed)
  } catch {
    return json
  }
}

export function detectLanguage(content: string): Language {
  const trimmed = content.trim()
  
  if (isValidJson(trimmed)) {
    return 'json'
  }
  
  if (trimmed.startsWith('<')) {
    if (trimmed.includes('<?xml') || trimmed.includes('<!DOCTYPE')) {
      return 'xml'
    }
    return 'html'
  }
  
  if (trimmed.startsWith('<script') || trimmed.includes('function') || trimmed.includes('const ') || trimmed.includes('let ')) {
    return 'javascript'
  }
  
  return 'plaintext'
}

function isValidJson(content: string): boolean {
  if (!content.startsWith('{') && !content.startsWith('[')) {
    return false
  }
  try {
    JSON.parse(content)
    return true
  } catch {
    return false
  }
}