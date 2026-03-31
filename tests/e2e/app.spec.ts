import { test, expect } from '@playwright/test'

test.describe('iApi E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.waitForLoadState('networkidle')
  })

  test.describe('Application Launch', () => {
    test('should load the application successfully', async ({ page }) => {
      await expect(page).toHaveTitle(/iApi/)
    })

    test('should display the main layout', async ({ page }) => {
      await expect(page.locator('.home-view')).toBeVisible({ timeout: 10000 })
    })

    test('should show empty state initially', async ({ page }) => {
      await expect(page.getByText('未命名请求')).toBeVisible()
      await expect(page.getByText('输入请求 URL 或粘贴 cURL 命令')).toBeVisible()
    })
  })

  test.describe('Request Creation Flow', () => {
    test('should create a new request', async ({ page }) => {
      const newRequestBtn = page.getByRole('button', { name: '新建请求' })
      await newRequestBtn.click()

      await expect(page.locator('.tab-item')).toHaveCount(1)
    })

    test('should fill in URL and method', async ({ page }) => {
      const newRequestBtn = page.getByRole('button', { name: '新建请求' })
      await newRequestBtn.click()

      const methodSelect = page.locator('.n-select').first()
      await methodSelect.click()
      await page.getByText('POST').click()

      const urlInput = page.locator('.url-input input')
      await urlInput.fill('https://api.example.com/users')

      await expect(urlInput).toHaveValue('https://api.example.com/users')
    })

    test('should switch between request tabs', async ({ page }) => {
      const newRequestBtn = page.getByRole('button', { name: '新建请求' })
      await newRequestBtn.click()
      await newRequestBtn.click()

      const tabs = page.locator('.tab-item')
      await expect(tabs).toHaveCount(2)

      await tabs.nth(1).click()
      await expect(tabs.nth(1)).toHaveClass(/active/)
    })

    test('should close a request tab', async ({ page }) => {
      const newRequestBtn = page.getByRole('button', { name: '新建请求' })
      await newRequestBtn.click()
      await newRequestBtn.click()

      const tabs = page.locator('.tab-item')
      await expect(tabs).toHaveCount(2)

      await tabs.nth(0).locator('.tab-close').click()
      await expect(tabs).toHaveCount(1)
    })
  })

  test.describe('Request Configuration', () => {
    test.beforeEach(async ({ page }) => {
      const newRequestBtn = page.getByRole('button', { name: '新建请求' })
      await newRequestBtn.click()
    })

    test('should add query parameters', async ({ page }) => {
      await page.getByText('参数').click()

      const paramInputs = page.locator('.key-value-editor .n-input')
      await paramInputs.nth(0).fill('page')
      await paramInputs.nth(1).fill('1')

      await expect(page.locator('.key-value-editor .n-input').first()).toHaveValue('page')
    })

    test('should add headers', async ({ page }) => {
      await page.getByText('请求头').click()

      const headerInputs = page.locator('.key-value-editor .n-input')
      await headerInputs.nth(0).fill('Content-Type')
      await headerInputs.nth(1).fill('application/json')

      await expect(headerInputs.first()).toHaveValue('Content-Type')
    })

    test('should configure body', async ({ page }) => {
      await page.getByText('请求体').click()

      const bodyTextarea = page.locator('.body-editor textarea, .body-editor .n-input')
      await bodyTextarea.fill('{"name":"test"}')

      await expect(bodyTextarea.first()).toHaveValue('{"name":"test"}')
    })

    test('should configure authentication', async ({ page }) => {
      await page.getByText('认证').click()

      const authSelect = page.locator('.auth-type-selector .n-select')
      await authSelect.click()
      await page.getByText('Basic 认证').click()

      const usernameInput = page.locator('input[placeholder="用户名"]')
      await usernameInput.fill('testuser')

      await expect(usernameInput).toHaveValue('testuser')
    })
  })

  test.describe('Collection Management', () => {
    test('should create a collection', async ({ page }) => {
      const collectionTab = page.locator('.n-tabs-tab').filter({ hasText: '集合' })
      await collectionTab.click()

      const addBtn = page.locator('.add-btn')
      await addBtn.click()

      await expect(page.locator('.list-item')).toHaveCount(1)
      await expect(page.locator('.list-item .item-name').first()).toContainText('新集合')
    })

    test('should save request to collection', async ({ page }) => {
      await page.getByRole('button', { name: '新建请求' }).click()

      const urlInput = page.locator('.url-input input')
      await urlInput.fill('https://api.example.com/test')

      const saveBtn = page.getByRole('button', { name: '保存' })
      await saveBtn.click()

      await page.getByText('新集合').first().click()
    })
  })

  test.describe('Environment Management', () => {
    test('should switch environments', async ({ page }) => {
      const envTab = page.locator('.n-tabs-tab').filter({ hasText: '环境' })
      await envTab.click()

      await page.locator('.add-btn').click()

      await expect(page.locator('.list-item')).toHaveCount(1)
    })
  })

  test.describe('Keyboard Shortcuts', () => {
    test('should create new request with Ctrl+N', async ({ page }) => {
      await page.keyboard.press('Control+n')

      await expect(page.locator('.tab-item')).toHaveCount(1)
    })

    test('should focus URL with keyboard navigation', async ({ page }) => {
      const urlInput = page.locator('.url-input input')
      await urlInput.click()

      await expect(urlInput).toBeFocused()
    })
  })

  test.describe('UI Theme', () => {
    test('should toggle theme', async ({ page }) => {
      const themeBtn = page.locator('.n-button[circle]').filter({ has: page.locator('.n-icon') }).last()
      await themeBtn.click()

      await page.waitForTimeout(500)
    })
  })
})
