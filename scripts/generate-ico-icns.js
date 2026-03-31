import sharp from 'sharp';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import pngToIco from 'png-to-ico';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const logoDir = path.join(__dirname, '../src/assets/logo');
const tauriIconsDir = path.join(__dirname, '../src-tauri/icons');
const appIconSvg = path.join(logoDir, 'app-icon.svg');

async function generateIco() {
  console.log('Generating .ico file for Windows...\n');
  
  const sizes = [16, 32, 48, 64, 128, 256];
  const pngBuffers = [];
  
  for (const size of sizes) {
    const buffer = await sharp(appIconSvg)
      .resize(size, size)
      .png()
      .toBuffer();
    pngBuffers.push(buffer);
    console.log(`✓ Prepared ${size}x${size} PNG`);
  }
  
  const icoBuffer = await pngToIco(pngBuffers);
  const icoPath = path.join(tauriIconsDir, 'icon.ico');
  fs.writeFileSync(icoPath, icoBuffer);
  console.log(`\n✅ Created: icon.ico`);
}

async function generateIcns() {
  console.log('\nGenerating .icns file for macOS...\n');
  
  const iconsetDir = path.join(tauriIconsDir, 'icon.iconset');
  if (!fs.existsSync(iconsetDir)) {
    fs.mkdirSync(iconsetDir, { recursive: true });
  }
  
  const sizes = [
    { name: 'icon_16x16.png', size: 16 },
    { name: 'icon_16x16@2x.png', size: 32 },
    { name: 'icon_32x32.png', size: 32 },
    { name: 'icon_32x32@2x.png', size: 64 },
    { name: 'icon_128x128.png', size: 128 },
    { name: 'icon_128x128@2x.png', size: 256 },
    { name: 'icon_256x256.png', size: 256 },
    { name: 'icon_256x256@2x.png', size: 512 },
    { name: 'icon_512x512.png', size: 512 },
    { name: 'icon_512x512@2x.png', size: 1024 },
  ];
  
  for (const { name, size } of sizes) {
    const outputPath = path.join(iconsetDir, name);
    await sharp(appIconSvg)
      .resize(size, size)
      .png()
      .toFile(outputPath);
    console.log(`✓ Created: ${name}`);
  }
  
  console.log('\n📝 To create .icns file, run:');
  console.log(`   iconutil -c icns "${iconsetDir}"`);
  console.log(`   This will create icon.icns in ${tauriIconsDir}`);
}

async function main() {
  try {
    await generateIco();
    await generateIcns();
    console.log('\n✅ All icon files generated!');
  } catch (error) {
    console.error('Error:', error.message);
  }
}

main();
