import { ref, computed, watch } from 'vue'
import { useEnvironmentStore } from '@/stores'

export function useVariableAutocomplete() {
  const environmentStore = useEnvironmentStore()
  const showSuggestions = ref(false)
  const selectedIndex = ref(-1)
  const triggerPosition = ref(-1)

  const currentInput = ref('')
  const currentValue = ref('')

  const suggestions = computed(() => {
    if (!showSuggestions.value || currentValue.value.length < 1) {
      return []
    }

    const lastBracePos = currentValue.value.lastIndexOf('{{')
    if (lastBracePos === -1) {
      showSuggestions.value = false
      return []
    }

    const query = currentValue.value.slice(lastBracePos + 2).toLowerCase()

    const variables = environmentStore.variables
    if (!query) {
      return variables.slice(0, 10)
    }

    return Object.entries(variables)
      .filter(([key]) => key.toLowerCase().includes(query))
      .slice(0, 10)
      .map(([key, value]) => ({ key, value }))
  })

  function updateValue(value: string) {
    currentValue.value = value
    const lastBracePos = value.lastIndexOf('{{')

    if (lastBracePos !== -1) {
      const afterBrace = value.slice(lastBracePos + 2)
      if (!afterBrace.includes('}}')) {
        showSuggestions.value = true
        selectedIndex.value = -1
        triggerPosition.value = lastBracePos
      } else {
        showSuggestions.value = false
      }
    } else {
      showSuggestions.value = false
    }
  }

  function selectSuggestion(suggestion: { key: string; value: string }) {
    if (triggerPosition.value === -1) return

    const before = currentValue.value.slice(0, triggerPosition.value)
    const after = currentValue.value.slice(triggerPosition.value + 2)

    let newValue = before + '{{' + suggestion.key + '}}'
    if (after.includes('}}')) {
      const afterCloseBrace = after.slice(after.indexOf('}}') + 2)
      newValue = before + '{{' + suggestion.key + '}}' + afterCloseBrace
    } else {
      newValue = before + '{{' + suggestion.key + '}}' + after
    }

    currentValue.value = newValue
    showSuggestions.value = false
    selectedIndex.value = -1
    triggerPosition.value = -1

    return newValue
  }

  function handleKeyDown(event: KeyboardEvent): boolean {
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
        return true
      case 'Enter':
      case 'Tab':
        if (selectedIndex.value >= 0 && selectedIndex.value < suggestions.value.length) {
          event.preventDefault()
          const selected = suggestions.value[selectedIndex.value]
          const result = selectSuggestion(selected)
          if (result) {
            currentValue.value = result
          }
          return true
        }
        return false
      case 'Escape':
        showSuggestions.value = false
        selectedIndex.value = -1
        return true
      case '}':
        if (currentValue.value.slice(-2) === '}}') {
          showSuggestions.value = false
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

  return {
    suggestions,
    showSuggestions,
    selectedIndex,
    currentValue,
    updateValue,
    selectSuggestion,
    handleKeyDown,
    hideSuggestions,
  }
}