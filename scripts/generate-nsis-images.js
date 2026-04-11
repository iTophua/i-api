import sharp from 'sharp'
import { readFileSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const iconsDir = join(__dirname, '../src-tauri/icons')

async function generateNsisImages() {
  const headerSvgPath = join(iconsDir, 'nsis-header.svg')
  const sidebarSvgPath = join(iconsDir, 'nsis-sidebar.svg')

  const headerPpmPath = join(iconsDir, 'nsis-header.ppm')
  const sidebarPpmPath = join(iconsDir, 'nsis-sidebar.ppm')

  const headerSvgBuffer = readFileSync(headerSvgPath)
  const sidebarSvgBuffer = readFileSync(sidebarSvgPath)

  await sharp(headerSvgBuffer)
    .resize(499, 60)
    .raw()
    .toFile(headerPpmPath)

  console.log('✅ Generated nsis-header.ppm (499x60)')

  await sharp(sidebarSvgBuffer)
    .resize(164, 314)
    .raw()
    .toFile(sidebarPpmPath)

  console.log('✅ Generated nsis-sidebar.ppm (164x314)')
}

generateNsisImages().catch(console.error)
