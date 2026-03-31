import { chromium, FullConfig } from '@playwright/test'

async function globalSetup(_config: FullConfig) {
  const browser = await chromium.launch()
  const context = await browser.newContext()
  const page = await context.newPage()

  await page.goto('http://localhost:5173')
  await page.waitForLoadState('networkidle')

  await browser.close()
}

export default globalSetup
