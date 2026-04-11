import { ref, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { Response } from '@/types'

interface StreamOptions {
  requestId: string
  url: string
  method: string
  params?: Array<{ key: string; value: string; enabled: boolean }>
  headers?: Array<{ key: string; value: string; enabled: boolean }>
  body?: any
  auth?: any
  timeout?: number
  returnBytes?: boolean
}

interface StreamChunk {
  chunk: number[]
  headers: Record<string, string>
  status: number
  responseTime: number
  isFinal: boolean
}

interface StreamComplete {
  body: string
  headers: Record<string, string>
  status: number
  responseTime: number
  responseSize: number
  isFinal: boolean
}

export function useStreamedRequest() {
  const isLoading = ref(false)
  const progress = ref(0)
  const receivedBytes = ref(0)
  const error = ref<string | null>(null)
  const partialResponse = ref<Partial<Response> | null>(null)

  const eventListeners: Array<() => void> = []

  async function sendStreamedRequest(
    options: StreamOptions,
    callbacks: {
      onHeaders?: (headers: Record<string, string>, status: number) => void
      onChunk?: (chunk: Uint8Array, progress: number) => void
      onComplete?: (response: Response) => void
      onError?: (error: string) => void
    }
  ) {
    const appWindow = getCurrentWindow()
    const eventId = options.requestId

    isLoading.value = true
    progress.value = 0
    receivedBytes.value = 0
    error.value = null
    partialResponse.value = null

    try {
      // 监听响应头事件
      const unlistenHeaders = await listen(
        `${eventId}-headers`,
        (event: any) => {
          if (callbacks.onHeaders) {
            callbacks.onHeaders(event.payload.headers, event.payload.status)
          }
        }
      )
      eventListeners.push(unlistenHeaders)

      // 监听数据块事件
      const unlistenChunk = await listen(
        `${eventId}-chunk`,
        (event: any) => {
          const payload = event.payload as StreamChunk
          
          if (payload.chunk && payload.chunk.length > 0) {
            receivedBytes.value += payload.chunk.length
            
            // 估算进度（基于接收的字节数，假设最大 10MB）
            const estimatedTotal = 10 * 1024 * 1024
            progress.value = Math.min(99, Math.round((receivedBytes.value / estimatedTotal) * 100))

            if (callbacks.onChunk) {
              const uint8Array = new Uint8Array(payload.chunk)
              callbacks.onChunk(uint8Array, progress.value)
            }
          }
        }
      )
      eventListeners.push(unlistenChunk)

      // 监听完成事件
      const unlistenComplete = await listen(
        `${eventId}-complete`,
        (event: any) => {
          const payload = event.payload as StreamComplete
          
          isLoading.value = false
          progress.value = 100

          const response: Response = {
            status: payload.status,
            statusText: '',
            headers: payload.headers,
            cookies: [],
            body: payload.body,
            responseTime: payload.responseTime,
            responseSize: payload.responseSize,
          }

          partialResponse.value = response

          if (callbacks.onComplete) {
            callbacks.onComplete(response)
          }
        }
      )
      eventListeners.push(unlistenComplete)

      // 监听错误事件
      const unlistenError = await listen(
        `${eventId}-error`,
        (event: any) => {
          isLoading.value = false
          error.value = event.payload.error

          if (callbacks.onError) {
            callbacks.onError(event.payload.error)
          }
        }
      )
      eventListeners.push(unlistenError)

      // 调用后端流式接口
      await invoke('send_http_request_stream', {
        request: options,
        eventId,
        window: appWindow,
        historyLimit: 100,
      })
    } catch (e) {
      isLoading.value = false
      error.value = String(e)
      
      if (callbacks.onError) {
        callbacks.onError(String(e))
      }
    }
  }

  function cancel() {
    eventListeners.forEach(unlisten => unlisten())
    eventListeners.length = 0
    isLoading.value = false
  }

  onUnmounted(() => {
    cancel()
  })

  return {
    isLoading,
    progress,
    receivedBytes,
    error,
    partialResponse,
    sendStreamedRequest,
    cancel,
  }
}
