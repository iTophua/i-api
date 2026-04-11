import type { Request } from '@/types'

export type CodeLanguage =
  | 'curl'
  | 'javascript-axios'
  | 'javascript-fetch'
  | 'python'
  | 'java'
  | 'go'

export interface CodeGeneratorOptions {
  replaceVariables?: boolean
  includeDependencies?: boolean
}

export function generateCode(
  request: Request,
  language: CodeLanguage,
  options: CodeGeneratorOptions = {}
): string {
  const { replaceVariables = true, includeDependencies = true } = options

  const processedRequest = replaceVariables
    ? {
        ...request,
        url: replaceEnvVariables(request.url),
        headers: request.headers.map((h) => ({
          ...h,
          value: replaceEnvVariables(h.value),
        })),
      }
    : request

  let code = ''
  switch (language) {
    case 'curl':
      code = generateCurl(processedRequest)
      break
    case 'javascript-axios':
      code = generateAxios(processedRequest)
      break
    case 'javascript-fetch':
      code = generateFetch(processedRequest)
      break
    case 'python':
      code = generatePython(processedRequest)
      break
    case 'java':
      code = generateJava(processedRequest)
      break
    case 'go':
      code = generateGo(processedRequest)
      break
    default:
      code = ''
  }

  if (includeDependencies) {
    const deps = getDependencies(language)
    if (deps) {
      code = `# 安装依赖\n${deps}\n\n# 代码\n${code}`
    }
  }

  return code
}

function replaceEnvVariables(text: string): string {
  const variablePattern = /\{\{([^}]+)\}\}/g
  return text.replace(variablePattern, (_match, varName) => {
    return `<${varName}>`
  })
}

function getDependencies(language: CodeLanguage): string | null {
  switch (language) {
    case 'curl':
      return `# cURL 通常系统自带，无需安装`
    case 'javascript-axios':
      return `# npm install axios`
    case 'javascript-fetch':
      return `# Node.js 18+ 内置，无需安装\n# 旧版本: npm install node-fetch`
    case 'python':
      return `# pip install requests`
    case 'java':
      return `# Maven pom.xml 添加:\n# <dependency>\n#     <groupId>com.squareup.okhttp3</groupId>\n#     <artifactId>okhttp</artifactId>\n#     <version>4.12.0</version>\n# </dependency>`
    case 'go':
      return `# go mod init yourmodule\n# go get github.com/google/go-httpclient (可选)`
    default:
      return null
  }
}

function generateCurl(request: Request): string {
  const url = JSON.stringify(request.url)
  const lines: string[] = [`curl -X ${request.method} ${url}`]

  const enabledHeaders = request.headers.filter((h) => h.enabled)
  for (const header of enabledHeaders) {
    lines.push(`  -H '${header.key}: ${header.value}'`)
  }

  if (request.body.mode === 'raw' && request.body.raw) {
    lines.push(`  -d '${request.body.raw.replace(/'/g, "'\\''")}'`)
  } else if (request.body.mode === 'form-data' && request.body.formData) {
    for (const field of request.body.formData) {
      if (field.enabled) {
        if (field.type === 'file' && field.filePath) {
          lines.push(`  -F '${field.key}=@${field.filePath}'`)
        } else {
          lines.push(`  -F '${field.key}=${field.value.replace(/'/g, "'\\''")}'`)
        }
      }
    }
  } else if (request.body.mode === 'urlencoded' && request.body.urlencoded) {
    for (const field of request.body.urlencoded) {
      if (field.enabled) {
        lines.push(`  --data-urlencode '${field.key}=${field.value.replace(/'/g, "'\\''")}'`)
      }
    }
  }

  return lines.join(' \\\n')
}

function generateAxios(request: Request): string {
  const url = JSON.stringify(request.url)
  const lines: string[] = [
    `import axios from 'axios';`,
    '',
    `const response = await axios({`,
    `  method: '${request.method.toLowerCase()}',`,
    `  url: ${url},`,
  ]

  const enabledHeaders = request.headers.filter((h) => h.enabled)
  if (enabledHeaders.length > 0) {
    lines.push('  headers: {')
    for (const header of enabledHeaders) {
      lines.push(`    '${header.key}': '${header.value}',`)
    }
    lines.push('  },')
  }

  if (request.body.mode === 'raw' && request.body.raw) {
    if (request.body.rawType === 'json') {
      lines.push(`  data: ${request.body.raw},`)
    } else {
      lines.push(`  data: '${request.body.raw}',`)
    }
  }

  lines.push('});')
  lines.push('')
  lines.push('console.log(response.data);')

  return lines.join('\n')
}

function generateFetch(request: Request): string {
  const lines: string[] = []

  const options: string[] = [`method: '${request.method}'`]

  const enabledHeaders = request.headers.filter((h) => h.enabled)
  if (enabledHeaders.length > 0) {
    const headersStr = enabledHeaders.map((h) => `'${h.key}': '${h.value}'`).join(',\n      ')
    options.push(`headers: {\n      ${headersStr}\n    }`)
  }

  if (request.body.mode === 'raw' && request.body.raw) {
    if (request.body.rawType === 'json') {
      options.push(`body: JSON.stringify(${request.body.raw})`)
    } else {
      options.push(`body: '${request.body.raw}'`)
    }
  }

  const url = JSON.stringify(request.url)
  lines.push(`const response = await fetch(${url}, {`)
  lines.push(`  ${options.join(',\n  ')}`)
  lines.push('});')
  lines.push('')
  lines.push('const data = await response.json();')
  lines.push('console.log(data);')

  return lines.join('\n')
}

function generatePython(request: Request): string {
  const url = JSON.stringify(request.url)
  const lines: string[] = [`import requests`, '', `url = ${url}`]

  const enabledHeaders = request.headers.filter((h) => h.enabled)
  if (enabledHeaders.length > 0) {
    lines.push('headers = {')
    for (const header of enabledHeaders) {
      lines.push(`    '${header.key}': '${header.value}',`)
    }
    lines.push('}')
  }

  if (request.body.mode === 'raw' && request.body.raw) {
    if (request.body.rawType === 'json') {
      lines.push(`json_data = ${request.body.raw}`)
      lines.push('')
      lines.push(
        `response = requests.${request.method.toLowerCase()}(url, headers=headers, json=json_data)`
      )
    } else {
      lines.push(`data = '${request.body.raw}'`)
      lines.push('')
      lines.push(
        `response = requests.${request.method.toLowerCase()}(url, headers=headers, data=data)`
      )
    }
  } else {
    lines.push('')
    lines.push(`response = requests.${request.method.toLowerCase()}(url, headers=headers)`)
  }

  lines.push('')
  lines.push('print(response.json())')

  return lines.join('\n')
}

function generateJava(request: Request): string {
  const lines: string[] = [
    `import okhttp3.*;`,
    `import java.io.IOException;`,
    '',
    `OkHttpClient client = new OkHttpClient();`,
    '',
  ]

  const method = request.method.toUpperCase()
  const bodyType = request.body.mode

  if (bodyType === 'raw' && request.body.raw) {
    lines.push(`MediaType mediaType = MediaType.parse("application/json");`)
    lines.push(
      `RequestBody body = RequestBody.create(mediaType, "${request.body.raw.replace(/"/g, '\\"')}");`
    )
  }

  lines.push(`Request request = new Request.Builder()`)
  lines.push(`    .url("${request.url}")`)

  const enabledHeaders = request.headers.filter((h) => h.enabled)
  for (const header of enabledHeaders) {
    lines.push(`    .addHeader("${header.key}", "${header.value}")`)
  }

  if (method === 'GET') {
    lines.push('    .get()')
  } else if (method === 'POST' && bodyType === 'raw') {
    lines.push('    .post(body)')
  } else if (method === 'PUT' && bodyType === 'raw') {
    lines.push('    .put(body)')
  } else if (method === 'DELETE') {
    lines.push('    .delete()')
  }

  lines.push('    .build();')
  lines.push('')
  lines.push('try (Response response = client.newCall(request).execute()) {')
  lines.push('    System.out.println(response.body().string());')
  lines.push('}')

  return lines.join('\n')
}

function generateGo(request: Request): string {
  const url = JSON.stringify(request.url)
  const lines: string[] = [
    `package main`,
    '',
    `import (`,
    `    "fmt"`,
    `    "io"`,
    `    "net/http"`,
    `    "strings"`,
    `)`,
    '',
    `func main() {`,
  ]

  const method = request.method.toUpperCase()

  if (request.body.mode === 'raw' && request.body.raw) {
    lines.push(`    body := strings.NewReader(\`${request.body.raw}\`)`)
    lines.push(`    req, _ := http.NewRequest("${method}", ${url}, body)`)
  } else {
    lines.push(`    req, _ := http.NewRequest("${method}", ${url}, nil)`)
  }

  const enabledHeaders = request.headers.filter((h) => h.enabled)
  for (const header of enabledHeaders) {
    lines.push(`    req.Header.Set("${header.key}", "${header.value}")`)
  }

  lines.push('')
  lines.push('    client := &http.Client{}')
  lines.push('    resp, _ := client.Do(req)')
  lines.push('    defer resp.Body.Close()')
  lines.push('')
  lines.push('    body, _ := io.ReadAll(resp.Body)')
  lines.push('    fmt.Println(string(body))')
  lines.push('}')

  return lines.join('\n')
}