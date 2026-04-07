const fs = require('fs');
const path = require('path');

// Create a minimal 32x32 .ico file
// This is a valid ICO file header + minimal bitmap data
const icoData = Buffer.from([
  // ICO header
  0x00, 0x00,                   // Reserved
  0x01, 0x00,                   // 1 image
  0x01, 0x00,                   // 1 image in directory

  // Image directory entry
  0x20,                         // 32x32
  0x20,                         // 32x32
  0x00,                         // No palette
  0x00, 0x00,                   // Reserved
  0x01, 0x00,                   // Color planes
  0x20, 0x00,                   // 32 bits per pixel
  0x00, 0x00, 0x00, 0x00,       // Size (placeholder)
  0x16, 0x00, 0x00, 0x00,       // Offset to bitmap data (22 bytes)

  // Minimal PNG data for a 32x32 image
  0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
  // IHDR chunk
  0x00, 0x00, 0x00, 0x1D, 0x49, 0x48, 0x44, 0x52, // IHDR
  0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x20, // 32x32
  0x08, 0x06, 0x00, 0x00, 0x00, 0x3A, 0x7E, 0x9B, // 8-bit, RGBA
  // IDAT chunk
  0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, // IDAT
  0x78, 0x9C, 0x61, 0x60, 0x00, 0x00, 0x00, 0x02, // Minimal compressed data
  0x00, 0x01, 0x00, 0x00, // IEND chunk
  0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND
  0xAE, 0x42, 0x60, 0x82
]);

const iconsDir = path.join(__dirname, 'src-tauri', 'icons');
if (!fs.existsSync(iconsDir)) {
  fs.mkdirSync(iconsDir, { recursive: true });
}

// Create icon.ico
const iconPath = path.join(iconsDir, 'icon.ico');
fs.writeFileSync(iconPath, icoData);
console.log('Created icon.ico');

// Also create 32x32.png (using the same PNG data embedded above)
const pngData = icoData.slice(22); // Skip ICO header
const pngPath = path.join(iconsDir, '32x32.png');
fs.writeFileSync(pngPath, pngData);
console.log('Created 32x32.png');

// Create 128x128.png (we'll just copy the 32x32 one)
const png128Path = path.join(iconsDir, '128x128.png');
fs.writeFileSync(png128Path, pngData);
console.log('Created 128x128.png');

// Create 128x128@2x.png
const png256Path = path.join(iconsDir, '128x128@2x.png');
fs.writeFileSync(png256Path, pngData);
console.log('Created 128x128@2x.png');

console.log('All icon files created successfully!');