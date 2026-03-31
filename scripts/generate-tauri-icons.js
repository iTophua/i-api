import sharp from 'sharp';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const logoDir = path.join(__dirname, '../src/assets/logo');
const tauriIconsDir = path.join(__dirname, '../src-tauri/icons');

const appIconSvg = path.join(logoDir, 'app-icon.svg');

async function generateTauriIcons() {
  console.log('Generating Tauri app icons...\n');
  
  const iconSizes = [
    { name: '32x32.png', size: 32 },
    { name: '128x128.png', size: 128 },
    { name: '128x128@2x.png', size: 256 },
    { name: 'icon.png', size: 1024 },
    { name: 'Square30x30Logo.png', size: 30 },
    { name: 'Square44x44Logo.png', size: 44 },
    { name: 'Square71x71Logo.png', size: 71 },
    { name: 'Square89x89Logo.png', size: 89 },
    { name: 'Square107x107Logo.png', size: 107 },
    { name: 'Square142x142Logo.png', size: 142 },
    { name: 'Square150x150Logo.png', size: 150 },
    { name: 'Square284x284Logo.png', size: 284 },
    { name: 'Square310x310Logo.png', size: 310 },
    { name: 'StoreLogo.png', size: 50 },
  ];
  
  for (const { name, size } of iconSizes) {
    const outputPath = path.join(tauriIconsDir, name);
    try {
      await sharp(appIconSvg)
        .resize(size, size)
        .png()
        .toFile(outputPath);
      console.log(`✓ Created: ${name}`);
    } catch (error) {
      console.error(`✗ Error creating ${name}:`, error.message);
    }
  }
  
  console.log('\n✅ Tauri icons generated successfully!');
}

generateTauriIcons().catch(console.error);
