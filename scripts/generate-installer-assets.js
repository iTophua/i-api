import { exec } from 'child_process'
import { promisify } from 'util'

const execAsync = promisify(exec)

async function generateInstallerAssets() {
  console.log('🎨 Generating installer assets...\n')
  
  console.log('📦 Generating DMG background...')
  await execAsync('node scripts/generate-dmg-background.js')
  
  console.log('\n🪟 Generating NSIS installer images...')
  await execAsync('node scripts/generate-nsis-images.js')
  
  console.log('\n✅ All installer assets generated successfully!')
}

generateInstallerAssets().catch(console.error)
