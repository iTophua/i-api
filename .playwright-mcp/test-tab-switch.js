async (page) => {
  const newBtn = page.getByRole('button', { name: '新建请求' })
  if (await newBtn.isVisible()) {
    await newBtn.click()
    await page.waitForTimeout(1000)
  }
  await page.locator('.request-panel .n-tabs-tab').filter({ hasText: '前置脚本' }).click()
  await page.waitForTimeout(1000)
  await page.locator('.request-panel .n-tabs-tab').filter({ hasText: '参数' }).click()
  await page.waitForTimeout(1000)
  const result = await page.evaluate(() => {
    const pane = document.querySelector('.request-panel .n-tab-pane')
    if (!pane) return { error: 'NO PANE' }
    const content = pane.querySelector('.tab-scroll-content')
    if (!content) return { error: 'NO CONTENT' }
    const rect = content.getBoundingClientRect()
    const style = window.getComputedStyle(content)
    return {
      height: rect.height,
      width: rect.width,
      visible: rect.height > 0,
      display: style.display,
      flex: style.flex,
    }
  })
  console.log('Params tab after switch:', JSON.stringify(result))
}
