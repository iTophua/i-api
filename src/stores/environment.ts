import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Environment, Variable } from '@/types'
import { safeParseDate } from '@/types'

export const useEnvironmentStore = defineStore('environment', () => {
  const environments = ref<Environment[]>([])
  const savedEnvId = (() => {
    try {
      return localStorage.getItem('iapi-current-environment') || 'default'
    } catch {
      return 'default'
    }
  })()
  const currentEnvironmentId = ref<string>(savedEnvId)
  const managerEnvironmentId = ref<string>('default')
  const isLoaded = ref(false)

  // 运行时变量（会话级临时变量，不持久化）
  // 由脚本（pre/post）执行后回传，优先级高于环境变量，用于实现跨请求的变量传递
  // 例如：登录请求的 post 脚本提取 token → 下一个请求的 url 中 {{token}} 自动替换
  const runtimeVariables = ref<Record<string, string>>({})

  const currentEnvironment = computed(() => {
    const env = environments.value.find((e: Environment) => e.id === currentEnvironmentId.value)
    if (env) return env
    if (environments.value.length > 0) return environments.value[0]
    return null
  })

  const managerEnvironment = computed(() => {
    const env = environments.value.find((e: Environment) => e.id === managerEnvironmentId.value)
    if (env) return env
    if (environments.value.length > 0) return environments.value[0]
    return null
  })

  const variables = computed(() => {
    const vars: Record<string, string> = {}
    // 环境变量（持久化）
    currentEnvironment.value?.variables
      .filter((v: Variable) => v.enabled)
      .forEach((v: Variable) => {
        vars[v.key] = v.value
      })
    // 运行时变量覆盖同名环境变量（脚本设置的变量优先级更高）
    Object.assign(vars, runtimeVariables.value)
    return vars
  })

  function setCurrentEnvironment(id: string) {
    currentEnvironmentId.value = id
    localStorage.setItem('iapi-current-environment', id)
  }

  function setManagerEnvironment(id: string) {
    managerEnvironmentId.value = id
  }

  async function createEnvironment(name: string): Promise<Environment> {
    const env: Environment = {
      id: crypto.randomUUID(),
      name,
      variables: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    }

    try {
      await invoke('save_environment', { environment: env })
      environments.value = [...environments.value, env]
      return env
    } catch (e) {
      console.error('创建环境失败:', e)
      throw e
    }
  }

  async function renameEnvironment(id: string, newName: string) {
    const env = environments.value.find((e: Environment) => e.id === id)
    if (env) {
      env.name = newName
      env.updatedAt = new Date().toISOString()
      try {
        await invoke('save_environment', { environment: env })
      } catch (e) {
        console.error('重命名环境失败:', e)
        throw e
      }
    }
  }

  async function duplicateEnvironment(id: string): Promise<Environment | null> {
    const sourceEnv = environments.value.find((e: Environment) => e.id === id)
    if (!sourceEnv) return null

    const newEnv: Environment = {
      id: crypto.randomUUID(),
      name: `${sourceEnv.name} (副本)`,
      variables: JSON.parse(JSON.stringify(sourceEnv.variables)),
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    }

    try {
      await invoke('save_environment', { environment: newEnv })
      environments.value = [...environments.value, newEnv]
      return newEnv
    } catch (e) {
      console.error('复制环境失败:', e)
      throw e
    }
  }

  async function updateEnvironment(id: string, updates: Partial<Environment>) {
    const env = environments.value.find((e: Environment) => e.id === id)
    if (env) {
      const updatedEnv = {
        ...env,
        ...updates,
        updatedAt: new Date().toISOString(),
      }

      try {
        await invoke('save_environment', { environment: updatedEnv })
        Object.assign(env, updatedEnv)
      } catch (e) {
        console.error('更新环境失败:', e)
        throw e
      }
    }
  }

  async function deleteEnvironment(id: string) {
    // 禁止删除默认环境
    if (id === 'default') {
      console.warn('不能删除默认环境')
      return
    }

    if (environments.value.length <= 1) return

    try {
      await invoke('delete_environment', { id })
      environments.value = environments.value.filter((e: Environment) => e.id !== id)
      if (currentEnvironmentId.value === id && environments.value.length > 0) {
        currentEnvironmentId.value = environments.value[0].id
      }
      if (managerEnvironmentId.value === id && environments.value.length > 0) {
        managerEnvironmentId.value = environments.value[0].id
      }
    } catch (e) {
      console.error('删除环境失败:', e)
      throw e
    }
  }

  async function addVariable(envId: string, variable: Variable) {
    const env = environments.value.find((e: Environment) => e.id === envId)
    if (env) {
      const variableWithId = {
        ...variable,
        id: variable.id || crypto.randomUUID(),
      }
      env.variables.push(variableWithId)
      await updateEnvironment(envId, { variables: env.variables })
    }
  }

  async function updateVariable(envId: string, index: number, variable: Variable) {
    const env = environments.value.find((e: Environment) => e.id === envId)
    if (env && env.variables[index]) {
      env.variables[index] = variable
      await updateEnvironment(envId, { variables: env.variables })
    }
  }

  async function deleteVariable(envId: string, index: number) {
    const env = environments.value.find((e: Environment) => e.id === envId)
    if (env) {
      env.variables.splice(index, 1)
      await updateEnvironment(envId, { variables: env.variables })
    }
  }

  function replaceVariables(template: string): string {
    if (!template) return template

    const maxIterations = 10
    let result = template
    let previousResult = ''
    let iteration = 0

    while (result !== previousResult && iteration < maxIterations) {
      previousResult = result
      result = replaceVariablesOnce(result)
      iteration++
    }

    return result
  }

  function replaceVariablesOnce(template: string): string {
    return template.replace(/\{\{([^}]+)\}\}/g, (match, expr: string) => {
      const trimmedExpr = expr.trim()

      const colonIndex = trimmedExpr.indexOf(':')
      if (colonIndex > 0) {
        const varName = trimmedExpr.substring(0, colonIndex).trim()
        const defaultValue = trimmedExpr.substring(colonIndex + 1).trim()

        if (variables.value[varName] !== undefined) {
          return variables.value[varName]
        }

        if (defaultValue.startsWith('"') && defaultValue.endsWith('"')) {
          return defaultValue.slice(1, -1)
        }
        if (defaultValue.startsWith("'") && defaultValue.endsWith("'")) {
          return defaultValue.slice(1, -1)
        }

        return defaultValue
      }

      if (variables.value[trimmedExpr] !== undefined) {
        return variables.value[trimmedExpr]
      }

      const dynamicValue = resolveDynamicVariable(trimmedExpr)
      if (dynamicValue !== null) {
        return dynamicValue
      }

      console.warn(`[iApi] 未定义的变量: ${trimmedExpr}`)
      return match
    })
  }

  function resolveDynamicVariable(expr: string): string | null {
    const dynamicFunctions: Record<string, () => string> = {
      $timestamp: () => Date.now().toString(),
      $timestampISO: () => new Date().toISOString(),
      $date: () => new Date().toISOString().split('T')[0],
      $time: () => new Date().toTimeString().split(' ')[0],
      $year: () => new Date().getFullYear().toString(),
      $month: () => (new Date().getMonth() + 1).toString().padStart(2, '0'),
      $day: () => new Date().getDate().toString().padStart(2, '0'),
      $hour: () => new Date().getHours().toString().padStart(2, '0'),
      $minute: () => new Date().getMinutes().toString().padStart(2, '0'),
      $second: () => new Date().getSeconds().toString().padStart(2, '0'),
      $randomInt: () => Math.floor(Math.random() * 1000000).toString(),
      $randomUUID: () => crypto.randomUUID(),
      $randomAlpha: () =>
        Array.from({ length: 8 }, () =>
          String.fromCharCode(65 + Math.floor(Math.random() * 26))
        ).join(''),
      $randomAlphaNumeric: () =>
        Array.from(
          { length: 8 },
          () =>
            'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'[
              Math.floor(Math.random() * 62)
            ]
        ).join(''),
    }

    if (expr.startsWith('$')) {
      const funcName = expr.split('(')[0]
      if (dynamicFunctions[funcName]) {
        return dynamicFunctions[funcName]()
      }

      if (expr.startsWith('$randomInt(') && expr.endsWith(')')) {
        const args = expr.slice(11, -1)
        const [min, max] = args.split(',').map((s) => parseInt(s.trim(), 10))
        if (!isNaN(min) && !isNaN(max)) {
          return Math.floor(Math.random() * (max - min + 1) + min).toString()
        }
      }

      if (expr.startsWith('$randomAlpha(') && expr.endsWith(')')) {
        const length = parseInt(expr.slice(13, -1), 10) || 8
        return Array.from({ length }, () =>
          String.fromCharCode(65 + Math.floor(Math.random() * 26))
        ).join('')
      }

      if (expr.startsWith('$randomAlphaNumeric(') && expr.endsWith(')')) {
        const length = parseInt(expr.slice(21, -1), 10) || 8
        const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
        return Array.from({ length }, () => chars[Math.floor(Math.random() * chars.length)]).join(
          ''
        )
      }
    }

    return null
  }

  function getVariable(name: string): string | undefined {
    return variables.value[name]
  }

  function setVariable(name: string, value: string): void {
    const env = currentEnvironment.value
    if (!env) return

    const existingIndex = env.variables.findIndex((v) => v.key === name)
    if (existingIndex >= 0) {
      env.variables[existingIndex].value = value
    } else {
      env.variables.push({
        key: name,
        value,
        enabled: true,
      })
    }

    updateEnvironment(env.id, { variables: env.variables })
  }

  // 批量写入运行时变量（供脚本执行后回传变量调用）
  function setRuntimeVariables(vars: Record<string, string>): void {
    runtimeVariables.value = { ...runtimeVariables.value, ...vars }
  }

  function clearRuntimeVariables(): void {
    runtimeVariables.value = {}
  }

  async function loadEnvironments() {
    if (isLoaded.value) return

    try {
      const loadedEnvs = await invoke<Environment[]>('get_all_environments')

      if (loadedEnvs && loadedEnvs.length > 0) {
        environments.value = loadedEnvs.map(env => ({
          ...env,
          createdAt: safeParseDate(env.createdAt),
          updatedAt: safeParseDate(env.updatedAt),
        }))
        
        // 检查是否存在默认环境，不存在则创建
        const hasDefaultEnv = environments.value.some(e => e.id === 'default')
        if (!hasDefaultEnv) {
          const defaultEnv: Environment = {
            id: 'default',
            name: '默认环境',
            variables: [],
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
          }
          await invoke('save_environment', { environment: defaultEnv })
          environments.value.unshift(defaultEnv)
        }

        if (!environments.value.some(e => e.id === currentEnvironmentId.value)) {
          currentEnvironmentId.value = 'default'
        }
      } else {
        const defaultEnv: Environment = {
          id: 'default',
          name: '默认环境',
          variables: [],
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        }
        await invoke('save_environment', { environment: defaultEnv })
        environments.value = [defaultEnv]
      }

      isLoaded.value = true
    } catch (e) {
      console.error('加载环境失败:', e)
      const defaultEnv: Environment = {
        id: 'default',
        name: '默认环境',
        variables: [],
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      }
      environments.value = [defaultEnv]
      isLoaded.value = true
    }
  }

  return {
    environments,
    currentEnvironmentId,
    managerEnvironmentId,
    currentEnvironment,
    managerEnvironment,
    variables,
    setCurrentEnvironment,
    setManagerEnvironment,
    createEnvironment,
    renameEnvironment,
    duplicateEnvironment,
    updateEnvironment,
    deleteEnvironment,
    addVariable,
    updateVariable,
    deleteVariable,
    replaceVariables,
    getVariable,
    setVariable,
    setRuntimeVariables,
    clearRuntimeVariables,
    loadEnvironments,
  }
})
