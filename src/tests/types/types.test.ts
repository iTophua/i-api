import { describe, it, expect } from 'vitest'
import {
  normalizeRequest,
  normalizeRequestBody,
  normalizeAuthConfig,
  toBackendRequest,
} from '@/types'

describe('Type Normalization Functions', () => {
  describe('normalizeRequest', () => {
    it('should handle request with basic fields', () => {
      const input = {
        id: 'test-id',
        name: 'Test Request',
        method: 'POST',
        url: 'https://api.example.com',
      }

      const result = normalizeRequest(input)

      expect(result.id).toBe('test-id')
      expect(result.name).toBe('Test Request')
      expect(result.method).toBe('POST')
    })

    it('should generate UUID if not provided', () => {
      const input = {
        name: 'Test',
        method: 'GET',
        url: 'https://api.example.com',
      }

      const result = normalizeRequest(input)

      expect(result.id).toBeDefined()
      expect(result.id.length).toBeGreaterThan(0)
    })

    it('should handle pre_script and post_script fields', () => {
      const input = {
        name: 'Test',
        method: 'POST',
        url: 'https://api.example.com',
        pre_script: 'console.log("pre")',
        post_script: 'console.log("post")',
      }

      const result = normalizeRequest(input)

      expect(result.preScript).toBe('console.log("pre")')
      expect(result.postScript).toBe('console.log("post")')
    })

    it('should normalize body correctly', () => {
      const input = {
        name: 'Test',
        method: 'POST',
        url: 'https://api.example.com',
        body: { mode: 'raw', raw: 'test', rawType: 'json' },
      }

      const result = normalizeRequest(input)

      expect(result.body.mode).toBe('raw')
      expect(result.body.raw).toBe('test')
    })

    it('should normalize auth correctly', () => {
      const input = {
        name: 'Test',
        method: 'POST',
        url: 'https://api.example.com',
        auth: { type: 'bearer', bearer: { token: 'my-token' } },
      }

      const result = normalizeRequest(input)

      expect(result.auth.type).toBe('bearer')
      expect(result.auth.bearer?.token).toBe('my-token')
    })
  })

  describe('normalizeRequestBody', () => {
    it('should normalize body with camelCase mode', () => {
      const input = { bodyMode: 'raw', raw: 'test', rawType: 'json' }

      const result = normalizeRequestBody(input)

      expect(result.mode).toBe('raw')
      expect(result.rawType).toBe('json')
    })

    it('should normalize body with snake_case mode', () => {
      const input = { body_mode: 'urlencoded', urlencoded: [] }

      const result = normalizeRequestBody(input)

      expect(result.mode).toBe('urlencoded')
    })

    it('should return default body when input is null', () => {
      const result = normalizeRequestBody(null)

      expect(result.mode).toBe('none')
    })

    it('should normalize form data type fields', () => {
      const input = {
        formData: [
          { key: 'file', value: '', enabled: true, type: 'file', filePath: '/path/to/file' },
        ],
      }

      const result = normalizeRequestBody(input)

      expect(result.formData).toBeDefined()
      expect(result.formData?.[0].type).toBe('file')
      expect(result.formData?.[0].filePath).toBe('/path/to/file')
    })
  })

  describe('normalizeAuthConfig', () => {
    it('should normalize basic auth', () => {
      const input = { type: 'basic', basic: { username: 'user', password: 'pass' } }

      const result = normalizeAuthConfig(input)

      expect(result.type).toBe('basic')
      expect(result.basic?.username).toBe('user')
      expect(result.basic?.password).toBe('pass')
    })

    it('should normalize bearer auth', () => {
      const input = { type: 'bearer', bearer: { token: 'my-token' } }

      const result = normalizeAuthConfig(input)

      expect(result.type).toBe('bearer')
      expect(result.bearer?.token).toBe('my-token')
    })

    it('should normalize apikey auth', () => {
      const input = { type: 'apikey', apikey: { key: 'X-API-Key', value: 'secret', add_to: 'header' } }

      const result = normalizeAuthConfig(input)

      expect(result.type).toBe('apikey')
      expect(result.apikey?.key).toBe('X-API-Key')
      expect(result.apikey?.addTo).toBe('header')
    })

    it('should default to none auth', () => {
      const input = {}

      const result = normalizeAuthConfig(input)

      expect(result.type).toBe('none')
    })

    it('should handle snake_case auth_type', () => {
      const input = { auth_type: 'bearer', bearer: { token: 'token' } }

      const result = normalizeAuthConfig(input)

      expect(result.type).toBe('bearer')
    })
  })

  describe('toBackendRequest', () => {
    it('should convert request to backend format', () => {
      const input = {
        id: 'test-id',
        name: 'Test',
        method: 'POST' as const,
        url: 'https://api.example.com',
        params: [],
        headers: [],
        body: { mode: 'raw' as const, raw: '{"test":true}', rawType: 'json' as const },
        auth: { type: 'bearer' as const, bearer: { token: 'token' } },
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      }

      const result = toBackendRequest(input)

      expect(result.id).toBe('test-id')
      expect(result.created_at).toBe('2024-01-01T00:00:00Z')
      expect(result.body.body_mode).toBe('raw')
    })

    it('should handle apikey add_to conversion', () => {
      const input = {
        id: 'test-id',
        name: 'Test',
        method: 'GET' as const,
        url: 'https://api.example.com',
        params: [],
        headers: [],
        body: { mode: 'none' as const },
        auth: { type: 'apikey' as const, apikey: { key: 'key', value: 'val', addTo: 'query' as const } },
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      }

      const result = toBackendRequest(input)

      expect(result.auth?.apikey?.add_to).toBe('query')
    })
  })
})
