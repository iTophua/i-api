import sharp from 'sharp'
import { readFileSync, writeFileSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const iconsDir = join(__dirname, '../src-tauri/icons')

async function generateDmgBackground() {
  const svgPath = join(iconsDir, 'dmg-background.svg')
  const pngPath = join(iconsDir, 'dmg-background.png')
  const png2xPath = join(iconsDir, 'dmg-background@2x.png')
  
  const svgBuffer = readFileSync(svgPath)
  
  await sharp(svgBuffer)
    .resize(660, 400)
    .png()
    .toFile(pngPath)
  
  console.log('✅ Generated dmg-background.png (660x400)')
  
  await sharp(svgBuffer)
    .resize(1320, 800)
    .png()
    .toFile(png2xPath)
  
  console.log('✅ Generated dmg-background@2x.png (1320x800)')
}

generateDmgBackground().catch(console.error)
