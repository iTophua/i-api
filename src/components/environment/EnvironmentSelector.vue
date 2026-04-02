<script setup lang="ts">
import { NSelect } from 'naive-ui'
import { computed } from 'vue'
import { useEnvironmentStore } from '@/stores'

const environmentStore = useEnvironmentStore()

const options = computed(() => {
  return environmentStore.environments.map((env) => ({
    label: `${env.name} (${env.variables.length}个变量)`,
    value: env.id,
  }))
})

function handleChange(value: string | number | null) {
  if (value) {
    environmentStore.setCurrentEnvironment(String(value))
  }
}
</script>

<template>
  <NSelect
    :value="environmentStore.currentEnvironmentId"
    :options="options"
    size="small"
    class="environment-selector"
    @update:value="handleChange"
  />
</template>

<style scoped>
.environment-selector {
  min-width: 180px;
}
</style>
