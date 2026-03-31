import { ref } from 'vue'
import { defineStore } from 'pinia'
import type { History } from '@/types'
import { useSettingsStore } from './settings'

export const useHistoryStore = defineStore('history', () => {
  const histories = ref<History[]>([])

  function addHistory(history: Omit<History, 'id' | 'createdAt'>) {
    const settingsStore = useSettingsStore()
    const maxHistory = settingsStore.settings?.historyLimit ?? 100
    const newHistory: History = {
      ...history,
      id: crypto.randomUUID(),
      createdAt: new Date().toISOString(),
    }
    histories.value.unshift(newHistory)
    if (histories.value.length > maxHistory) {
      histories.value = histories.value.slice(0, maxHistory)
    }
  }

  function clearHistory() {
    histories.value = []
  }

  function deleteHistory(id: string) {
    histories.value = histories.value.filter((h: History) => h.id !== id)
  }

  return {
    histories,
    addHistory,
    clearHistory,
    deleteHistory,
  }
})
