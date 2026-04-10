export interface CommonHeader {
  key: string
  value?: string
  description: string
}

export const COMMON_HEADERS: CommonHeader[] = [
  { key: 'Accept', value: 'application/json', description: '客户端可接受的响应内容类型' },
  { key: 'Accept-Encoding', value: 'gzip, deflate, br', description: '客户端支持的压缩编码' },
  { key: 'Accept-Language', value: 'zh-CN,zh;q=0.9,en;q=0.8', description: '客户端语言偏好' },
  { key: 'Authorization', description: '认证信息（Bearer Token / Basic Auth）' },
  { key: 'Cache-Control', value: 'no-cache', description: '缓存控制策略' },
  { key: 'Content-Type', description: '请求体的媒体类型' },
  { key: 'Cookie', description: '发送的 Cookie 数据' },
  { key: 'Host', description: '目标主机' },
  { key: 'Origin', description: '请求来源（用于 CORS）' },
  { key: 'Referer', description: '请求来源页面' },
  { key: 'User-Agent', value: 'iApi/1.0', description: '用户代理标识' },
  { key: 'X-Requested-With', value: 'XMLHttpRequest', description: 'AJAX 请求标识' },
  { key: 'X-API-Key', description: 'API 密钥' },
  { key: 'X-Auth-Token', description: '认证令牌' },
  { key: 'X-Request-ID', description: '请求唯一标识' },
  { key: 'If-Modified-Since', description: '条件请求：资源修改时间' },
  { key: 'If-None-Match', description: '条件请求：资源 ETag' },
  { key: 'If-Match', description: '条件请求：资源 ETag' },
  { key: 'Range', description: '范围请求：请求的资源范围' },
  { key: 'Authorization', description: 'OAuth 2.0 Bearer Token' },
]

export const CONTENT_TYPE_OPTIONS = [
  { label: 'application/json', value: 'application/json', description: 'JSON 数据' },
  { label: 'application/xml', value: 'application/xml', description: 'XML 数据' },
  { label: 'application/x-www-form-urlencoded', value: 'application/x-www-form-urlencoded', description: '表单编码数据' },
  { label: 'multipart/form-data', value: 'multipart/form-data', description: '多部分表单数据' },
  { label: 'text/plain', value: 'text/plain', description: '纯文本' },
  { label: 'text/html', value: 'text/html', description: 'HTML 文档' },
  { label: 'application/octet-stream', value: 'application/octet-stream', description: '二进制数据' },
  { label: 'image/png', value: 'image/png', description: 'PNG 图片' },
  { label: 'image/jpeg', value: 'image/jpeg', description: 'JPEG 图片' },
  { label: 'multipart/byteranges', value: 'multipart/byteranges', description: '多部分字节范围' },
]

export function getHeaderSuggestions(query: string): CommonHeader[] {
  if (!query) return COMMON_HEADERS.slice(0, 8)
  const lowerQuery = query.toLowerCase()
  return COMMON_HEADERS.filter(
    h => h.key.toLowerCase().includes(lowerQuery) ||
         (h.description && h.description.toLowerCase().includes(lowerQuery))
  ).slice(0, 8)
}