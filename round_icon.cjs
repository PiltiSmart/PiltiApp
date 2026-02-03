const fs = require('fs');
const { createCanvas, loadImage } = require('canvas');

async function roundCorners(inputPath, outputPath, size, radius) {
    const canvas = createCanvas(size, size);
    const ctx = canvas.getContext('2d');

    // Create rounded path
    ctx.beginPath();
    ctx.moveTo(radius, 0);
    ctx.lineTo(size - radius, 0);
    ctx.quadraticCurveTo(size, 0, size, radius);
    ctx.lineTo(size, size - radius);
    ctx.quadraticCurveTo(size, size, size - radius, size);
    ctx.lineTo(radius, size);
    ctx.quadraticCurveTo(0, size, 0, size - radius);
    ctx.lineTo(0, radius);
    ctx.quadraticCurveTo(0, 0, radius, 0);
    ctx.closePath();
    ctx.clip(); // Clip subsequent drawing to this path

    const image = await loadImage(inputPath);
    ctx.drawImage(image, 0, 0, size, size);

    const buffer = canvas.toBuffer('image/png');
    fs.writeFileSync(outputPath, buffer);
}

// macOS style rounding is typically ~22% of dimension
// For 1024px icon, radius is ~225px
roundCorners(process.argv[2], process.argv[3], 1024, 225).catch(console.error);
