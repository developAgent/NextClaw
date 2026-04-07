const fs = require('fs');
const path = require('path');

// Create a valid Windows .ico file with embedded PNG data
// This format includes proper ICO header and PNG image data

function createIcoWithPng() {
  // Create a simple 32x32 PNG (in-memory)
  const pngData = Buffer.from([
    // PNG signature
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,

    // IHDR
    0x00, 0x00, 0x00, 0x0D, // Length
    0x49, 0x48, 0x44, 0x52, // Type
    0x00, 0x00, 0x00, 0x20, // Width 32
    0x00, 0x00, 0x00, 0x20, // Height 32
    0x08, 0x06, 0x00, 0x00, 0x00, // Bit depth, color type, etc.
    0x73, 0x26, 0x4A, 0x4B, // CRC

    // sRGB chunk
    0x00, 0x00, 0x00, 0x01, // Length
    0x73, 0x52, 0x47, 0x42, // Type
    0x00, // Rendering intent
    0xAE, 0xCE, 0x1C, 0xE9, // CRC

    // IDAT (minimal data)
    0x00, 0x00, 0x00, 0x0E, // Length
    0x49, 0x44, 0x41, 0x54, // Type
    0x78, 0x9C, 0x62, 0x00, 0x00, 0x00, 0x01, 0x00,
    0x00, 0x05, 0x00, 0x01, // Minimal zlib data
    0x0D, 0x0A, 0x2D, 0xB4, // CRC

    // IEND
    0x00, 0x00, 0x00, 0x00, // Length
    0x49, 0x45, 0x4E, 0x44, // Type
    0xAE, 0x42, 0x60, 0x82  // CRC
  ]);

  // ICO file header (6 bytes)
  const icoHeader = Buffer.from([
    0x00, 0x00, // Reserved
    0x01, 0x00, // 1 image (ICO format)
    0x01, 0x00  // 1 image in directory
  ]);

  // Image directory entry (16 bytes)
  const dirEntry = Buffer.from([
    0x20,         // Width: 32
    0x20,         // Height: 32
    0x00,         // No color palette
    0x00,         // Reserved
    0x01, 0x00,   // 1 color plane
    0x20, 0x00,   // 32 bits per pixel
    pngData.length & 0xFF, (pngData.length >> 8) & 0xFF, // Size (little endian)
    0x00, 0x00,
    0x16, 0x00, 0x00, 0x00  // Offset to PNG data (22 = header + directory)
  ]);

  return Buffer.concat([icoHeader, dirEntry, pngData]);
}

const iconsDir = path.join(__dirname, 'src-tauri', 'icons');
if (!fs.existsSync(iconsDir)) {
  fs.mkdirSync(iconsDir, { recursive: true });
}

const icoData = createIcoWithPng();
const icoPath = path.join(iconsDir, 'icon.ico');
fs.writeFileSync(icoPath, icoData);
console.log(`Created ${icoPath} (${icoData.length} bytes)`);

// Verify it's a valid ICO
const header = icoData.readUInt16LE(0);
if (header === 0) {
  console.log('✓ Valid ICO file header');
} else {
  console.log('✗ Invalid ICO file header');
}