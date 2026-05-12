export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'OPTIONS' | 'HEAD'

// HTTP 方法颜色配置（统一颜色定义）
export const HTTP_METHOD_COLORS: Record<
  HttpMethod,
  { color: string; background: string }
> = {
  GET: { color: '#61affe', background: 'rgba(97, 175, 254, 0.1)' },
  POST: { color: '#49cc90', background: 'rgba(73, 204, 144, 0.1)' },
  PUT: { color: '#fca130', background: 'rgba(252, 161, 48, 0.1)' },
  DELETE: { color: '#f93e3e', background: 'rgba(249, 62, 62, 0.1)' },
  PATCH: { color: '#50e3c2', background: 'rgba(80, 227, 194, 0.1)' },
  OPTIONS: { color: '#9012fe', background: 'rgba(144, 18, 254, 0.1)' },
  HEAD: { color: '#9012fe', background: 'rgba(144, 18, 254, 0.1)' },
}

export type Locale = 'zh-CN' | 'en-US'

export interface KeyValuePair {
  id?: string
  key: string
  value: string
  description?: string
  enabled: boolean
}

export type AuthType = 'none' | 'basic' | 'bearer' | 'apikey'

export interface BasicAuth {
  username: string
  password: string
}

export interface BearerAuth {
  token: string
}

export interface ApiKeyAuth {
  key: string
  value: string
  addTo: 'header' | 'query'
}

export interface AuthConfig {
  type: AuthType
  basic?: BasicAuth
  bearer?: BearerAuth
  apikey?: ApiKeyAuth
}

export type BodyMode = 'none' | 'form-data' | 'urlencoded' | 'raw' | 'binary'

export type RawType = 'json' | 'xml' | 'text' | 'html'

export interface FormDatum {
  id?: string
  key: string
  value: string
  description?: string
  enabled: boolean
  type: 'text' | 'file'
  filePath?: string
}

export interface RequestBody {
  mode: BodyMode
  raw?: string
  rawType?: RawType
  formData?: FormDatum[]
  urlencoded?: KeyValuePair[]
  binary?: string
}

export interface Request {
  id: string
  name: string
  description?: string
  method: HttpMethod
  url: string
  params: KeyValuePair[]
  headers: KeyValuePair[]
  body: RequestBody
  auth: AuthConfig
  preScript?: string
  postScript?: string
  returnBytes?: boolean
  proxy?: ProxyConfig
  followRedirects?: boolean
  verifySsl?: boolean
  createdAt: string
  updatedAt: string
}

export interface ProxyConfig {
  enabled: boolean
  host: string
  port: number
  username?: string
  password?: string
}

export interface Folder {
  id: string
  name: string
  description?: string
  folders: Folder[]
  requests: Request[]
  createdAt: string
  updatedAt: string
}

export interface Collection {
  id: string
  name: string
  description?: string
  folders: Folder[]
  requests: Request[]
  createdAt: string
  updatedAt: string
}

export type Variable = KeyValuePair

export interface Environment {
  id: string
  name: string
  variables: Variable[]
  createdAt: string
  updatedAt: string
}

export interface History {
  id: string
  requestId?: string
  method: HttpMethod
  url: string
  status: number
  responseTime: number
  responseSize: number
  createdAt: string
  params?: KeyValuePair[]
  headers?: KeyValuePair[]
  body?: RequestBody
  auth?: AuthConfig
}

export interface Cookie {
  name: string
  value: string
  domain?: string
  path?: string
  expires?: string
  httpOnly?: boolean
  secure?: boolean
}

export interface Response {
  status: number
  statusText: string
  headers: Record<string, string>
  cookies: Cookie[]
  body: string
  bodyBytes?: number[]
  responseTime: number
  responseSize: number
}

export interface ShortcutBinding {
  key: string
  ctrl?: boolean
  shift?: boolean
  alt?: boolean
  meta?: boolean
}

export interface Settings {
  theme: 'light' | 'dark' | 'system'
  language: Locale
  historyLimit: number
  timeout: number
  downloadPath?: string
  downloadAsk?: boolean
  proxy?: {
    enabled: boolean
    host: string
    port: number
    username?: string
    password?: string
  }
  followRedirects?: boolean
  verifySsl?: boolean
  shortcuts?: Record<string, ShortcutBinding | null>
}

export type TreeNode = Collection | Folder | Request

export interface RequestTab {
  id: string
  request: Request
  isDirty: boolean
  isTemporary: boolean
  collectionId?: string
  testResults?: {
    total: number
    passed: number
    failed: number
    assertions: Array<{
      name: string
      passed: boolean
      message?: string
    }>
  }
}

export interface AppState {
  currentTabId?: string
  currentEnvironmentId?: string
  sidebarCollapsed: boolean
}

export interface RawRequestBody {
  mode?: BodyMode
  bodyMode?: BodyMode
  body_mode?: BodyMode
  raw?: string
  rawType?: RawType
  raw_type?: RawType
  formData?: RawFormDatum[]
  form_data?: RawFormDatum[]
  urlencoded?: KeyValuePair[]
  binary?: string
}

export interface RawFormDatum {
  key?: string
  value?: string
  description?: string
  enabled?: boolean
  type?: 'text' | 'file'
  formType?: 'text' | 'file'
  form_type?: 'text' | 'file'
  filePath?: string
  file_path?: string
}

export interface RawAuthConfig {
  type?: AuthType
  authType?: AuthType
  auth_type?: AuthType
  basic?: {
    username?: string
    password?: string
  }
  bearer?: {
    token?: string
  }
  apikey?: {
    key?: string
    value?: string
    addTo?: 'header' | 'query'
    add_to?: 'header' | 'query'
  }
}

export interface RawRequest {
  id?: string
  name?: string
  description?: string
  method?: HttpMethod
  url?: string
  params?: KeyValuePair[]
  headers?: KeyValuePair[]
  body?: RawRequestBody
  auth?: RawAuthConfig
  preScript?: string
  pre_script?: string
  postScript?: string
  post_script?: string
  proxy?: ProxyConfig
  followRedirects?: boolean
  follow_redirects?: boolean
  verifySsl?: boolean
  verify_ssl?: boolean
  createdAt?: string
  created_at?: string
  updatedAt?: string
  updated_at?: string
}

export function normalizeRequestBody(body: RawRequestBody | null | undefined): RequestBody {
  if (!body) return { mode: 'none' }

  const normalized = {
    mode: body.mode || body.bodyMode || body.body_mode || 'none',
    raw: body.raw,
    rawType: body.rawType || body.raw_type,
    formData: normalizeFormData(body.formData || body.form_data),
    urlencoded: body.urlencoded,
    binary: body.binary,
  }

  return normalized
}

function normalizeFormData(formData: RawFormDatum[] | undefined): FormDatum[] | undefined {
  if (!formData || !Array.isArray(formData)) return undefined

  return formData.map((item) => ({
    key: item.key || '',
    value: item.value || '',
    description: item.description,
    enabled: item.enabled !== false,
    type: (item.type || item.formType || item.form_type || 'text') as 'text' | 'file',
    filePath: item.filePath || item.file_path,
  }))
}

export function normalizeAuthConfig(auth: RawAuthConfig | null | undefined): AuthConfig {
  if (!auth) return { type: 'none' }

  const authType = auth.type || auth.authType || auth.auth_type || 'none'

  return {
    type: authType,
    basic: auth.basic
      ? {
          username: auth.basic.username || '',
          password: auth.basic.password || '',
        }
      : undefined,
    bearer: auth.bearer
      ? {
          token: auth.bearer.token || '',
        }
      : undefined,
    apikey: auth.apikey
      ? {
          key: auth.apikey.key || '',
          value: auth.apikey.value || '',
          addTo: auth.apikey.addTo || auth.apikey.add_to || 'header',
        }
      : undefined,
  }
}

/**
 * 安全地解析日期时间字符串，处理空值、null 或无效格式
 * @param dateStr 日期字符串（可能为空、null 或无效格式）
 * @returns 有效的 ISO 日期字符串
 */
export function safeParseDate(dateStr: string | null | undefined, defaultDate?: string): string {
  if (!dateStr || dateStr.trim() === '') {
    return defaultDate || new Date().toISOString()
  }
  
  // 尝试解析日期
  const date = new Date(dateStr)
  if (isNaN(date.getTime())) {
    // 如果解析失败，返回默认值
    return defaultDate || new Date().toISOString()
  }
  
  // 返回标准化的 ISO 格式
  return date.toISOString()
}

export function normalizeRequest(request: RawRequest): Request {
  return {
    id: request.id || crypto.randomUUID(),
    name: request.name || '未命名请求',
    description: request.description,
    method: request.method || 'GET',
    url: request.url || '',
    params: request.params || [],
    headers: request.headers || [],
    body: normalizeRequestBody(request.body),
    auth: normalizeAuthConfig(request.auth),
    preScript: request.preScript || request.pre_script,
    postScript: request.postScript || request.post_script,
    proxy: request.proxy,
    followRedirects: request.followRedirects ?? request.follow_redirects,
    verifySsl: request.verifySsl ?? request.verify_ssl,
    createdAt: safeParseDate(request.createdAt || request.created_at),
    updatedAt: safeParseDate(request.updatedAt || request.updated_at),
  }
}

export interface BackendRequestBody {
  body_mode: BodyMode
  raw?: string
  raw_type?: RawType
  form_data?: {
    key: string
    value: string
    description?: string
    enabled: boolean
    form_type: 'text' | 'file'
    file_path?: string
  }[]
  urlencoded?: KeyValuePair[]
  binary?: string
}

export interface BackendAuthConfig {
  auth_type: AuthType
  basic?: BasicAuth
  bearer?: BearerAuth
  apikey?: {
    key: string
    value: string
    add_to: 'header' | 'query'
  } | null
}

export interface BackendRequest {
  id: string
  name: string
  description?: string
  method: HttpMethod
  url: string
  params: KeyValuePair[]
  headers: KeyValuePair[]
  body: BackendRequestBody | null
  auth: BackendAuthConfig | null
  pre_script?: string
  post_script?: string
  proxy?: ProxyConfig
  follow_redirects?: boolean
  verify_ssl?: boolean
  created_at: string
  updated_at: string
}

export function toBackendRequest(request: Request): BackendRequest {
  return {
    id: request.id,
    name: request.name,
    description: request.description,
    method: request.method,
    url: request.url,
    params: request.params,
    headers: request.headers,
    body: request.body
      ? {
          body_mode: request.body.mode,
          raw: request.body.raw,
          raw_type: request.body.rawType,
          form_data: request.body.formData?.map((item) => ({
            key: item.key,
            value: item.value,
            description: item.description,
            enabled: item.enabled,
            form_type: item.type,
            file_path: item.filePath,
          })),
          urlencoded: request.body.urlencoded,
          binary: request.body.binary,
        }
      : null,
    auth: request.auth
      ? {
          auth_type: request.auth.type,
          basic: request.auth.basic,
          bearer: request.auth.bearer,
          apikey: request.auth.apikey
            ? {
                key: request.auth.apikey.key,
                value: request.auth.apikey.value,
                add_to: request.auth.apikey.addTo,
              }
            : null,
        }
      : null,
    pre_script: request.preScript,
    post_script: request.postScript,
    proxy: request.proxy,
    follow_redirects: request.followRedirects,
    verify_ssl: request.verifySsl,
    created_at: request.createdAt,
    updated_at: request.updatedAt,
  }
}
