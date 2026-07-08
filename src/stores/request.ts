import { ref, computed, watch, shallowRef } from 'vue'
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
import { safeParseDate } from '@/types'

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

// 解析 URL 中的查询参数
function parseUrlParams(url: string): { baseUrl: string; params: KeyValuePair[] } {
  const questionMarkIndex = url.indexOf('?')
  if (questionMarkIndex === -1) {
    return { baseUrl: url, params: [] }
  }

  const baseUrl = url.substring(0, questionMarkIndex)
  const queryString = url.substring(questionMarkIndex + 1)
  const params: KeyValuePair[] = []

  if (queryString) {
    for (const paramPair of queryString.split('&')) {
      if (!paramPair) continue
      const equalIndex = paramPair.indexOf('=')
      if (equalIndex === -1) {
        // 只有 key 没有 value
        params.push({
          id: crypto.randomUUID(),
          key: decodeURIComponent(paramPair),
          value: '',
          description: '',
          enabled: true,
        })
      } else {
        const key = paramPair.substring(0, equalIndex)
        const value = paramPair.substring(equalIndex + 1)
        params.push({
          id: crypto.randomUUID(),
          key: decodeURIComponent(key),
          value: decodeURIComponent(value),
          description: '',
          enabled: true,
        })
      }
    }
  }

  return { baseUrl, params }
}

// 从 params 重建 URL 查询字符串
function buildUrlWithParams(baseUrl: string, params: KeyValuePair[]): string {
  const enabledParams = params.filter((p) => p.enabled && p.key)
  if (enabledParams.length === 0) {
    // 移除已有的查询参数
    const questionMarkIndex = baseUrl.indexOf('?')
    if (questionMarkIndex !== -1) {
      return baseUrl.substring(0, questionMarkIndex)
    }
    return baseUrl
  }

  const queryString = enabledParams
    .map(
      (p) => `${encodeURIComponent(p.key)}=${encodeURIComponent(p.value)}`
    )
    .join('&')

  // 移除已有的查询参数
  const questionMarkIndex = baseUrl.indexOf('?')
  const cleanBaseUrl =
    questionMarkIndex !== -1 ? baseUrl.substring(0, questionMarkIndex) : baseUrl

  return `${cleanBaseUrl}?${queryString}`
}

export const useRequestStore = defineStore('request', () => {
  const collections = ref<Collection[]>([])
  const tabs = ref<RequestTab[]>([])
  const activeTabId = ref<string | null>(null)
  const responses = shallowRef<Record<string, Response>>({})

  function setResponse(tabId: string, response: Response) {
    responses.value = { ...responses.value, [tabId]: response }
  }

  function clearTabResponse(tabId: string) {
    delete responses.value[tabId]
  }

  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const pendingRequestId = ref<string | null>(null)
  const uploadProgress = ref<number>(0)
  const downloadProgress = ref<number>(0)

  function setUploadProgress(progress: number) {
    uploadProgress.value = progress
  }

  function setDownloadProgress(progress: number) {
    downloadProgress.value = progress
  }

  function resetProgress() {
    uploadProgress.value = 0
    downloadProgress.value = 0
  }

  const currentTab = computed(() => tabs.value.find((t) => t.id === activeTabId.value) || null)

  const currentRequest = computed(() => currentTab.value?.request || createDefaultRequest())

  const currentResponse = computed(() => {
    if (!activeTabId.value) return null
    return responses.value[activeTabId.value] || null
  })

  const LARGE_RESPONSE_THRESHOLD = 5 * 1024 * 1024 // 5MB

  function isLargeResponse(response: Response): boolean {
    return response.responseSize > LARGE_RESPONSE_THRESHOLD
  }

  function getOptimizedResponse(response: Response): Response {
    if (response.responseSize <= LARGE_RESPONSE_THRESHOLD) {
      return response
    }

    const maxDisplaySize = 1 * 1024 * 1024 // 1MB
    let optimizedBody = response.body

    if (response.body.length > maxDisplaySize) {
      optimizedBody = response.body.substring(0, maxDisplaySize) + '\n\n[... 响应内容已被截断以提升性能 ...]\n' +
        `原始大小: ${formatBytes(response.responseSize)}`
    }

    return {
      ...response,
      body: optimizedBody,
      bodyBytes: undefined,
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

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
    clearTabResponse(tabId)

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
      clearTabResponse(tab.id)
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
      clearTabResponse(tab.id)
    })

    if (!tabs.value.find((t) => t.id === activeTabId.value)) {
      activeTabId.value = tabs.value[tabs.value.length - 1]?.id || null
    }
  }

  function closeAllTabs() {
    tabs.value.forEach((tab) => {
      clearTabResponse(tab.id)
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

  function duplicateTab(tabId: string) {
    const tab = tabs.value.find((t) => t.id === tabId)
    if (!tab) return

    const newRequest: Request = {
      ...JSON.parse(JSON.stringify(tab.request)),
      id: crypto.randomUUID(),
      name: `${tab.request.name} (副本)`,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    }

    openRequest(newRequest, tab.collectionId, tab.isTemporary)
  }

  function updateRequest(updates: Partial<Request>) {
    if (!currentTab.value) return
    const updatedRequest = {
      ...JSON.parse(JSON.stringify(currentTab.value.request)),
      ...updates,
      updatedAt: new Date().toISOString(),
    }
    currentTab.value.request = updatedRequest
    currentTab.value.isDirty = true
  }

  function updateUrl(url: string) {
    // 解析 URL 中的查询参数
    const { params: urlParams } = parseUrlParams(url)
    
    // 获取当前参数
    const currentParams = currentTab.value?.request.params || []
    
    // 创建 URL 参数的 key 集合
    const urlParamKeys = new Set(urlParams.map(p => p.key))
    
    // 保留手动禁用的参数（不在 URL 中且原本是禁用状态）
    const disabledParams = currentParams
      .filter(p => !urlParamKeys.has(p.key) && p.key && !p.enabled)
    
    // 合并：URL 参数（启用）+ 手动禁用的参数
    const mergedParams = [...urlParams, ...disabledParams]
    
    // 直接更新，不触发 updateParams
    if (!currentTab.value) return
    const updatedRequest = {
      ...JSON.parse(JSON.stringify(currentTab.value.request)),
      url,
      params: mergedParams,
      updatedAt: new Date().toISOString(),
    }
    currentTab.value.request = updatedRequest
    currentTab.value.isDirty = true
  }

  function updateMethod(method: Request['method']) {
    updateRequest({ method })
  }

  function updateParams(params: KeyValuePair[]) {
    // 从 params 重建 URL
    const currentUrl = currentTab.value?.request.url || ''
    const newUrl = buildUrlWithParams(currentUrl, params)
    updateRequest({ params, url: newUrl })
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
      // 同步 request 的归属字段，保证内存与 DB 一致
      request.collectionId = collectionId
      request.folderId = folderId || undefined

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

  async function createFolder(
    collectionId: string,
    name: string,
    parentFolderId?: string
  ): Promise<Folder | null> {
    const collection = collections.value.find((c: Collection) => c.id === collectionId)
    if (!collection) return null

    const now = new Date().toISOString()
    const folder: Folder = {
      id: crypto.randomUUID(),
      name,
      folders: [],
      requests: [],
      collectionId,
      parentFolderId: parentFolderId || undefined,
      createdAt: now,
      updatedAt: now,
    }

    // 组装到内存树
    if (parentFolderId) {
      const parent = findFolderRecursive(collection.folders, parentFolderId)
      if (parent) {
        parent.folders.push(folder)
      } else {
        // 父文件夹找不到时回退到集合根部
        collection.folders.push(folder)
        folder.parentFolderId = undefined
      }
    } else {
      collection.folders.push(folder)
    }

    // 持久化到后端
    try {
      await invoke('save_folder', { folder })
    } catch (e) {
      console.error('创建文件夹失败:', e)
    }

    return folder
  }

  async function deleteFolder(collectionId: string, folderId: string) {
    const collection = collections.value.find((c: Collection) => c.id === collectionId)
    if (!collection) return

    // 递归从内存树移除
    const removeFromTree = (folders: Folder[]): boolean => {
      const idx = folders.findIndex((f) => f.id === folderId)
      if (idx >= 0) {
        folders.splice(idx, 1)
        return true
      }
      return folders.some((f) => removeFromTree(f.folders))
    }
    removeFromTree(collection.folders)

    // 持久化（后端会递归删子文件夹及其 requests）
    try {
      await invoke('delete_folder', { id: folderId })
    } catch (e) {
      console.error('删除文件夹失败:', e)
    }
  }

  async function renameFolder(collectionId: string, folderId: string, newName: string) {
    const collection = collections.value.find((c: Collection) => c.id === collectionId)
    if (!collection) return

    const folder = findFolderRecursive(collection.folders, folderId)
    if (!folder) return

    folder.name = newName
    folder.updatedAt = new Date().toISOString()

    try {
      await invoke('save_folder', { folder })
    } catch (e) {
      console.error('重命名文件夹失败:', e)
    }
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

  function reorderCollection(fromIndex: number, toIndex: number) {
    const newCollections = [...collections.value]
    const [removed] = newCollections.splice(fromIndex, 1)
    newCollections.splice(toIndex, 0, removed)
    collections.value = newCollections
  }

  function reorderRequest(collectionId: string, fromIndex: number, toIndex: number) {
    const collection = collections.value.find((c: Collection) => c.id === collectionId)
    if (!collection) return

    const newRequests = [...collection.requests]
    const [removed] = newRequests.splice(fromIndex, 1)
    newRequests.splice(toIndex, 0, removed)
    collection.requests = newRequests
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

  /**
   * 将扁平 folder 列表组装成嵌套树（按 parent_folder_id）
   * 只组装属于指定 collection 的 folders
   */
  function buildFolderTree(flatFolders: Folder[], collectionId: string): Folder[] {
    const collectionFolders = flatFolders.filter((f) => f.collectionId === collectionId)
    const folderMap = new Map<string, Folder>()
    const roots: Folder[] = []

    // 初始化每个 folder 的 folders/requests 子数组
    for (const f of collectionFolders) {
      folderMap.set(f.id, {
        ...f,
        folders: [],
        requests: [],
      })
    }

    // 按 parentFolderId 挂载
    for (const f of collectionFolders) {
      const node = folderMap.get(f.id)!
      if (f.parentFolderId && folderMap.has(f.parentFolderId)) {
        folderMap.get(f.parentFolderId)!.folders.push(node)
      } else {
        roots.push(node)
      }
    }

    return roots
  }

  /**
   * 递归在 folders 树中查找指定 id 的 folder
   */
  function findFolderRecursive(folders: Folder[], folderId: string): Folder | null {
    for (const f of folders) {
      if (f.id === folderId) return f
      const found = findFolderRecursive(f.folders, folderId)
      if (found) return found
    }
    return null
  }

  async function loadCollections() {
    try {
      const loadedCollections = await invoke<Collection[]>('get_all_collections')
      if (loadedCollections) {
        // 加载所有 folders（扁平列表，含 collectionId/parentFolderId）
        const allFolders = await invoke<Folder[]>('get_all_folders').catch(() => [])

        for (const collection of loadedCollections) {
          // 确保时间字段有效
          collection.createdAt = safeParseDate(collection.createdAt)
          collection.updatedAt = safeParseDate(collection.updatedAt)

          // 组装 folder 嵌套树
          collection.folders = buildFolderTree(allFolders, collection.id)

          // 加载该集合下所有 requests（含 folderId），按 folderId 归位
          const requests = await invoke<Request[]>('get_requests_by_collection', {
            collectionId: collection.id,
          })
          const parsedRequests = (requests || []).map((request) => ({
            ...request,
            createdAt: safeParseDate(request.createdAt),
            updatedAt: safeParseDate(request.updatedAt),
          }))

          collection.requests = []
          for (const request of parsedRequests) {
            if (request.folderId) {
              const folder = findFolderRecursive(collection.folders, request.folderId)
              if (folder) {
                folder.requests.push(request)
              } else {
                // folder 找不到时回退到集合根部，避免请求丢失
                collection.requests.push(request)
              }
            } else {
              collection.requests.push(request)
            }
          }
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

  // ===== 批量操作功能 =====

  // 选中的请求 ID 集合
  const selectedRequestIds = ref<Set<string>>(new Set())
  // 是否启用选择模式
  const isSelectionMode = ref(false)

  function toggleSelection(requestId: string) {
    if (selectedRequestIds.value.has(requestId)) {
      selectedRequestIds.value.delete(requestId)
    } else {
      selectedRequestIds.value.add(requestId)
    }
  }

  function selectRequest(requestId: string) {
    selectedRequestIds.value.add(requestId)
  }

  function deselectRequest(requestId: string) {
    selectedRequestIds.value.delete(requestId)
  }

  function clearSelection() {
    selectedRequestIds.value.clear()
  }

  function toggleSelectionMode() {
    isSelectionMode.value = !isSelectionMode.value
    if (!isSelectionMode.value) {
      clearSelection()
    }
  }

  function selectAllRequests(requests: Request[]) {
    requests.forEach((req) => selectedRequestIds.value.add(req.id))
  }

  function isInSelection(requestId: string): boolean {
    return selectedRequestIds.value.has(requestId)
  }

  // 批量删除请求
  async function batchDeleteRequests(collectionId: string, requestIds: string[]) {
    try {
      await invoke('batch_delete_requests', { requestIds })

      const collection = collections.value.find((c: Collection) => c.id === collectionId)
      if (collection) {
        collection.requests = collection.requests.filter((r: Request) => !requestIds.includes(r.id))

        const deleteFromFolder = (folders: Folder[]) => {
          for (const folder of folders) {
            folder.requests = folder.requests.filter((r: Request) => !requestIds.includes(r.id))
            deleteFromFolder(folder.folders)
          }
        }
        deleteFromFolder(collection.folders)
      }

      const tabsToRemove = tabs.value.filter((t) => requestIds.includes(t.request.id))
      tabsToRemove.forEach((tab) => {
        closeTab(tab.id)
      })

      clearSelection()
    } catch (e) {
      console.error('批量删除请求失败:', e)
      throw e
    }
  }

  // 批量移动请求到文件夹
  async function batchMoveRequests(
    fromCollectionId: string,
    toCollectionId: string,
    toFolderId: string | null,
    requestIds: string[]
  ) {
    try {
      // 获取要移动的请求
      const fromCollection = collections.value.find((c) => c.id === fromCollectionId)
      if (!fromCollection) throw new Error('源集合不存在')

      const requestsToMove: Request[] = []
      const findRequests = (folders: Folder[]) => {
        for (const folder of folders) {
          requestsToMove.push(...folder.requests.filter((r) => requestIds.includes(r.id)))
          findRequests(folder.folders)
        }
      }
      requestsToMove.push(...fromCollection.requests.filter((r) => requestIds.includes(r.id)))
      findRequests(fromCollection.folders)

      // 添加到目标集合或文件夹
      const toCollection = collections.value.find((c) => c.id === toCollectionId)
      if (!toCollection) throw new Error('目标集合不存在')

      for (const request of requestsToMove) {
        await invoke('save_request_to_collection', {
          request,
          collectionId: toCollectionId,
          folderId: toFolderId,
        })

        // 从原位置删除
        await invoke('delete_request_from_collection', { id: request.id })
      }

      // 更新本地状态
      fromCollection.requests = fromCollection.requests.filter((r) => !requestIds.includes(r.id))

      if (toFolderId) {
        const addToFolder = (folders: Folder[]): boolean => {
          for (const folder of folders) {
            if (folder.id === toFolderId) {
              folder.requests.push(...requestsToMove)
              return true
            }
            if (addToFolder(folder.folders)) return true
          }
          return false
        }
        addToFolder(toCollection.folders)
      } else {
        toCollection.requests.push(...requestsToMove)
      }

      clearSelection()
    } catch (e) {
      console.error('批量移动请求失败:', e)
      throw e
    }
  }

  // 批量复制请求
  async function batchCopyRequests(
    toCollectionId: string,
    toFolderId: string | null,
    requestIds: string[]
  ) {
    try {
      const toCollection = collections.value.find((c) => c.id === toCollectionId)
      if (!toCollection) throw new Error('目标集合不存在')

      // 查找所有要复制的请求
      const requestsToCopy: Request[] = []
      const findRequests = (collection: Collection) => {
        requestsToCopy.push(...collection.requests.filter((r) => requestIds.includes(r.id)))
        const searchFolders = (folders: Folder[]) => {
          for (const folder of folders) {
            requestsToCopy.push(...folder.requests.filter((r) => requestIds.includes(r.id)))
            searchFolders(folder.folders)
          }
        }
        searchFolders(collection.folders)
      }

      for (const collection of collections.value) {
        findRequests(collection)
      }

      // 创建副本并添加
      for (const request of requestsToCopy) {
        const newRequest: Request = {
          ...JSON.parse(JSON.stringify(request)),
          id: crypto.randomUUID(),
          name: `${request.name} (副本)`,
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        }

        await invoke('save_request_to_collection', {
          request: newRequest,
          collectionId: toCollectionId,
          folderId: toFolderId,
        })
      }

      clearSelection()
    } catch (e) {
      console.error('批量复制请求失败:', e)
      throw e
    }
  }

  function moveRequest(
    _requestId: string,
    fromCollectionId: string,
    toCollectionId: string,
    fromIndex: number,
    toIndex: number,
    fromFolderId?: string,
    toFolderId?: string
  ) {
    const fromCollection = collections.value.find((c) => c.id === fromCollectionId)
    const toCollection = collections.value.find((c) => c.id === toCollectionId)

    if (!fromCollection || !toCollection) return

    if (fromCollectionId === toCollectionId && !fromFolderId && !toFolderId) {
      const requests = [...fromCollection.requests]
      const [removed] = requests.splice(fromIndex, 1)
      requests.splice(toIndex, 0, removed)
      fromCollection.requests = requests
    } else if (fromFolderId && toFolderId && fromFolderId === toFolderId) {
      const fromFolder = findFolderRecursive(fromCollection.folders, fromFolderId)
      if (fromFolder) {
        const requests = [...fromFolder.requests]
        const [removed] = requests.splice(fromIndex, 1)
        requests.splice(toIndex, 0, removed)
        fromFolder.requests = requests
      }
    } else {
      const getSourceRequests = (c: Collection, folderId?: string) => {
        if (folderId) {
          const folder = findFolderRecursive(c.folders, folderId)
          return folder?.requests || []
        }
        return c.requests
      }

      const sourceRequests = getSourceRequests(fromCollection, fromFolderId)
      const targetRequests = getSourceRequests(toCollection, toFolderId)

      const [removed] = sourceRequests.splice(fromIndex, 1)
      // 更新被移动 request 的 folderId
      removed.folderId = toFolderId
      targetRequests.splice(toIndex, 0, removed)

      // 持久化被移动 request 的 folderId 归属
      invoke('save_request_to_collection', {
        request: removed,
        collectionId: toCollectionId,
        folderId: toFolderId || null,
      }).catch(console.error)
    }

    invoke('save_collection', { collection: fromCollection }).catch(console.error)
    if (fromCollectionId !== toCollectionId) {
      invoke('save_collection', { collection: toCollection }).catch(console.error)
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
    uploadProgress,
    downloadProgress,
    currentCollection,
    selectedRequestIds,
    isSelectionMode,
    openRequest,
    closeTab,
    closeTabsToLeft,
    closeTabsToRight,
    closeAllTabs,
    moveTab,
    switchTab,
    newRequest,
    duplicateTab,
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
    deleteFolder,
    renameFolder,
    deleteRequest,
    deleteCollection,
    reorderCollection,
    reorderRequest,
    saveTemporaryRequest,
    loadTemporaryRequest,
    clearTemporaryRequest,
    cancelCurrentRequest,
    setPendingRequestId,
    setResponse,
    clearTabResponse,
    loadCollections,
    renameRequest,
    saveTabs,
    loadTabs,
    initTabsPersistence,
    setUploadProgress,
    setDownloadProgress,
    resetProgress,
    isLargeResponse,
    getOptimizedResponse,
    formatBytes,
    // 批量操作
    toggleSelection,
    selectRequest,
    deselectRequest,
    clearSelection,
    toggleSelectionMode,
    selectAllRequests,
    isInSelection,
    batchDeleteRequests,
    batchMoveRequests,
    batchCopyRequests,
    moveRequest,
  }
})
