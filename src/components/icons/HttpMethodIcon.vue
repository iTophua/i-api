<script setup lang="ts">
import { computed } from 'vue'

type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'OPTIONS' | 'HEAD'

interface Props {
  method: HttpMethod
  size?: number
  filled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  size: 16,
  filled: false,
})

const iconPaths: Record<HttpMethod, string> = {
  GET: 'M12 4L8 8H11V16H13V8H16L12 4ZM6 10V12H10V10H6ZM6 14V16H10V14H6Z',
  POST: 'M8 4V16H10V12H14V16H16V4H14V10H10V4H8Z',
  PUT: 'M8 4V16H10V12H14V16H16V4H14V10H10V4H8ZM11 6H13V8H11V6Z',
  DELETE: 'M7 4V6H17V4H7ZM9 8V16H11V8H9ZM13 8V16H15V8H13ZM6 10V16H8V10H6ZM16 10V16H18V10H16Z',
  PATCH: 'M12 4L8 8H11V16H13V12H15L16 13V16H18V12L14 8H11V7.41L13.29 9.71L14.71 8.29L12 5.59V4ZM11 10H13V11H11V10Z',
  OPTIONS: 'M12 4C14.21 4 16 5.79 16 8C16 10.21 14.21 12 12 12C9.79 12 8 10.21 8 8C8 5.79 9.79 4 12 4M12 6C10.9 6 10 6.9 10 8C10 9.1 10.9 10 12 10C13.1 10 14 9.1 14 8C14 6.9 13.1 6 12 6M8 12V16H10V14H14V16H16V12H8Z',
  HEAD: 'M8 4V16H10V10H13L15 16H17.21L14.5 8.73L17 4H14.5L13 7.5L11.5 4H8Z',
}

/* 颜色统一来自 theme.css 的 --color-method-*，与 tab 标签、方法选择器保持一致 */
const methodVar: Record<HttpMethod, string> = {
  GET: '--color-method-get',
  POST: '--color-method-post',
  PUT: '--color-method-put',
  DELETE: '--color-method-delete',
  PATCH: '--color-method-patch',
  OPTIONS: '--color-method-options',
  HEAD: '--color-method-head',
}

const viewBox = computed(() => `0 0 24 24`)
const pathStyle = computed(() => ({ color: `var(${methodVar[props.method]})` }))
</script>

<template>
  <svg
    :width="size"
    :height="size"
    :viewBox="viewBox"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    class="http-method-icon"
  >
    <path
      :d="iconPaths[method]"
      :style="pathStyle"
      :fill="filled ? 'currentColor' : 'none'"
      :stroke="filled ? 'none' : 'currentColor'"
      :stroke-width="filled ? 0 : 1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</template>

<style scoped>
.http-method-icon {
  display: inline-block;
  vertical-align: middle;
}
</style>
