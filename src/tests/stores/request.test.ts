import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useRequestStore } from '@/stores/request'
import type { Request } from '@/types'

vi.mock('@/stores/settings', () => ({
  useSettingsStore: vi.fn().mockReturnValue({
    settings: { theme: 'light', language: 'zh-CN' },
  }),
}))

describe('Request Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('createDefaultRequest', () => {
    it('should create a request with default values', () => {
      const store = useRequestStore()
      const request = store.currentRequest

      expect(request.method).toBe('GET')
      expect(request.name).toBe('未命名请求')
      expect(request.url).toBe('')
      expect(request.params).toEqual([])
      expect(request.headers).toEqual([])
      expect(request.body.mode).toBe('none')
      expect(request.auth.type).toBe('none')
    })
  })

  describe('openRequest', () => {
    it('should open a request in a new tab', () => {
      const store = useRequestStore()
      const request: Request = {
        id: 'test-id',
        name: 'Test Request',
        method: 'POST',
        url: 'https://api.example.com/test',
        params: [],
        headers: [],
        body: { mode: 'raw', raw: '{"key":"value"}' },
        auth: { type: 'bearer', bearer: { token: 'test-token' } },
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      }

      store.openRequest(request)

      expect(store.tabs.length).toBe(1)
      expect(store.activeTabId).toBe(store.tabs[0].id)
      expect(store.currentRequest.name).toBe('Test Request')
    })

    it('should switch to existing tab if request is already open', () => {
      const store = useRequestStore()
      const request: Request = {
        id: 'test-id',
        name: 'Test Request',
        method: 'POST',
        url: 'https://api.example.com/test',
        params: [],
        headers: [],
        body: { mode: 'none' },
        auth: { type: 'none' },
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      }

      store.openRequest(request)
      store.openRequest(request)

      expect(store.tabs.length).toBe(1)
    })
  })

  describe('closeTab', () => {
    it('should close tab and switch to another', () => {
      const store = useRequestStore()

      store.openRequest({ ...store.currentRequest, id: '1', name: 'Request 1' })
      store.openRequest({ ...store.currentRequest, id: '2', name: 'Request 2' })
      const firstTabId = store.tabs[0].id

      store.closeTab(store.tabs[1].id)

      expect(store.tabs.length).toBe(1)
      expect(store.tabs[0].id).toBe(firstTabId)
    })
  })

  describe('updateMethod', () => {
    it('should update request method', () => {
      const store = useRequestStore()
      store.newRequest()

      store.updateMethod('POST')

      expect(store.currentRequest.method).toBe('POST')
    })
  })

  describe('updateUrl', () => {
    it('should update request URL', () => {
      const store = useRequestStore()
      store.newRequest()

      store.updateUrl('https://api.example.com/users')

      expect(store.currentRequest.url).toBe('https://api.example.com/users')
    })
  })

  describe('updateParams', () => {
    it('should update request params', () => {
      const store = useRequestStore()
      store.newRequest()

      const params = [
        { key: 'page', value: '1', enabled: true },
        { key: 'limit', value: '10', enabled: true },
      ]

      store.updateParams(params)

      expect(store.currentRequest.params).toEqual(params)
    })
  })

  describe('updateHeaders', () => {
    it('should update request headers', () => {
      const store = useRequestStore()
      store.newRequest()

      const headers = [
        { key: 'Content-Type', value: 'application/json', enabled: true },
        { key: 'Authorization', value: 'Bearer token', enabled: true },
      ]

      store.updateHeaders(headers)

      expect(store.currentRequest.headers).toEqual(headers)
    })
  })

  describe('updateBody', () => {
    it('should update request body', () => {
      const store = useRequestStore()
      store.newRequest()

      const body = { mode: 'raw' as const, raw: '{"test":true}', rawType: 'json' as const }

      store.updateBody(body)

      expect(store.currentRequest.body.mode).toBe('raw')
      expect(store.currentRequest.body.raw).toBe('{"test":true}')
    })
  })

  describe('updateAuth', () => {
    it('should update auth configuration', () => {
      const store = useRequestStore()
      store.newRequest()

      const auth = { type: 'basic' as const, basic: { username: 'user', password: 'pass' } }

      store.updateAuth(auth)

      expect(store.currentRequest.auth.type).toBe('basic')
      expect(store.currentRequest.auth.basic?.username).toBe('user')
    })
  })

  describe('createCollection', () => {
    it('should create a new collection', async () => {
      const store = useRequestStore()

      await store.createCollection('My Collection')

      expect(store.collections.length).toBe(1)
      expect(store.collections[0].name).toBe('My Collection')
    })
  })

  describe('deleteCollection', () => {
    it('should delete a collection', async () => {
      const store = useRequestStore()

      const collection = await store.createCollection('Test Collection')
      await store.deleteCollection(collection.id)

      expect(store.collections.length).toBe(0)
    })
  })

  describe('setPendingRequestId', () => {
    it('should track pending request', () => {
      const store = useRequestStore()

      store.setPendingRequestId('test-request-id')

      expect(store.pendingRequestId).toBe('test-request-id')

      store.setPendingRequestId(null)

      expect(store.pendingRequestId).toBeNull()
    })
  })
})
