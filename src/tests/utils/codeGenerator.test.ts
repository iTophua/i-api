import { describe, it, expect } from 'vitest'
import { generateCode } from '@/utils/codeGenerator'
import type { Request } from '@/types'

const createMockRequest = (overrides: Partial<Request> = {}): Request => ({
  id: 'test-id',
  name: 'Test Request',
  method: 'GET',
  url: 'https://api.example.com/users',
  params: [],
  headers: [],
  body: { mode: 'none' },
  auth: { type: 'none' },
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
  ...overrides,
} as Request)

describe('Code Generator', () => {
  describe('cURL', () => {
    it('should generate basic GET request', () => {
      const request = createMockRequest({ method: 'GET', url: 'https://api.example.com/users' })

      const result = generateCode(request, 'curl')

      expect(result).toContain('curl')
      expect(result).toContain('https://api.example.com/users')
      expect(result).toContain('-X GET')
    })

    it('should generate POST request with JSON body', () => {
      const request = createMockRequest({
        method: 'POST',
        url: 'https://api.example.com/users',
        body: { mode: 'raw', raw: '{"Name":"test"}', rawType: 'json' },
      })

      const result = generateCode(request, 'curl')

      expect(result).toContain('-X POST')
      expect(result).toContain("'{\"Name\":\"test\"}'")
    })

    it('should include headers', () => {
      const request = createMockRequest({
        method: 'GET',
        url: 'https://api.example.com/users',
        headers: [
          { key: 'Authorization', value: 'Bearer token123', enabled: true },
          { key: 'X-Custom', value: 'custom-value', enabled: true },
        ],
      })

      const result = generateCode(request, 'curl')

      expect(result).toContain('Authorization: Bearer token123')
      expect(result).toContain('X-Custom: custom-value')
    })

    it('should include body for raw requests', () => {
      const request = createMockRequest({
        method: 'POST',
        url: 'https://api.example.com/users',
        body: { mode: 'raw', raw: 'test data', rawType: 'text' },
      })

      const result = generateCode(request, 'curl')

      expect(result).toContain('-d')
    })

    it('should exclude disabled headers', () => {
      const request = createMockRequest({
        method: 'GET',
        headers: [
          { key: 'Enabled', value: 'yes', enabled: true },
          { key: 'Disabled', value: 'no', enabled: false },
        ],
      })

      const result = generateCode(request, 'curl')

      expect(result).toContain('Enabled')
      expect(result).not.toContain('Disabled')
    })
  })

  describe('JavaScript (fetch)', () => {
    it('should generate fetch GET request', () => {
      const request = createMockRequest({ method: 'GET' })

      const result = generateCode(request, 'javascript-fetch')

      expect(result).toContain('fetch(')
      expect(result).toContain("method: 'GET'")
    })

    it('should generate fetch POST with body', () => {
      const request = createMockRequest({
        method: 'POST',
        body: { mode: 'raw', raw: '{"test":true}', rawType: 'json' },
      })

      const result = generateCode(request, 'javascript-fetch')

      expect(result).toContain("method: 'POST'")
      expect(result).toContain('body:')
    })
  })

  describe('Python (requests)', () => {
    it('should generate requests GET request', () => {
      const request = createMockRequest({ method: 'GET' })

      const result = generateCode(request, 'python')

      expect(result).toContain('requests.')
      expect(result).toContain('url')
    })

    it('should generate python with url', () => {
      const request = createMockRequest({ method: 'GET', url: 'https://api.example.com' })

      const result = generateCode(request, 'python')

      expect(result).toContain('https://api.example.com')
    })
  })

  describe('Axios', () => {
    it('should generate axios import', () => {
      const request = createMockRequest({ method: 'GET' })

      const result = generateCode(request, 'javascript-axios')

      expect(result).toContain('axios')
    })

    it('should generate axios POST request', () => {
      const request = createMockRequest({ method: 'POST' })

      const result = generateCode(request, 'javascript-axios')

      expect(result).toContain("method: 'post'")
    })
  })
})
