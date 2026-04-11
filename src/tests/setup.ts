import { vi } from 'vitest'

const mockCrypto = {
  randomUUID: () => Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15),
  getRandomValues: <T extends Uint8Array>(arr: T): T => {
    for (let i = 0; i < arr.length; i++) {
      arr[i] = Math.floor(Math.random() * 256) as unknown as T[number]
    }
    return arr
  },
  subtle: {} as Record<string, never>,
  webcrypto: {} as Record<string, never>,
}

Object.defineProperty(globalThis, 'crypto', {
  value: mockCrypto,
  writable: true,
  configurable: true,
})

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

window.scrollTo = vi.fn()
