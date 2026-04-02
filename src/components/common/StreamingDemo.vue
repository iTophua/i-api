<script setup lang="ts">
import { NButton, NProgress, NCard } from 'naive-ui'
import { useStreamedRequest } from '@/composables/useStreamedRequest'
import { ref } from 'vue'

const responseText = ref('')

const { isLoading, progress, receivedBytes, error, partialResponse, sendStreamedRequest } =
  useStreamedRequest()

async function handleSendStream() {
  responseText.value = ''

  await sendStreamedRequest(
    {
      requestId: `stream-${Date.now()}`,
      url: 'https://httpbin.org/stream/20',
      method: 'GET',
      headers: [],
      timeout: 30000,
    },
    {
      onHeaders: (_headers, _status) => {},
      onChunk: (chunk, _prog) => {
        const text = new TextDecoder().decode(chunk)
        responseText.value += text
      },
      onComplete: (_response) => {},
      onError: (_err) => {},
    }
  )
}
</script>

<template>
  <div class="streaming-demo" style="padding: 20px">
    <h2>HTTP 流式响应演示</h2>

    <NButton type="primary" :loading="isLoading" @click="handleSendStream">
      {{ isLoading ? '请求中...' : '发送流式请求' }}
    </NButton>

    <div v-if="isLoading" style="margin-top: 16px">
      <NProgress
        :percentage="progress"
        :status="progress < 100 ? 'success' : 'success'"
        :show-indicator="true"
      />
      <p>已接收字节：{{ receivedBytes }}</p>
    </div>

    <NCard v-if="error" title="错误" style="margin-top: 16px; background: var(--n-color-error)">
      {{ error }}
    </NCard>

    <NCard v-if="partialResponse" title="响应预览" style="margin-top: 16px">
      <p>状态：{{ partialResponse.status }}</p>
      <p>响应时间：{{ partialResponse.responseTime }}ms</p>
      <p>响应大小：{{ partialResponse.responseSize }} bytes</p>
    </NCard>

    <pre
      v-if="responseText"
      style="
        margin-top: 16px;
        padding: 12px;
        background: var(--n-color-hover);
        border-radius: 4px;
        max-height: 400px;
        overflow: auto;
      "
      >{{ responseText }}</pre
    >
  </div>
</template>
