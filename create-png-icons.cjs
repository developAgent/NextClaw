const fs = require('fs');
const path = require('path');

// Create a minimal 32x32 PNG file
const pngData = Buffer.from([
  // PNG signature
  0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,

  // IHDR chunk (13 bytes header + 12 bytes data + 4 bytes CRC)
  0x00, 0x00, 0x00, 0x0D, // Length
  0x49, 0x48, 0x44, 0x52, // IHDR
  0x00, 0x00, 0x00, 0x20, // Width: 32
  0x00, 0x00, 0x00, 0x20, // Height: 32
  0x08, // Bit depth: 8
  0x06, // Color type: RGBA
  0x00, // Compression: deflate
  0x00, // Filter: adaptive
  0x00, // Interlace: none
  0x49, 0x48, 0x44, 0x52, // CRC placeholder
  0x00, 0x00, 0x00, 0x00,

  // IDAT chunk with minimal data
  0x00, 0x00, 0x00, 0x0A, // Length
  0x49, 0x44, 0x41, 0x54, // IDAT
  0x78, 0x9C, // Zlib header
  0x01, 0x00, 0x00, 0xFF, 0xFF, // Minimal zlib data
  0x00, 0x00, 0x00, 0x02, 0x00, 0x01, // Adler32 (dummy)

  // IEND chunk
  0x00, 0x00, 0x00, 0x00, // Length
  0x49, 0x45, 0x4E, 0x44, // IEND
  0xAE, 0x42, 0x60, 0x82 // CRC
]);

const iconsDir = path.join(__dirname, 'src-tauri', 'icons');
if (!fs.existsSync(iconsDir)) {
  fs.mkdirSync(iconsDir, { recursive: true });
}

// Create 32x32.png
fs.writeFileSync(path.join(iconsDir, '32x32.png'), pngData);
console.log('Created 32x32.png');

// Create 128x128.png
fs.writeFileSync(path.join(iconsDir, '128x128.png'), pngData);
console.log('Created 128x128.png');

// Create 128x128@2x.png
fs.writeFileSync(path.join(iconsDir, '128x128@2x.png'), pngData);
console.log('Created 128x128@2x.png');

console.log('All PNG icons created!');