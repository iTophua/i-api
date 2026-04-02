<script setup lang="ts">
interface Props {
  status: number
  size?: number
}

withDefaults(defineProps<Props>(), {
  size: 16,
})

const getStatusColor = (status: number): string => {
  if (status >= 200 && status < 300) return '#10b981'
  if (status >= 300 && status < 400) return '#3b82f6'
  if (status >= 400 && status < 500) return '#f59e0b'
  return '#ef4444'
}

const getStatusIcon = (status: number): string => {
  if (status >= 200 && status < 300) {
    // Success checkmark
    return 'M9 12L11 14L15 10M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z'
  } else if (status >= 300 && status < 400) {
    // Arrow/redirect
    return 'M13 5L20 5L20 12M8 16L16 8M20 12L11 21C9.89543 22.1046 8.10457 22.1046 7 21C5.89543 19.8954 5.89543 18.1046 7 17L16 8M12 4L5 11'
  } else if (status >= 400 && status < 500) {
    // Warning triangle
    return 'M12 9V11M12 15H12.01M5.07183 19H18.9282C20.4678 19 21.4301 17.3333 20.6603 16L13.7321 4C12.9623 2.66667 11.0377 2.66667 10.2679 4L3.33975 16C2.56994 17.3333 3.53223 19 5.07183 19Z'
  } else {
    // Error X
    return 'M18 6L6 18M6 6L18 18M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z'
  }
}
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="status-icon"
  >
    <path
      :d="getStatusIcon(status)"
      :stroke="getStatusColor(status)"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</template>

<style scoped>
.status-icon {
  display: inline-block;
  vertical-align: middle;
}
</style>
