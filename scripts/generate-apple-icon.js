import sharp from 'sharp';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const logoDir = path.join(__dirname, '../src/assets/logo');
const pngDir = path.join(logoDir, 'png');
const appIconSvg = path.join(logoDir, 'app-icon.svg');

async function generateAppleTouchIcon() {
  console.log('Generating Apple Touch Icon (180x180)...\n');
  
  const outputPath = path.join(pngDir, 'app-icon-180x180.png');
  
  await sharp(appIconSvg)
    .resize(180, 180)
    .png()
    .toFile(outputPath);
  
  console.log(`✅ Created: app-icon-180x180.png`);
}

generateAppleTouchIcon().catch(console.error);
