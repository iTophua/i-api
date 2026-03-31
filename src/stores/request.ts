import { ref, computed, watch } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type {
  Collection,
  Request,
  Folder,
  KeyValuePair,
  RequestBody,
  AuthConfig,
  Response,
  RequestTab,
} from '@/types'

const createDefaultRequest = (): Request => ({
  id: crypto.randomUUID(),
  name: '未命名请求',
  method: 'GET',
  url: '',
  params: [],
  headers: [],
  body: { mode: 'none' },
  auth: { type: 'none' },
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
})

export const useRequestStore = defineStore('request', () => {
  const collections = ref<Collection[]>([])
  const tabs = ref<RequestTab[]>([])
  const activeTabId = ref<string | null>(null)
  const responses = ref<Record<string, Response>>({})

  function setResponse(tabId: string, response: Response) {
    responses.value[tabId] = response
  }
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const pendingRequestId = ref<string | null>(null)

  const currentTab = computed(() => tabs.value.find((t) => t.id === activeTabId.value) || null)

  const currentRequest = computed(() => currentTab.value?.request || createDefaultRequest())

  const currentResponse = computed(() => {
    if (!activeTabId.value) return null
    return responses.value[activeTabId.value] || null
  })

  const currentCollection = computed(() => collections.value[0] || null)

  function openRequest(request: Request, collectionId?: string, isTemporary = false) {
    const existingTab = tabs.value.find((t) => t.request.id === request.id)
    if (existingTab) {
      activeTabId.value = existingTab.id
      return existingTab
    }

    const tab: RequestTab = {
      id: crypto.randomUUID(),
      request: JSON.parse(JSON.stringify(request)),
      isDirty: false,
      isTemporary,
      collectionId,
    }
    tabs.value.push(tab)
    activeTabId.value = tab.id
    return tab
  }

  function closeTab(tabId: string) {
    const index = tabs.value.findIndex((t) => t.id === tabId)
    if (index === -1) return

    tabs.value.splice(index, 1)
    delete responses.value[tabId]

    if (activeTabId.value === tabId) {
      if (tabs.value.length > 0) {
        const newIndex = Math.min(index, tabs.value.length - 1)
        activeTabId.value = tabs.value[newIndex].id
      } else {
        activeTabId.value = null
      }
    }
  }

  function closeTabsToLeft(tabId: string) {
    const index = tabs.value.findIndex((t) => t.id === tabId)
    if (index <= 0) return

    const removedTabs = tabs.value.splice(0, index)
    removedTabs.forEach((tab) => {
      delete responses.value[tab.id]
    })

    if (!tabs.value.find((t) => t.id === activeTabId.value)) {
      activeTabId.value = tabs.value[0]?.id || null
    }
  }

  function closeTabsToRight(tabId: string) {
    const index = tabs.value.findIndex((t) => t.id === tabId)
    if (index === -1 || index >= tabs.value.length - 1) return

    const removedTabs = tabs.value.splice(index + 1)
    removedTabs.forEach((tab) => {
      delete responses.value[tab.id]
    })

    if (!tabs.value.find((t) => t.id === activeTabId.value)) {
      activeTabId.value = tabs.value[tabs.value.length - 1]?.id || null
    }
  }

  function closeAllTabs() {
    tabs.value.forEach((tab) => {
      delete responses.value[tab.id]
    })
    tabs.value = []
    activeTabId.value = null
  }

  function moveTab(fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return
    if (fromIndex < 0 || fromIndex >= tabs.value.length) return
    if (toIndex < 0 || toIndex >= tabs.value.length) return

    const [tab] = tabs.value.splice(fromIndex, 1)
    tabs.value.splice(toIndex, 0, tab)
  }

  function switchTab(tabId: string) {
    if (tabs.value.find((t) => t.id === tabId)) {
      activeTabId.value = tabId
    }
  }

  function newRequest() {
    const request = createDefaultRequest()
    openRequest(request, undefined, true)
  }

  function updateRequest(updates: Partial<Request>) {
    if (!currentTab.value) return
    Object.assign(currentTab.value.request, updates, { updatedAt: new Date().toISOString() })
    currentTab.value.isDirty = true
  }

  function updateUrl(url: string) {
    updateRequest({ url })
  }

  function updateMethod(method: Request['method']) {
    updateRequest({ method })
  }

  function updateParams(params: KeyValuePair[]) {
    updateRequest({ params })
  }

  function updateHeaders(headers: KeyValuePair[]) {
    updateRequest({ headers })
  }

  function updateBody(body: RequestBody) {
    updateRequest({ body })
  }

  function updateAuth(auth: AuthConfig) {
    updateRequest({ auth })
  }

  async function saveRequest(collectionId: string, folderId?: string) {
    if (!currentTab.value) return

    const request = JSON.parse(JSON.stringify(currentTab.value.request))
    request.name = request.name || '未命名请求'

    try {
      await invoke('save_request_to_collection', {
        request,
        collectionId,
        folderId: folderId || null,
      })

      const collection = collections.value.find((c: Collection) => c.id === collectionId)
      if (collection) {
        if (folderId) {
          const addToFolder = (folders: Folder[]): boolean => {
            for (const folder of folders) {
              if (folder.id === folderId) {
                const existingIndex = folder.requests.findIndex((r: Request) => r.id === request.id)
                if (existingIndex >= 0) {
                  folder.requests[existingIndex] = request
                } else {
                  folder.requests.push(request)
                }
                return true
              }
              if (addToFolder(folder.folders)) return true
            }
            return false
          }
          addToFolder(collection.folders)
        } else {
          const existingIndex = collection.requests.findIndex((r: Request) => r.id === request.id)
          if (existingIndex >= 0) {
            collection.requests[existingIndex] = request
          } else {
            collection.requests.push(request)
          }
        }
      }

      currentTab.value.isDirty = false
      currentTab.value.isTemporary = false
      currentTab.value.collectionId = collectionId
    } catch (e) {
      console.error('保存请求失败:', e)
      throw e
    }
  }

  async function createCollection(name: string) {
    const collection: Collection = {
      id: crypto.randomUUID(),
      name,
      folders: [],
      requests: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    }
    try {
      await invoke('save_collection', { collection })
      collections.value.push(collection)
      return collection
    } catch (e) {
      console.error('创建集合失败:', e)
      throw e
    }
  }

  async function renameCollection(collectionId: string, newName: string) {
    const collection = collections.value.find((c: Collection) => c.id === collectionId)
    if (collection) {
      try {
        await invoke('rename_collection', { id: collectionId, name: newName })
        collection.name = newName
        collection.updatedAt = new Date().toISOString()
      } catch (e) {
        console.error('重命名集合失败:', e)
        throw e
      }
    }
  }

  function createFolder(
    collectionId: string,
    name: string,
    parentFolderId?: string
  ): Folder | null {
    const collection = collections.value.find((c: Collection) => c.id === collectionId)
    if (!collection) return null

    const folder: Folder = {
      id: crypto.randomUUID(),
      name,
      folders: [],
      requests: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    }

    if (parentFolderId) {
      const addToParent = (folders: Folder[]): boolean => {
        for (const f of folders) {
          if (f.id === parentFolderId) {
            f.folders.push(folder)
            return true
          }
          if (addToParent(f.folders)) return true
        }
        return false
      }
      addToParent(collection.folders)
    } else {
      collection.folders.push(folder)
    }

    return folder
  }

  async function deleteRequest(collectionId: string, requestId: string) {
    try {
      await invoke('delete_request_from_collection', { id: requestId })
      const collection = collections.value.find((c: Collection) => c.id === collectionId)
      if (!collection) return

      collection.requests = collection.requests.filter((r: Request) => r.id !== requestId)

      const deleteFromFolder = (folders: Folder[]) => {
        for (const folder of folders) {
          folder.requests = folder.requests.filter((r: Request) => r.id !== requestId)
          deleteFromFolder(folder.folders)
        }
      }
      deleteFromFolder(collection.folders)
    } catch (e) {
      console.error('删除请求失败:', e)
      throw e
    }
  }

  async function deleteCollection(collectionId: string) {
    try {
      await invoke('delete_collection', { id: collectionId })
      collections.value = collections.value.filter((c: Collection) => c.id !== collectionId)
    } catch (e) {
      console.error('删除集合失败:', e)
      throw e
    }
  }

  async function saveTemporaryRequest() {
    if (!currentTab.value) return
    try {
      await invoke('save_temporary_request', { request: currentTab.value.request })
    } catch (e) {
      console.error('保存临时请求失败:', e)
    }
  }

  async function loadTemporaryRequest() {
    try {
      const request = await invoke<Request | null>('get_temporary_request')
      if (request) {
        openRequest(request, undefined, true)
      }
    } catch (e) {
      console.error('加载临时请求失败:', e)
    }
  }

  async function clearTemporaryRequest() {
    try {
      await invoke('clear_temporary_request')
    } catch (e) {
      console.error('清除临时请求失败:', e)
    }
  }

  async function loadCollections() {
    try {
      const loadedCollections = await invoke<Collection[]>('get_all_collections')
      if (loadedCollections) {
        for (const collection of loadedCollections) {
          const requests = await invoke<Request[]>('get_requests_by_collection', {
            collectionId: collection.id,
          })
          collection.requests = requests || []
          collection.folders = []
        }
        collections.value = loadedCollections
      }
    } catch (e) {
      console.error('加载集合失败:', e)
    }
  }

  async function renameRequest(requestId: string, newName: string) {
    try {
      await invoke('rename_request', { id: requestId, name: newName })
      for (const collection of collections.value) {
        const request = collection.requests.find((r: Request) => r.id === requestId)
        if (request) {
          request.name = newName
          request.updatedAt = new Date().toISOString()
          break
        }
      }
    } catch (e) {
      console.error('重命名请求失败:', e)
      throw e
    }
  }

  async function saveTabs() {
    try {
      const tabData = {
        tabs: tabs.value.map((tab) => ({
          id: tab.id,
          request: tab.request,
          collectionId: tab.collectionId,
          isTemporary: tab.isTemporary,
          isDirty: tab.isDirty,
        })),
        activeTabId: activeTabId.value,
      }
      await invoke('save_open_tabs', {
        tabsData: JSON.stringify(tabData),
        activeTabId: activeTabId.value,
      })
    } catch (e) {
      console.error('保存 tabs 失败:', e)
    }
  }

  async function loadTabs() {
    try {
      const result = await invoke<[string, string | null] | null>('get_open_tabs')
      if (result) {
        const [tabsJson, savedActiveTabId] = result
        const tabData = JSON.parse(tabsJson)
        if (tabData.tabs && Array.isArray(tabData.tabs)) {
          tabs.value = tabData.tabs
          activeTabId.value = savedActiveTabId || tabData.activeTabId
        }
      }
    } catch (e) {
      console.error('加载 tabs 失败:', e)
    }
  }

  let saveTimeout: ReturnType<typeof setTimeout> | null = null
  function debouncedSaveTabs() {
    if (saveTimeout) {
      clearTimeout(saveTimeout)
    }
    saveTimeout = setTimeout(() => {
      saveTabs()
    }, 300)
  }

  function initTabsPersistence() {
    watch(
      () => tabs.value,
      () => {
        debouncedSaveTabs()
      },
      { deep: true }
    )

    watch(
      () => activeTabId.value,
      () => {
        debouncedSaveTabs()
      }
    )
  }

  async function cancelCurrentRequest(): Promise<boolean> {
    if (!pendingRequestId.value) return false

    try {
      const result = await invoke<boolean>('cancel_http_request', {
        requestId: pendingRequestId.value,
      })
      if (result) {
        isLoading.value = false
        pendingRequestId.value = null
      }
      return result
    } catch (e) {
      console.error('取消请求失败:', e)
      return false
    }
  }

  function setPendingRequestId(id: string | null) {
    pendingRequestId.value = id
  }

  return {
    collections,
    tabs,
    activeTabId,
    responses,
    currentTab,
    currentRequest,
    currentResponse,
    isLoading,
    error,
    pendingRequestId,
    currentCollection,
    openRequest,
    closeTab,
    closeTabsToLeft,
    closeTabsToRight,
    closeAllTabs,
    moveTab,
    switchTab,
    newRequest,
    updateRequest,
    updateUrl,
    updateMethod,
    updateParams,
    updateHeaders,
    updateBody,
    updateAuth,
    saveRequest,
    createCollection,
    renameCollection,
    createFolder,
    deleteRequest,
    deleteCollection,
    saveTemporaryRequest,
    loadTemporaryRequest,
    clearTemporaryRequest,
    cancelCurrentRequest,
    setPendingRequestId,
    setResponse,
    loadCollections,
    renameRequest,
    saveTabs,
    loadTabs,
    initTabsPersistence,
  }
})
