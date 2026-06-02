import sharp from 'sharp';
import fs from 'fs';
import path from 'path';

const iconDir = '/Users/carroo/claude/rclone_ui/src-tauri/icons';

// Read the link SVG and replace color with white
const svgContent = fs.readFileSync('/Users/carroo/Downloads/挂载 (1).svg', 'utf8');
const whiteSvg = svgContent.replace(/#515151/g, '#FFFFFF');
const pathMatch = whiteSvg.match(/<path[^>]*d="([^"]+)"/);
const iconPath = pathMatch ? pathMatch[1] : '';

// Generate app icon using a single flat SVG (no nested svg tags)
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

// Copy to standard names for bundle
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

// Generate tray icons (transparent, using the same icon as app but original gray color for template image)
const traySvgContent = fs.readFileSync('/Users/carroo/Downloads/挂载 (1).svg', 'utf8');
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
