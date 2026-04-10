import { ref, computed, watch } from 'vue'
import { useHistoryStore } from '@/stores'

export function useUrlAutocomplete() {
  const historyStore = useHistoryStore()
  const showSuggestions = ref(false)
  const selectedIndex = ref(-1)

  const currentInput = ref('')

  const suggestions = computed(() => {
    if (!currentInput.value || currentInput.value.length < 2) {
      return []
    }

    const query = currentInput.value.toLowerCase()
    const uniqueUrls = new Map<string, { url: string; method: string; count: number }>()

    historyStore.histories.forEach((h) => {
      if (h.url.toLowerCase().includes(query)) {
        const existing = uniqueUrls.get(h.url)
        if (existing) {
          existing.count++
        } else {
          uniqueUrls.set(h.url, { url: h.url, method: h.method, count: 1 })
        }
      }
    })

    return Array.from(uniqueUrls.values())
      .sort((a, b) => b.count - a.count)
      .slice(0, 8)
  })

  function updateInput(value: string) {
    currentInput.value = value
    showSuggestions.value = value.length >= 2
    selectedIndex.value = -1
  }

  function selectSuggestion(suggestion: { url: string; method: string }) {
    currentInput.value = suggestion.url
    showSuggestions.value = false
    selectedIndex.value = -1
    return suggestion
  }

  function handleKeyDown(event: KeyboardEvent, callback: (url: string) => void) {
    if (!showSuggestions.value || suggestions.value.length === 0) {
      return false
    }

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault()
        selectedIndex.value = Math.min(selectedIndex.value + 1, suggestions.value.length - 1)
        return true
      case 'ArrowUp':
        event.preventDefault()
        selectedIndex.value = Math.max(selectedIndex.value - 1, -1)
        if (selectedIndex.value === -1) {
          return true
        }
        return true
      case 'Enter':
        if (selectedIndex.value >= 0 && selectedIndex.value < suggestions.value.length) {
          event.preventDefault()
          const selected = suggestions.value[selectedIndex.value]
          selectSuggestion(selected)
          callback(selected.url)
          return true
        }
        return false
      case 'Escape':
        showSuggestions.value = false
        selectedIndex.value = -1
        return true
      case 'Tab':
        if (selectedIndex.value >= 0 && selectedIndex.value < suggestions.value.length) {
          event.preventDefault()
          const selected = suggestions.value[selectedIndex.value]
          currentInput.value = selected.url
          callback(selected.url)
          showSuggestions.value = false
          return true
        }
        return false
      default:
        return false
    }
  }

  function hideSuggestions() {
    showSuggestions.value = false
    selectedIndex.value = -1
  }

  function show() {
    if (suggestions.value.length > 0) {
      showSuggestions.value = true
    }
  }

  return {
    currentInput,
    suggestions,
    showSuggestions,
    selectedIndex,
    updateInput,
    selectSuggestion,
    handleKeyDown,
    hideSuggestions,
    show,
  }
}