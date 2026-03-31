import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useEnvironmentStore } from '@/stores/environment'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue([]),
}))

describe('Environment Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('should have empty environments by default', () => {
      const store = useEnvironmentStore()

      expect(store.environments).toEqual([])
      expect(store.currentEnvironmentId).toBe('default')
    })
  })

  describe('createEnvironment', () => {
    it('should create a new environment', async () => {
      const store = useEnvironmentStore()

      const env = await store.createEnvironment('Test Environment')

      expect(store.environments.length).toBe(1)
      expect(store.environments[0].name).toBe('Test Environment')
      expect(env.name).toBe('Test Environment')
    })
  })

  describe('setCurrentEnvironment', () => {
    it('should set the current environment', async () => {
      const store = useEnvironmentStore()

      await store.createEnvironment('Env 1')
      await store.createEnvironment('Env 2')
      const env2Id = store.environments[1].id

      store.setCurrentEnvironment(env2Id)

      expect(store.currentEnvironmentId).toBe(env2Id)
    })
  })

  describe('variables computation', () => {
    it('should return enabled variables as key-value map', async () => {
      const store = useEnvironmentStore()

      await store.createEnvironment('Test Env')
      await store.addVariable(store.environments[0].id, {
        key: 'API_URL',
        value: 'https://api.example.com',
        enabled: true,
      })
      await store.addVariable(store.environments[0].id, {
        key: 'DISABLED_VAR',
        value: 'should not appear',
        enabled: false,
      })

      expect(store.variables).toEqual({
        API_URL: 'https://api.example.com',
      })
    })
  })

  describe('addVariable', () => {
    it('should add a variable to environment', async () => {
      const store = useEnvironmentStore()

      await store.createEnvironment('Test')
      const envId = store.environments[0].id
      await store.addVariable(envId, { key: 'KEY', value: 'VALUE', enabled: true })

      expect(store.environments[0].variables.length).toBe(1)
      expect(store.environments[0].variables[0].key).toBe('KEY')
    })
  })

  describe('updateVariable', () => {
    it('should update an existing variable', async () => {
      const store = useEnvironmentStore()

      await store.createEnvironment('Test')
      const envId = store.environments[0].id
      await store.addVariable(envId, { key: 'KEY', value: 'OLD', enabled: true })
      await store.updateVariable(envId, 0, { key: 'KEY', value: 'NEW', enabled: true })

      expect(store.environments[0].variables[0].value).toBe('NEW')
    })
  })

  describe('deleteVariable', () => {
    it('should delete a variable', async () => {
      const store = useEnvironmentStore()

      await store.createEnvironment('Test')
      const envId = store.environments[0].id
      await store.addVariable(envId, { key: 'TO_DELETE', value: 'VALUE', enabled: true })
      await store.deleteVariable(envId, 0)

      expect(store.environments[0].variables.length).toBe(0)
    })
  })

  describe('replaceVariables', () => {
    it('should replace simple variables', async () => {
      const store = useEnvironmentStore()

      await store.createEnvironment('Test')
      await store.addVariable(store.environments[0].id, {
        key: 'BASE_URL',
        value: 'https://api.example.com',
        enabled: true,
      })

      const result = store.replaceVariables('{{BASE_URL}}/users')

      expect(result).toBe('https://api.example.com/users')
    })

    it('should replace multiple variables', async () => {
      const store = useEnvironmentStore()

      await store.createEnvironment('Test')
      await store.addVariable(store.environments[0].id, { key: 'HOST', value: 'api', enabled: true })
      await store.addVariable(store.environments[0].id, { key: 'PORT', value: '8080', enabled: true })

      const result = store.replaceVariables('{{HOST}}:{{PORT}}')

      expect(result).toBe('api:8080')
    })

    it('should handle undefined variables', async () => {
      const store = useEnvironmentStore()

      const result = store.replaceVariables('{{NONEXISTENT}}')

      expect(result).toBe('{{NONEXISTENT}}')
    })

    it('should support default values', async () => {
      const store = useEnvironmentStore()

      await store.createEnvironment('Test')

      const result = store.replaceVariables('{{UNDEFINED:default_value}}')

      expect(result).toBe('default_value')
    })

    it('should support quoted default values', async () => {
      const store = useEnvironmentStore()

      await store.createEnvironment('Test')

      const result = store.replaceVariables('{{UNDEFINED:"quoted"}}')

      expect(result).toBe('quoted')
    })
  })

  describe('dynamic variables', () => {
    it('should resolve $timestamp', () => {
      const store = useEnvironmentStore()

      const result = store.replaceVariables('{{$timestamp}}')

      expect(result).toMatch(/^\d+$/)
    })

    it('should resolve $timestampISO', () => {
      const store = useEnvironmentStore()

      const result = store.replaceVariables('{{$timestampISO}}')

      expect(result).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/)
    })

    it('should resolve $date', () => {
      const store = useEnvironmentStore()

      const result = store.replaceVariables('{{$date}}')

      expect(result).toMatch(/^\d{4}-\d{2}-\d{2}$/)
    })

    it('should resolve $randomInt', () => {
      const store = useEnvironmentStore()

      const result = store.replaceVariables('{{$randomInt}}')

      expect(result).toMatch(/^\d+$/)
    })

    it('should resolve $randomUUID', () => {
      const store = useEnvironmentStore()

      const result = store.replaceVariables('{{$randomUUID}}')

      expect(result).toMatch(/^[0-9a-z]+$/)
    })
  })

  describe('nested variable replacement', () => {
    it('should handle nested variable references', async () => {
      const store = useEnvironmentStore()

      await store.createEnvironment('Test')
      await store.addVariable(store.environments[0].id, { key: 'BASE', value: 'https://api', enabled: true })
      await store.addVariable(store.environments[0].id, { key: 'ENDPOINT', value: '{{BASE}}/v1', enabled: true })

      const result = store.replaceVariables('{{ENDPOINT}}')

      expect(result).toBe('https://api/v1')
    })
  })
})
