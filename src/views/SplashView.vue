<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSettingsStore, useRequestStore, useEnvironmentStore, useHistoryStore } from '@/stores'

const router = useRouter()
const settingsStore = useSettingsStore()
const requestStore = useRequestStore()
const environmentStore = useEnvironmentStore()
const historyStore = useHistoryStore()

const progress = ref(0)
const status = ref('正在初始化...')
const currentStep = ref(0)
let redirectTimer: ReturnType<typeof setTimeout> | null = null

const loadingSteps = [
  { id: 1, text: '加载设置', weight: 12 },
  { id: 2, text: '加载环境变量', weight: 12 },
  { id: 3, text: '加载历史记录', weight: 6 },
  { id: 4, text: '加载集合', weight: 6 },
  { id: 5, text: '恢复上次会话', weight: 12 },
  { id: 6, text: '加载临时请求', weight: 12 },
  { id: 7, text: '恢复应用状态', weight: 20 },
  { id: 8, text: '准备就绪', weight: 20 },
]

onUnmounted(() => {
  if (redirectTimer) {
    clearTimeout(redirectTimer)
    redirectTimer = null
  }
})

onMounted(async () => {
  try {
    currentStep.value = 0
    status.value = '加载设置...'
    progress.value = 12
    await settingsStore.loadSettings()

    currentStep.value = 1
    status.value = '加载环境变量...'
    progress.value = 24
    await environmentStore.loadEnvironments()

    currentStep.value = 2
    status.value = '加载历史记录...'
    progress.value = 30
    await historyStore.loadHistory()

    currentStep.value = 3
    status.value = '加载集合...'
    progress.value = 36
    await requestStore.loadCollections()

    currentStep.value = 4
    status.value = '恢复上次会话...'
    progress.value = 48
    await requestStore.loadTabs()

    currentStep.value = 5
    status.value = '加载临时请求...'
    progress.value = 60
    await requestStore.loadTemporaryRequest()

    currentStep.value = 6
    status.value = '恢复应用状态...'
    progress.value = 80
    await settingsStore.loadAppState()

    currentStep.value = 7
    status.value = '准备就绪'
    progress.value = 100

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
    <div class="grid-background"></div>
    <div class="glow-orb glow-orb-1"></div>
    <div class="glow-orb glow-orb-2"></div>
    
    <div class="splash-content">
      <div class="logo-section">
        <div class="logo-wrapper">
          <div class="logo-icon">
            <svg viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
              <rect width="48" height="48" rx="12" fill="url(#gradient)" />
              <path d="M14 16L24 12L34 16V24L24 36L14 24V16Z" fill="white" fill-opacity="0.95" />
              <path
                d="M24 12V36M14 16L24 24L34 16"
                stroke="white"
                stroke-width="2"
                stroke-opacity="0.6"
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
                  <stop stop-color="#3b82f6" />
                  <stop offset="1" stop-color="#2563eb" />
                </linearGradient>
              </defs>
            </svg>
          </div>
          <div class="logo-glow"></div>
        </div>
        
        <div class="brand-text">
          <h1 class="logo-title">iApi</h1>
          <p class="logo-subtitle">API 调试工具</p>
        </div>
      </div>

      <div class="loading-section">
        <div class="progress-container">
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: `${progress}%` }">
              <div class="progress-glow"></div>
            </div>
          </div>
          <div class="progress-info">
            <span class="progress-percent">{{ progress }}%</span>
            <span class="progress-status">{{ status }}</span>
          </div>
        </div>

        <div class="steps-container">
          <div 
            v-for="(step, index) in loadingSteps" 
            :key="step.id"
            class="step-item"
            :class="{ 
              active: index === currentStep, 
              completed: index < currentStep 
            }"
          >
            <div class="step-indicator">
              <div class="step-dot"></div>
              <div v-if="index < loadingSteps.length - 1" class="step-line"></div>
            </div>
            <span class="step-text">{{ step.text }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="version-info">v1.0.0</div>
  </div>
</template>

<style scoped>
.splash-container {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #0a0e27 0%, #1a1f3a 50%, #0f1428 100%);
  position: relative;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

.grid-background {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-image: 
    linear-gradient(rgba(59, 130, 246, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(59, 130, 246, 0.03) 1px, transparent 1px);
  background-size: 50px 50px;
  animation: gridMove 20s linear infinite;
}

@keyframes gridMove {
  0% {
    transform: translate(0, 0);
  }
  100% {
    transform: translate(50px, 50px);
  }
}

.glow-orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.15;
  animation: orbFloat 8s ease-in-out infinite;
}

.glow-orb-1 {
  width: 400px;
  height: 400px;
  background: radial-gradient(circle, #3b82f6 0%, transparent 70%);
  top: -100px;
  right: -100px;
  animation-delay: 0s;
}

.glow-orb-2 {
  width: 300px;
  height: 300px;
  background: radial-gradient(circle, #2563eb 0%, transparent 70%);
  bottom: -50px;
  left: -50px;
  animation-delay: 4s;
}

@keyframes orbFloat {
  0%, 100% {
    transform: translate(0, 0) scale(1);
  }
  50% {
    transform: translate(30px, -30px) scale(1.1);
  }
}

.splash-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 60px;
  z-index: 1;
  animation: fadeInUp 0.8s ease-out;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(30px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.logo-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
}

.logo-wrapper {
  position: relative;
  animation: logoFloat 3s ease-in-out infinite;
}

@keyframes logoFloat {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

.logo-icon {
  width: 100px;
  height: 100px;
  position: relative;
  z-index: 2;
}

.logo-icon svg {
  width: 100%;
  height: 100%;
  filter: drop-shadow(0 8px 32px rgba(59, 130, 246, 0.4));
}

.logo-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 150px;
  height: 150px;
  background: radial-gradient(circle, rgba(59, 130, 246, 0.3) 0%, transparent 70%);
  border-radius: 50%;
  filter: blur(20px);
  z-index: 1;
  animation: glowPulse 2s ease-in-out infinite;
}

@keyframes glowPulse {
  0%, 100% {
    opacity: 0.5;
    transform: translate(-50%, -50%) scale(1);
  }
  50% {
    opacity: 0.8;
    transform: translate(-50%, -50%) scale(1.2);
  }
}

.brand-text {
  text-align: center;
}

.logo-title {
  font-size: 48px;
  font-weight: 700;
  color: #ffffff;
  letter-spacing: 4px;
  margin: 0;
  text-shadow: 0 0 20px rgba(59, 130, 246, 0.5);
  font-feature-settings: 'liga' 1, 'calt' 1;
}

.logo-subtitle {
  font-size: 16px;
  color: rgba(255, 255, 255, 0.6);
  margin: 8px 0 0 0;
  letter-spacing: 2px;
  font-weight: 300;
}

.loading-section {
  width: 400px;
  display: flex;
  flex-direction: column;
  gap: 32px;
  animation: fadeInUp 0.8s ease-out 0.2s both;
}

.progress-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.progress-bar {
  width: 100%;
  height: 6px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 3px;
  overflow: hidden;
  position: relative;
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.3);
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #3b82f6 0%, #60a5fa 100%);
  border-radius: 3px;
  transition: width 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  box-shadow: 0 0 20px rgba(59, 130, 246, 0.5);
}

.progress-glow {
  position: absolute;
  top: 0;
  right: 0;
  width: 20px;
  height: 100%;
  background: linear-gradient(90deg, transparent 0%, rgba(255, 255, 255, 0.6) 100%);
  animation: progressGlow 1.5s ease-in-out infinite;
}

@keyframes progressGlow {
  0%, 100% {
    opacity: 0.6;
  }
  50% {
    opacity: 1;
  }
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-percent {
  font-size: 14px;
  font-weight: 600;
  color: #60a5fa;
  font-variant-numeric: tabular-nums;
}

.progress-status {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
}

.steps-container {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.step-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  opacity: 0.3;
  transition: all 0.3s ease;
}

.step-item.active {
  opacity: 1;
}

.step-item.completed {
  opacity: 0.6;
}

.step-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  position: relative;
}

.step-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.3);
  transition: all 0.3s ease;
  position: relative;
  z-index: 2;
}

.step-item.active .step-dot {
  background: #3b82f6;
  box-shadow: 0 0 12px rgba(59, 130, 246, 0.8);
  animation: dotPulse 1.5s ease-in-out infinite;
}

.step-item.completed .step-dot {
  background: #60a5fa;
}

@keyframes dotPulse {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.3);
  }
}

.step-line {
  position: absolute;
  top: 8px;
  left: 50%;
  transform: translateX(-50%);
  width: 2px;
  height: 24px;
  background: rgba(255, 255, 255, 0.1);
}

.step-item.completed .step-line {
  background: rgba(59, 130, 246, 0.3);
}

.step-text {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.6);
  transition: all 0.3s ease;
}

.step-item.active .step-text {
  color: #ffffff;
  font-weight: 500;
}

.step-item.completed .step-text {
  color: rgba(255, 255, 255, 0.5);
}

.version-info {
  position: absolute;
  bottom: 24px;
  right: 24px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.3);
  font-weight: 300;
  letter-spacing: 1px;
}

@media (prefers-reduced-motion: reduce) {
  .grid-background,
  .glow-orb,
  .logo-wrapper,
  .logo-glow,
  .progress-glow,
  .step-item.active .step-dot {
    animation: none;
  }
  
  .progress-fill,
  .step-item,
  .step-dot,
  .step-text {
    transition: none;
  }
}
</style>
