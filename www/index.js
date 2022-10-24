import { CPU } from "viper";
import { memory } from "viper/viper_bg";

// define some constants 
const FRAME_RELATIVE_SIZE = 0.8;  // frame size relative to window
const PIXEL_SIZE = 20;            // px, size of each screen pixel in the canvas
const SCREEN_START_X = 7;         // px 
const SCREEN_START_Y = 11;        // px

// define the canvas
const canvas = document.getElementById("display-canvas");
const ctx = canvas.getContext('2d');

// fetch the image
const frameImg = new Image();
frameImg.src = "img/frame.png";

frameImg.onload = function (e)
{
    // get frame image size 
    const frameWidth = frameImg.width;
    const frameHeight = frameImg.height;

    // set the internal width and height of canvas
    canvas.width = frameWidth;
    canvas.height = frameHeight;

    // scale the canvas element to take up the correct amount of the screen
    const scale = FRAME_RELATIVE_SIZE * Math.min(
        window.innerHeight / frameHeight, 
        window.innerWidth / frameWidth
    );
    canvas.style.width = `${frameWidth * scale}px`;
    canvas.style.height = `${frameHeight * scale}px`;

    // finally, draw the frame
    ctx.drawImage(frameImg, 0, 0, frameWidth, frameHeight);
}

const cpu = CPU.new();

const romSelector = document.getElementById('rom-selector');

romSelector.addEventListener('change', (event) => {
    const reader = new FileReader();
    const romFile = event.target.files[0];

    reader.onload = () => {
        const romData = new Uint8Array(reader.result);
        cpu.load(romData);
        requestAnimationFrame(renderLoop);
    }

    reader.readAsArrayBuffer(romFile);
});

const displayWidth = cpu.display_width();
const displayHeight = cpu.display_height();
const pixelsPtr = cpu.display_pixels();
const pixels = new Uint8Array(memory.buffer, pixelsPtr, displayWidth * displayHeight / 8);

const pixelIsOn = (row, col, pixels) => {
    const idx = row * displayWidth / 8 + Math.floor(col / 8);
    console.log(`LOC: ${row} ${col}`)
    console.log(`PIXEL: ${pixels[idx]}`);
    const mask = 1 << (7 - col % 8);
    console.log(`MASK: ${mask}`);
    console.log(`RESULT: ${pixels[idx] & mask}`);
    return (pixels[idx] & mask) != 0;
}

const renderLoop = () => {
    cpu.step();
    ctx.beginPath();

    ctx.fillStyle = "#FFFFFF";
    for (let row = 0; row < displayHeight; row++) { 
        for (let col = 0; col < displayWidth; col++) {
            if (pixelIsOn(row, col, pixels)) {
                console.log(`PIXEL at ${col}`);
                ctx.fillRect((SCREEN_START_X + col) * PIXEL_SIZE, 
                    (SCREEN_START_Y + row) * PIXEL_SIZE, 
                    PIXEL_SIZE, 
                    PIXEL_SIZE);
            }
        }
    }

    ctx.fillStyle = "#000000";
    for (let row = 0; row < displayHeight; row++) {
        for (let col = 0; col < displayWidth; col++) {
            if (!pixelIsOn(row, col, pixels)) {
                ctx.fillRect((SCREEN_START_X + col) * PIXEL_SIZE, 
                    (SCREEN_START_Y + row) * PIXEL_SIZE, 
                    PIXEL_SIZE, 
                    PIXEL_SIZE);
            }
        }
    }

    ctx.stroke();

    /* for (let row = 0; row < displayHeight; row++) {
        for (let col = 0; col < displayWidth; col++) {
            if pixelIsOn(row, col, displayPtr) {
                ctx.
            }
        }
    } */

    requestAnimationFrame(renderLoop);
}