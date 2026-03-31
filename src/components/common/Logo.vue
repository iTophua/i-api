<script setup lang="ts">
defineProps<{
  size?: number
  showText?: boolean
}>()
</script>

<template>
  <div class="logo" :style="{ height: `${size || 32}px` }">
    <svg viewBox="0 0 60 60" :width="size || 32" :height="size || 32" class="logo-icon">
      <defs>
        <linearGradient id="iconGradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" style="stop-color:#7C3AED"/>
          <stop offset="100%" style="stop-color:#A855F7"/>
        </linearGradient>
        <linearGradient id="boltGradientIcon" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" style="stop-color:#FCD34D"/>
          <stop offset="100%" style="stop-color:#F59E0B"/>
        </linearGradient>
      </defs>
      <rect x="0" y="0" width="60" height="60" rx="14" fill="url(#iconGradient)"/>
      <circle cx="30" cy="16" r="6" fill="white"/>
      <path d="M26 26 L34 26 L31 42 L36 42 L24 58 L29 45 L22 45 Z" fill="url(#boltGradientIcon)"/>
    </svg>
    <span v-if="showText !== false" class="logo-text">
      <span class="logo-i">i</span><span class="logo-api">Api</span>
    </span>
  </div>
</template>

<style scoped>
.logo {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.logo-icon {
  flex-shrink: 0;
}

.logo-text {
  font-size: 1.2em;
  font-weight: 600;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  letter-spacing: -0.02em;
}

.logo-i {
  background: linear-gradient(135deg, #7C3AED 0%, #A855F7 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.logo-api {
  color: inherit;
}
</style>
