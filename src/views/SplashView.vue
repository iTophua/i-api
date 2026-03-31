<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSettingsStore, useRequestStore, useEnvironmentStore } from '@/stores'

const router = useRouter()
const settingsStore = useSettingsStore()
const requestStore = useRequestStore()
const environmentStore = useEnvironmentStore()

const progress = ref(0)
const status = ref('正在初始化...')
let redirectTimer: ReturnType<typeof setTimeout> | null = null

onUnmounted(() => {
  if (redirectTimer) {
    clearTimeout(redirectTimer)
    redirectTimer = null
  }
})

onMounted(async () => {
  try {
    // 阶段1: 加载设置
    status.value = '加载设置...'
    progress.value = 12
    await settingsStore.loadSettings()

    // 阶段2: 加载环境
    status.value = '加载环境变量...'
    progress.value = 24
    await environmentStore.loadEnvironments()

    // 阶段3: 加载集合
    status.value = '加载集合...'
    progress.value = 36
    await requestStore.loadCollections()

    // 阶段4: 加载上次打开的 tabs
    status.value = '恢复上次会话...'
    progress.value = 48
    await requestStore.loadTabs()

    // 阶段5: 加载临时请求
    status.value = '加载临时请求...'
    progress.value = 60
    await requestStore.loadTemporaryRequest()

    // 阶段6: 加载应用状态
    status.value = '恢复应用状态...'
    progress.value = 80
    await settingsStore.loadAppState()

    // 阶段7: 完成
    status.value = '准备就绪'
    progress.value = 100

    // 短暂延迟后进入主页
    redirectTimer = setTimeout(() => {
      router.replace('/')
    }, 500)
  } catch (e) {
    console.error('初始化失败:', e)
    status.value = '初始化失败'
    setTimeout(() => {
      router.replace('/')
    }, 1000)
  }
})
</script>

<template>
  <div class="splash-container">
    <div class="splash-content">
      <!-- Logo 区域 -->
      <div class="logo-area">
        <div class="logo-icon">
          <svg viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect width="48" height="48" rx="12" fill="url(#gradient)" />
            <path d="M14 16L24 12L34 16V24L24 36L14 24V16Z" fill="white" fill-opacity="0.9" />
            <path
              d="M24 12V36M14 16L24 24L34 16"
              stroke="white"
              stroke-width="2"
              stroke-opacity="0.5"
            />
            <defs>
              <linearGradient
                id="gradient"
                x1="0"
                y1="0"
                x2="48"
                y2="48"
                gradientUnits="userSpaceOnUse"
              >
                <stop stop-color="#6366f1" />
                <stop offset="1" stop-color="#8b5cf6" />
              </linearGradient>
            </defs>
          </svg>
        </div>
        <h1 class="logo-text">iApi</h1>
        <p class="logo-subtitle">API 调试工具</p>
      </div>

      <!-- 加载进度 -->
      <div class="loading-area">
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: `${progress}%` }"></div>
        </div>
        <p class="status-text">{{ status }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.splash-container {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f0f23 100%);
  overflow: hidden;
}

.splash-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 48px;
}

.logo-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  animation: fadeInDown 0.6s ease-out;
}

.logo-icon {
  width: 80px;
  height: 80px;
  animation: pulse 2s ease-in-out infinite;
}

.logo-icon svg {
  width: 100%;
  height: 100%;
  filter: drop-shadow(0 4px 20px rgba(99, 102, 241, 0.4));
}

.logo-text {
  font-size: 36px;
  font-weight: 700;
  color: #ffffff;
  letter-spacing: 2px;
  margin: 0;
}

.logo-subtitle {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.6);
  margin: 0;
}

.loading-area {
  width: 240px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  animation: fadeInUp 0.6s ease-out 0.3s both;
}

.progress-bar {
  width: 100%;
  height: 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #6366f1, #8b5cf6);
  border-radius: 2px;
  transition: width 0.3s ease-out;
}

.status-text {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
  margin: 0;
}

@keyframes fadeInDown {
  from {
    opacity: 0;
    transform: translateY(-20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes pulse {
  0%,
  100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.05);
  }
}
</style>
