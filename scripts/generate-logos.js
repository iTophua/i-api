import sharp from 'sharp';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const logoDir = path.join(__dirname, '../src/assets/logo');
const outputDir = path.join(logoDir, 'png');
const jpgDir = path.join(logoDir, 'jpg');

if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}
if (!fs.existsSync(jpgDir)) {
  fs.mkdirSync(jpgDir, { recursive: true });
}

const sizes = [16, 32, 48, 64, 128, 256, 512, 1024];

async function convertSvgToPng(svgFile, baseName) {
  const svgPath = path.join(logoDir, svgFile);
  if (!fs.existsSync(svgPath)) {
    console.log(`Skipping ${svgFile} - file not found`);
    return;
  }
  
  for (const size of sizes) {
    const outputPath = path.join(outputDir, `${baseName}-${size}x${size}.png`);
    try {
      await sharp(svgPath)
        .resize(size, size)
        .png()
        .toFile(outputPath);
      console.log(`Created: ${outputPath}`);
    } catch (error) {
      console.error(`Error creating ${outputPath}:`, error.message);
    }
  }
}

async function convertSvgToJpg(svgFile, baseName) {
  const svgPath = path.join(logoDir, svgFile);
  if (!fs.existsSync(svgPath)) {
    return;
  }
  
  const jpgSizes = [256, 512, 1024];
  for (const size of jpgSizes) {
    const outputPath = path.join(jpgDir, `${baseName}-${size}x${size}.jpg`);
    try {
      await sharp(svgPath)
        .resize(size, size)
        .flatten({ background: { r: 255, g: 255, b: 255 } })
        .jpeg({ quality: 90 })
        .toFile(outputPath);
      console.log(`Created: ${outputPath}`);
    } catch (error) {
      console.error(`Error creating ${outputPath}:`, error.message);
    }
  }
}

async function generateHorizontalPng() {
  const svgPath = path.join(logoDir, 'logo-horizontal-light.svg');
  if (fs.existsSync(svgPath)) {
    const widths = [200, 400, 800, 1200];
    for (const width of widths) {
      const outputPath = path.join(outputDir, `logo-horizontal-${width}w.png`);
      try {
        await sharp(svgPath)
          .resize(width)
          .png()
          .toFile(outputPath);
        console.log(`Created: ${outputPath}`);
      } catch (error) {
        console.error(`Error creating ${outputPath}:`, error.message);
      }
    }
  }
}

async function main() {
  console.log('Converting SVG logos to PNG and JPG formats...\n');
  
  await convertSvgToPng('icon.svg', 'icon');
  await convertSvgToPng('favicon.svg', 'favicon');
  await convertSvgToPng('app-icon.svg', 'app-icon');
  await convertSvgToPng('icon-mono.svg', 'icon-mono');
  
  await convertSvgToJpg('icon.svg', 'icon');
  await convertSvgToJpg('app-icon.svg', 'app-icon');
  
  await generateHorizontalPng();
  
  console.log('\n✅ Logo conversion complete!');
}

main().catch(console.error);
