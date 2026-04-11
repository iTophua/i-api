import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { History } from '@/types'
import { useSettingsStore } from './settings'
import { safeParseDate } from '@/types'

export const useHistoryStore = defineStore('history', () => {
  const histories = ref<History[]>([])
  const searchQuery = ref('')
  const filterMethod = ref<string>('all')
  const filterStatus = ref<string>('all')
  const dateRange = ref<{ start: number | null; end: number | null }>({ start: null, end: null })

  const settingsStore = useSettingsStore()

  const filteredHistories = computed(() => {
    let result = [...histories.value]

    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      result = result.filter(
        (h) =>
          h.url.toLowerCase().includes(query) ||
          h.method.toLowerCase().includes(query)
      )
    }

    if (filterMethod.value !== 'all') {
      result = result.filter((h) => h.method === filterMethod.value)
    }

    if (filterStatus.value !== 'all') {
      if (filterStatus.value === '2xx') {
        result = result.filter((h) => h.status >= 200 && h.status < 300)
      } else if (filterStatus.value === '3xx') {
        result = result.filter((h) => h.status >= 300 && h.status < 400)
      } else if (filterStatus.value === '4xx') {
        result = result.filter((h) => h.status >= 400 && h.status < 500)
      } else if (filterStatus.value === '5xx') {
        result = result.filter((h) => h.status >= 500 && h.status < 600)
      }
    }

    if (dateRange.value.start) {
      const startDate = new Date(dateRange.value.start)
      if (!isNaN(startDate.getTime())) {
        result = result.filter((h) => new Date(safeParseDate(h.createdAt)) >= startDate)
      }
    }

    if (dateRange.value.end) {
      const endDate = new Date(dateRange.value.end)
      if (!isNaN(endDate.getTime())) {
        endDate.setHours(23, 59, 59, 999)
        result = result.filter((h) => new Date(safeParseDate(h.createdAt)) <= endDate)
      }
    }

    return result
  })

  const statistics = computed(() => {
    const total = histories.value.length
    const successCount = histories.value.filter((h) => h.status >= 200 && h.status < 300).length
    const errorCount = histories.value.filter((h) => h.status >= 400).length
    const avgResponseTime =
      histories.value.length > 0
        ? Math.round(histories.value.reduce((sum, h) => sum + h.responseTime, 0) / histories.value.length)
        : 0

    const methodStats: Record<string, number> = {}
    histories.value.forEach((h) => {
      methodStats[h.method] = (methodStats[h.method] || 0) + 1
    })

    return {
      total,
      successCount,
      errorCount,
      avgResponseTime,
      methodStats,
    }
  })

  async function loadHistory(limit?: number) {
    try {
      const historyLimit = limit ?? settingsStore.settings?.historyLimit ?? 100
      const result = await invoke<History[]>('get_recent_history', { limit: historyLimit })
      histories.value = result
    } catch (error) {
      console.error('加载历史记录失败:', error)
    }
  }

  function addHistory(history: Omit<History, 'id' | 'createdAt'>) {
    // 后端会自动保存历史记录，前端只需更新内存状态
    const newHistory: History = {
      ...history,
      id: crypto.randomUUID(),
      createdAt: new Date().toISOString(),
    }
    histories.value.unshift(newHistory)
    const maxHistory = settingsStore.settings?.historyLimit ?? 100
    if (histories.value.length > maxHistory) {
      histories.value = histories.value.slice(0, maxHistory)
    }
  }

  async function clearHistory() {
    try {
      await invoke('clear_history')
      histories.value = []
      searchQuery.value = ''
      filterMethod.value = 'all'
      filterStatus.value = 'all'
      dateRange.value = { start: null, end: null }
    } catch (error) {
      console.error('清空历史记录失败:', error)
    }
  }

  async function deleteHistory(id: string) {
    try {
      await invoke('delete_history', { id })
      histories.value = histories.value.filter((h: History) => h.id !== id)
    } catch (error) {
      console.error('删除历史记录失败:', error)
    }
  }

  function clearFiltered() {
    searchQuery.value = ''
    filterMethod.value = 'all'
    filterStatus.value = 'all'
    dateRange.value = { start: null, end: null }
  }

  function setSearchQuery(query: string) {
    searchQuery.value = query
  }

  function setFilterMethod(method: string) {
    filterMethod.value = method
  }

  function setFilterStatus(status: string) {
    filterStatus.value = status
  }

  function setDateRange(start: number | null, end: number | null) {
    dateRange.value = { start, end }
  }

  function getHistoryById(id: string): History | undefined {
    return histories.value.find((h) => h.id === id)
  }

  function exportHistory(): string {
    return JSON.stringify(histories.value, null, 2)
  }

  function importHistory(data: string): boolean {
    try {
      const imported = JSON.parse(data) as History[]
      if (Array.isArray(imported)) {
        histories.value = [...imported, ...histories.value]
        const maxHistory = settingsStore.settings?.historyLimit ?? 100
        if (histories.value.length > maxHistory) {
          histories.value = histories.value.slice(0, maxHistory)
        }
        return true
      }
      return false
    } catch {
      return false
    }
  }

  return {
    histories,
    filteredHistories,
    statistics,
    searchQuery,
    filterMethod,
    filterStatus,
    dateRange,
    loadHistory,
    addHistory,
    clearHistory,
    deleteHistory,
    clearFiltered,
    setSearchQuery,
    setFilterMethod,
    setFilterStatus,
    setDateRange,
    getHistoryById,
    exportHistory,
    importHistory,
  }
})
