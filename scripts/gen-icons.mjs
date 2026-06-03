/**
 * Icon generator — reads an SVG source and produces all required
 * PNG icons for the Tauri bundle (app icons + tray icons).
 *
 * Usage:
 *   ICON_SVG=path/to/icon.svg node scripts/gen-icons.mjs
 *
 * If ICON_SVG is not set, defaults to scripts/assets/icon.svg.
 */
import sharp from 'sharp';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const iconDir = path.join(__dirname, '..', 'src-tauri', 'icons');

// SVG source path — override via ICON_SVG env var, or fall back to scripts/assets/icon.svg
const svgSource = process.env.ICON_SVG || path.join(__dirname, 'assets', 'icon.svg');

// Read the SVG and replace the default gray fill with white for app icons
const svgContent = fs.readFileSync(svgSource, 'utf8');
const whiteSvg = svgContent.replace(/#515151/g, '#FFFFFF');
const pathMatch = whiteSvg.match(/<path[^>]*d="([^"]+)"/);
const iconPath = pathMatch ? pathMatch[1] : '';

// Generate app icons — blue rounded square with white icon
const sizes = [16, 32, 64, 128, 256, 512, 1024];
const cornerRadius = 230;

for (const size of sizes) {
  const scale = size / 1024;
  const rx = Math.round(cornerRadius * scale);
  const iconScale = 0.6;
  const iconOffset = (size - size * iconScale) / 2;

  const compositeSvg = `<svg width="${size}" height="${size}" viewBox="0 0 ${size} ${size}" xmlns="http://www.w3.org/2000/svg">
    <rect width="${size}" height="${size}" rx="${rx}" ry="${rx}" fill="#0A84FF"/>
    <g transform="translate(${iconOffset}, ${iconOffset}) scale(${iconScale})">
      <path d="${iconPath}" fill="#FFFFFF"/>
    </g>
  </svg>`;

  await sharp(Buffer.from(compositeSvg))
    .png()
    .toFile(path.join(iconDir, `app-${size}.png`));
  console.log(`app-${size}.png done`);
}

// Copy to Tauri's expected icon filenames for bundling
const names = {
  'app-16.png': '16x16.png',
  'app-32.png': '32x32.png',
  'app-128.png': '128x128.png',
  'app-256.png': '128x128@2x.png',
  'app-256.png': '256x256.png',
  'app-512.png': '512x512.png',
  'app-1024.png': '1024x1024.png',
};
for (const [src, dst] of Object.entries(names)) {
  fs.copyFileSync(path.join(iconDir, src), path.join(iconDir, dst));
}

// Generate tray icons — gray icon on transparent background (macOS template image)
const traySvgContent = fs.readFileSync(svgSource, 'utf8');
const trayPathMatch = traySvgContent.match(/<path[^>]*d="([^"]+)"/);
const trayIconPath = trayPathMatch ? trayPathMatch[1] : '';
for (const size of [16, 32]) {
  const traySvg = `<svg width="${size}" height="${size}" viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg">
    <path d="${trayIconPath}" fill="#515151"/>
  </svg>`;
  await sharp(Buffer.from(traySvg))
    .png()
    .toFile(path.join(iconDir, `tray-${size}.png`));
  console.log(`tray-${size}.png done`);
}

console.log('All icons generated!');
