import { CPU } from "viper";
import { memory } from "viper/viper_bg";

// define some constants 
const FRAME_RELATIVE_SIZE = 0.8;  // frame size relative to window
const PIXEL_SIZE = 20;            // px, size of each screen pixel in the canvas
const SCREEN_START_X = 7;         // px 
const SCREEN_START_Y = 7;        // px
const PIXEL_OFF_COLOR = "#000000";
const PIXEL_ON_COLOR = "#FFFFFF";

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

// whenever we select a new ROM, load it into our CPU
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

// get the display properties
const displayWidth = cpu.display_width();
const displayHeight = cpu.display_height();
const pixelsPtr = cpu.pixels();
const pixels = new Uint8Array(memory.buffer, pixelsPtr, displayWidth * displayHeight / 8);

// detect if a given pixel is on
const pixelIsOn = (row, col, pixels) => {
    const idx = row * displayWidth / 8 + Math.floor(col / 8);
    const mask = 1 << (7 - col % 8);
    return (pixels[idx] & mask) != 0;
}

// perform the actual rendering
const renderLoop = () => {
    cpu.step();
    ctx.beginPath();

    // render the ON pixels
    ctx.fillStyle = PIXEL_ON_COLOR;
    for (let row = 0; row < displayHeight; row++) { 
        for (let col = 0; col < displayWidth; col++) {
            if (pixelIsOn(row, col, pixels)) {
                ctx.fillRect((SCREEN_START_X + col) * PIXEL_SIZE, 
                    (SCREEN_START_Y + row) * PIXEL_SIZE, 
                    PIXEL_SIZE, 
                    PIXEL_SIZE);
            }
        }
    }

    // render the OFF pixels
    ctx.fillStyle = PIXEL_OFF_COLOR;
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

    // draw and request next frame
    ctx.stroke();
    requestAnimationFrame(renderLoop);
}